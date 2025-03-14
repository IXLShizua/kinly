use auth_proxy_gl::{args, config, config::Config as AppConfig, routes, state, state::Sockets};
use clap::Parser;
use figment::{providers, providers::Format};
use futures::StreamExt;
use openssl::rsa;
use std::{
    error::Error,
    fs,
    io::Write,
    net::SocketAddr,
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    sync::Arc,
};
use tokio::{net, signal};
use tracing::{debug, info, span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    // Настраиваем логирование с фильтрацией уровня через переменную окружения.
    let env_filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .with_env_var("LOGGING_LEVEL")
        .from_env()?;
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    let args = args::Args::parse();

    if !args.config_path.exists() {
        info!("Config file not found. Saving default.");

        let default_config = serde_json::to_string_pretty(&config::default())?;
        fs::write(&args.config_path, default_config)?;
        return Ok(());
    }

    let key_pair = {
        let span = span!(Level::DEBUG, "keys");
        let _enter = span.enter();

        let keys_dir = args.data_dir.join("keys");
        debug!(
            "Keys directory:: {:?}",
            keys_dir.canonicalize().unwrap_or_else(|_| keys_dir.clone())
        );

        fs::create_dir_all(&keys_dir)?;

        let private_key_path = keys_dir.join("private.pem");
        debug!(
            "Path to private key: {:?}",
            private_key_path
                .canonicalize()
                .unwrap_or_else(|_| private_key_path.clone())
        );

        let rsa_private = if !private_key_path.exists() {
            debug!("Private key not found. Generation.");

            let rsa = rsa::Rsa::generate(4096)?;

            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true).mode(0o600);

            let mut file = options.open(&private_key_path)?;
            file.write_all(&rsa.private_key_to_pem()?)?;

            rsa
        } else {
            debug!("Private key found. Trying to read.");

            let key_data = fs::read(&private_key_path)?;
            rsa::Rsa::private_key_from_pem(&key_data)?
        };

        let public_key = String::from_utf8(rsa_private.public_key_to_pem()?)?;

        state::KeyPair {
            rsa: rsa_private,
            public: public_key,
        }
    };

    let config = figment::Figment::new()
        .join(providers::Json::file(&args.config_path))
        .extract::<AppConfig>()?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(_main(config, args.data_dir, key_pair))
}

async fn _main(
    config: AppConfig,
    data_dir: PathBuf,
    key_pair: state::KeyPair,
) -> Result<(), Box<dyn Error>> {
    let sockets = Arc::new(Sockets::from_servers(&config.servers).await);
    let router = axum::Router::new()
        .nest(
            "/:server_id",
            axum::Router::new()
                .merge(routes::root::router())
                .nest("/api", routes::api::router())
                .nest("/sessionserver", routes::sessionserver::router())
                .nest("/assets", routes::assets::router()),
        )
        .with_state(state::State {
            config: Arc::new(config.clone()),
            key_pair: Arc::new(key_pair),
            data_dir,
            servers: Arc::new(config.servers),
            sockets: sockets.clone(),
        });

    let addr = format!("{}:{}", config.api.host, config.api.port).parse::<SocketAddr>()?;
    let listener = net::TcpListener::bind(addr).await?;
    info!("Proxy listening on address {}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            signal::ctrl_c().await.unwrap();

            futures::stream::iter(sockets.inner())
                .for_each(|socket| socket.shutdown())
                .await;

            debug!("Ctrl^C signal received. Quitting.");
        })
        .await?;

    Ok(())
}
