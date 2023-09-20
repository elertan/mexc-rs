use async_trait::async_trait;
use crate::ws::{ClientMessagePayload, MexcWsClient};

#[derive(Debug)]
pub struct SubscribeParams {
    pub subscription_requests: Vec<SubscriptionRequest>,
}

#[derive(Debug)]
pub enum SubscriptionRequest {
    SpotDeals(SpotDealsSubscriptionRequest),
}

#[derive(Debug)]
pub struct SpotDealsSubscriptionRequest {
    pub symbol: String,
}

impl SubscriptionRequest {
    pub fn to_subscription_param(&self) -> String {
        match self {
            SubscriptionRequest::SpotDeals(spot_deals_sr) => format!("spot@public.deals.v3.api@{}", spot_deals_sr.symbol)
        }
    }
}

#[derive(Debug)]
pub struct SubscribeOutput {}

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("Failed to serialize payload to JSON: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

#[async_trait]
pub trait Subscribe {
    async fn subscribe(&self, params: SubscribeParams) -> Result<SubscribeOutput, SubscribeError>;
}

#[async_trait]
impl Subscribe for MexcWsClient {
    async fn subscribe(&self, params: SubscribeParams) -> Result<SubscribeOutput, SubscribeError> {
        let payload_params = params.subscription_requests
            .iter()
            .map(|sr| sr.to_subscription_param())
            .collect::<Vec<String>>();
        let payload = ClientMessagePayload {
            method: "SUBSCRIBE",
            params: payload_params,
        };
        let payload_str = serde_json::to_string(&payload)?;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use super::*;

    #[tokio::test]
    async fn test_subscription_spot_deals() {
        let ws_client = MexcWsClient::default();
        let subscribe_params = SubscribeParams {
            subscription_requests: vec![
                SubscriptionRequest::SpotDeals(SpotDealsSubscriptionRequest {
                    symbol: "KASUSDT".to_string(),
                }),
            ],
        };
        let subscribe_output = ws_client.subscribe(subscribe_params).await.expect("Failed to subscribe");
        eprintln!("{:?}", subscribe_output);

        let message_result = ws_client.stream().next().await;
        eprintln!("{:?}", message_result);

        assert!(message_result.is_some());
    }
}
