use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub api: Api,
    pub servers: HashMap<String, server::Server>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Api {
    pub host: String,
    pub port: u16,
}

pub mod server {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Server {
        pub api: url::Url,
        pub token: String,
        pub meta: meta::Meta,
    }

    pub mod meta {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Meta {
            pub assets: Assets,
        }

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(untagged)]
        pub enum Assets {
            AllInOne(Vec<String>),
            Separated {
                skins: Vec<String>,
                capes: Vec<String>,
            },
        }
    }
}

pub fn default() -> Config {
    Config {
        api: Api {
            host: "0.0.0.0".to_string(),
            port: 10000,
        },
        servers: HashMap::new(),
    }
}
