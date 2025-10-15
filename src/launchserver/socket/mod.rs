mod error;
mod events;
mod options;

pub use error::*;
pub use options::*;

use crate::launchserver::{
    socket::events::{input, output},
    types::{request, response},
};
use futures_util::{
    SinkExt,
    StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::{collections::HashMap, time::Duration};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
    task,
    time,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite};
use tracing::debug;
use uuid::Uuid;

const CAPACITY: usize = 256;
const CONCURRENCY: usize = 5120;

/// Represents a message sent to the actor.
struct ActorMessage {
    /// Sender for sending back the response.
    sender: oneshot::Sender<response::any::Kind>,
    /// The actual request to be processed.
    request: request::any::Any,
}

type WebSocketSender = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>;
type WebSocketReceiver = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Represents the WebSocket client.
pub struct Socket {
    /// Sender to communicate with the actor handling the loop.
    actor_sender: mpsc::Sender<input::Loop>,
    actor_handle: task::JoinHandle<()>,

    options: options::SocketOptions,
}

impl Socket {
    /// Creates a new `Socket` instance and starts the actor loop.
    ///
    /// # Arguments
    ///
    /// * `addr` - The URL of the WebSocket server to connect to.
    ///
    /// # Returns
    ///
    /// A new instance of `Socket`.
    pub fn new(addr: impl Into<url::Url>, options: options::SocketOptions) -> Socket {
        let (actor_sender, actor_receiver) = mpsc::channel(CAPACITY);
        let actor_handle = tokio::spawn(start_handle_loop(
            addr.into(),
            options.reconnection_timeout,
            actor_receiver,
        ));

        Socket {
            actor_sender,
            actor_handle,

            options,
        }
    }

    /// Sends a request through the WebSocket and awaits a response.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to be sent.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response on success or an `Error` on failure.
    pub async fn send_request(
        &self,
        request: request::any::Any,
        timeout: Duration,
    ) -> Result<response::any::Kind, Error> {
        let request_id = request.id;

        let (tx, rx) = oneshot::channel();

        self.actor_sender
            .send(input::Loop::Message(ActorMessage {
                sender: tx,
                request,
            }))
            .await
            .expect("actor channel is closed");

        match time::timeout(timeout, rx).await {
            Ok(res) => Ok(res.expect("sender of the callback cannot be closed")),
            Err(_) => {
                let _ = self
                    .actor_sender
                    .send(input::Loop::CancelMessage(request_id))
                    .await;

                Err(Error::ResponseNotReceived)
            }
        }
    }

    /// Initiates a graceful shutdown of the WebSocket connection.
    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();

        let _ = self.actor_sender.send(input::Loop::Shutdown(tx)).await;
        let _ = time::timeout(self.options.timeout, rx).await;
        if !self.actor_handle.is_finished() {
            self.actor_handle.abort();
        }
    }
}

