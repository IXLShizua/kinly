use crate::launchserver;
use openssl::{pkey, rsa};
use std::{collections::HashMap, sync::Arc};

pub type ClonableState = Arc<State>;

pub struct State {
    pub servers: HashMap<String, Arc<Server>>,
}

pub struct Server {
    pub key_pair: ServerKeyPair,
    pub assets: Vec<String>,
    pub client: launchserver::Client,
}

pub struct ServerKeyPair {
    pub private: rsa::Rsa<pkey::Private>,
    pub public: String,
}
