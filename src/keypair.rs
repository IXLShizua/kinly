use std::{
    fs,
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::Path,
};

use openssl::{pkey, rsa};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum LoadKeyPairError {
    #[snafu(display("creating directory for private key store"))]
    DirCreation {
        #[snafu(source)]
        source: io::Error,
    },

    #[snafu(display("writing new private key"))]
    Write {
        #[snafu(source)]
        source: WritePrivateKeyError,
    },

    #[snafu(display("loading existing private key"))]
    Load {
        #[snafu(source)]
        source: LoadPrivateKeyError,
    },
}

pub struct KeyPair {
    pub private: rsa::Rsa<pkey::Private>,
    pub public: String,
}

pub fn load_or_create_key_pair(dir: &Path) -> Result<KeyPair, LoadKeyPairError> {
    fs::create_dir_all(dir).context(DirCreationSnafu)?;

    let private_key_path = dir.join("private.pem");
    let private_key = match private_key_path.exists() {
        true => load_private_key(&private_key_path).context(LoadSnafu)?,
        false => write_private_key(&private_key_path).context(WriteSnafu)?,
    };

    let public_key = String::from_utf8(private_key.public_key_to_pem().unwrap()).unwrap();

    Ok(KeyPair {
        private: private_key,
        public: public_key,
    })
}

#[derive(Debug, Snafu)]
pub enum WritePrivateKeyError {
    #[snafu(display("generating private key"))]
    GeneratePrivateKey {
        #[snafu(source)]
        source: openssl::error::ErrorStack,
    },

    #[snafu(display("writing private key"))]
    WritePrivateKey {
        #[snafu(source)]
        source: io::Error,
    },
}

fn write_private_key(path: &Path) -> Result<rsa::Rsa<pkey::Private>, WritePrivateKeyError> {
    let rsa = rsa::Rsa::generate(4096).context(GeneratePrivateKeySnafu)?;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .context(WritePrivateKeySnafu)?;

    file.write_all(&rsa.private_key_to_pem().unwrap())
        .context(WritePrivateKeySnafu)?;

    Ok(rsa)
}

#[derive(Debug, Snafu)]
pub enum LoadPrivateKeyError {
    #[snafu(display("reading private key"))]
    ReadPrivateKey {
        #[snafu(source)]
        source: io::Error,
    },

    #[snafu(display("malformed private key"))]
    MalformedPrivateKey {
        #[snafu(source)]
        source: openssl::error::ErrorStack,
    },
}

fn load_private_key(path: &Path) -> Result<rsa::Rsa<pkey::Private>, LoadPrivateKeyError> {
    let key_data = fs::read(path).context(ReadPrivateKeySnafu)?;
    rsa::Rsa::private_key_from_pem(&key_data).context(MalformedPrivateKeySnafu)
}
