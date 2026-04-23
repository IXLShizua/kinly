use crate::http::{
    dto::response::profile::Profile,
    extractors::current_server::CurrentServerHandle,
};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

#[derive(Deserialize)]
pub struct GetProfilesByUsernamesRequest(pub Vec<String>);

#[derive(Serialize)]
pub struct GetProfilesByUsernamesResponse(pub Vec<Profile>);

#[instrument(
    name = "get_profiles_by_usernames",
    level = "debug",
    skip_all,
    fields(
        server.name = %current_server.name(),
        usernames = ?usernames,
    )
)]
pub async fn get_profiles_by_usernames(
    current_server: CurrentServerHandle,
    Json(GetProfilesByUsernamesRequest(usernames)): Json<GetProfilesByUsernamesRequest>,
) -> impl IntoResponse {
    debug!("handling get_profiles_by_usernames request");

    let Ok(profiles) = current_server
        .client()
        .batch_profiles_by_usernames(usernames.clone())
        .await
        .inspect(|value| debug!(value = ?value.player_profiles, "launcher::get_profiles_by_usernames succeeded"))
        .inspect_err(|error| debug!(error = %error, "launcher::get_profiles_by_usernames failed"))
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let response = profiles
        .player_profiles
        .into_iter()
        .filter_map(|profile| {
            profile.map(|profile| Profile {
                id: profile.uuid.simple().to_string(),
                name: profile.username,
                properties: Vec::new(),
            })
        })
        .collect::<Vec<_>>();

    debug!(response = ?response);

    (
        StatusCode::OK,
        Json(GetProfilesByUsernamesResponse(response)),
    )
        .into_response()
}
