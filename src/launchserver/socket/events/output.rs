pub mod loopback {
    pub enum Loop<R, W> {
        SocketConnected { read: R, write: W },
    }
}

pub mod websocket {
    use tokio_tungstenite::tungstenite;

    pub enum Loop {
        Message(tungstenite::Message),
        FailedToSend(tungstenite::Message, tungstenite::Error),
        Disconnect,
    }

    pub mod sender {
        use tokio_tungstenite::tungstenite;

        pub enum Loop {
            FailedToSend(tungstenite::Message, tungstenite::Error),
        }
    }

    pub mod receiver {
        use tokio_tungstenite::tungstenite;

        pub enum Loop {
            Message(tungstenite::Message),
        }
    }
}
