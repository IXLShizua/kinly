use crate::http::{
    dto::response::profile::Profile,
    extractors::current_server::CurrentServerHandle,
    routes::sessionserver::mapper::map_profile,
};
use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct GetProfileByUuidPath {
    pub profile_id: uuid::Uuid,
}

#[derive(Deserialize)]
pub struct GetProfileByUuidQuery {
    pub unsigned: bool,
}

#[derive(Serialize)]
pub struct GetProfileByUuidResponse(pub Profile);

pub async fn get_profile_by_uuid(
    Path(GetProfileByUuidPath { profile_id }): Path<GetProfileByUuidPath>,
    Query(GetProfileByUuidQuery { unsigned }): Query<GetProfileByUuidQuery>,
    current_server: CurrentServerHandle,
) -> impl IntoResponse {
    let Ok(profile) = current_server
        .client()
        .get_profile_by_uuid(profile_id)
        .await
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let response = map_profile(
        profile.player_profile,
        &current_server.keypair().private,
        now,
        !unsigned,
    );

    (StatusCode::OK, Json(GetProfileByUuidResponse(response))).into_response()
}
