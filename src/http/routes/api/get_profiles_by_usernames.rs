use crate::http::{
    dto::response::profile::Profile,
    extractors::current_server::CurrentServerHandle,
};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetProfilesByUsernamesRequest(pub Vec<String>);

#[derive(Serialize)]
pub struct GetProfilesByUsernamesResponse(pub Vec<Profile>);

pub async fn get_profiles_by_usernames(
    current_server: CurrentServerHandle,
    Json(GetProfilesByUsernamesRequest(usernames)): Json<GetProfilesByUsernamesRequest>,
) -> impl IntoResponse {
    let Ok(profiles) = current_server
        .client()
        .batch_profiles_by_usernames(usernames.clone())
        .await
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

    (
        StatusCode::OK,
        Json(GetProfilesByUsernamesResponse(response)),
    )
        .into_response()
}
