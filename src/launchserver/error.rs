use crate::launchserver::{socket, types::response};
use std::fmt::{Display, Formatter};

// The Error::UnexpectedResponse variant, although much heavier than Error::Internal, is the most common variant.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Error {
    UnexpectedResponse(response::any::Kind),
    Internal(socket::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedResponse(err) => {
                write!(fmt, "unexpected response received: {:?}", err)
            }
            Error::Internal(err) => write!(fmt, "internal socket error: {}", err),
        }
    }
}

impl From<socket::Error> for Error {
    fn from(value: socket::Error) -> Self {
        Error::Internal(value)
    }
}
