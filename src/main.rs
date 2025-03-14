use auth_proxy_gl::{args, config, config::Config as AppConfig, routes, state, state::Sockets};
use clap::Parser;
use figment::{providers, providers::Format};
use futures::StreamExt;
use openssl::{rsa, rsa::Rsa};
use std::{error::Error, fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::{net, runtime, signal};
use tracing::{debug, info, span, Level};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

fn main() -> Result<(), Box<dyn Error>> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("LOGGING_LEVEL")
        .from_env()?;
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    let args = args::Args::parse();

    if !fs::exists(&args.config_path)? {
        info!("Config file not found. Saving default.");

        let default_config = serde_json::to_string_pretty(&config::default())?;

        fs::write(&args.config_path, default_config)?;

        return Ok(());
    }

    let key_pair = {
        let span = span!(Level::DEBUG, "keys");
        let span = span.entered();

        let keys_dir = args.data_dir.join("keys");
        debug!("Keys directory:: {:?}", keys_dir.canonicalize()?);
        if !fs::exists(&keys_dir)? {
            debug!("Key directory not found. Creation");
            fs::create_dir_all(&keys_dir)?;
        }

        let private_key_path = keys_dir.join("private.pem");
        debug!(
            "Path to private key: {:?}",
            private_key_path.canonicalize()?
        );
        let rsa_private = if !fs::exists(&private_key_path)? {
            debug!("Private key not found. Generation");
            let rsa = rsa::Rsa::generate(4096)?;

            let _ = fs::remove_file(&private_key_path);
            fs::write(&private_key_path, rsa.private_key_to_pem()?)?;

            rsa
        } else {
            debug!("Private key found. Trying to read");
            let bytes = fs::read(&private_key_path)?;
            Rsa::private_key_from_pem(&bytes)?
        };
        let public_key = String::from_utf8(rsa_private.public_key_to_pem()?)?;

        let _ = span.exit();

        state::KeyPair {
            rsa: rsa_private,
            public: public_key,
        }
    };

    let config = figment::Figment::new()
        .join(providers::Json::file(&args.config_path))
        .extract::<AppConfig>()?;

    let rt = runtime::Builder::new_multi_thread().enable_all().build()?;

    rt.block_on(_main(config, key_pair))
}

async fn _main(config: AppConfig, key_pair: state::KeyPair) -> Result<(), Box<dyn Error>> {
    let sockets = Arc::new(Sockets::from_servers(&config.servers).await);
    let router = axum::Router::new()
        .nest(
            "/:server_id",
            axum::Router::new()
                .merge(routes::root::router())
                .nest("/api", routes::api::router())
                .nest("/sessionserver", routes::sessionserver::router()),
        )
        .with_state(state::State {
            servers: Arc::new(config.servers),
            key_pair: Arc::new(key_pair),
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
