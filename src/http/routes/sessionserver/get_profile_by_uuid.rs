use crate::http::{
    dto::response::profile::Profile,
    extractors::current_server::CurrentServerHandle,
    routes::sessionserver::mapper::map_player_profile,
};
use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

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

#[instrument(
    name = "get_profile_by_uuid",
    level = "debug",
    skip_all,
    fields(
        unsigned = %unsigned,
        profile.id = %profile_id,
        server.name = %current_server.name(),
    )
)]
pub async fn get_profile_by_uuid(
    Path(GetProfileByUuidPath { profile_id }): Path<GetProfileByUuidPath>,
    Query(GetProfileByUuidQuery { unsigned }): Query<GetProfileByUuidQuery>,
    current_server: CurrentServerHandle,
) -> impl IntoResponse {
    debug!("handling get_profile_by_uuid request");

    let Ok(profile) = current_server
        .client()
        .get_profile_by_uuid(profile_id)
        .await
        .inspect(|value| debug!(value = ?value, "launcher::get_profile_by_uuid succeeded"))
        .inspect_err(|error| debug!(error = %error, "launcher::get_profile_by_uuid failed"))
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let response = map_player_profile(
        profile.player_profile,
        current_server.keypair().private.as_ref(),
        !unsigned,
    );

    debug!(response = ?response);

    (StatusCode::OK, Json(GetProfileByUuidResponse(response))).into_response()
}
