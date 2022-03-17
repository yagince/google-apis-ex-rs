use reqwest::{header::HeaderMap, StatusCode, Url};

use crate::{
    drive::{
        client::{File, GoogleDriveError},
        Client, UploadFileMetadata,
    },
    error::Error,
    mime,
};

const UPLOAD_PATH: &str = "upload/drive/v3/files";
const HEADER_UPLOAD_CONTENT_TYPE: &'static str = "X-Upload-Content-Type";
const HEADER_UPLOAD_CONTENT_LENGTH: &'static str = "X-Upload-Content-Length";

pub async fn upload_file(
    http: &reqwest::Client,
    headers: HeaderMap,
    data: impl Into<Vec<u8>>,
    metadata: UploadFileMetadata,
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
    metadata: &UploadFileMetadata,
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
    metadata: &UploadFileMetadata,
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
    metadata: &UploadFileMetadata,
) -> Result<reqwest::Response, Error> {
    let resume_url = Client::build_uri(UPLOAD_PATH, &[("uploadType", "resumable")])?;

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
