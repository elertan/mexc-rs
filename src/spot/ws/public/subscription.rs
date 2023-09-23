use std::collections::HashMap;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use crate::spot::ws::public::{AcquireWebsocketError, PublicClientMessagePayload, MexcSpotPublicWsClient, PublicRawMexcSpotWsMessage};
use crate::spot::ws::public::kline::KlineIntervalSubscription;

#[derive(Debug)]
pub struct PublicSubscribeParams {
    pub subscription_topics: Vec<PublicSubscriptionTopic>,
    /// Wait for subscription confirmation response
    ///
    /// If `None`, defaults to `true`
    pub wait_for_confirmation: Option<bool>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PublicSubscriptionTopic {
    SpotDeals(PublicSpotDealsSubscriptionTopic),
    SpotKline(PublicSpotKlineSubscriptionTopic),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PublicSpotDealsSubscriptionTopic {
    pub symbol: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PublicSpotKlineSubscriptionTopic {
    pub symbol: String,
    pub interval: KlineIntervalSubscription,
}

impl PublicSubscriptionTopic {
    pub fn to_subscription_param(&self) -> String {
        match self {
            PublicSubscriptionTopic::SpotDeals(spot_deals_sr) => format!("spot@public.deals.v3.api@{}", spot_deals_sr.symbol),
            PublicSubscriptionTopic::SpotKline(spot_kline_sr) => format!("spot@public.kline.v3.api@{}@{}", spot_kline_sr.symbol, spot_kline_sr.interval.as_ref()),
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
        let mut subscription_topics = vec![];
        for subscription_topic in params.subscription_topics.iter() {
            let has_subscription = self.has_subscription_topic(subscription_topic).await;
            if has_subscription {
                tracing::debug!("Already subscribed to {:?}, skipping...", subscription_topic);
                continue;
            }
            subscription_topics.push(subscription_topic.clone());
        }
        if subscription_topics.is_empty() {
            tracing::debug!("No new subscriptions to make, returning early...");
            return Ok(PublicSubscribeOutput {});
        }

        let subscription_params = subscription_topics
            .iter()
            .map(|sr| sr.to_subscription_param())
            .collect::<Vec<String>>();
        let payload = PublicClientMessagePayload {
            method: "SUBSCRIPTION",
            params: subscription_params.clone(),
        };
        let payload_str = serde_json::to_string(&payload)?;
        let message = Message::Text(payload_str);

        {
            tracing::debug!("Acquiring websocket...");
            let awo = self.acquire_websocket().await?;
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
        }

        for subscription_topic in subscription_topics {
            self.add_subscription_topic(subscription_topic).await;
        }

        Ok(PublicSubscribeOutput {})
    }
}

pub struct PublicUnsubscribeParams {
    pub subscription_topics: Vec<PublicSubscriptionTopic>,
    /// Wait for unsubscription confirmation response
    ///
    /// If `None`, defaults to `true`
    pub wait_for_confirmation: Option<bool>,
}

#[derive(Debug)]
pub struct PublicUnsubscribeOutput {}

#[derive(Debug, thiserror::Error)]
pub enum PublicUnsubscribeError {
    #[error("Failed to serialize payload to JSON: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Failed to acquire websocket: {0}")]
    AcquireWebsocketError(#[from] AcquireWebsocketError),
    #[error("Failed to send message: {0}")]
    SendMessageError(#[from] async_channel::SendError<Message>),
}

#[async_trait]
pub trait PublicUnsubscribe {
    async fn public_unsubscribe(&self, params: PublicUnsubscribeParams) -> Result<PublicUnsubscribeOutput, PublicUnsubscribeError>;
}

#[async_trait]
impl PublicUnsubscribe for MexcSpotPublicWsClient {
    async fn public_unsubscribe(&self, params: PublicUnsubscribeParams) -> Result<PublicUnsubscribeOutput, PublicUnsubscribeError> {
        let mut subscription_topics_to_remove = vec![];
        for subscription_topic in params.subscription_topics.iter() {
            let has_subscription = self.has_subscription_topic(subscription_topic).await;
            if !has_subscription {
                tracing::debug!("Not subscribed to {:?}, skipping...", subscription_topic);
                continue;
            }
            subscription_topics_to_remove.push(subscription_topic.clone());
        }

        if subscription_topics_to_remove.is_empty() {
            tracing::debug!("No subscriptions to remove, returning early...");
            return Ok(PublicUnsubscribeOutput {});
        }

        let subscription_params = subscription_topics_to_remove
            .iter()
            .map(|sr| sr.to_subscription_param())
            .collect::<Vec<String>>();

        let payload = PublicClientMessagePayload {
            method: "UNSUBSCRIPTION",
            params: subscription_params.clone(),
        };
        let payload_str = serde_json::to_string(&payload)?;
        let message = Message::Text(payload_str);

        {
            tracing::debug!("Acquiring websocket...");
            let awo = self.acquire_websocket().await?;
            let ws_tx = awo.message_sender();

            tracing::debug!("Sending message: {:?}", &message);
            ws_tx.send(message).await?;

            if params.wait_for_confirmation.unwrap_or(true) {
                tracing::debug!("Waiting for unsubscription confirmation response...");
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
                tracing::debug!("Unsubscription confirmed");
            }
        }

        for subscription_topic in subscription_topics_to_remove {
            self.remove_subscription_topic(&subscription_topic).await;
        }

        Ok(PublicUnsubscribeOutput {})
    }
}

#[cfg(test)]
mod tests { }
