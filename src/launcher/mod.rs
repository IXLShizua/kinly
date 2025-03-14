pub mod error;
pub mod socket;
pub mod types;

use crate::{
    config,
    launcher::types::{request, response},
};
use std::{collections::HashMap, future::Future, time::Duration};
use uuid::Uuid;

macro_rules! extract_response {
    ($response:expr, $kind:path) => {
        if let $kind(value) = $response {
            Ok(value)
        } else {
            Err(error::Error::UnexpectedResponse($response))
        }
    };
}

pub struct Api {
    socket: socket::Socket,
}

impl Api {
    pub fn new(addr: impl Into<url::Url>, timeout: impl Into<Option<Duration>>) -> Api {
        Api {
            socket: socket::Socket::new(addr, timeout),
        }
    }

    pub async fn restore_token(
        &self,
        pair: request::restore_token::Pair,
        user_info: bool,
    ) -> Result<response::restore_token::RestoreToken, error::Error> {
        let response = self
            .socket
            .send_request(request::Request {
                id: Uuid::new_v4(),
                body: request::any::Kind::RestoreToken(request::restore_token::RestoreToken {
                    extended: HashMap::from([(pair.name, pair.value)]),
                    need_user_info: user_info,
                }),
            })
            .await?;

        extract_response!(response, response::any::Kind::RestoreToken)
    }

    pub async fn check_server(
        &self,
        username: impl Into<String>,
        server_id: impl Into<String>,
        need_hardware: bool,
        need_properties: bool,
    ) -> Result<response::check_server::CheckServer, error::Error> {
        let response = self
            .socket
            .send_request(request::Request {
                id: Uuid::new_v4(),
                body: request::any::Kind::CheckServer(request::check_server::CheckServer {
                    username: username.into(),
                    server_id: server_id.into(),
                    need_hardware,
                    need_properties,
                }),
            })
            .await?;

        extract_response!(response, response::any::Kind::CheckServer)
    }

    pub async fn get_profile_by_uuid(
        &self,
        uuid: Uuid,
    ) -> Result<response::get_profile_by_uuid::GetProfileByUuid, error::Error> {
        let response = self
            .socket
            .send_request(request::Request {
                id: Uuid::new_v4(),
                body: request::any::Kind::GetProfileByUuid(
                    request::get_profile_by_uuid::GetProfileByUuid { uuid },
                ),
            })
            .await?;

        extract_response!(response, response::any::Kind::GetProfileByUuid)
    }

    pub async fn get_profile_by_username(
        &self,
        username: impl Into<String>,
    ) -> Result<response::get_profile_by_username::GetProfileByUsername, error::Error> {
        let response = self
            .socket
            .send_request(request::Request {
                id: Uuid::new_v4(),
                body: request::any::Kind::GetProfileByUsername(
                    request::get_profile_by_username::GetProfileByUsername {
                        username: username.into(),
                    },
                ),
            })
            .await?;

        extract_response!(response, response::any::Kind::GetProfileByUsername)
    }

    pub async fn batch_profiles_by_usernames(
        &self,
        usernames: Vec<impl Into<String>>,
    ) -> Result<response::batch_profiles_by_usernames::BatchProfilesByUsernames, error::Error> {
        let response = self
            .socket
            .send_request(request::Request {
                id: Uuid::new_v4(),
                body: request::any::Kind::BatchProfilesByUsernames(
                    request::batch_profiles_by_usernames::BatchProfilesByUsernames {
                        list: usernames
                            .into_iter()
                            .map(|username| request::batch_profiles_by_usernames::Entry {
                                username: username.into(),
                            })
                            .collect::<Vec<_>>(),
                    },
                ),
            })
            .await?;

        extract_response!(response, response::any::Kind::BatchProfilesByUsernames)
    }

    pub async fn with_token_restore<T, F, Fut>(
        &self,
        server: &config::server::Server,
        action: F,
    ) -> Result<T, error::Error>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, error::Error>>,
    {
        match action().await {
            Ok(result) => Ok(result),
            Err(err) => match err {
                error::Error::UnexpectedResponse(response::any::Kind::Error(
                    response::error::Error {
                        kind: response::error::Kind::PermissionsDenied,
                    },
                )) => {
                    let res = self
                        .restore_token(
                            request::restore_token::Pair {
                                name: "checkServer".to_string(),
                                value: server.token.clone(),
                            },
                            false,
                        )
                        .await;

                    if res.map(|v| v.invalid_tokens.is_empty()).unwrap_or(false) {
                        action().await
                    } else {
                        Err(err)
                    }
                }
                _ => Err(err),
            },
        }
    }

    pub async fn shutdown(&self) {
        self.socket.shutdown().await;
    }
}
