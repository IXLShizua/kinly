use clap::Parser;
use figment::{providers, providers::Format};
use kinly::{args, config, config::server::meta::Assets, http, http::state, launchserver};
use openssl::{pkey, rsa};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io,
    io::Write,
    os::unix::fs::OpenOptionsExt,
    time,
};
use tokio::net;
use tracing::{Level, info};
use tracing_subscriber::{Layer, filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(
            fmt::layer().with_filter(
                filter::EnvFilter::builder()
                    .with_default_directive(Level::INFO.into())
                    .with_env_var("LOG_LEVEL")
                    .from_env()?,
            ),
        )
        .with({
            let timestamp = time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs();

            let file = {
                if let Err(err) = fs::create_dir("logs")
                    && err.kind() != io::ErrorKind::AlreadyExists
                {
                    return Err(err.into());
                }

                fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(format!("logs/debug-{}.log", timestamp))?
            };

            fmt::layer()
                .with_ansi(false)
                .with_writer(file)
                .with_filter(filter::LevelFilter::from_level(Level::DEBUG))
        })
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
                    Assets::Separated { mut skins, capes } => {
                        skins.extend(capes);
                        skins
                    }
                },
                socket: launchserver::Client::new(
                    server.token,
                    server.api,
                    time::Duration::from_secs(5),
                ),
            },
        )
    });

    let state = state::State {
        servers: HashMap::from_iter(servers),
    };

    http::init(listener, state).await?;

    Ok(())
}