/// Starts the main loop that manages the WebSocket connection and message handling.
///
/// # Arguments
///
/// * `addr` - The URL of the WebSocket server to connect to.
/// * `reconnection_timeout` - Optional duration to wait before attempting reconnection.
/// * `ev_receiver` - Receiver for incoming events from the actor.
async fn start_handle_loop(
    addr: impl Into<url::Url>,
    reconnection_timeout: impl Into<Option<Duration>>,
    mut ev_receiver: mpsc::Receiver<input::Loop>,
) {
    let addr = addr.into();
    let reconnection_timeout = reconnection_timeout.into();

    // Map to correlate requests with their response senders using UUIDs.
    let mut requests_callbacks: HashMap<Uuid, oneshot::Sender<response::any::Kind>> =
        HashMap::new();

    // Channels for WebSocket input and output events.
    let (mut ws_input_ev_sender, mut ws_input_ev_receiver) =
        mpsc::channel::<input::websocket::Loop>(CAPACITY);
    let (mut ws_output_ev_sender, mut ws_output_ev_receiver) =
        mpsc::channel::<output::websocket::Loop>(CAPACITY);

    // Channels for loopback (connection management) events.
    let (loopback_input_ev_sender, loopback_input_ev_receiver) =
        mpsc::channel::<input::loopback::Loop>(1);
    let (loopback_output_ev_sender, mut loopback_output_ev_receiver) =
        mpsc::channel::<output::loopback::Loop<WebSocketReceiver, WebSocketSender>>(1);

    // Flags to track connection state.
    let mut ws_is_connected = false;

    // Initiate connection to the WebSocket server.
    let _ = loopback_input_ev_sender
        .send(input::loopback::Loop::ConnectSocket {
            addr: addr.clone(),
            timeout: reconnection_timeout,
        })
        .await;

    // Spawn the loopback handler loop.
    tokio::spawn(start_loopback_handle_loop(
        loopback_output_ev_sender,
        loopback_input_ev_receiver,
    ));

    // Main event loop.
    loop {
        tokio::select! {
            // Handle events from the loopback output (e.g., successful connection)
            Some(event) = loopback_output_ev_receiver.recv() => {
                match event {
                    output::loopback::Loop::SocketConnected { read, write } => {
                        // Reinitialize WebSocket input and output channels upon connection.
                        (ws_input_ev_sender, ws_input_ev_receiver) =
                            mpsc::channel::<input::websocket::Loop>(CAPACITY);
                        (ws_output_ev_sender, ws_output_ev_receiver) =
                            mpsc::channel::<output::websocket::Loop>(CAPACITY);

                        // Spawn the WebSocket handler loop.
                        tokio::spawn(start_ws_handle_loop(
                            ws_output_ev_sender,
                            ws_input_ev_receiver,
                            read,
                            write,
                        ));

                        ws_is_connected = true;
                    }
                }
            }

            Some(event) = ev_receiver.recv() => {
                match event {
                    input::Loop::Message(msg) => {
                        let Ok(json_request) = serde_json::to_string(&msg.request) else {
                            continue;
                        };

                        requests_callbacks.insert(msg.request.id, msg.sender);

                        let _ = ws_input_ev_sender
                            .send(input::websocket::Loop::Message(tungstenite::Message::text(json_request)))
                            .await;
                    },
                    input::Loop::CancelMessage(request_id) => {
                        if requests_callbacks.remove(&request_id).is_none() {
                            debug!("failed to remove message {} with exceeded timeout", request_id);
                        }
                    }
                    input::Loop::Shutdown(sender) => {
                        debug!("start_handle_loop shutdown received");

                        if ws_is_connected {
                            // Initiate shutdown of the WebSocket handler.
                            let (ws_tx, ws_rx) = oneshot::channel();
                            let _ = ws_input_ev_sender
                                .send(input::websocket::Loop::Shutdown(ws_tx))
                                .await;
                            let _ = ws_rx.await;
                        }

                        // Initiate shutdown of the loopback handler.
                        let (loopback_tx, loopback_rx) = oneshot::channel();
                        let _ = loopback_input_ev_sender
                            .send(input::loopback::Loop::Shutdown(loopback_tx))
                            .await;
                        let _ = loopback_rx.await;

                        // Confirm shutdown to the caller.
                        let _ = sender.send(());

                        break;
                    }
                }
            }

            // Handle events from the WebSocket output (e.g., incoming messages, errors).
            Some(event) = ws_output_ev_receiver.recv() => {
                match event {
                    output::websocket::Loop::Message(msg) => {
                        let tungstenite::Message::Text(text) = msg else {
                            continue
                        };

                        let response = match serde_json::from_str::<response::any::Any>(&text) {
                            Ok(response) => response,
                            Err(err) => {
                                debug!("failed to deserialize response: {}, error: {}", text, err);
                                continue;
                            }
                        };

                        let Some(sender) = requests_callbacks.remove(&response.id) else {
                            debug!("failed to find callback with id {}", response.id);
                            continue;
                        };

                        // Send the response back through the oneshot channel.
                        if sender.send(response.body).is_err() {
                            debug!("failed to send response to channel");
                        }
                    }
                    output::websocket::Loop::Disconnect => {
                        ws_is_connected = false;

                        // Initiate shutdown of the WebSocket handler.
                        let (ws_tx, ws_rx) = oneshot::channel();
                        let _ = ws_input_ev_sender
                            .send(input::websocket::Loop::Shutdown(ws_tx))
                            .await;
                        let _ = ws_rx.await;

                        // Attempt to reconnect by sending a connect request to loopback.
                        let _ = loopback_input_ev_sender
                        .send(input::loopback::Loop::ConnectSocket {
                            addr: addr.clone(),
                            timeout: reconnection_timeout
                        })
                        .await;
                    }
                }
            }
        }
    }
}

