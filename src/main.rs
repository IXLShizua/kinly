use kinly::{
    args,
    config::{self, server::meta::Assets},
    http::{self, state},
    keypair,
    launchserver,
    logging,
};
use snafu::{Report, ResultExt, Snafu};
use std::{collections::HashMap, io, time};
use tokio::net;
use tracing::info;

#[derive(Debug, Snafu)]
enum ApplicationError {
    #[snafu(display("initializing app logging"))]
    InitLogging {
        #[snafu(source)]
        source: logging::InitLoggingError,
    },

    #[snafu(display("loading keypair"))]
    LoadKeyPair {
        #[snafu(source)]
        source: keypair::LoadKeyPairError,
    },

    #[snafu(display("loading config"))]
    LoadConfig {
        #[snafu(source)]
        source: config::LoadConfigError,
    },

    #[snafu(display("binding TCP listener"))]
    BindListener {
        #[snafu(source)]
        source: io::Error,
    },

    #[snafu(display("serving HTTP API"))]
    ServeHttp {
        #[snafu(source)]
        source: io::Error,
    },
}

fn main() -> Report<ApplicationError> {
    Report::capture(common_main)
}

fn common_main() -> Result<(), ApplicationError> {
    let args = args::load();

    logging::load(&args.logs_dir).context(InitLoggingSnafu)?;

    let key_pair =
        keypair::load_or_create_key_pair(&args.data_dir.join("keys")).context(LoadKeyPairSnafu)?;

    let config = match config::load_or_create_config(&args.config_path).context(LoadConfigSnafu)? {
        config::ConfigSource::Created(config) => {
            info!(
                "application config not found. created new ({:?}): {:?}",
                args.config_path.canonicalize().unwrap(),
                config
            );
            return Ok(());
        }
        config::ConfigSource::Loaded(config) => config,
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main(args, config, key_pair))
}

async fn async_main(
    _args: args::Args,
    config: config::Config,
    keypair: keypair::KeyPair,
) -> Result<(), ApplicationError> {
    let addr = std::net::SocketAddr::from((config.binds.host, config.binds.port));
    let listener = net::TcpListener::bind(addr)
        .await
        .context(BindListenerSnafu)?;
    info!("proxy listening on address {}", addr);

    let servers = config
        .servers
        .into_iter()
        .map(|server| {
            (
                server.name,
                state::Server {
                    key_pair: state::KeyPair {
                        private: keypair.private.clone(),
                        public: keypair.public.clone(),
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
        })
        .collect::<HashMap<_, _>>();

    let state = state::State { servers };
    http::init(listener, state).await.context(ServeHttpSnafu)?;

    Ok(())
}
