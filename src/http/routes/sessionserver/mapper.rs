use crate::{
    http::dto::response::{
        profile,
        profile::property::{
            textures,
            textures::kind::{cape, skin, skin::metadata},
        },
    },
    launchserver,
    launchserver::types::response::base::profile::skin::metadata::Model,
};
use openssl::{base64, hash, pkey, sign};
use std::time::{SystemTime, UNIX_EPOCH};

const TEXTURES_PROPERTY_NAME: &str = "textures";

pub fn map_player_profile(
    profile: launchserver::types::response::base::profile::Profile,
    private_key: &pkey::PKeyRef<pkey::Private>,
    signed: bool,
) -> profile::Profile {
    let skin = profile.assets.skin.map(|skin| skin::Skin {
        url: skin.url.to_string(),
        metadata: skin.metadata.and_then(|meta| match meta.model {
            Model::Default => None,
            Model::Slim => Some(metadata::Metadata {
                model: metadata::Model::Slim,
            }),
        }),
    });

    let cape = profile.assets.cape.map(|cape| cape::Cape {
        url: cape.url.to_string(),
    });

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let textures = textures::Textures {
        timestamp: now.as_millis(),
        profile_id: profile.uuid.simple().to_string(),
        profile_name: profile.username.clone(),
        signature_required: signed,
        textures: textures::kind::Kind { skin, cape },
    };

    let base64_encoded = serde_json::to_vec(&textures)
        .map(|bytes| base64::encode_block(bytes.as_slice()))
        .unwrap();

    let base64_encoded_signature = match signed {
        true => compute_textures_signature_sha1_with_rsa(private_key, &base64_encoded),
        false => None,
    };

    profile::Profile {
        id: profile.uuid.simple().to_string(),
        name: profile.username,
        properties: vec![profile::property::Property {
            name: TEXTURES_PROPERTY_NAME.to_string(),
            value: base64_encoded,
            signature: base64_encoded_signature,
        }],
    }
}

fn compute_textures_signature_sha1_with_rsa(
    private_key: &pkey::PKeyRef<pkey::Private>,
    textures: &str,
) -> Option<String> {
    let mut signer = sign::Signer::new(hash::MessageDigest::sha1(), private_key).ok()?;
    signer.update(textures.as_bytes()).ok()?;

    let signature = signer.sign_to_vec().ok()?;
    Some(base64::encode_block(signature.as_slice()))
}
