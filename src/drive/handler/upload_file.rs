use reqwest::{header::HeaderMap, Url};

use crate::{
    drive::{Client, UploadFileMetadata},
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
) -> Result<(), Error> {
    let resume_url = Client::build_uri(UPLOAD_PATH, &[("uploadType", "resumable")])?;
    let data = data.into();

    let res = http
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
        .await?;

    let url = dbg!(res)
        .headers()
        .get(reqwest::header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<Url>()?;

    let res = http
        .put(url)
        .headers(headers)
        .header(reqwest::header::CONTENT_TYPE, &metadata.mime_type)
        .header(reqwest::header::CONTENT_LENGTH, data.len())
        .body(data)
        .send()
        .await?;
    dbg!(dbg!(res).text().await?);
    Ok(())
}
