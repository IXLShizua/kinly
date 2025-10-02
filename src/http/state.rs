use crate::launchserver;
use openssl::{pkey, rsa};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct ClonableState(Arc<State>);

impl ClonableState {
    pub fn new(state: State) -> ClonableState {
        ClonableState(Arc::new(state))
    }

    pub fn servers(&self) -> &HashMap<String, Server> {
        &self.0.servers
    }
}

pub struct State {
    pub servers: HashMap<String, Server>,
}

pub struct Server {
    pub key_pair: KeyPair,
    pub assets: Vec<String>,
    pub socket: launchserver::Client,
}

pub struct KeyPair {
    pub private: rsa::Rsa<pkey::Private>,
    pub public: String,
}
