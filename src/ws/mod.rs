use std::sync::Arc;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use futures::{StreamExt};
use futures::stream::{BoxStream, SplitSink};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use crate::ws::spot_deals::{channel_message_to_spot_deals_message, SpotDealsMessage};

pub mod subscription;
pub mod spot_deals;

pub enum MexcWsEndpoint {
    Base,
    Custom(String),
}

impl AsRef<str> for MexcWsEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MexcWsEndpoint::Base => "wss://wbs.mexc.com/ws",
            MexcWsEndpoint::Custom(endpoint) => endpoint,
        }
    }
}

struct MexcWsClientInner {
    websocket_sink: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
}

pub struct MexcWsClient {
    endpoint: MexcWsEndpoint,
    ws_message_tx: async_broadcast::Sender<Arc<MexcWsMessage>>,
    ws_message_rx: async_broadcast::Receiver<Arc<MexcWsMessage>>,
    ws_raw_message_tx: async_broadcast::Sender<Arc<RawMexcWsMessage>>,
    ws_raw_message_rx: async_broadcast::Receiver<Arc<RawMexcWsMessage>>,
    inner: Arc<Mutex<MexcWsClientInner>>,
}

impl MexcWsClient {
    pub fn new(endpoint: MexcWsEndpoint) -> Self {
        let (ws_message_tx, ws_message_rx) = async_broadcast::broadcast(1000000);
        let (ws_raw_message_tx, ws_raw_message_rx) = async_broadcast::broadcast(1000000);
        let inner = Arc::new(Mutex::new(MexcWsClientInner {
            websocket_sink: None,
        }));

        Self { endpoint, ws_message_tx, ws_message_rx, ws_raw_message_tx, ws_raw_message_rx, inner }
    }

    async fn acquire_websocket(&self) -> Result<AcquireWebsocketOutput, AcquireWebsocketError> {
        let mut inner = self.inner.lock().await;
        if inner.websocket_sink.is_some() {
            return Ok(AcquireWebsocketOutput { inner_guard: inner });
        }

        let (ws, _) = tokio_tungstenite::connect_async(self.endpoint.as_ref()).await?;
        let (ws_tx, mut ws_rx) = ws.split();
        inner.websocket_sink = Some(ws_tx);

        tokio::spawn({
            let ws_message_tx = self.ws_message_tx.clone();
            let ws_raw_message_tx = self.ws_raw_message_tx.clone();
            async move {
                while let Some(message_result) = ws_rx.next().await {
                    let message = match message_result {
                        Ok(message) => message,
                        Err(err) => {
                            tracing::error!("Failed to receive message: {:?}", err);
                            break;
                        }
                    };
                    let text = match message {
                        Message::Text(text) => text,
                        _ => {
                            tracing::debug!("Received non-text ws message: {:?}", message);
                            continue;
                        }
                    };
                    let raw_mexc_ws_message = match serde_json::from_str::<RawMexcWsMessage>(&text) {
                        Ok(raw_mexc_ws_message) => Arc::new(raw_mexc_ws_message),
                        Err(err) => {
                            tracing::error!("Failed to parse mexc ws message: {:?}", err);
                            break;
                        }
                    };
                    let mexc_message_opt = match raw_mexc_ws_message.as_ref() {
                        RawMexcWsMessage::ChannelMessage(channel_message) => {
                            let channel = &channel_message.channel;
                            if channel.starts_with("spot@public.deals.v3.api@") {
                                let result = channel_message_to_spot_deals_message(channel_message);
                                match result {
                                    Ok(spot_deals_message) => Some(Arc::new(MexcWsMessage::SpotDeals(spot_deals_message))),
                                    Err(err) => {
                                        tracing::error!("Failed to convert channel message to spot deals message: {:?}", err);
                                        break;
                                    }
                                }
                            } else {
                                None
                            }
                        }
                        _ => {
                            tracing::debug!("Received raw mexc ws message that won't be used for direct consumption by api consumer");
                            None
                        }
                    };

                    match ws_raw_message_tx.broadcast(raw_mexc_ws_message).await {
                        Ok(_) => {}
                        Err(err) => {
                            tracing::error!("Failed to broadcast raw message: {:?}", err);
                            break;
                        }
                    };

                    if let Some(mexc_message) = mexc_message_opt {
                        match ws_message_tx.broadcast(mexc_message).await {
                            Ok(_) => {}
                            Err(err) => {
                                tracing::error!("Failed to broadcast message: {:?}", err);
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(AcquireWebsocketOutput { inner_guard: inner })
    }

    pub fn stream(&self) -> BoxStream<Arc<MexcWsMessage>> {
        let mut mr = self.ws_message_rx.clone();
        let stream = async_stream::stream! {
            while let Ok(message) = mr.recv().await {
                yield message;
            }
        };
        stream.boxed()
    }

    fn stream_raw(&self) -> BoxStream<Arc<RawMexcWsMessage>> {
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

pub struct AcquireWebsocketOutput<'a> {
    inner_guard: MutexGuard<'a, MexcWsClientInner>,
}

impl<'a> AcquireWebsocketOutput<'a> {
    pub fn websocket_sink(&mut self) -> &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message> {
        self.inner_guard.websocket_sink.as_mut().expect("Websocket should be set here")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AcquireWebsocketError {
    #[error("Failed to connect to websocket: {0}")]
    TungesteniteError(#[from] tokio_tungstenite::tungstenite::Error),
}

impl Default for MexcWsClient {
    fn default() -> Self {
        Self::new(MexcWsEndpoint::Base)
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientMessagePayload<'a, T> {
    pub method: &'a str,
    pub params: T,
}

#[derive(Debug)]
pub enum MexcWsMessage {
    SpotDeals(SpotDealsMessage),
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum RawMexcWsMessage {
    IdCodeMsg { id: i64, code: i32, msg: String },
    ChannelMessage(ChannelMessage),
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ChannelMessage {
    #[serde(rename = "c")]
    pub channel: String,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "t", with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "d")]
    pub data: ChannelMessageData,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelMessageData {
    pub deals: Option<Vec<ChannelMessageDeal>>,
    #[serde(rename = "e")]
    pub event_type: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelMessageDeal {
    #[serde(rename = "p")]
    pub price: BigDecimal,
    #[serde(rename = "v")]
    pub quantity: BigDecimal,
    #[serde(rename = "t", with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "S")]
    pub trade_type: i32,
}
