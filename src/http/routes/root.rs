use crate::http::state::ClonableState;
use axum::{
    Json,
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{MethodFilter, on},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

#[derive(Deserialize)]
pub struct RootPath {
    pub server_name: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RootResponse {
    pub meta: meta::Meta,
    pub skin_domains: Vec<String>,
    pub signature_public_key: String,
}

pub mod meta {
    use serde::Serialize;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Meta {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub server_name: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub implementation_name: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub implementation_version: Option<String>,
    }
}

pub fn router() -> Router<ClonableState> {
    Router::new().route("/", on(MethodFilter::GET, root))
}

#[instrument(
    name = "root",
    level = "debug",
    skip_all,
    fields(
        server.name = %server_name,
    )
)]
async fn root(
    State(state): State<ClonableState>,
    Path(RootPath { server_name }): Path<RootPath>,
) -> impl IntoResponse {
    debug!("handling root request");

    let Some(server) = state.servers.get(&server_name) else {
        debug!("server not found");
        return StatusCode::NO_CONTENT.into_response();
    };

    let response = RootResponse {
        meta: meta::Meta {
            server_name: Some(server_name),
            implementation_name: Some("Kinly".to_string()),
            implementation_version: None,
        },
        skin_domains: server.assets.clone(),
        signature_public_key: server.key_pair.public.clone(),
    };

    debug!(response=?response);

    (StatusCode::OK, Json(response)).into_response()
}
