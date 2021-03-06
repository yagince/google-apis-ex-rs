use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig},
    IntoRequest, Request,
};

use crate::{
    auth::TokenManager,
    error::Error,
    proto::{
        google::cloud::kms::v1::{
            key_management_service_client::KeyManagementServiceClient, DecryptRequest,
            DecryptResponse, EncryptRequest, EncryptResponse, ListCryptoKeysRequest,
            ListCryptoKeysResponse, ListKeyRingsRequest, ListKeyRingsResponse,
        },
        TLS_CERT,
    },
    util::construct_request,
};

pub const DOMAIN_NAME: &str = "cloudkms.googleapis.com";
pub const ENDPOINT: &str = "https://cloudkms.googleapis.com";
pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/cloudkms",
];

pub struct KmsClient {
    token_manager: TokenManager,
    client: KeyManagementServiceClient<Channel>,
}

impl KmsClient {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            token_manager: TokenManager::new(&SCOPES).await?,
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
        &mut self,
        request: T,
        headers: Vec<(&str, &str)>,
    ) -> Result<Request<T>, Error> {
        construct_request(
            request,
            self.token_manager.get_token().await?.as_str(),
            headers,
        )
        .await
    }

    /// # Arguments
    /// * `parent` - in the format `projects/*/locations/*`
    pub async fn list_key_rings(&mut self, parent: &str) -> Result<ListKeyRingsResponse, Error> {
        let request = self
            .construct_request(
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
        let request = self
            .construct_request(
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
        data: impl Into<Vec<u8>>,
    ) -> Result<EncryptResponse, Error> {
        let request = self
            .construct_request(
                EncryptRequest {
                    name: key_name.to_owned(),
                    plaintext: data.into(),
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
        data: impl Into<Vec<u8>>,
    ) -> Result<DecryptResponse, Error> {
        let request = self
            .construct_request(
                DecryptRequest {
                    name: key_name.to_owned(),
                    ciphertext: data.into(),
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
