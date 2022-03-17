use std::path::Path;

use gcp_auth::{AuthenticationManager, Token};

use crate::error::AuthError;

pub struct TokenManager {
    auth_manager: AuthenticationManager,
    token: Option<Token>,
    scopes: Vec<&'static str>,
}

impl TokenManager {
    pub async fn new(scopes: &[&'static str]) -> Result<Self, AuthError> {
        Ok(Self {
            auth_manager: gcp_auth::init().await?,
            token: None,
            scopes: scopes.to_owned(),
        })
    }

    pub async fn from_credential_file<T: AsRef<Path>>(
        path: T,
        scopes: &[&'static str],
    ) -> Result<Self, AuthError> {
        Ok(Self {
            auth_manager: gcp_auth::from_credentials_file(path).await?,
            token: None,
            scopes: scopes.to_owned(),
        })
    }

    pub async fn get_token(&mut self) -> Result<&Token, AuthError> {
        match &self.token {
            None => {
                let token = self._get_token().await?;
                self.token = Some(token);
            }
            Some(current_token) => {
                if current_token.has_expired() {
                    let token = self._get_token().await?;
                    self.token = Some(token);
                }
            }
        }

        Ok(self.token.as_ref().unwrap())
    }

    async fn _get_token(&mut self) -> Result<Token, AuthError> {
        Ok(self.auth_manager.get_token(&self.scopes).await?)
    }
}
