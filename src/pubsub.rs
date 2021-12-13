use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig},
    IntoRequest, Request,
};

use crate::{
    auth::TokenManager,
    error::Error,
    proto::{
        google::pubsub::v1::{
            publisher_client::PublisherClient, PublishRequest, PublishResponse, PubsubMessage,
        },
        TLS_CERT,
    },
    util::construct_request,
};

pub const DOMAIN_NAME: &str = "pubsub.googleapis.com";
pub const ENDPOINT: &str = "https://pubsub.googleapis.com";
pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/pubsub",
];

pub struct PubSubClient {
    token_manager: TokenManager,
    publisher_client: PublisherClient<Channel>,
}

impl PubSubClient {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            token_manager: TokenManager::new(&SCOPES).await?,
            publisher_client: Self::publisher_client().await?,
        })
    }

    async fn publisher_client() -> Result<PublisherClient<Channel>, Error> {
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERT))
            .domain_name(DOMAIN_NAME);

        let channel = Channel::from_static(ENDPOINT)
            .tls_config(tls_config)?
            .connect()
            .await?;

        Ok(PublisherClient::new(channel))
    }

    async fn construct_request<T: IntoRequest<T>>(
        &mut self,
        request: T,
    ) -> Result<Request<T>, Error> {
        construct_request(
            request,
            self.token_manager.get_token().await?.as_str(),
            vec![],
        )
        .await
    }

    /// # Arguments
    /// * `name` - in the format `projects/{project}/topics/{topic}`
    pub async fn publish(
        &mut self,
        topic: impl ToOwned<Owned = String>,
        data: impl Into<Vec<u8>>,
    ) -> Result<PublishResponse, Error> {
        let request = self
            .construct_request(PublishRequest {
                topic: topic.to_owned(),
                messages: vec![PubsubMessage {
                    data: data.into(),
                    attributes: Default::default(),
                    message_id: Default::default(),
                    publish_time: None,
                    ordering_key: Default::default(),
                }],
            })
            .await?;
        let res = self.publisher_client.publish(request).await?;
        Ok(res.into_inner())
    }
}
