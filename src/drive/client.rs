use reqwest::{header::HeaderMap, Url};
use serde::Serialize;
use std::path::Path;

use crate::{
    auth::TokenManager,
    error::{AuthError, Error},
};

use super::handler::upload_file::upload_file;

const ENDPOINT: &str = "https://www.googleapis.com";

pub struct Client {
    token_manager: TokenManager,
    http: reqwest::Client,
}

#[derive(PartialEq, Eq, Hash)]
pub enum Scope {
    /// See, edit, create, and delete all of your Google Drive files
    Full,

    /// See, create, and delete its own configuration data in your Google Drive
    Appdata,

    /// See, edit, create, and delete only the specific Google Drive files you use with this app
    File,

    /// View and manage metadata of files in your Google Drive
    Metadata,

    /// See information about your Google Drive files
    MetadataReadonly,

    /// View the photos, videos and albums in your Google Photos
    PhotoReadonly,

    /// See and download all your Google Drive files
    Readonly,

    /// Modify your Google Apps Script scripts' behavior
    Script,
}

impl AsRef<str> for Scope {
    fn as_ref(&self) -> &str {
        match *self {
            Scope::Full => "https://www.googleapis.com/auth/drive",
            Scope::Appdata => "https://www.googleapis.com/auth/drive.appdata",
            Scope::File => "https://www.googleapis.com/auth/drive.file",
            Scope::Metadata => "https://www.googleapis.com/auth/drive.metadata",
            Scope::MetadataReadonly => "https://www.googleapis.com/auth/drive.metadata.readonly",
            Scope::PhotoReadonly => "https://www.googleapis.com/auth/drive.photos.readonly",
            Scope::Readonly => "https://www.googleapis.com/auth/drive.readonly",
            Scope::Script => "https://www.googleapis.com/auth/drive.scripts",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileMetadata {
    pub name: String,
    pub mime_type: String,
    pub parents: Vec<String>,
}

impl Client {
    pub async fn new() -> Result<Self, AuthError> {
        Ok(Self {
            token_manager: TokenManager::new(&[
                "https://www.googleapis.com/auth/cloud-platform",
                Scope::Full.as_ref(),
                Scope::File.as_ref(),
            ])
            .await?,
            http: reqwest::Client::new(),
        })
    }

    pub async fn from_credential_file<T: AsRef<Path>>(path: T) -> Result<Self, AuthError> {
        Ok(Self {
            token_manager: TokenManager::from_credential_file(
                path,
                &[
                    "https://www.googleapis.com/auth/cloud-platform",
                    Scope::Full.as_ref(),
                    Scope::File.as_ref(),
                ],
            )
            .await?,
            http: reqwest::Client::new(),
        })
    }

    pub async fn upload(
        &mut self,
        data: impl Into<Vec<u8>>,
        metadata: UploadFileMetadata,
    ) -> Result<(), Error> {
        let headers = self.headers().await?;
        let data = data.into();

        upload_file(&self.http, headers, data, metadata).await
    }

    pub(crate) fn build_uri(path: &str, params: &[(&str, &str)]) -> Result<Url, Error> {
        let mut uri = Url::parse(ENDPOINT)?.join(path)?;
        for (key, value) in params {
            uri.query_pairs_mut().append_pair(key, value);
        }
        Ok(uri)
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
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_build_uri() -> anyhow::Result<()> {
        assert_eq!(
            Client::build_uri("hoge/goo.txt", &[])?,
            Url::parse("https://www.googleapis.com/hoge/goo.txt")?
        );

        assert_eq!(
            Client::build_uri("/hoge/goo.txt", &[])?,
            Url::parse("https://www.googleapis.com/hoge/goo.txt")?
        );

        assert_eq!(
            dbg!(Client::build_uri("/hoge/goo.txt", &[("hoge", "foo")])?),
            Url::parse("https://www.googleapis.com/hoge/goo.txt?hoge=foo")?
        );

        Ok(())
    }
}
