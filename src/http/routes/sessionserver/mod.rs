mod get_profile_by_uuid;
mod mapper;
mod player_has_joined;

use crate::http::state::ClonableState;
use axum::{
    routing::{on, MethodFilter},
    Router,
};

pub fn router() -> Router<ClonableState> {
    Router::new().nest(
        "/session/minecraft",
        Router::new()
            .route(
                "/hasJoined",
                on(MethodFilter::GET, player_has_joined::player_has_joined),
            )
            .route(
                "/profile/:uuid",
                on(MethodFilter::GET, get_profile_by_uuid::get_profile_by_uuid),
            ),
    )
}
