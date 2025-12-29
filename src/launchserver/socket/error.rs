use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    ResponseNotReceived,
    Internal,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ResponseNotReceived => write!(fmt, "response not received"),
            Error::Internal => write!(fmt, "internal error occurred"),
        }
    }
}
