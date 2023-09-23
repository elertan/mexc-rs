use futures::stream::{BoxStream, SplitSink};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use rust_decimal::Decimal;
use crate::spot::MexcSpotApiClientWithAuthentication;
use crate::spot::v3::ApiError;
use crate::spot::v3::create_user_data_stream::CreateUserDataStreamEndpoint;
use crate::spot::v3::enums::ChangedType;
use crate::spot::ws::MexcSpotWsEndpoint;
use crate::spot::ws::private::account_update::channel_message_to_account_update_message;

pub mod subscription;
pub mod account_update;

struct MexcSpotPrivateWsClientInner {
    websocket_sink: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
}

pub struct MexcSpotPrivateWsClient {
    endpoint: MexcSpotWsEndpoint,
    ws_message_tx: async_broadcast::Sender<Arc<PrivateMexcSpotWsMessage>>,
    ws_message_rx: async_broadcast::Receiver<Arc<PrivateMexcSpotWsMessage>>,
    ws_raw_message_tx: async_broadcast::Sender<Arc<PrivateRawMexcSpotWsMessage>>,
    ws_raw_message_rx: async_broadcast::Receiver<Arc<PrivateRawMexcSpotWsMessage>>,
    inner: Arc<Mutex<MexcSpotPrivateWsClientInner>>,
    spot_client_with_auth: MexcSpotApiClientWithAuthentication,
}

impl MexcSpotPrivateWsClient {
    pub fn new(endpoint: MexcSpotWsEndpoint, spot_client_with_auth: MexcSpotApiClientWithAuthentication) -> Self {
        let (ws_message_tx, ws_message_rx) = async_broadcast::broadcast(1000000);
        let (ws_raw_message_tx, ws_raw_message_rx) = async_broadcast::broadcast(1000000);
        let inner = Arc::new(Mutex::new(MexcSpotPrivateWsClientInner {
            websocket_sink: None,
        }));

        Self { endpoint, ws_message_tx, ws_message_rx, ws_raw_message_tx, ws_raw_message_rx, inner, spot_client_with_auth }
    }

    async fn acquire_websocket(&self) -> Result<AcquireWebsocketOutput, AcquireWebsocketError> {
        let mut inner = self.inner.lock().await;
        if inner.websocket_sink.is_some() {
            return Ok(AcquireWebsocketOutput { inner_guard: inner });
        }

        tracing::debug!("Creating listen key for private websocket...");
        let user_data_stream_output = self.spot_client_with_auth.create_user_data_stream().await?;
        tracing::debug!("Listen key created: {}", &user_data_stream_output.listen_key);

        let ws_url = format!("{}?listenKey={}", self.endpoint.as_ref(), user_data_stream_output.listen_key);
        let (ws, _) = tokio_tungstenite::connect_async(&ws_url).await?;
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
                    let raw_mexc_ws_message = match serde_json::from_str::<PrivateRawMexcSpotWsMessage>(&text) {
                        Ok(raw_mexc_ws_message) => Arc::new(raw_mexc_ws_message),
                        Err(err) => {
                            tracing::error!("Failed to parse mexc ws message: {:?}", err);
                            break;
                        }
                    };
                    let mexc_message_opt = match raw_mexc_ws_message.as_ref() {
                        PrivateRawMexcSpotWsMessage::ChannelMessage(channel_message) => {
                            let channel = &channel_message.channel;
                            if channel == "spot@private.account.v3.api" {
                                let result = channel_message_to_account_update_message(channel_message);
                                match result {
                                    Ok(account_update_message) => Some(Arc::new(PrivateMexcSpotWsMessage::AccountUpdate(account_update_message))),
                                    Err(err) => {
                                        tracing::error!("Failed to convert channel message to account update message: {:?}", err);
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

    pub fn stream(&self) -> BoxStream<Arc<PrivateMexcSpotWsMessage>> {
        let mut mr = self.ws_message_rx.clone();
        let stream = async_stream::stream! {
            while let Ok(message) = mr.recv().await {
                yield message;
            }
        };
        stream.boxed()
    }

    fn stream_raw(&self) -> BoxStream<Arc<PrivateRawMexcSpotWsMessage>> {
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
    inner_guard: MutexGuard<'a, MexcSpotPrivateWsClientInner>,
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
    #[error("Create user data stream error: {0}")]
    CreateUserDataStreamError(#[from] ApiError)
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateClientMessagePayload<'a, T> {
    pub method: &'a str,
    pub params: T,
}

#[derive(Debug)]
pub enum PrivateMexcSpotWsMessage {
    AccountUpdate(account_update::AccountUpdateMessage),
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum PrivateRawMexcSpotWsMessage {
    IdCodeMsg { id: i64, code: i32, msg: String },
    ChannelMessage(PrivateChannelMessage),
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct PrivateChannelMessage {
    #[serde(rename = "c")]
    pub channel: String,
    #[serde(rename = "t", with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "d")]
    pub data: PrivateChannelMessageData,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub(crate) enum PrivateChannelMessageData {
    AccountUpdate(AccountUpdateRawChannelMessageData)
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct AccountUpdateRawChannelMessageData {
    pub a: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub c: DateTime<Utc>,
    pub f: Decimal,
    pub fd: Decimal,
    pub l: Decimal,
    pub ld: Decimal,
    pub o: ChangedType,
}
