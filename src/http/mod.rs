use crate::http::state::ClonableState;
use axum::Router;
use futures_util::StreamExt;
use routes::{api, root, sessionserver};
use tokio::{io, net, signal};
use tracing::debug;

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
                .nest("/http", api::router())
                .nest("/sessionserver", sessionserver::router()),
        )
        .with_state(state.clone());

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            signal::ctrl_c().await.unwrap();

            let sockets = state.servers().values().map(|server| &server.socket);
            futures::stream::iter(sockets)
                .for_each(|socket| async move {
                    socket.shutdown().await;
                })
                .await;

            debug!("Ctrl^C signal received. Quitting.");
        })
        .await
}
