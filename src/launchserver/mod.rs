pub mod error;
pub mod socket;
pub mod types;

use crate::launchserver::types::{request, response};
use futures_util::TryFutureExt;
use std::{collections::HashMap, time::Duration};
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

pub struct Client {
    token: String,
    timeout: Duration,
    socket: socket::Socket,
}

impl Client {
    pub fn new(
        token: impl Into<String>,
        addr: impl Into<url::Url>,
        timeout: impl Into<Option<Duration>>,
    ) -> Client {
        let timeout = timeout.into();
        let options = socket::SocketOptions::builder()
            .with_timeout(timeout)
            .with_reconnection_timeout(timeout)
            .build();

        Client {
            token: token.into(),
            timeout: options.timeout,
            socket: socket::Socket::new(addr, options),
        }
    }

    pub async fn check_server(
        &self,
        username: impl Into<String>,
        server_id: impl Into<String>,
        need_hardware: bool,
        need_properties: bool,
    ) -> Result<response::check_server::CheckServer, error::Error> {
        let username = username.into();
        let server_id = server_id.into();

        let response = self
            .send_safely_request(request::Request {
                id: Uuid::new_v4(),
                body: request::any::Kind::CheckServer(request::check_server::CheckServer {
                    username: username.clone(),
                    server_id: server_id.clone(),
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
            .send_safely_request(request::Request {
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
        let username = username.into();

        let response = self
            .send_safely_request(request::Request {
                id: Uuid::new_v4(),
                body: request::any::Kind::GetProfileByUsername(
                    request::get_profile_by_username::GetProfileByUsername {
                        username: username.clone(),
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
        let usernames = usernames
            .into_iter()
            .map(|username| request::batch_profiles_by_usernames::Entry {
                username: username.into(),
            })
            .collect::<Vec<_>>();

        let response = self
            .send_safely_request(request::Request {
                id: Uuid::new_v4(),
                body: request::any::Kind::BatchProfilesByUsernames(
                    request::batch_profiles_by_usernames::BatchProfilesByUsernames {
                        list: usernames,
                    },
                ),
            })
            .await?;

        extract_response!(response, response::any::Kind::BatchProfilesByUsernames)
    }

    async fn send_safely_request(
        &self,
        request: request::any::Any,
    ) -> Result<response::any::Kind, error::Error> {
        let response = self
            .socket
            .send_request(request.clone(), self.timeout)
            .await;

        match response {
            Ok(
                kind @ response::any::Kind::Error(response::error::Error {
                    kind: response::error::Kind::PermissionsDenied,
                }),
            ) => {
                let res = self
                    .restore_token(
                        request::restore_token::Pair {
                            name: "checkServer".to_string(),
                            value: self.token.clone(),
                        },
                        false,
                    )
                    .await;

                if res.map(|v| v.invalid_tokens.is_empty()).unwrap_or(false) {
                    self.socket
                        .send_request(request, self.timeout)
                        .map_err(|err| err.into())
                        .await
                } else {
                    Err(error::Error::UnexpectedResponse(kind))
                }
            }
            Ok(result) => Ok(result),
            other => other.map_err(|err| err.into()),
        }
    }

    async fn restore_token(
        &self,
        pair: request::restore_token::Pair,
        user_info: bool,
    ) -> Result<response::restore_token::RestoreToken, error::Error> {
        let response = self
            .socket
            .send_request(
                request::Request {
                    id: Uuid::new_v4(),
                    body: request::any::Kind::RestoreToken(request::restore_token::RestoreToken {
                        extended: HashMap::from([(pair.name, pair.value)]),
                        need_user_info: user_info,
                    }),
                },
                self.timeout,
            )
            .await?;

        extract_response!(response, response::any::Kind::RestoreToken)
    }

    pub async fn shutdown(&self) {
        self.socket.shutdown().await;
    }
}
