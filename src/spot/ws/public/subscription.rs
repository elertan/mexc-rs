use std::collections::HashMap;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use crate::spot::ws::public::{AcquireWebsocketError, PublicClientMessagePayload, MexcSpotPublicWsClient, PublicRawMexcSpotWsMessage};

#[derive(Debug)]
pub struct PublicSubscribeParams {
    pub subscription_requests: Vec<PublicSubscriptionRequest>,
    /// Wait for subscription confirmation response
    ///
    /// If `None`, defaults to `true`
    pub wait_for_confirmation: Option<bool>,
}

#[derive(Debug)]
pub enum PublicSubscriptionRequest {
    SpotDeals(PublicSpotDealsSubscriptionRequest),
}

#[derive(Debug)]
pub struct PublicSpotDealsSubscriptionRequest {
    pub symbol: String,
}

impl PublicSubscriptionRequest {
    pub fn to_subscription_param(&self) -> String {
        match self {
            PublicSubscriptionRequest::SpotDeals(spot_deals_sr) => format!("spot@public.deals.v3.api@{}", spot_deals_sr.symbol)
        }
    }
}

#[derive(Debug)]
pub struct PublicSubscribeOutput {}

#[derive(Debug, thiserror::Error)]
pub enum PublicSubscribeError {
    #[error("Failed to serialize payload to JSON: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Failed to acquire websocket: {0}")]
    AcquireWebsocketError(#[from] AcquireWebsocketError),
    #[error("Failed to send message: {0}")]
    SendMessageError(#[from] async_channel::SendError<Message>),
    #[error("Too many active subscriptions")]
    TooManyActiveSubscriptions,
}

#[async_trait]
pub trait PublicSubscribe {
    async fn public_subscribe(&self, params: PublicSubscribeParams) -> Result<PublicSubscribeOutput, PublicSubscribeError>;
}

#[async_trait]
impl PublicSubscribe for MexcSpotPublicWsClient {
    async fn public_subscribe(&self, params: PublicSubscribeParams) -> Result<PublicSubscribeOutput, PublicSubscribeError> {
        let subscription_params = params.subscription_requests
            .iter()
            .map(|sr| sr.to_subscription_param())
            .collect::<Vec<String>>();
        let payload = PublicClientMessagePayload {
            method: "SUBSCRIPTION",
            params: subscription_params.clone(),
        };
        let payload_str = serde_json::to_string(&payload)?;
        let message = Message::Text(payload_str);

        tracing::debug!("Acquiring websocket...");
        let mut awo = self.acquire_websocket().await?;
        let message_tx = awo.message_sender();

        tracing::debug!("Sending message: {:?}", &message);
        message_tx.send(message).await?;

        if params.wait_for_confirmation.unwrap_or(true) {
            tracing::debug!("Waiting for subscription confirmation response...");
            let mut raw_stream = self.stream_raw();
            while let Some(raw_msg) = raw_stream.next().await {
                match raw_msg.as_ref() {
                    PublicRawMexcSpotWsMessage::IdCodeMsg { msg, .. } => {
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

        Ok(PublicSubscribeOutput {})
    }
}

#[cfg(test)]
mod tests { }
