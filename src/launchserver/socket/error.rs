use std::fmt::{Display, Formatter};
use tokio::{
    sync::{mpsc, oneshot},
    time,
};
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub enum Error {
    Send,
    Recv(oneshot::error::RecvError),
    Socket(tungstenite::Error),
    TimeoutExceeded(time::error::Elapsed),
    Serialize(serde_json::error::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Socket(err) => write!(fmt, "Socket: {}", err),
            Error::Send => write!(fmt, "Send to channel"),
            Error::Recv(err) => write!(fmt, "Recv from channel: {}", err),
            Error::TimeoutExceeded(err) => {
                write!(fmt, "Response wait timeout exceeded: {}", err)
            }
            Error::Serialize(err) => write!(fmt, "Failed to serialize request: {}", err),
        }
    }
}

impl From<tungstenite::Error> for Error {
    fn from(value: tungstenite::Error) -> Self {
        Error::Socket(value)
    }
}

impl<T> From<mpsc::error::SendError<T>> for Error {
    fn from(_value: mpsc::error::SendError<T>) -> Self {
        Error::Send
    }
}

impl From<oneshot::error::RecvError> for Error {
    fn from(value: oneshot::error::RecvError) -> Self {
        Error::Recv(value)
    }
}

impl From<time::error::Elapsed> for Error {
    fn from(value: time::error::Elapsed) -> Self {
        Error::TimeoutExceeded(value)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(value: serde_json::error::Error) -> Self {
        Error::Serialize(value)
    }
}
