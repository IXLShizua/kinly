use clap::Parser;
use figment::{providers, providers::Format};
use kinly::{args, config, config::server::meta::Assets, http, http::state, launchserver};
use openssl::{pkey, rsa};
use std::{collections::HashMap, error::Error, fs, io::Write, os::unix::fs::OpenOptionsExt};
use tokio::net;
use tracing::{info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var("LOG_LEVEL")
        .from_env()?;
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    let args = args::Args::parse();

    if !args.config_path.exists() {
        let config = config::Config::default();
        info!(
            "Config file not found. Saving default with content {:?}.",
            config
        );

        let serialized = serde_json::to_string_pretty(&config)?;
        fs::write(&args.config_path, serialized)?;

        return Ok(());
    }

    let key_pair = {
        let keys_dir = args.data_dir.join("keys");
        fs::create_dir_all(&keys_dir)?;

        let private_key_path = keys_dir.join("private.pem");

        let rsa_private = if !private_key_path.exists() {
            let rsa = rsa::Rsa::generate(4096)?;

            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true).mode(0o600);

            let mut file = options.open(&private_key_path)?;
            file.write_all(&rsa.private_key_to_pem()?)?;

            rsa
        } else {
            let key_data = fs::read(&private_key_path)?;
            rsa::Rsa::private_key_from_pem(&key_data)?
        };

        let public_key = String::from_utf8(rsa_private.public_key_to_pem()?)?;

        (rsa_private, public_key)
    };

    let config = figment::Figment::new()
        .join(providers::Json::file(&args.config_path))
        .extract::<config::Config>()?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(_main(args, config, key_pair))
}

async fn _main(
    _args: args::Args,
    config: config::Config,
    (private_key, public_key): (rsa::Rsa<pkey::Private>, String),
) -> Result<(), Box<dyn Error>> {
    let addr = std::net::SocketAddr::from((config.binds.host, config.binds.port));
    let listener = net::TcpListener::bind(addr).await?;
    info!("Proxy listening on address {}", addr);

    let servers = config.servers.into_iter().map(|server| {
        (
            server.name,
            state::Server {
                key_pair: state::KeyPair {
                    private: private_key.clone(),
                    public: public_key.clone(),
                },
                assets: match server.meta.assets {
                    Assets::AllInOne(values) => values,
                    Assets::Separated { skins, capes } => {
                        let mut values = skins
                            .into_iter()
                            .chain(capes.into_iter())
                            .collect::<Vec<_>>();
                        values.dedup();
                        values
                    }
                },
                socket: launchserver::Client::new(server.token, server.api, None),
            },
        )
    });

    let state = state::State {
        servers: HashMap::from_iter(servers),
    };

    http::init(listener, state).await?;

    Ok(())
}
