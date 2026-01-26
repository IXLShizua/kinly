use crate::{
    http::state::{ClonableState, Server, ServerKeyPair},
    launchserver,
};
use axum::{
    extract::{FromRequestParts, Path},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::sync::Arc;

pub struct CurrentServerHandle {
    server: Arc<Server>,
}

impl CurrentServerHandle {
    pub fn keypair(&self) -> &ServerKeyPair {
        &self.server.key_pair
    }

    pub fn assets(&self) -> &[String] {
        &self.server.assets
    }

    pub fn client(&self) -> &launchserver::Client {
        &self.server.client
    }
}

impl FromRequestParts<ClonableState> for CurrentServerHandle {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ClonableState,
    ) -> Result<Self, Self::Rejection> {
        let Path(path) = Path::<PathWithServerId>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        let server = state
            .servers
            .get(&path.server_id)
            .ok_or_else(|| StatusCode::NO_CONTENT.into_response())?;

        Ok(CurrentServerHandle {
            server: Arc::clone(server),
        })
    }
}

#[derive(Deserialize)]
struct PathWithServerId {
    server_id: String,
}
