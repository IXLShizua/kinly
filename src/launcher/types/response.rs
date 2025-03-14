use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct Response<T: Serialize + DeserializeOwned> {
    #[serde(rename = "requestUUID")]
    pub id: Uuid,

    #[serde(flatten)]
    pub body: T,
}

pub mod any {
    use crate::launcher::types::response::{
        batch_profiles_by_usernames, check_server, error, get_profile_by_username,
        get_profile_by_uuid, restore_token, Response,
    };
    use serde::{Deserialize, Serialize};

    pub type Any = Response<Kind>;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "type")]
    pub enum Kind {
        #[serde(rename = "restore")]
        RestoreToken(restore_token::RestoreToken),

        #[serde(rename = "checkServer")]
        CheckServer(check_server::CheckServer),

        #[serde(rename = "profileByUUID")]
        GetProfileByUuid(get_profile_by_uuid::GetProfileByUuid),

        #[serde(rename = "profileByUsername")]
        GetProfileByUsername(get_profile_by_username::GetProfileByUsername),

        #[serde(rename = "batchProfileByUsername")]
        BatchProfilesByUsernames(batch_profiles_by_usernames::BatchProfilesByUsernames),

        #[serde(rename = "error")]
        Error(error::Error),
    }
}

pub mod error {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Error {
        #[serde(rename = "error")]
        pub kind: Kind,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Kind {
        #[serde(rename = "User not found")]
        UserNotFound,

        #[serde(rename = "User not verified")]
        UserNotVerified,

        #[serde(rename = "Permissions denied")]
        PermissionsDenied,

        #[serde(untagged)]
        Other(String),
    }
}

pub mod restore_token {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RestoreToken {
        #[serde(rename = "invalidTokens")]
        pub invalid_tokens: Vec<String>,
    }
}

pub mod check_server {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CheckServer {
        pub uuid: Uuid,

        #[serde(rename = "playerProfile")]
        pub profile: Profile,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Profile {
        pub uuid: Uuid,
        pub username: String,
    }
}

pub mod get_profile_by_uuid {
    use crate::launcher::types::response::base;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GetProfileByUuid {
        #[serde(rename = "playerProfile")]
        pub player_profile: base::profile::Profile,
    }
}

pub mod get_profile_by_username {
    use crate::launcher::types::response::base;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GetProfileByUsername {
        #[serde(rename = "playerProfile")]
        pub player_profile: base::profile::Profile,
    }
}

pub mod base {
    pub mod profile {
        use serde::{Deserialize, Serialize};
        use uuid::Uuid;

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Profile {
            pub uuid: Uuid,
            pub username: String,
            pub assets: Assets,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Assets {
            #[serde(rename = "SKIN", skip_serializing_if = "Option::is_none")]
            pub skin: Option<skin::Skin>,

            #[serde(rename = "CAPE", skip_serializing_if = "Option::is_none")]
            pub cape: Option<cape::Cape>,
        }

        pub mod skin {
            use serde::{Deserialize, Serialize};

            #[derive(Serialize, Deserialize, Debug)]
            pub struct Skin {
                pub url: url::Url,
                pub digest: String,

                #[serde(skip_serializing_if = "Option::is_none")]
                pub metadata: Option<metadata::Metadata>,
            }

            pub mod metadata {
                use serde::{Deserialize, Serialize};

                #[derive(Serialize, Deserialize, Debug)]
                pub struct Metadata {
                    #[serde(default)]
                    pub model: Model,
                }

                #[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq)]
                pub enum Model {
                    #[serde(rename = "slim")]
                    Slim,

                    #[serde(other)]
                    #[default]
                    Default,
                }
            }
        }

        pub mod cape {
            use serde::{Deserialize, Serialize};

            #[derive(Serialize, Deserialize, Debug)]
            pub struct Cape {
                pub url: url::Url,
                pub digest: String,
            }
        }
    }
}

pub mod batch_profiles_by_usernames {
    use crate::launcher::types::response::base::profile;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct BatchProfilesByUsernames {
        #[serde(rename = "playerProfiles")]
        pub player_profiles: Vec<Option<profile::Profile>>,
    }
}
