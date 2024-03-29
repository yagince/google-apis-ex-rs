use reqwest::{header::HeaderMap, StatusCode, Url};

use crate::{
    auth::TokenManager,
    error::{AuthError, Error},
};

use super::object::ObjectResource;

const ENDPOINT: &str = "https://storage.googleapis.com/storage/v1";
const ENDPOINT_UPLOAD: &str = "https://storage.googleapis.com/upload/storage/v1";
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

    pub async fn object(&mut self, bucket: &str, object: &str) -> Result<Vec<u8>, Error> {
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

    pub async fn create_object(
        &mut self,
        bucket: &str,
        name: &str,
        object: impl Into<Vec<u8>>,
        mime_type: impl AsRef<str>,
    ) -> Result<ObjectResource, Error> {
        let url = Self::build_upload_uri(bucket, Some(""))?;
        let data = object.into();

        let res = self
            .http
            .post(url)
            .query(&[
                // cf. https://cloud.google.com/storage/docs/json_api/v1/objects/insert#parameters
                ("uploadType", "media"),
                ("name", name),
            ])
            .headers(self.headers().await?)
            .header("content-type", mime_type.as_ref())
            .header("content-length", data.len())
            .body(data)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.json().await?)
        } else {
            Err(CloudStorageError::ErrorResponse {
                status: res.status(),
                response: res.text().await?,
            }
            .into())
        }
    }

    fn build_uri<T: AsRef<str>>(bucket: &str, object: Option<T>) -> Result<Url, url::ParseError> {
        Self::build_uri_by_endpoint(ENDPOINT, bucket, object)
    }

    fn build_upload_uri<T: AsRef<str>>(
        bucket: &str,
        object: Option<T>,
    ) -> Result<Url, url::ParseError> {
        Self::build_uri_by_endpoint(ENDPOINT_UPLOAD, bucket, object)
    }

    fn build_uri_by_endpoint<T: AsRef<str>>(
        endpoint: &str,
        bucket: &str,
        object: Option<T>,
    ) -> Result<Url, url::ParseError> {
        let mut url = Url::parse(endpoint)?;
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
        Some("hoge/foo/bar.yaml"),
        "https://storage.googleapis.com/storage/v1/b/test-bucket/o/hoge%2Ffoo%2Fbar.yaml"
    )]
    #[case(
        "test-bucket",
        Some(""),
        "https://storage.googleapis.com/storage/v1/b/test-bucket/o/"
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
