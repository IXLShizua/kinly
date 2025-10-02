use crate::launchserver::{socket, types::response};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    UnexpectedResponse(response::any::Kind),
    Socket(socket::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedResponse(err) => write!(fmt, "Unexpected response: {:?}", err),
            Error::Socket(err) => write!(fmt, "Socket error: {}", err),
        }
    }
}

impl From<socket::Error> for Error {
    fn from(value: socket::Error) -> Self {
        Error::Socket(value)
    }
}
