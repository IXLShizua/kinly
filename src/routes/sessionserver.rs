use crate::{
    config::{server::experimental::rewrite::Rewrite, Meta},
    injector::types::{
        request,
        response::{
            profile,
            profile::property::{
                textures,
                textures::kind::{cape, skin, skin::metadata},
            },
        },
    },
    launcher,
    launcher::types::response::base::profile::skin::metadata::Model,
    routes::assets,
    state,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{on, MethodFilter},
    Json,
    Router,
};
use openssl::{base64, pkey, rsa, rsa::Padding};
use std::time::{self, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn router() -> Router<state::State> {
    Router::new().nest(
        "/session/minecraft",
        Router::new()
            .route("/hasJoined", on(MethodFilter::GET, has_joined))
            .route("/profile/:uuid", on(MethodFilter::GET, profile_by_uuid)),
    )
}

async fn has_joined(
    State(state): State<state::State>,
    Path(server_id): Path<String>,
    Query(query): Query<request::has_joined::Query>,
) -> impl IntoResponse {
    let Some(current_server) = state.servers.get(&server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };
    let Some(socket) = state.sockets.socket(&server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(check_server) = socket
        .with_token_restore(current_server, || {
            socket.check_server(
                query.username.clone(),
                query.server_id.clone(),
                false,
                false,
            )
        })
        .await
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(profile) = socket
        .with_token_restore(current_server, || {
            socket.get_profile_by_uuid(check_server.uuid)
        })
        .await
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let response = map_profile(
        profile.player_profile,
        &state.key_pair.rsa,
        now,
        false,
        server_id,
        current_server
            .experimental
            .as_ref()
            .and_then(|v| v.rewrite.as_ref())
            .map(|v| (&state.config.meta, v)),
    );

    (StatusCode::OK, Json(response)).into_response()
}

async fn profile_by_uuid(
    State(state): State<state::State>,
    Path((server_id, uuid)): Path<(String, Uuid)>,
    Query(request::profile_by_uuid::Query { unsigned }): Query<request::profile_by_uuid::Query>,
) -> impl IntoResponse {
    let Some(current_server) = state.servers.get(&server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };
    let Some(socket) = state.sockets.socket(&server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(profile) = socket
        .with_token_restore(current_server, || socket.get_profile_by_uuid(uuid))
        .await
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let response = map_profile(
        profile.player_profile,
        &state.key_pair.rsa,
        now,
        !unsigned,
        server_id,
        current_server
            .experimental
            .as_ref()
            .and_then(|v| v.rewrite.as_ref())
            .map(|v| (&state.config.meta, v)),
    );

    (StatusCode::OK, Json(response)).into_response()
}

fn map_profile(
    profile: launcher::types::response::base::profile::Profile,
    rsa: &rsa::Rsa<pkey::Private>,
    now: time::Duration,
    signed: bool,
    server_id: impl Into<String>,
    rewrite: Option<(&Meta, &Rewrite)>,
) -> profile::Profile {
    let server_id = server_id.into();

    let (rewritten_skin_url, rewritten_cape_url) = match rewrite {
        None => (None, None),
        Some((meta, rewrite)) => {
            let assets = meta.public.join(&format!("{}/assets/", server_id)).ok();

            match rewrite {
                Rewrite::AllInOne(v) if *v => (assets.clone(), assets),
                Rewrite::AllInOne(_) => (None, None),
                Rewrite::Separated {
                    skins: skins_flag,
                    capes: capes_flag,
                } => match (*skins_flag, *capes_flag) {
                    (true, true) => (assets.clone(), assets),
                    (true, false) => (assets, None),
                    (false, true) => (None, assets),
                    (false, false) => (None, None),
                },
            }
        }
    };

    let skin = profile.assets.skin.map(|skin| skin::Skin {
        url: rewritten_skin_url
            .and_then(|v| {
                let asset = assets::Pair {
                    hash: skin.digest,
                    original: skin.url.clone(),
                };
                let serialized =
                    serde_json::to_string(&asset).expect("This should not fail with an error");
                let encoded = base64::encode_block(serialized.as_bytes());

                v.join(&encoded).ok()
            })
            .map(|v| v.to_string())
            .unwrap_or(skin.url.to_string()),
        metadata: skin.metadata.and_then(|meta| match meta.model {
            Model::Default => None,
            Model::Slim => Some(metadata::Metadata {
                model: metadata::Model::Slim,
            }),
        }),
    });
    let cape = profile.assets.cape.map(|cape| cape::Cape {
        url: rewritten_cape_url
            .and_then(|v| {
                let asset = assets::Pair {
                    hash: cape.digest,
                    original: cape.url.clone(),
                };
                let serialized =
                    serde_json::to_string(&asset).expect("This should not fail with an error");
                let encoded = base64::encode_block(serialized.as_bytes());

                v.join(&encoded).ok()
            })
            .map(|v| v.to_string())
            .unwrap_or(cape.url.to_string()),
    });

    let textures = textures::Textures {
        timestamp: now.as_millis(),
        profile_id: profile.uuid.simple().to_string(),
        profile_name: profile.username.clone(),
        signature_required: signed,
        textures: textures::kind::Kind { skin, cape },
    };
    let serialized_textures = serde_json::to_string(&textures).unwrap();

    let encoded = base64::encode_block(serialized_textures.as_bytes());
    let encoded_signature = match signed {
        true => {
            let mut bytes = vec![0u8; encoded.len()];
            let _ = rsa
                .private_encrypt(encoded.as_bytes(), &mut bytes, Padding::PKCS1)
                .unwrap();
            let encoded_encrypted = base64::encode_block(&bytes);

            Some(encoded_encrypted)
        }
        false => None,
    };

    profile::Profile {
        id: profile.uuid.simple().to_string(),
        name: profile.username,
        properties: vec![profile::property::Property {
            name: "textures".to_string(),
            value: encoded,
            signature: encoded_signature,
        }],
    }
}
