use std::time::Duration;

#[derive(Clone)]
pub struct SocketOptions {
    pub timeout: Duration,
    pub reconnection_timeout: Duration,
}

impl SocketOptions {
    pub fn builder() -> SocketOptionsBuilder {
        SocketOptionsBuilder {
            timeout: None,
            reconnection_timeout: None,
        }
    }
}

pub struct SocketOptionsBuilder {
    timeout: Option<Duration>,
    reconnection_timeout: Option<Duration>,
}

impl SocketOptionsBuilder {
    pub fn with_timeout(mut self, timeout: impl Into<Option<Duration>>) -> SocketOptionsBuilder {
        self.timeout = timeout.into();
        self
    }

    pub fn with_reconnection_timeout(
        mut self,
        reconnection_timeout: impl Into<Option<Duration>>,
    ) -> SocketOptionsBuilder {
        self.reconnection_timeout = reconnection_timeout.into();
        self
    }

    pub fn build(self) -> SocketOptions {
        SocketOptions {
            timeout: Duration::from_secs(5),
            reconnection_timeout: Duration::from_secs(5),
        }
    }
}
