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
use openssl::{base64, pkey, rsa, rsa::Padding};
use std::time;

pub fn map_profile(
    profile: launchserver::types::response::base::profile::Profile,
    rsa: &rsa::Rsa<pkey::Private>,
    now: time::Duration,
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

    let textures = textures::Textures {
        timestamp: now.as_millis(),
        profile_id: profile.uuid.simple().to_string(),
        profile_name: profile.username.clone(),
        signature_required: signed,
        textures: textures::kind::Kind { skin, cape },
    };
    let serialized_textures = serde_json::to_string(&textures).unwrap();

    let encoded = base64::encode_block(serialized_textures.as_bytes());
    let encoded_signature = if signed {
        let mut buf = vec![0u8; rsa.size() as usize];

        let len = rsa
            .private_encrypt(encoded.as_bytes(), &mut buf, Padding::PKCS1)
            .unwrap();

        Some(base64::encode_block(&buf[..len]))
    } else {
        None
    };

    profile::Profile {
        id: profile.uuid.simple().to_string(),
        name: profile.username,
        properties: vec![profile::property::Property {
            name: "textures".to_string(),
            value: encoded,
            signature: encoded_signature,
        }],
    }
}
