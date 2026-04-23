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
    let filter = filter::EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var("LOG_LEVEL")
        .from_env()
        .context(InvalidEnvVariableSnafu)?;

    let stdout = fmt::layer().with_filter(filter.clone());

    let file = {
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
                .open(dir.join(format!("{}.log", timestamp)))
                .unwrap()
        };

        fmt::layer()
            .with_ansi(false)
            .with_writer(file)
            .with_filter(filter)
    };

    // Keep the file layer before the stdout layer.
    // With multiple `fmt` layers, ANSI formatting from an ANSI-enabled stdout layer can leak
    // into span fields written by the file layer if the layers are reordered.
    // The file layer must be registered first so plain log files stay free of ANSI escape codes.
    tracing_subscriber::registry()
        .with(file)
        .with(stdout)
        .init();

    Ok(())
}
