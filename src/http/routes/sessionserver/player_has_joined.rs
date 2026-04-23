use crate::http::{
    dto::response::profile::Profile,
    extractors::current_server::CurrentServerHandle,
    routes::sessionserver::mapper::map_player_profile,
};
use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

#[derive(Deserialize)]
pub struct PlayerHasJoinedQuery {
    pub username: String,

    #[serde(rename = "serverId")]
    pub server_id: String,

    pub ip: Option<String>,
}

#[derive(Serialize)]
pub struct PlayerHasJoinedResponse(pub Profile);

#[instrument(
    name = "player_has_joined",
    level = "debug",
    skip_all,
    fields(
        server.name = %current_server.name(),
        server.id = %server_id,
        username = %username,
    )
)]
pub async fn player_has_joined(
    Query(PlayerHasJoinedQuery {
        username,
        server_id,
        ip: _,
    }): Query<PlayerHasJoinedQuery>,
    current_server: CurrentServerHandle,
) -> impl IntoResponse {
    debug!("checking whether player has joined server");

    let Ok(check_server) = current_server
        .client()
        .check_server(username.clone(), server_id.clone(), false, false)
        .await
        .inspect(|value| debug!(value = ?value, "launcher::check_server succeeded"))
        .inspect_err(|error| debug!(error = %error, "launcher::check_server failed"))
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(profile) = current_server
        .client()
        .get_profile_by_uuid(check_server.uuid)
        .await
        .inspect(|value| debug!(value = ?value.player_profile, "launcher::get_profile_by_uuid succeeded"))
        .inspect_err(|error| debug!(error = %error, "launcher::get_profile_by_uuid failed"))
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let response = map_player_profile(
        profile.player_profile,
        &current_server.keypair().private,
        false,
    );

    debug!(response = ?response);

    (StatusCode::OK, Json(PlayerHasJoinedResponse(response))).into_response()
}
