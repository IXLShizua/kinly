use crate::{
    injector,
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
    let Some(socket) = state.sockets.socket(server_id) else {
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

    let response = transform_profile(now, false, &state.key_pair.rsa, profile.player_profile);

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
    let Some(socket) = state.sockets.socket(server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(profile) = socket
        .with_token_restore(current_server, || socket.get_profile_by_uuid(uuid))
        .await
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let response = transform_profile(now, !unsigned, &state.key_pair.rsa, profile.player_profile);

    (StatusCode::OK, Json(response)).into_response()
}

fn transform_profile(
    now: time::Duration,
    signed: bool,
    rsa: &rsa::Rsa<pkey::Private>,
    profile: launcher::types::response::base::profile::Profile,
) -> profile::Profile {
    let skin = profile.assets.skin.map(|skin| skin::Skin {
        url: skin.url,
        metadata: skin.metadata.and_then(|meta| match meta.model {
            Model::Default => None,
            Model::Slim => Some(metadata::Metadata {
                model: metadata::Model::Slim,
            }),
        }),
    });
    let cape = profile.assets.cape.map(|cape| cape::Cape { url: cape.url });

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
