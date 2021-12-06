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

    pub async fn get_token(&mut self) -> Result<Token, AuthError> {
        match &self.token {
            None => {
                let token = self._get_token().await?;
                self.token = Some(token.clone());
                Ok(token)
            }
            Some(current_token) => {
                let token = if current_token.has_expired() {
                    let token = self._get_token().await?;
                    self.token = Some(token.clone());
                    token
                } else {
                    current_token.clone()
                };
                Ok(token)
            }
        }
    }

    async fn _get_token(&mut self) -> Result<Token, AuthError> {
        Ok(self.auth_manager.get_token(&self.scopes).await?)
    }
}