/// Handles the loopback for managing socket connections, including reconnections.
///
/// # Arguments
///
/// * `ev_sender` - Sender to communicate events back to the main loop.
/// * `ev_receiver` - Receiver for incoming loopback events.
async fn start_loopback_handle_loop(
    ev_sender: mpsc::Sender<output::loopback::Loop<WebSocketReceiver, WebSocketSender>>,
    mut ev_receiver: mpsc::Receiver<input::loopback::Loop>,
) {
    let mut connect_task: Option<task::JoinHandle<()>> = None;

    while let Some(event) = ev_receiver.recv().await {
        match event {
            input::loopback::Loop::ConnectSocket { addr, timeout } => {
                if let Some(task) = connect_task {
                    task.abort();
                }

                connect_task = Some(tokio::spawn({
                    let ev_sender = ev_sender.clone();

                    async move {
                        let (ws_stream, _) = loop {
                            match tokio_tungstenite::connect_async(addr.clone()).await {
                                Ok(ws_stream) => {
                                    debug!("successfully connected to socket with addr: {}", addr);

                                    break ws_stream;
                                }
                                Err(err) => {
                                    debug!("error with connect to socket: {}", err);

                                    if let Some(timeout) = timeout {
                                        tokio::time::sleep(timeout).await;
                                    }
                                }
                            }
                        };

                        let (ws_sender, ws_receiver) = ws_stream.split();

                        // Notify the main loop of the successful connection.
                        let _ = ev_sender
                            .send(output::loopback::Loop::SocketConnected {
                                read: ws_receiver,
                                write: ws_sender,
                            })
                            .await;
                    }
                }));
            }
            input::loopback::Loop::Shutdown(sender) => {
                debug!("start_loopback_handle_loop shutdown received");

                if let Some(task) = connect_task {
                    task.abort();
                }

                // Confirm shutdown to the caller.
                let _ = sender.send(());
                break;
            }
        }
    }
}

/// Handles the WebSocket communication by spawning separate tasks for sending and receiving messages.
///
/// # Arguments
///
/// * `ev_sender` - Sender to communicate events back to the main loop.
/// * `ev_receiver` - Receiver for incoming WebSocket events.
/// * `ws_receiver` - The split WebSocket receiver stream.
/// * `ws_sender` - The split WebSocket sender sink.
async fn start_ws_handle_loop(
    ev_sender: mpsc::Sender<output::websocket::Loop>,
    mut ev_receiver: mpsc::Receiver<input::websocket::Loop>,
    ws_receiver: WebSocketReceiver,
    ws_sender: WebSocketSender,
) {
    // Channels for handling outgoing WebSocket messages.
    let (ws_sender_input_ev_tx, ws_sender_input_ev_rx) =
        mpsc::channel::<input::websocket::sender::Loop>(CAPACITY);
    let (ws_sender_output_ev_tx, mut ws_sender_output_ev_rx) =
        mpsc::channel::<output::websocket::sender::Loop>(CAPACITY);

    // Channels for handling incoming WebSocket messages.
    let (ws_receiver_input_ev_tx, ws_receiver_input_ev_rx) =
        mpsc::channel::<input::websocket::receiver::Loop>(CAPACITY);
    let (ws_receiver_output_ev_tx, mut ws_receiver_output_ev_rx) =
        mpsc::channel::<output::websocket::receiver::Loop>(CAPACITY);

    // Spawn tasks to handle outgoing and incoming WebSocket messages.
    tokio::spawn(start_handle_outgoing_messages(
        ws_sender_output_ev_tx,
        ws_sender_input_ev_rx,
        ws_sender,
    ));
    tokio::spawn(start_handle_incoming_messages(
        ws_receiver_output_ev_tx,
        ws_receiver_input_ev_rx,
        ws_receiver,
    ));

    loop {
        tokio::select! {
            // Handle incoming events from the main loop.
            Some(event) = ev_receiver.recv() => {
                match event {
                    input::websocket::Loop::Message(msg) => {
                        let _ = ws_sender_input_ev_tx
                            .send(input::websocket::sender::Loop::Message(msg))
                            .await;
                    }
                    input::websocket::Loop::Shutdown(sender) => {
                        debug!("start_ws_handle_loop shutdown received");

                        // Create oneshot channels to confirm shutdown of sender and receiver.
                        let (outgoing_tx, outgoing_rx) = oneshot::channel::<()>();
                        let (incoming_tx, incoming_rx) = oneshot::channel::<()>();

                        // Send shutdown signals to both outgoing and incoming handlers.
                        let _ = tokio::join!(
                            ws_sender_input_ev_tx.send(input::websocket::sender::Loop::Shutdown(outgoing_tx)),
                            outgoing_rx,

                            ws_receiver_input_ev_tx.send(input::websocket::receiver::Loop::Shutdown(incoming_tx)),
                            incoming_rx
                        );

                        // Confirm shutdown to the caller.
                        let _ = sender.send(());

                        break;
                    }
                }
            }

            // Handle events from the outgoing message handler.
            Some(ws_sender_output_ev) = ws_sender_output_ev_rx.recv() => {
                match ws_sender_output_ev {
                    output::websocket::sender::Loop::FailedToSend(msg, err) => {
                        debug!("failed to send message: {} {}", msg, err);

                        if matches!(
                            err,
                            tungstenite::Error::ConnectionClosed
                                | tungstenite::Error::AlreadyClosed
                                | tungstenite::Error::Io(_)
                                | tungstenite::Error::Tls(_)
                                | tungstenite::Error::Protocol(_)
                        ) {
                            let _ = ev_sender.send(output::websocket::Loop::Disconnect).await;
                        }
                    }
                }
            }

            // Handle events from the incoming message handler.
            Some(ws_receiver_output_ev) = ws_receiver_output_ev_rx.recv() => {
                match ws_receiver_output_ev {
                    output::websocket::receiver::Loop::Message(msg) => {
                        let _ = ev_sender.send(output::websocket::Loop::Message(msg)).await;
                    }
                }
            }
        }
    }
}

