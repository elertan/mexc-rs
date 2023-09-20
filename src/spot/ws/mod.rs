use std::sync::Arc;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use futures::{StreamExt};
use futures::stream::{BoxStream, SplitSink};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use crate::spot::ws::spot_deals::{channel_message_to_spot_deals_message, SpotDealsMessage};

pub mod subscription;
pub mod spot_deals;

pub enum MexcSportWsEndpoint {
    Base,
    Custom(String),
}

impl AsRef<str> for MexcSportWsEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MexcSportWsEndpoint::Base => "wss://wbs.mexc.com/ws",
            MexcSportWsEndpoint::Custom(endpoint) => endpoint,
        }
    }
}

struct MexcSpotWsClientInner {
    websocket_sink: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
}

pub struct MexcSpotWsClient {
    endpoint: MexcSportWsEndpoint,
    ws_message_tx: async_broadcast::Sender<Arc<MexcSpotWsMessage>>,
    ws_message_rx: async_broadcast::Receiver<Arc<MexcSpotWsMessage>>,
    ws_raw_message_tx: async_broadcast::Sender<Arc<RawMexcSpotWsMessage>>,
    ws_raw_message_rx: async_broadcast::Receiver<Arc<RawMexcSpotWsMessage>>,
    inner: Arc<Mutex<MexcSpotWsClientInner>>,
}

impl MexcSpotWsClient {
    pub fn new(endpoint: MexcSportWsEndpoint) -> Self {
        let (ws_message_tx, ws_message_rx) = async_broadcast::broadcast(1000000);
        let (ws_raw_message_tx, ws_raw_message_rx) = async_broadcast::broadcast(1000000);
        let inner = Arc::new(Mutex::new(MexcSpotWsClientInner {
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
                    let raw_mexc_ws_message = match serde_json::from_str::<RawMexcSpotWsMessage>(&text) {
                        Ok(raw_mexc_ws_message) => Arc::new(raw_mexc_ws_message),
                        Err(err) => {
                            tracing::error!("Failed to parse mexc ws message: {:?}", err);
                            break;
                        }
                    };
                    let mexc_message_opt = match raw_mexc_ws_message.as_ref() {
                        RawMexcSpotWsMessage::ChannelMessage(channel_message) => {
                            let channel = &channel_message.channel;
                            if channel.starts_with("spot@public.deals.v3.api@") {
                                let result = channel_message_to_spot_deals_message(channel_message);
                                match result {
                                    Ok(spot_deals_message) => Some(Arc::new(MexcSpotWsMessage::SpotDeals(spot_deals_message))),
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

    pub fn stream(&self) -> BoxStream<Arc<MexcSpotWsMessage>> {
        let mut mr = self.ws_message_rx.clone();
        let stream = async_stream::stream! {
            while let Ok(message) = mr.recv().await {
                yield message;
            }
        };
        stream.boxed()
    }

    fn stream_raw(&self) -> BoxStream<Arc<RawMexcSpotWsMessage>> {
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
    inner_guard: MutexGuard<'a, MexcSpotWsClientInner>,
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

impl Default for MexcSpotWsClient {
    fn default() -> Self {
        Self::new(MexcSportWsEndpoint::Base)
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientMessagePayload<'a, T> {
    pub method: &'a str,
    pub params: T,
}

#[derive(Debug)]
pub enum MexcSpotWsMessage {
    SpotDeals(SpotDealsMessage),
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum RawMexcSpotWsMessage {
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
