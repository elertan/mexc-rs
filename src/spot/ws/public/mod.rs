use std::ops::ControlFlow;
use futures::stream::{BoxStream};
use tokio::sync::{Mutex, MutexGuard};
use tokio_tungstenite::tungstenite::{ Message};
use std::sync::Arc;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use tokio_util::sync::CancellationToken;
use crate::spot::ws::MexcSpotWsEndpoint;
use crate::spot::ws::public::spot_deals::{channel_message_to_spot_deals_message, SpotDealsMessage};

pub mod subscription;
pub mod spot_deals;

struct MexcSpotPublicWsClientInner {
    message_sink_tx: Option<async_channel::Sender<Message>>,
}

pub struct MexcSpotPublicWsClient {
    endpoint: MexcSpotWsEndpoint,
    ws_message_tx: async_broadcast::Sender<Arc<PublicMexcSpotWsMessage>>,
    ws_message_rx: async_broadcast::Receiver<Arc<PublicMexcSpotWsMessage>>,
    ws_raw_message_tx: async_broadcast::Sender<Arc<PublicRawMexcSpotWsMessage>>,
    ws_raw_message_rx: async_broadcast::Receiver<Arc<PublicRawMexcSpotWsMessage>>,
    inner: Arc<Mutex<MexcSpotPublicWsClientInner>>,
}

impl MexcSpotPublicWsClient {
    pub fn new(endpoint: MexcSpotWsEndpoint) -> Self {
        let (ws_message_tx, ws_message_rx) = async_broadcast::broadcast(1000000);
        let (ws_raw_message_tx, ws_raw_message_rx) = async_broadcast::broadcast(1000000);
        let inner = Arc::new(Mutex::new(MexcSpotPublicWsClientInner {
            message_sink_tx: None,
        }));

        Self { endpoint, ws_message_tx, ws_message_rx, ws_raw_message_tx, ws_raw_message_rx, inner }
    }

