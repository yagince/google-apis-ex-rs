use gcp_auth::Token;

use crate::error::AuthError;

pub async fn get_token(scopes: &[&str]) -> Result<Token, AuthError> {
    Ok(gcp_auth::init().await?.get_token(scopes).await?)
}
