use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct Request<T: Serialize + DeserializeOwned> {
    #[serde(rename = "requestUUID")]
    pub id: Uuid,

    #[serde(flatten)]
    pub body: T,
}

pub mod any {
    use crate::launchserver::types::request::{
        Request,
        batch_profiles_by_usernames,
        check_server,
        get_profile_by_username,
        get_profile_by_uuid,
        restore_token,
    };
    use serde::{Deserialize, Serialize};

    pub type Any = Request<Kind>;

    #[derive(Serialize, Deserialize, Debug, Clone)]
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
    }
}

pub mod restore_token {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RestoreToken {
        pub extended: HashMap<String, String>,

        #[serde(rename = "needUserInfo")]
        pub need_user_info: bool,
    }

    pub struct Pair {
        pub name: String,
        pub value: String,
    }
}

pub mod check_server {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct CheckServer {
        pub username: String,

        #[serde(rename = "serverID")]
        pub server_id: String,

        #[serde(rename = "needHardware")]
        pub need_hardware: bool,

        #[serde(rename = "needProperties")]
        pub need_properties: bool,
    }
}

pub mod get_profile_by_uuid {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GetProfileByUuid {
        pub uuid: Uuid,
    }
}

pub mod get_profile_by_username {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GetProfileByUsername {
        pub username: String,
    }
}

pub mod batch_profiles_by_usernames {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct BatchProfilesByUsernames {
        pub list: Vec<Entry>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Entry {
        pub username: String,
    }
}
