pub mod join {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Serialize, Deserialize)]
    pub struct Body {
        #[serde(rename = "accessToken")]
        pub access_token: String,

        #[serde(rename = "selectedProfile")]
        pub selected_profile: Uuid,

        #[serde(rename = "serverId")]
        pub server_id: String,
    }
}

pub mod has_joined {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Query {
        pub username: String,

        #[serde(rename = "serverId")]
        pub server_id: String,
    }
}

pub mod profile_by_uuid {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Query {
        pub unsigned: bool,
    }
}

pub mod profiles_by_usernames {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Body(pub Vec<String>);
}
