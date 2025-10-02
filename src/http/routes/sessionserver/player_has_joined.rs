use crate::http::{
    dto::response::profile::Profile,
    routes::sessionserver::mapper::map_profile,
    state::ClonableState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct PlayerHasJoinedPath {
    pub server_id: String,
}

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
    State(state): State<ClonableState>,
    Path(PlayerHasJoinedPath {
        server_id: path_server_id,
    }): Path<PlayerHasJoinedPath>,
    Query(PlayerHasJoinedQuery {
        username,
        server_id,
        ip: _,
    }): Query<PlayerHasJoinedQuery>,
) -> impl IntoResponse {
    let Some(server) = state.servers().get(&path_server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(check_server) = server
        .socket
        .check_server(username.clone(), server_id.clone(), false, false)
        .await
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(profile) = server.socket.get_profile_by_uuid(check_server.uuid).await else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let response = map_profile(profile.player_profile, &server.key_pair.private, now, false);

    (StatusCode::OK, Json(PlayerHasJoinedResponse(response))).into_response()
}
