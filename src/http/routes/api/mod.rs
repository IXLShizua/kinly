mod get_profiles_by_usernames;

use crate::http::state::ClonableState;
use axum::{
    Router,
    routing::{MethodFilter, on},
};

pub fn router() -> Router<ClonableState> {
    Router::new().route(
        "/profiles/minecraft",
        on(
            MethodFilter::POST,
            get_profiles_by_usernames::get_profiles_by_usernames,
        ),
    )
}
