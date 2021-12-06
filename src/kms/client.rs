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
            key_management_service_client::KeyManagementServiceClient, ListKeyRingsRequest,
            ListKeyRingsResponse,
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
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERT))
            .domain_name(DOMAIN_NAME);

        let channel = Channel::from_static(ENDPOINT)
            .tls_config(tls_config)?
            .connect()
            .await?;

        Ok(Self {
            token: get_token(&SCOPES).await?,
            client: KeyManagementServiceClient::new(channel),
        })
    }

    pub(crate) async fn construct_request<T: IntoRequest<T>>(
        &self,
        request: T,
    ) -> Result<Request<T>, Error> {
        let mut request = request.into_request();
        let token = self.token.as_str();
        let metadata = request.metadata_mut();
        metadata.insert("authorization", token.parse().unwrap());
        Ok(request)
    }

    /// # Arguments
    /// * `parent` - in the format `projects/*/locations/*`
    pub async fn list_key_rings(&mut self, parent: &str) -> Result<ListKeyRingsResponse, Error> {
        let request = Self::construct_request(
            self,
            ListKeyRingsRequest {
                parent: parent.to_owned(),
                page_size: 0,
                page_token: Default::default(),
                filter: Default::default(),
                order_by: Default::default(),
            },
        )
        .await?;

        let response = self.client.list_key_rings(request).await?;
        Ok(response.into_inner())
    }
}
