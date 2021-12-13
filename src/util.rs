use tonic::{IntoRequest, Request};

use crate::error::Error;

pub(crate) async fn construct_request<T: IntoRequest<T>>(
    request: T,
    token: &str,
    headers: Vec<(&str, &str)>,
) -> Result<Request<T>, Error> {
    let mut request = request.into_request();
    let metadata = request.metadata_mut();
    metadata.insert(
        "authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );

    request.metadata_mut().insert(
        "x-goog-request-params",
        headers
            .into_iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<_>>()
            .join("&")
            .parse()?,
    );
    Ok(request)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    use crate::proto::google::cloud::kms::v1::ListKeyRingsRequest;

    #[tokio::test]
    async fn test_construct_request() -> anyhow::Result<()> {
        let token = "token";
        let parent = "projects/test-proj/locations/asia-northeast1";

        let req = ListKeyRingsRequest {
            parent: parent.to_owned(),
            page_size: 100,
            page_token: Default::default(),
            filter: Default::default(),
            order_by: Default::default(),
        };

        let actual = construct_request(
            req.clone(),
            token,
            vec![("parent", parent), ("key", "value")],
        )
        .await?;

        assert_eq!(
            actual.metadata().get("authorization"),
            Some(&"Bearer token".parse().unwrap())
        );
        assert_eq!(
            actual.metadata().get("x-goog-request-params"),
            Some(
                &"parent=projects/test-proj/locations/asia-northeast1&key=value"
                    .parse()
                    .unwrap()
            )
        );
        assert_eq!(actual.into_inner(), req);

        Ok(())
    }
}
