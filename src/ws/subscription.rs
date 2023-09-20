use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use crate::ws::{AcquireWebsocketError, ClientMessagePayload, MexcWsClient};

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
    #[error("Failed to acquire websocket: {0}")]
    AcquireWebsocketError(#[from] AcquireWebsocketError),
    #[error("Failed to send message: {0}")]
    TungesteniteError(#[from] tokio_tungstenite::tungstenite::Error),
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
            method: "SUBSCRIPTION",
            params: payload_params,
        };
        let payload_str = serde_json::to_string(&payload)?;
        let message = Message::Text(payload_str);

        let mut awo = self.acquire_websocket().await?;
        let ws = awo.websocket_sink();

        ws.send(message).await?;
        let mut i = 0;
        let mut raw_stream = self.stream_raw();
        while let Some(raw_msg) = raw_stream.next().await {
            tracing::info!("Raw message: {:?}", raw_msg);
            i += 1;
            if i > 5 {
                todo!();
            }
        }

        Ok(SubscribeOutput {})
    }
}

#[cfg(test)]
mod tests { }