    async fn acquire_websocket(&self) -> Result<AcquireWebsocketOutput, AcquireWebsocketError> {
        let mut inner = self.inner.lock().await;
        if inner.message_sink_tx.is_some() {
            return Ok(AcquireWebsocketOutput { inner_guard: inner });
        }

        let (ws, _) = tokio_tungstenite::connect_async(self.endpoint.as_ref()).await?;
        let (mut ws_tx, mut ws_rx) = ws.split();

        let cancellation_token = CancellationToken::new();

        let (message_sink_tx, message_sink_rx) = async_channel::unbounded();
        inner.message_sink_tx = Some(message_sink_tx.clone());

        tokio::spawn({
            let cancellation_token = cancellation_token.clone();
            let message_sink_tx = message_sink_tx.clone();
            async move {
                loop {
                    tokio::select! {
                        _ = cancellation_token.cancelled() => {
                            tracing::debug!("Cancelling message sink");
                            break;
                        }
                        message_result = message_sink_rx.recv() => {
                            let message = match message_result {
                                    Ok(message) => message,
                                Err(err) => {
                                    tracing::error!("Failed to receive message: {:?}", err);
                                    break;
                                }
                            };
                            match ws_tx.send(message).await {
                                Ok(_) => {}
                                Err(err) => match err {
                                    tokio_tungstenite::tungstenite::Error::ConnectionClosed => {
                                        tracing::debug!("Failed to send websocket message, connection closed");
                                        break;
                                    }
                                    tokio_tungstenite::tungstenite::Error::AlreadyClosed => {
                                        tracing::debug!("Failed to send websocket message, connection already closed");
                                        break;
                                    }
                                    _ => {
                                        tracing::error!("Failed to send websocket message: {:?}", err);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        tokio::spawn({
            let cancellation_token = cancellation_token.clone();
            let message_sink_tx = message_sink_tx.clone();
            async move {
                loop {
                    tokio::select! {
                        _ = cancellation_token.cancelled() => {
                            tracing::debug!("Cancelling ping ws sender");
                            break;
                        }
                        _ = tokio::time::sleep(std::time::Duration::from_secs(20)) => {
                            tracing::trace!("Sending ping");
                            let message = Message::Text(r#"{"method":"PING"}"#.to_string());
                            match message_sink_tx.send(message).await {
                                Ok(_) => {}
                                Err(err) => {
                                    tracing::error!("Failed to send ping: {:?}", err);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        tokio::spawn({
            let cancellation_token = cancellation_token.clone();
            let ws_message_tx = self.ws_message_tx.clone();
            let ws_raw_message_tx = self.ws_raw_message_tx.clone();
            async move {
                loop {
                    tokio::select! {
                        _ = cancellation_token.cancelled() => {
                            tracing::debug!("Cancelling ws message receiver");
                            break;
                        }
                        message_opt = ws_rx.next() => {
                            let control_flow = ws_message_receiver(message_opt, &ws_raw_message_tx, &ws_message_tx).await;
                            match control_flow {
                                ControlFlow::Break(_) => {
                                    break;
                                }
                                ControlFlow::Continue(_) => {}
                            }
                        }
                    }
                }
            }
        });

        Ok(AcquireWebsocketOutput { inner_guard: inner })
    }

    pub fn stream(&self) -> BoxStream<Arc<PublicMexcSpotWsMessage>> {
        let mut mr = self.ws_message_rx.clone();
        let stream = async_stream::stream! {
            while let Ok(message) = mr.recv().await {
                yield message;
            }
        };
        stream.boxed()
    }

    fn stream_raw(&self) -> BoxStream<Arc<PublicRawMexcSpotWsMessage>> {
        let mut mr = self.ws_raw_message_rx.clone();
        let stream = async_stream::stream! {
            while let Ok(message) = mr.recv().await {
                tracing::debug!("Received raw mexc ws message: {:?}", &message);
                yield message;
            }
        };
        stream.boxed()
    }
}

async fn ws_message_receiver(message_opt: Option<Result<Message, tokio_tungstenite::tungstenite::Error>>, ws_raw_message_tx: &async_broadcast::Sender<Arc<PublicRawMexcSpotWsMessage>>, ws_message_tx: &async_broadcast::Sender<Arc<PublicMexcSpotWsMessage>>) -> ControlFlow<()> {
    let Some(message_result) = message_opt else {
        return ControlFlow::Break(());
    };
    let message = match message_result {
        Ok(message) => message,
        Err(err) => {
            match err {
                tokio_tungstenite::tungstenite::Error::ConnectionClosed => {
                    tracing::debug!("Connection closed");
                }
                tokio_tungstenite::tungstenite::Error::AlreadyClosed => {
                    tracing::debug!("Connection already closed");
                }
                _ => {
                    tracing::error!("Failed to receive message: {:?}", &err);
                }
            }
            return ControlFlow::Break(());
        }
    };
    let text = match message {
        Message::Text(text) => text,
        _ => {
            tracing::debug!("Received non-text ws message: {:?}", message);
            return ControlFlow::Break(());
        }
    };
    let raw_mexc_ws_message = match serde_json::from_str::<PublicRawMexcSpotWsMessage>(&text) {
        Ok(raw_mexc_ws_message) => Arc::new(raw_mexc_ws_message),
        Err(err) => {
            tracing::error!("Failed to parse mexc ws message: {:?}", err);
            return ControlFlow::Break(());
        }
    };
    let mexc_message_opt = match raw_mexc_ws_message.as_ref() {
        PublicRawMexcSpotWsMessage::ChannelMessage(channel_message) => {
            let channel = &channel_message.channel;
            if channel.starts_with("spot@public.deals.v3.api@") {
                let result = channel_message_to_spot_deals_message(channel_message);
                match result {
                    Ok(spot_deals_message) => Some(Arc::new(PublicMexcSpotWsMessage::SpotDeals(spot_deals_message))),
                    Err(err) => {
                        tracing::error!("Failed to convert channel message to spot deals message: {:?}", err);
                        return ControlFlow::Break(());
                    }
                }
            } else {
                None
            }
        }
        _ => {
            tracing::trace!("Received raw mexc ws message that won't be used for direct consumption by api consumer");
            None
        }
    };

    match ws_raw_message_tx.broadcast(raw_mexc_ws_message).await {
        Ok(_) => {}
        Err(err) => {
            tracing::error!("Failed to broadcast raw message: {:?}", err);
            return ControlFlow::Break(());
        }
    };

    if let Some(mexc_message) = mexc_message_opt {
        match ws_message_tx.broadcast(mexc_message).await {
            Ok(_) => {}
            Err(err) => {
                tracing::error!("Failed to broadcast message: {:?}", err);
                return ControlFlow::Break(());
            }
        }
    }

    ControlFlow::Continue(())
}

pub struct AcquireWebsocketOutput<'a> {
    inner_guard: MutexGuard<'a, MexcSpotPublicWsClientInner>,
}

impl<'a> AcquireWebsocketOutput<'a> {
    pub fn message_sender(&self) -> &async_channel::Sender<Message> {
        self.inner_guard.message_sink_tx.as_ref().expect("Websocket should be set here")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AcquireWebsocketError {
    #[error("Failed to connect to websocket: {0}")]
    TungesteniteError(#[from] tokio_tungstenite::tungstenite::Error),
}

impl Default for MexcSpotPublicWsClient {
    fn default() -> Self {
        Self::new(MexcSpotWsEndpoint::Base)
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicClientMessagePayload<'a, T> {
    pub method: &'a str,
    pub params: T,
}

#[derive(Debug)]
pub enum PublicMexcSpotWsMessage {
    SpotDeals(SpotDealsMessage),
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum PublicRawMexcSpotWsMessage {
    IdCodeMsg { id: i64, code: i32, msg: String },
    ChannelMessage(PublicChannelMessage),
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct PublicChannelMessage {
    #[serde(rename = "c")]
    pub channel: String,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "t", with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "d")]
    pub data: PublicChannelMessageData,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublicChannelMessageData {
    pub deals: Option<Vec<PublicChannelMessageDeal>>,
    #[serde(rename = "e")]
    pub event_type: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublicChannelMessageDeal {
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "v")]
    pub quantity: Decimal,
    #[serde(rename = "t", with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "S")]
    pub trade_type: i32,
}
