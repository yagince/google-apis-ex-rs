use gcp_auth::Token;
use reqwest::{header::HeaderMap, Url};

const ENDPOINT: &'static str = "https://storage.googleapis.com/storage/v1";
const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/devstorage.full_control",
];

#[derive(thiserror::Error, Debug)]
pub enum CloudStorageError {
    #[error("{0:?}")]
    ErrorResponse(String),
}

pub struct Client {
    token: Token,
    http: reqwest::Client,
}

impl Client {
    pub async fn new() -> Result<Self, gcp_auth::Error> {
        let auth = gcp_auth::init().await?;
        Ok(Self {
            token: auth.get_token(&SCOPES).await?,
            http: reqwest::Client::new(),
        })
    }

    pub async fn object(
        &self,
        bucket: &str,
        object: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let url = Self::build_uri(bucket, Some(object))?;
        let res = self
            .http
            .get(url)
            .headers(self.headers())
            .query(&[("alt", "media")])
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();
        Ok(res)
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

    fn headers(&self) -> HeaderMap {
        let mut header = HeaderMap::new();
        header.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.token.as_str()).parse().unwrap(),
        );
        header
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
