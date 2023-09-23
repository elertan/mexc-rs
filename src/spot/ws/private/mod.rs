use std::ops::ControlFlow;
use futures::stream::{BoxStream, SplitSink};
use tokio::sync::{Mutex, MutexGuard};
use tokio_tungstenite::tungstenite::Message;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use futures::{StreamExt, SinkExt};
use rust_decimal::Decimal;
use tokio_util::sync::CancellationToken;
use crate::spot::MexcSpotApiClientWithAuthentication;
use crate::spot::v3::ApiError;
use crate::spot::v3::create_user_data_stream::CreateUserDataStreamEndpoint;
use crate::spot::v3::enums::ChangedType;
use crate::spot::v3::keep_alive_user_data_stream::{KeepAliveUserDataStreamEndpoint, KeepAliveUserDataStreamParams};
use crate::spot::ws::MexcSpotWsEndpoint;
use crate::spot::ws::private::account_update::channel_message_to_account_update_message;

pub mod subscription;
pub mod account_update;

struct MexcSpotPrivateWsClientInner {
    message_sink_tx: Option<async_channel::Sender<Message>>,
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
            message_sink_tx: None,
        }));

        Self { endpoint, ws_message_tx, ws_message_rx, ws_raw_message_tx, ws_raw_message_rx, inner, spot_client_with_auth }
    }

    async fn acquire_websocket(&self) -> Result<AcquireWebsocketOutput, AcquireWebsocketError> {
        let mut inner = self.inner.lock().await;
        if inner.message_sink_tx.is_some() {
            return Ok(AcquireWebsocketOutput { inner_guard: inner });
        }

        tracing::debug!("Creating listen key for private websocket...");
        let user_data_stream_output = self.spot_client_with_auth.create_user_data_stream().await?;
        tracing::debug!("Listen key created: {}", &user_data_stream_output.listen_key);

        let ws_url = format!("{}?listenKey={}", self.endpoint.as_ref(), &user_data_stream_output.listen_key);
        let (ws, _) = tokio_tungstenite::connect_async(&ws_url).await?;
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
            let spot_client_with_auth = self.spot_client_with_auth.clone();

            async move {
                loop {
                    tokio::select! {
                        _ = cancellation_token.cancelled() => {
                            tracing::debug!("Cancelling keep alive ws");
                            break;
                        }
                        _ = tokio::time::sleep(std::time::Duration::from_secs(60 * 30)) => {
                            tracing::debug!("Sending keep alive for user data stream");

                            match spot_client_with_auth.keep_alive_user_data_stream(KeepAliveUserDataStreamParams {
                                listen_key: &user_data_stream_output.listen_key,
                            }).await {
                                Ok(_) => {},
                                Err(err) => {
                                    tracing::error!("Failed to keep alive user data stream: {:?}", err);
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

async fn ws_message_receiver(message_opt: Option<Result<Message, tokio_tungstenite::tungstenite::Error>>, ws_raw_message_tx: &async_broadcast::Sender<Arc<PrivateRawMexcSpotWsMessage>>, ws_message_tx: &async_broadcast::Sender<Arc<PrivateMexcSpotWsMessage>>) -> ControlFlow<()> {
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
    let raw_mexc_ws_message = match serde_json::from_str::<PrivateRawMexcSpotWsMessage>(&text) {
        Ok(raw_mexc_ws_message) => Arc::new(raw_mexc_ws_message),
        Err(err) => {
            tracing::error!("Failed to parse mexc ws message: {:?}", err);
            return ControlFlow::Break(());
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
    inner_guard: MutexGuard<'a, MexcSpotPrivateWsClientInner>,
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
    #[error("Create user data stream error: {0}")]
    CreateUserDataStreamError(#[from] ApiError),
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
