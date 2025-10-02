use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub binds: Binds,
    pub servers: Vec<server::Server>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Binds {
    pub host: std::net::Ipv4Addr,
    pub port: u16,
}

pub mod server {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Server {
        pub name: String,
        pub api: url::Url,
        pub token: String,
        pub meta: meta::Meta,
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
}

impl Default for Config {
    fn default() -> Self {
        Config {
            binds: Binds {
                host: "0.0.0.0"
                    .parse()
                    .expect("The correct host should be parsed"),
                port: 10000,
            },
            servers: Vec::default(),
        }
    }
}
