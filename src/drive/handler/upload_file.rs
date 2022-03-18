use reqwest::{header::HeaderMap, StatusCode, Url};

use crate::{
    drive::{
        client::{File, GoogleDriveError},
        Client,
    },
    error::Error,
    mime,
};

const UPLOAD_PATH: &str = "upload/drive/v3/files";
const HEADER_UPLOAD_CONTENT_TYPE: &str = "X-Upload-Content-Type";
const HEADER_UPLOAD_CONTENT_LENGTH: &str = "X-Upload-Content-Length";

pub async fn upload_file(
    http: &reqwest::Client,
    headers: HeaderMap,
    data: impl Into<Vec<u8>>,
    metadata: File,
) -> Result<File, Error> {
    let data = data.into();
    let url = resume_url(http, &data, headers.clone(), &metadata).await?;
    upload_request(http, url, data, headers, &metadata).await
}

async fn upload_request(
    http: &reqwest::Client,
    url: Url,
    data: Vec<u8>,
    headers: HeaderMap,
    metadata: &File,
) -> Result<File, Error> {
    Ok(http
        .put(url)
        .headers(headers)
        .header(reqwest::header::CONTENT_TYPE, &metadata.mime_type)
        .header(reqwest::header::CONTENT_LENGTH, data.len())
        .body(data)
        .send()
        .await?
        .json()
        .await?)
}

async fn resume_url(
    http: &reqwest::Client,
    data: &[u8],
    headers: HeaderMap,
    metadata: &File,
) -> Result<Url, Error> {
    let res = resume_request(http, data, headers, metadata).await?;
    match res.status() {
        StatusCode::OK => {
            if let Some(x) = res.headers().get(reqwest::header::LOCATION) {
                return Ok(x.to_str()?.parse()?);
            }
            return Err(GoogleDriveError::ResumeUrlNotFound {
                response: res.text().await?,
            }
            .into());
        }
        _ => Err(GoogleDriveError::UnexpectedResponse {
            status: res.status(),
            response: res.text().await?,
        }
        .into()),
    }
}

async fn resume_request(
    http: &reqwest::Client,
    data: &[u8],
    headers: HeaderMap,
    metadata: &File,
) -> Result<reqwest::Response, Error> {
    let resume_url = Client::build_drive_uri(UPLOAD_PATH, &[("uploadType", "resumable")])?;

    Ok(http
        .post(resume_url)
        .headers(headers.clone())
        .header(HEADER_UPLOAD_CONTENT_TYPE, &metadata.mime_type)
        .header(HEADER_UPLOAD_CONTENT_LENGTH, data.len())
        .header(
            reqwest::header::CONTENT_TYPE,
            mime::APPLICATION_JSON.as_ref(),
        )
        .json(&metadata)
        .send()
        .await?)
}

#[cfg(test)]
mod tests {
    use mockito::Matcher;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_resume_url() -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = "https://hoge.com";
        let metadata = File {
            name: "test.rb".into(),
            mime_type: "text/plain".into(),
            ..Default::default()
        };
        let headers: HeaderMap = vec![(reqwest::header::AUTHORIZATION, "Bearer token".try_into()?)]
            .into_iter()
            .collect();
        let data = [1, 2, 3];

        let _mock = mockito::mock("POST", format!("/{}", UPLOAD_PATH).as_str())
            .match_query(Matcher::UrlEncoded("uploadType".into(), "resumable".into()))
            .match_header(reqwest::header::AUTHORIZATION.as_str(), "Bearer token")
            .match_header(
                reqwest::header::CONTENT_TYPE.as_str(),
                mime::APPLICATION_JSON.as_ref(),
            )
            .match_header(HEADER_UPLOAD_CONTENT_TYPE, metadata.mime_type.as_str())
            .match_header(
                HEADER_UPLOAD_CONTENT_LENGTH,
                data.len().to_string().as_str(),
            )
            .with_status(200)
            .with_header(reqwest::header::LOCATION.as_str(), url)
            .create();

        let resumable_url = resume_url(&client, &data, headers, &metadata).await?;

        _mock.assert();
        assert_eq!(resumable_url, Url::parse(url)?);

        Ok(())
    }

    #[tokio::test]
    async fn test_upload_file() -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = [mockito::server_url().as_str(), "/upload_file"].join("");
        let metadata = File {
            name: "test.rb".into(),
            mime_type: "text/plain".into(),
            ..Default::default()
        };
        let headers: HeaderMap = vec![(reqwest::header::AUTHORIZATION, "Bearer token".try_into()?)]
            .into_iter()
            .collect();
        let data = [1, 2, 3];

        let resume_mock = mockito::mock("POST", format!("/{}", UPLOAD_PATH).as_str())
            .match_query(Matcher::UrlEncoded("uploadType".into(), "resumable".into()))
            .match_header(reqwest::header::AUTHORIZATION.as_str(), "Bearer token")
            .with_status(200)
            .with_header(reqwest::header::LOCATION.as_str(), &url)
            .create();

        let upload_mock = mockito::mock("PUT", "/upload_file")
            .match_header(reqwest::header::AUTHORIZATION.as_str(), "Bearer token")
            .match_body(data.clone().to_vec())
            .with_status(200)
            .with_body(
                json!({
                    "kind": "drive#file",
                    "id": "asehdgjfhkjkasdflkajsdfalksjdfa",
                    "name": metadata.name.clone(),
                    "mimeType": metadata.mime_type.clone(),
                })
                .to_string(),
            )
            .create();

        let file = upload_file(&client, headers, data, metadata.clone()).await?;

        resume_mock.assert();
        upload_mock.assert();

        assert_eq!(
            file,
            File {
                kind: Some("drive#file".into()),
                id: Some("asehdgjfhkjkasdflkajsdfalksjdfa".into()),
                ..metadata
            }
        );

        Ok(())
    }
}
