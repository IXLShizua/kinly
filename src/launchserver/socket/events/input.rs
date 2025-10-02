use crate::launchserver::socket::ActorMessage;
use tokio::sync::oneshot;

pub enum Loop {
    Message(ActorMessage),
    Shutdown(oneshot::Sender<()>),
}

pub mod loopback {
    use std::time::Duration;
    use tokio::sync::oneshot;

    pub enum Loop {
        ConnectSocket {
            addr: url::Url,
            timeout: Option<Duration>,
        },
        Shutdown(oneshot::Sender<()>),
    }
}

pub mod websocket {
    use tokio::sync::oneshot;
    use tokio_tungstenite::tungstenite;

    pub enum Loop {
        Message(tungstenite::Message),
        Shutdown(oneshot::Sender<()>),
    }

    pub mod sender {
        use tokio::sync::oneshot;
        use tokio_tungstenite::tungstenite;

        pub enum Loop {
            Message(tungstenite::Message),
            Shutdown(oneshot::Sender<()>),
        }
    }

    pub mod receiver {
        use tokio::sync::oneshot;

        pub enum Loop {
            Shutdown(oneshot::Sender<()>),
        }
    }
}
