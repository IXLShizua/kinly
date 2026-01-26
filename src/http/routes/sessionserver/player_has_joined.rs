use crate::http::{
    dto::response::profile::Profile,
    extractors::current_server::CurrentServerHandle,
    routes::sessionserver::mapper::map_profile,
};
use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct PlayerHasJoinedQuery {
    pub username: String,

    #[serde(rename = "serverId")]
    pub server_id: String,

    pub ip: Option<String>,
}

#[derive(Serialize)]
pub struct PlayerHasJoinedResponse(pub Profile);

pub async fn player_has_joined(
    Query(PlayerHasJoinedQuery {
        username,
        server_id,
        ip: _,
    }): Query<PlayerHasJoinedQuery>,
    current_server: CurrentServerHandle,
) -> impl IntoResponse {
    let Ok(check_server) = current_server
        .client()
        .check_server(username.clone(), server_id.clone(), false, false)
        .await
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(profile) = current_server
        .client()
        .get_profile_by_uuid(check_server.uuid)
        .await
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let response = map_profile(
        profile.player_profile,
        &current_server.keypair().private,
        now,
        false,
    );

    (StatusCode::OK, Json(PlayerHasJoinedResponse(response))).into_response()
}
