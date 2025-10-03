use crate::http::{
    dto::response::profile::Profile,
    routes::sessionserver::mapper::map_profile,
    state::ClonableState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct GetProfileByUuidPath {
    pub server_id: String,
    pub profile_id: uuid::Uuid,
}

#[derive(Deserialize)]
pub struct GetProfileByUuidQuery {
    pub unsigned: bool,
}

#[derive(Serialize)]
pub struct GetProfileByUuidResponse(pub Profile);

pub async fn get_profile_by_uuid(
    State(state): State<ClonableState>,
    Path(GetProfileByUuidPath {
        server_id,
        profile_id,
    }): Path<GetProfileByUuidPath>,
    Query(GetProfileByUuidQuery { unsigned }): Query<GetProfileByUuidQuery>,
) -> impl IntoResponse {
    let Some(server) = state.servers().get(&server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(profile) = server.socket.get_profile_by_uuid(profile_id).await else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let response = map_profile(
        profile.player_profile,
        &server.key_pair.private,
        now,
        !unsigned,
    );

    (StatusCode::OK, Json(GetProfileByUuidResponse(response))).into_response()
}
