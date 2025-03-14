use crate::state;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response,
    response::IntoResponse,
    routing::{on, MethodFilter},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{error, info};

pub fn router() -> Router<state::State> {
    Router::new().route("/:encoded", on(MethodFilter::GET, asset_by_hash))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pair {
    pub hash: String,
    pub original: url::Url,
}

pub async fn asset_by_hash(
    State(state): State<state::State>,
    Path((_, encoded)): Path<(String, String)>,
) -> impl IntoResponse {
    let Ok(bytes) = openssl::base64::decode_block(&encoded) else {
        return StatusCode::NO_CONTENT.into_response();
    };
    let Ok(deserialized) = serde_json::from_slice::<Pair>(&bytes) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let dir = state.data_dir.join("assets").join(&deserialized.hash[0..2]);
    let asset_path = dir.join(&deserialized.hash);

    let bytes = if !asset_path.exists() {
        let Ok(_) = fs::create_dir_all(&dir).await else {
            return StatusCode::NO_CONTENT.into_response();
        };

        let Ok(response) = reqwest::get(deserialized.original).await else {
            return StatusCode::NO_CONTENT.into_response();
        };
        let Ok(bytes) = response.bytes().await else {
            return StatusCode::NO_CONTENT.into_response();
        };

        let bytes = bytes.to_vec();
        let Ok(_) = fs::write(&asset_path, &bytes).await else {
            return StatusCode::NO_CONTENT.into_response();
        };

        bytes
    } else {
        let Ok(bytes) = fs::read(&asset_path).await else {
            return StatusCode::NO_CONTENT.into_response();
        };

        bytes
    };

    let mut response = response::Response::new(axum::body::Body::from(bytes));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    response
}
