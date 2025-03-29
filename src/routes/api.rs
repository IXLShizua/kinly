use crate::{
    injector::types::{request::profiles_by_usernames, response::profile},
    state,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{on, MethodFilter},
    Json,
    Router,
};

pub fn router() -> Router<state::State> {
    Router::new().route(
        "/profiles/minecraft",
        on(MethodFilter::POST, profiles_by_usernames),
    )
}

async fn profiles_by_usernames(
    State(state): State<state::State>,
    Path(server_id): Path<String>,
    Json(profiles_by_usernames::Body(usernames)): Json<profiles_by_usernames::Body>,
) -> impl IntoResponse {
    let Some(current_server) = state.servers.get(&server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };
    let Some(socket) = state.sockets.socket(server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let Ok(profiles) = socket
        .with_token_restore(current_server, || {
            socket.batch_profiles_by_usernames(usernames.clone())
        })
        .await
    else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let response = profiles
        .player_profiles
        .into_iter()
        .filter_map(|profile| {
            profile.map(|profile| profile::Profile {
                id: profile.uuid.simple().to_string(),
                name: profile.username,
                properties: Vec::new(),
            })
        })
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(response)).into_response()
}
