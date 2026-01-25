use crate::http::state::ClonableState;
use axum::Router;
use futures::future::select;
use futures_util::StreamExt;
use routes::{api, root, sessionserver};
use tokio::{
    io,
    net,
    signal::unix::{SignalKind, signal},
};
use tracing::info;

pub mod dto;
mod routes;
pub mod state;

pub async fn init(listener: net::TcpListener, state: state::State) -> Result<(), io::Error> {
    let state = ClonableState::new(state);

    let router = Router::new()
        .nest(
            "/{server_id}",
            Router::new()
                .merge(root::router())
                .nest("/api", api::router())
                .nest("/sessionserver", sessionserver::router()),
        )
        .with_state(state.clone());

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let mut sigterm =
                signal(SignalKind::terminate()).expect("failed to construct SIGTERM signal");
            let mut sigint =
                signal(SignalKind::interrupt()).expect("failed to construct SIGINT signal");

            tokio::select! {
                _ = sigterm.recv() => info!("SIGTERM received. Quitting."),
                _ = sigint.recv() => info!("SIGINT received. Quitting."),
            }

            let sockets = state.servers().values().map(|server| &server.socket);
            futures::stream::iter(sockets)
                .for_each(|socket| async move {
                    socket.shutdown().await;
                })
                .await;

            info!("Ctrl^C signal received. Quitting.");
        })
        .await
}
