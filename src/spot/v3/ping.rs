use crate::spot::v3::ApiResult;
use crate::spot::MexcSpotApiTrait;
use async_trait::async_trait;

#[async_trait]
pub trait PingEndpoint {
    /// Test connectivity to the Rest API.
    async fn ping(&self) -> ApiResult<()>;
}

#[async_trait]
impl<T: MexcSpotApiTrait + Sync> PingEndpoint for T {
    async fn ping(&self) -> ApiResult<()> {
        let endpoint = format!("{}/api/v3/ping", self.endpoint().as_ref());
        self.reqwest_client().get(&endpoint).send().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::spot::MexcSpotApiClient;

    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let client = MexcSpotApiClient::default();
        let result = client.ping().await;
        assert!(result.is_ok());
    }
}
