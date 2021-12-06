use reqwest::{header::HeaderMap, StatusCode, Url};

use crate::{auth::TokenManager, error::AuthError};

const ENDPOINT: &'static str = "https://storage.googleapis.com/storage/v1";
const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/devstorage.full_control",
];

#[derive(thiserror::Error, Debug)]
pub enum CloudStorageError {
    #[error("Status: {status:?} Res: {response}")]
    ErrorResponse {
        status: StatusCode,
        response: String,
    },
}

pub struct Client {
    token_manager: TokenManager,
    http: reqwest::Client,
}

impl Client {
    pub async fn new() -> Result<Self, AuthError> {
        Ok(Self {
            token_manager: TokenManager::new(&SCOPES).await?,
            http: reqwest::Client::new(),
        })
    }

    pub async fn object(
        &mut self,
        bucket: &str,
        object: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let url = Self::build_uri(bucket, Some(object))?;
        let res = self
            .http
            .get(url)
            .headers(self.headers().await?)
            .query(&[("alt", "media")])
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.bytes().await?.to_vec())
        } else {
            Err(CloudStorageError::ErrorResponse {
                status: res.status(),
                response: res.text().await?,
            }
            .into())
        }
    }

    fn build_uri<T: AsRef<str>>(bucket: &str, object: Option<T>) -> Result<Url, url::ParseError> {
        let mut url = Url::parse(ENDPOINT)?;
        url.path_segments_mut().unwrap().push("b").push(bucket);

        if let Some(object) = object {
            url.path_segments_mut()
                .unwrap()
                .push("o")
                .push(object.as_ref());
        }

        Ok(url)
    }

    async fn headers(&mut self) -> Result<HeaderMap, AuthError> {
        let mut header = HeaderMap::new();
        header.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.token_manager.get_token().await?.as_str())
                .parse()
                .unwrap(),
        );
        Ok(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case(
        "test-bucket",
        Some("hoge.yaml"),
        "https://storage.googleapis.com/storage/v1/b/test-bucket/o/hoge.yaml"
    )]
    #[case(
        "test-bucket",
        None,
        "https://storage.googleapis.com/storage/v1/b/test-bucket"
    )]
    #[test]
    fn test_build_uri(
        #[case] bucket: &str,
        #[case] object: Option<&str>,
        #[case] expected: &str,
    ) -> anyhow::Result<()> {
        assert_eq!(
            Client::build_uri(bucket, object)?,
            Url::parse(expected).unwrap()
        );
        Ok(())
    }
}
