use std::{fs, io, path::Path, time};

use snafu::{ResultExt, Snafu};
use tracing::Level;
use tracing_subscriber::{Layer, filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Snafu)]
pub enum InitLoggingError {
    #[snafu(display("parsing env variable value"))]
    InvalidEnvVariable {
        #[snafu(source)]
        source: filter::FromEnvError,
    },

    #[snafu(display("creating logs directory"))]
    DirCreation {
        #[snafu(source)]
        source: io::Error,
    },
}

pub fn load(dir: &Path) -> Result<(), InitLoggingError> {
    tracing_subscriber::registry()
        .with(
            fmt::layer().with_filter(
                filter::EnvFilter::builder()
                    .with_default_directive(Level::INFO.into())
                    .with_env_var("LOG_LEVEL")
                    .from_env()
                    .context(InvalidEnvVariableSnafu)?,
            ),
        )
        .with({
            let timestamp = time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let file = {
                if let Err(err) = fs::create_dir(dir)
                    && err.kind() != io::ErrorKind::AlreadyExists
                {
                    return Err(InitLoggingError::DirCreation { source: err });
                }

                fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(format!("logs/debug-{}.log", timestamp))
                    .unwrap()
            };

            fmt::layer()
                .with_ansi(false)
                .with_writer(file)
                .with_filter(filter::LevelFilter::from_level(Level::DEBUG))
        })
        .init();

    Ok(())
}
