use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub api: Api,
    pub meta: Meta,
    pub servers: HashMap<String, server::Server>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Api {
    pub host: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meta {
    pub public: url::Url,
}

pub mod server {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Server {
        pub api: url::Url,
        pub token: String,
        pub meta: meta::Meta,
        pub experimental: Option<experimental::Experimental>,
    }

    pub mod meta {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Clone, Debug)]
        pub struct Meta {
            pub assets: Assets,
        }

        #[derive(Serialize, Deserialize, Clone, Debug)]
        #[serde(untagged)]
        pub enum Assets {
            AllInOne(Vec<String>),
            Separated {
                skins: Vec<String>,
                capes: Vec<String>,
            },
        }
    }

    pub mod experimental {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Clone, Debug)]
        pub struct Experimental {
            pub rewrite: Option<rewrite::Rewrite>,
        }

        pub mod rewrite {
            use serde::{Deserialize, Serialize};

            #[derive(Serialize, Deserialize, Clone, Debug)]
            #[serde(untagged)]
            pub enum Rewrite {
                AllInOne(bool),
                Separated { skins: bool, capes: bool },
            }
        }
    }
}

pub fn default() -> Config {
    Config {
        api: Api {
            host: "0.0.0.0".to_string(),
            port: 10000,
        },
        meta: Meta {
            public: url::Url::from_str("http://0.0.0.0:10000")
                .expect("The correct url should be parsed"),
        },
        servers: HashMap::new(),
    }
}
