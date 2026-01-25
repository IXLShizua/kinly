use std::{fs, path::Path};

use figment::providers::{self, Format};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use tokio::io;

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

#[derive(Debug, Snafu)]
pub enum LoadConfigError {
    #[snafu(display("writing new config"))]
    Write {
        #[snafu(source)]
        source: io::Error,
    },

    #[snafu(display("extracting existing config"))]
    Extract {
        #[snafu(source)]
        source: Box<figment::Error>,
    },
}

pub enum ConfigSource {
    Created(Config),
    Loaded(Config),
}

pub fn load_or_create_config(path: &Path) -> Result<ConfigSource, LoadConfigError> {
    if !path.exists() {
        let config = Config {
            binds: Binds {
                host: "0.0.0.0"
                    .parse()
                    .expect("The correct host should be parsed"),
                port: 10000,
            },
            servers: Vec::default(),
        };

        let serialized = serde_json::to_string_pretty(&config).unwrap();
        fs::write(path, serialized).context(WriteSnafu)?;

        return Ok(ConfigSource::Created(config));
    }

    let config = figment::Figment::new()
        .join(providers::Json::file(path))
        .extract::<Config>()
        .map_err(|err| LoadConfigError::Extract {
            source: Box::new(err),
        })?;

    Ok(ConfigSource::Loaded(config))
}
