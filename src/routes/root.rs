use crate::{config::server, injector::types::response, state};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{on, MethodFilter},
    Json,
    Router,
};

pub fn router() -> Router<state::State> {
    Router::new().route("/", on(MethodFilter::GET, root))
}

async fn root(
    State(state): State<state::State>,
    Path(server_id): Path<String>,
) -> impl IntoResponse {
    let Some(server) = state.servers.get(&server_id) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let skin_domains = match &server.meta.assets {
        server::meta::Assets::AllInOne(v) => v.clone(),
        server::meta::Assets::Separated { skins, capes } => skins
            .iter()
            .cloned()
            .chain(capes.iter().cloned())
            .collect::<Vec<_>>(),
    };

    let response = response::root::Root {
        meta: response::root::meta::Meta {
            server_name: Some(server_id),
            implementation_name: Some("Auth-Proxy-GL".to_string()),
            implementation_version: None,
        },
        skin_domains,
        signature_public_key: state.key_pair.public.clone(),
    };

    (StatusCode::OK, Json(response)).into_response()
}
