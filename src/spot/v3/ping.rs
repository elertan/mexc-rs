use async_trait::async_trait;
use crate::spot::{MexcSpotApiClient, MexcSpotApiClientWithAuthentication, MexcSpotApiEndpoint};
use crate::spot::v3::ApiResult;

#[async_trait]
pub trait PingEndpoint {
    /// Test connectivity to the Rest API.
    async fn ping(&self) -> ApiResult<()>;
}

async fn ping_impl(endpoint: &MexcSpotApiEndpoint, client: &reqwest::Client) -> ApiResult<()> {
    let endpoint = format!("{}/api/v3/ping", endpoint.as_ref());
    client.get(&endpoint).send().await?;

    Ok(())
}

#[async_trait]
impl PingEndpoint for MexcSpotApiClient {
    async fn ping(&self) -> ApiResult<()> {
        ping_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[async_trait]
impl PingEndpoint for MexcSpotApiClientWithAuthentication {
    async fn ping(&self) -> ApiResult<()> {
        ping_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let client = MexcSpotApiClient::default();
        let result = client.ping().await;
        assert!(result.is_ok());
    }
}
