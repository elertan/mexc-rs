use std::collections::HashMap;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use crate::spot::ws::{AcquireWebsocketError, ClientMessagePayload, MexcSpotWsClient, RawMexcSpotWsMessage};

#[derive(Debug)]
pub struct SubscribeParams {
    pub subscription_requests: Vec<SubscriptionRequest>,
    /// Wait for subscription confirmation response
    ///
    /// If `None`, defaults to `true`
    pub wait_for_confirmation: Option<bool>,
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
impl Subscribe for MexcSpotWsClient {
    async fn subscribe(&self, params: SubscribeParams) -> Result<SubscribeOutput, SubscribeError> {
        let subscription_params = params.subscription_requests
            .iter()
            .map(|sr| sr.to_subscription_param())
            .collect::<Vec<String>>();
        let payload = ClientMessagePayload {
            method: "SUBSCRIPTION",
            params: subscription_params.clone(),
        };
        let payload_str = serde_json::to_string(&payload)?;
        let message = Message::Text(payload_str);

        tracing::debug!("Acquiring websocket...");
        let mut awo = self.acquire_websocket().await?;
        let ws = awo.websocket_sink();

        tracing::debug!("Sending message: {:?}", &message);
        ws.send(message).await?;

        if params.wait_for_confirmation.unwrap_or(true) {
            tracing::debug!("Waiting for subscription confirmation response...");
            let mut raw_stream = self.stream_raw();
            while let Some(raw_msg) = raw_stream.next().await {
                match raw_msg.as_ref() {
                    RawMexcSpotWsMessage::IdCodeMsg { msg, .. } => {
                        if msg.is_empty() {
                            continue;
                        }

                        let mut map = subscription_params.iter().map(|param| (param.as_str(), false)).collect::<HashMap<&str, bool>>();
                        let parts = msg.split(',');
                        for part in parts {
                            let has_part = subscription_params.iter().any(|param| param == part);
                            if has_part {
                                map.insert(part, true);
                            }
                        }
                        let has_all = map.values().all(|v| *v);
                        if has_all {
                            break;
                        }
                    }
                    _ => {}
                }
            }
            tracing::debug!("Subscription confirmed");
        }

        Ok(SubscribeOutput {})
    }
}

#[cfg(test)]
mod tests { }