/// Handles sending outgoing WebSocket messages.
///
/// # Arguments
///
/// * `ev_sender` - Sender to communicate events back to the WebSocket handler.
/// * `ev_receiver` - Receiver for incoming outgoing message events.
/// * `ws_sender` - The WebSocket sender sink.
async fn start_handle_outgoing_messages(
    ev_sender: mpsc::Sender<output::websocket::sender::Loop>,
    mut ev_receiver: mpsc::Receiver<input::websocket::sender::Loop>,
    mut ws_sender: WebSocketSender,
) {
    let mut join_set = tokio::task::JoinSet::<()>::new();

    loop {
        tokio::select! {
            Some(event) = ev_receiver.recv() => {
                match event {
                    input::websocket::sender::Loop::Message(msg) => {
                        while join_set.len() >= CONCURRENCY {
                            join_set.join_next().await;
                        }

                        debug!("raw outgoing message: {:?}", msg);
                        if let Err(err) = ws_sender.send(msg.clone()).await {
                            join_set.spawn({
                                let ev_sender = ev_sender.clone();

                                async move {
                                    let _ = ev_sender
                                        .send(output::websocket::sender::Loop::FailedToSend(msg, err))
                                        .await;
                                }
                            });
                        }
                    }
                    input::websocket::sender::Loop::Shutdown(sender) => {
                        debug!("start_handle_outgoing_messages shutdown received");

                        join_set.join_all().await;
                        let _ = sender.send(());

                        break;
                    }
                }
            }
        }
    }
}

/// Handles receiving incoming WebSocket messages.
///
/// # Arguments
///
/// * `ev_sender` - Sender to communicate events back to the WebSocket handler.
/// * `ev_receiver` - Receiver for incoming message events.
/// * `ws_receiver` - The WebSocket receiver stream.
async fn start_handle_incoming_messages(
    ev_sender: mpsc::Sender<output::websocket::receiver::Loop>,
    mut ev_receiver: mpsc::Receiver<input::websocket::receiver::Loop>,
    mut ws_receiver: WebSocketReceiver,
) {
    let mut join_set = tokio::task::JoinSet::<()>::new();

    loop {
        tokio::select! {
            Some(msg) = ws_receiver.next() => {
                match msg {
                    Ok(msg) => {
                        while join_set.len() >= CONCURRENCY {
                            join_set.join_next().await;
                        }

                        debug!("raw incoming message: {:?}", msg);
                        join_set.spawn({
                            let ev_sender = ev_sender.clone();

                            async move {
                                let _ = ev_sender
                                    .send(output::websocket::receiver::Loop::Message(msg))
                                    .await;
                            }
                        });
                    },
                    Err(err) => {
                        debug!("socket receiver error: {}", err)
                    }
                }
            },
            Some(event) = ev_receiver.recv() => {
                match event {
                    input::websocket::receiver::Loop::Shutdown(sender) => {
                        debug!("start_handle_incoming_messages shutdown received");

                        join_set.join_all().await;
                        let _ = sender.send(());

                        break;
                    }
                }
            }
        }
    }
}
