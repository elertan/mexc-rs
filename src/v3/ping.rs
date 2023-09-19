use async_trait::async_trait;
use crate::{MexcApiClient, MexcApiClientWithAuthentication, MexcApiEndpoint};
use crate::v3::ApiV3Result;

#[async_trait]
pub trait PingEndpoint {
    /// Test connectivity to the Rest API.
    async fn ping(&self) -> ApiV3Result<()>;
}

async fn ping_impl(endpoint: &MexcApiEndpoint, client: &reqwest::Client) -> ApiV3Result<()> {
    let endpoint = format!("{}/api/v3/ping", endpoint.as_ref());
    client.get(&endpoint).send().await?;

    Ok(())
}

#[async_trait]
impl PingEndpoint for MexcApiClient {
    async fn ping(&self) -> ApiV3Result<()> {
        ping_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[async_trait]
impl PingEndpoint for MexcApiClientWithAuthentication {
    async fn ping(&self) -> ApiV3Result<()> {
        ping_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let client = MexcApiClient::default();
        let result = client.ping().await;
        assert!(result.is_ok());
    }
}
