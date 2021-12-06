use gcp_auth::Token;
use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig},
    IntoRequest, Request,
};

use crate::{
    auth::get_token,
    error::Error,
    proto::{
        google::cloud::kms::v1::{
            key_management_service_client::KeyManagementServiceClient, DecryptRequest,
            DecryptResponse, EncryptRequest, EncryptResponse, ListCryptoKeysRequest,
            ListCryptoKeysResponse, ListKeyRingsRequest, ListKeyRingsResponse,
        },
        TLS_CERT,
    },
};

pub const DOMAIN_NAME: &'static str = "cloudkms.googleapis.com";
pub const ENDPOINT: &'static str = "https://cloudkms.googleapis.com";
pub const SCOPES: [&'static str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/cloudkms",
];

pub struct KmsClient {
    token: Token,
    client: KeyManagementServiceClient<Channel>,
}

impl KmsClient {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            token: get_token(&SCOPES).await?,
            client: Self::kms_client().await?,
        })
    }

    async fn kms_client() -> Result<KeyManagementServiceClient<Channel>, Error> {
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERT))
            .domain_name(DOMAIN_NAME);

        let channel = Channel::from_static(ENDPOINT)
            .tls_config(tls_config)?
            .connect()
            .await?;

        Ok(KeyManagementServiceClient::new(channel))
    }

    pub(crate) async fn construct_request<T: IntoRequest<T>>(
        &self,
        request: T,
        headers: Vec<(&str, &str)>,
    ) -> Result<Request<T>, Error> {
        construct_request(request, self.token.as_str(), headers).await
    }

    /// # Arguments
    /// * `parent` - in the format `projects/*/locations/*`
    pub async fn list_key_rings(&mut self, parent: &str) -> Result<ListKeyRingsResponse, Error> {
        let request = Self::construct_request(
            self,
            ListKeyRingsRequest {
                parent: parent.to_owned(),
                page_size: 100,
                page_token: Default::default(),
                filter: Default::default(),
                order_by: Default::default(),
            },
            vec![("parent", parent)],
        )
        .await?;

        let response = self.client.list_key_rings(request).await?;
        Ok(response.into_inner())
    }

    /// # Arguments
    /// * `parent` - in the format `projects/*/locations/*/keyRings/*`
    pub async fn list_crypto_keys(
        &mut self,
        parent: &str,
    ) -> Result<ListCryptoKeysResponse, Error> {
        let request = Self::construct_request(
            self,
            ListCryptoKeysRequest {
                parent: parent.to_owned(),
                page_size: 100,
                page_token: Default::default(),
                filter: Default::default(),
                order_by: Default::default(),
                version_view: 0,
            },
            vec![("parent", parent)],
        )
        .await?;

        let response = self.client.list_crypto_keys(request).await?;
        Ok(response.into_inner())
    }

    /// # Arguments
    /// * `key_name` - in the format `projects/*/locations/*/keyRings/*/cryptoKeys/*`
    /// * `data`     - to be encrypted.
    pub async fn encrypt(
        &mut self,
        key_name: &str,
        data: Vec<u8>,
    ) -> Result<EncryptResponse, Error> {
        let request = Self::construct_request(
            self,
            EncryptRequest {
                name: key_name.to_owned(),
                plaintext: data,
                additional_authenticated_data: vec![],
                plaintext_crc32c: None,
                additional_authenticated_data_crc32c: None,
            },
            vec![("name", key_name)],
        )
        .await?;

        let response = self.client.encrypt(request).await?;
        Ok(response.into_inner())
    }

    /// # Arguments
    /// * `key_name` - in the format `projects/*/locations/*/keyRings/*/cryptoKeys/*`
    /// * `data`     - to be decrypted.
    pub async fn decrypt(
        &mut self,
        key_name: &str,
        data: Vec<u8>,
    ) -> Result<DecryptResponse, Error> {
        let request = Self::construct_request(
            self,
            DecryptRequest {
                name: key_name.to_owned(),
                ciphertext: data,
                additional_authenticated_data: vec![],
                ciphertext_crc32c: None,
                additional_authenticated_data_crc32c: None,
            },
            vec![("name", key_name)],
        )
        .await?;

        let response = self.client.decrypt(request).await?;
        Ok(response.into_inner())
    }
}

async fn construct_request<T: IntoRequest<T>>(
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
