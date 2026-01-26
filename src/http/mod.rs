use axum::Router;
use futures_util::StreamExt;
use routes::{api, root, sessionserver};
use tokio::{
    io,
    net
    ,
};

pub mod dto;
mod routes;
pub mod state;

pub async fn init(
    listener: net::TcpListener,
    state: state::ClonableState,
) -> Result<(), io::Error> {
    let router = Router::new()
        .nest(
            "/{server_id}",
            Router::new()
                .merge(root::router())
                .nest("/api", api::router())
                .nest("/sessionserver", sessionserver::router()),
        )
        .with_state(state.clone());

    axum::serve(listener, router).await
}
