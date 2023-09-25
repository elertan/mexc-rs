use crate::spot::v3::create_user_data_stream::CreateUserDataStreamEndpoint;
use crate::spot::v3::keep_alive_user_data_stream::{
    KeepAliveUserDataStreamEndpoint, KeepAliveUserDataStreamParams,
};
use crate::spot::v3::ApiError;
use crate::spot::ws::auth::WebsocketAuth;
use crate::spot::ws::topic::Topic;
use crate::spot::ws::{message, Inner, MexcSpotWebsocketClient, SendableMessage, WebsocketEntry};
use crate::spot::MexcSpotApiClientWithAuthentication;
use async_channel::Sender;
use async_trait::async_trait;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct AcquireWebsocketsForTopicsParams {
    pub auth: Option<WebsocketAuth>,
    pub for_topics: Vec<Topic>,
}

impl Default for AcquireWebsocketsForTopicsParams {
    fn default() -> Self {
        Self::new(None, Vec::new())
    }
}

impl AcquireWebsocketsForTopicsParams {
    pub fn new(auth: Option<WebsocketAuth>, topics: Vec<Topic>) -> Self {
        Self {
            auth,
            for_topics: topics,
        }
    }

    pub fn for_topic(mut self, topic: Topic) -> Self {
        self.for_topics.push(topic);
        self
    }

    pub fn for_topics(mut self, topics: Vec<Topic>) -> Self {
        self.for_topics.extend(topics);
        self
    }

    pub fn with_auth(mut self, auth: WebsocketAuth) -> Self {
        self.auth = Some(auth);
        self
    }
}

#[derive(Debug)]
pub(crate) struct AcquireWebsocketsForTopicsOutput {
    pub websockets: Vec<AcquiredWebsocket>,
}

#[derive(Debug)]
pub struct AcquiredWebsocket {
    pub websocket_entry: Arc<WebsocketEntry>,
    pub for_topics: Vec<Topic>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AcquireWebsocketForTopicsError {
    /// There is a hard limit of 5 websocket connections per listen key, and a limit of 60 active
    /// listen keys per user id. And each connection can subscribe to up to 30 topics.
    /// Therefore, the maximum number of topics that can be subscribed to per user is 9000.
    ///
    /// It cannot be over 9000!
    #[error("Maximum amount of topics for user will be exceeded")]
    MaximumAmountOfTopicsForUserWillBeExceeded,

    #[error("Requested topics require authentication")]
    RequestedTopicsRequireAuthentication,

    #[error("Tungestenite error: {0}")]
    TungesteniteError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Could not create datastream (listen key)")]
    CouldNotCreateDataStream(#[from] ApiError),
}

#[async_trait]
pub(crate) trait AcquireWebsocketsForTopics {
    async fn acquire_websockets_for_topics(
        self: Arc<Self>,
        params: AcquireWebsocketsForTopicsParams,
    ) -> Result<AcquireWebsocketsForTopicsOutput, AcquireWebsocketForTopicsError>;
}

#[async_trait]
impl AcquireWebsocketsForTopics for MexcSpotWebsocketClient {
    async fn acquire_websockets_for_topics(
        self: Arc<Self>,
        params: AcquireWebsocketsForTopicsParams,
    ) -> Result<AcquireWebsocketsForTopicsOutput, AcquireWebsocketForTopicsError> {
        let (private_topics, public_topics) = params
            .for_topics
            .into_iter()
            .partition::<Vec<_>, _>(|topic| topic.requires_auth());

        if params.auth.is_none() && !private_topics.is_empty() {
            return Err(AcquireWebsocketForTopicsError::RequestedTopicsRequireAuthentication);
        }

        let mut inner = self.inner.write().await;

        let mut acquired_websockets =
            match acquire_websockets_for_public_topics(self.clone(), &mut inner, public_topics)
                .await
            {
                Ok(x) => x,
                Err(err) => match err {
                    AcquireWebsocketsForPublicTopicsError::TungesteniteError(err) => {
                        return Err(AcquireWebsocketForTopicsError::TungesteniteError(err));
                    }
                },
            };

        if let Some(auth) = params.auth {
            let private_acquired_websockets = match acquire_websockets_for_private_topics(self.clone(), &mut inner, &auth, private_topics)
                .await {
                Ok(x) => x,
                Err(err) => match err {
                    AcquireWebsocketsForPrivateTopicsError::MaximumAmountOfTopicsForUserWillBeExceeded => {
                        return Err(AcquireWebsocketForTopicsError::MaximumAmountOfTopicsForUserWillBeExceeded);
                    }
                    AcquireWebsocketsForPrivateTopicsError::TungesteniteError(err) => {
                        return Err(AcquireWebsocketForTopicsError::TungesteniteError(err));
                    }
                    AcquireWebsocketsForPrivateTopicsError::CouldNotCreateDataStream(err) => {
                        return Err(AcquireWebsocketForTopicsError::CouldNotCreateDataStream(err));
                    }
                }
            };
            acquired_websockets.extend(private_acquired_websockets);
        }

        Ok(AcquireWebsocketsForTopicsOutput {
            websockets: acquired_websockets,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AcquireWebsocketsForPublicTopicsError {
    #[error("Tungestenite error: {0}")]
    TungesteniteError(#[from] tokio_tungstenite::tungstenite::Error),
}

async fn acquire_websockets_for_public_topics(
    this: Arc<MexcSpotWebsocketClient>,
    inner: &mut Inner,
    public_topics: Vec<Topic>,
) -> Result<Vec<AcquiredWebsocket>, AcquireWebsocketsForPublicTopicsError> {
    // Look for existing websockets that have a subscription to one or
    // more of these topics.

    let mut matching_websockets = vec![];
    for websocket_entry in inner.websockets.iter() {
        let topics = websocket_entry.topics.read().await;
        let topics_facilitated_by_websocket = topics
            .iter()
            .filter_map(|t| public_topics.contains(t).then(|| t.clone()))
            .collect::<Vec<_>>();
        if topics_facilitated_by_websocket.is_empty() {
            continue;
        }
        matching_websockets.push((websocket_entry.clone(), topics_facilitated_by_websocket));
    }

    let topics_not_covered_matching_websockets = public_topics
        .iter()
        .filter_map(|topic| {
            matching_websockets
                .iter()
                .all(|(_, topics)| !topics.contains(topic))
                .then(|| topic.clone())
        })
        .collect::<Vec<_>>();

    if topics_not_covered_matching_websockets.is_empty() {
        // We can reuse the websocket(s) that we found.
        return Ok(matching_websockets
            .into_iter()
            .map(|(ws_entry, topics)| AcquiredWebsocket {
                websocket_entry: ws_entry,
                for_topics: topics.iter().map(|t| (*t).clone()).collect::<Vec<Topic>>(),
            })
            .collect());
    }

    // Check whether one of the websockets have enough space to accommodate the topics.
    let mut websocket_that_can_accommodate = None;
    for websocket in inner.websockets.iter() {
        if websocket.auth.as_ref() == None
            && websocket.topics.read().await.len() + public_topics.len() <= 30
        {
            websocket_that_can_accommodate = Some(websocket.clone());
            break;
        }
    }

    if let Some(websocket_entry) = websocket_that_can_accommodate {
        // Check whether this websocket is part of the matching websockets via id
        if matching_websockets
            .iter()
            .any(|(matching_websocket, _)| matching_websocket.id == websocket_entry.id)
        {
            // This is an already matching websocket
            return Ok(matching_websockets
                .into_iter()
                .map(|(ws_entry, topics)| {
                    let mut for_topics =
                        topics.iter().map(|t| (*t).clone()).collect::<Vec<Topic>>();
                    if ws_entry.id == websocket_entry.id {
                        // Extend this websocket with the topics that are not yet covered
                        for_topics.extend(topics_not_covered_matching_websockets.iter().cloned());
                    }
                    AcquiredWebsocket {
                        websocket_entry: ws_entry,
                        for_topics,
                    }
                })
                .collect());
        } else {
            // This is another socket which we can put the topics onto
            return Ok([(
                websocket_entry.clone(),
                topics_not_covered_matching_websockets,
            )]
            .into_iter()
            .chain(matching_websockets.into_iter())
            .map(|(ws_entry, topics)| AcquiredWebsocket {
                websocket_entry: ws_entry.clone(),
                for_topics: topics.iter().map(|t| (*t).clone()).collect::<Vec<Topic>>(),
            })
            .collect());
        }
    }

    // Create new websocket for the topics
    let websocket_entry = match create_public_websocket(this.clone(), inner).await {
        Ok(x) => x,
        Err(err) => match err {
            CreatePublicWebsocketError::TungesteniteError(err) => {
                return Err(AcquireWebsocketsForPublicTopicsError::TungesteniteError(
                    err,
                ));
            }
        },
    };

    Ok([(websocket_entry, topics_not_covered_matching_websockets)]
        .into_iter()
        .chain(matching_websockets.into_iter())
        .map(|(ws_entry, topics)| AcquiredWebsocket {
            websocket_entry: ws_entry.clone(),
            for_topics: topics.iter().map(|t| (*t).clone()).collect::<Vec<Topic>>(),
        })
        .collect())
}

#[derive(Debug, thiserror::Error)]
pub enum AcquireWebsocketsForPrivateTopicsError {
    #[error("Maximum amount of topics for user will be exceeded")]
    MaximumAmountOfTopicsForUserWillBeExceeded,

    #[error("Tungestenite error: {0}")]
    TungesteniteError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Could not create datastream (listen key)")]
    CouldNotCreateDataStream(#[from] ApiError),
}

async fn acquire_websockets_for_private_topics(
    this: Arc<MexcSpotWebsocketClient>,
    inner: &mut Inner,
    auth: &WebsocketAuth,
    private_topics: Vec<Topic>,
) -> Result<Vec<AcquiredWebsocket>, AcquireWebsocketsForPrivateTopicsError> {
    // Look for existing websockets with the same auth, that have a subscription to one or more of
    // these topics.
    // If we find one, we can reuse it.
    // Otherwise, we have to set up a new websocket that we could subscribe/unsubscribe to.
    // We can assume once we find a topic for a websocket with the same auth, that there won't be
    // any other left
    let mut matching_websockets = vec![];
    for websocket_entry in inner.websockets.iter() {
        if websocket_entry.auth.as_ref() != Some(auth) {
            continue;
        }
        let topics = websocket_entry.topics.read().await;
        let topics_facilitated_by_websocket = topics
            .iter()
            .filter_map(|t| private_topics.contains(t).then(|| t.clone()))
            .collect::<Vec<_>>();
        if topics_facilitated_by_websocket.is_empty() {
            continue;
        }
        matching_websockets.push((websocket_entry.clone(), topics_facilitated_by_websocket));
    }

    let topics_not_covered_matching_websockets = private_topics
        .iter()
        .filter_map(|topic| {
            matching_websockets
                .iter()
                .all(|(_, topics)| !topics.contains(topic))
                .then(|| topic.clone())
        })
        .collect::<Vec<_>>();

    if topics_not_covered_matching_websockets.is_empty() {
        // We can reuse the websocket(s) that we found.
        return Ok(matching_websockets
            .into_iter()
            .map(|(ws_entry, topics)| AcquiredWebsocket {
                websocket_entry: ws_entry,
                for_topics: topics.iter().map(|t| (*t).clone()).collect::<Vec<Topic>>(),
            })
            .collect());
    }

    // Check whether one of the websockets have enough space to accommodate the topics.
    let mut websocket_that_can_accommodate = None;
    for websocket in inner.websockets.iter() {
        if websocket.auth.as_ref() == Some(auth)
            && websocket.topics.read().await.len() + private_topics.len() <= 30
        {
            websocket_that_can_accommodate = Some(websocket.clone());
            break;
        }
    }

    if let Some(websocket_entry) = websocket_that_can_accommodate {
        // Check whether this websocket is part of the matching websockets via id
        if matching_websockets
            .iter()
            .any(|(matching_websocket, _)| matching_websocket.id == websocket_entry.id)
        {
            // This is an already matching websocket
            return Ok(matching_websockets
                .into_iter()
                .map(|(ws_entry, topics)| {
                    let mut for_topics =
                        topics.iter().map(|t| (*t).clone()).collect::<Vec<Topic>>();
                    if ws_entry.id == websocket_entry.id {
                        // Extend this websocket with the topics that are not yet covered
                        for_topics.extend(topics_not_covered_matching_websockets.iter().cloned());
                    }
                    AcquiredWebsocket {
                        websocket_entry: ws_entry,
                        for_topics,
                    }
                })
                .collect());
        } else {
            // This is another socket which we can put the topics onto
            return Ok([(
                websocket_entry.clone(),
                topics_not_covered_matching_websockets,
            )]
            .into_iter()
            .chain(matching_websockets.into_iter())
            .map(|(ws_entry, topics)| AcquiredWebsocket {
                websocket_entry: ws_entry.clone(),
                for_topics: topics.iter().map(|t| (*t).clone()).collect::<Vec<Topic>>(),
            })
            .collect());
        }
    }

    // Create new websocket for the topics
    let websocket_entry = match create_private_websocket(this.clone(), inner, auth.clone()).await {
        Ok(x) => x,
        Err(err) => match err {
            CreatePrivateWebsocketError::MaximumAmountOfTopicsForUserWillBeExceeded => {
                return Err(AcquireWebsocketsForPrivateTopicsError::MaximumAmountOfTopicsForUserWillBeExceeded);
            }
            CreatePrivateWebsocketError::TungesteniteError(err) => {
                return Err(AcquireWebsocketsForPrivateTopicsError::TungesteniteError(
                    err,
                ));
            }
            CreatePrivateWebsocketError::CouldNotCreateDataStream(err) => {
                return Err(AcquireWebsocketsForPrivateTopicsError::CouldNotCreateDataStream(err));
            }
        },
    };

    Ok([(websocket_entry, topics_not_covered_matching_websockets)]
        .into_iter()
        .chain(matching_websockets.into_iter())
        .map(|(ws_entry, topics)| AcquiredWebsocket {
            websocket_entry: ws_entry.clone(),
            for_topics: topics.iter().map(|t| (*t).clone()).collect::<Vec<Topic>>(),
        })
        .collect())
}

#[derive(Debug, thiserror::Error)]
pub enum CreatePrivateWebsocketError {
    #[error("Maximum amount of topics for user will be exceeded")]
    MaximumAmountOfTopicsForUserWillBeExceeded,

    #[error("Tungestenite error: {0}")]
    TungesteniteError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Could not create datastream (listen key)")]
    CouldNotCreateDataStream(#[from] ApiError),
}

async fn create_private_websocket(
    this: Arc<MexcSpotWebsocketClient>,
    inner: &mut Inner,
    auth: WebsocketAuth,
) -> Result<Arc<WebsocketEntry>, CreatePrivateWebsocketError> {
    // Check whether we can create a new websocket for the topics
    let amount_of_websockets_for_auth = inner
        .websockets
        .iter()
        .filter(|websocket| websocket.auth.as_ref() == Some(&auth))
        .count();
    if amount_of_websockets_for_auth >= 5 {
        return Err(CreatePrivateWebsocketError::MaximumAmountOfTopicsForUserWillBeExceeded);
    }

    tracing::debug!("Creating listen key for private websocket...");
    let spot_client_with_auth = MexcSpotApiClientWithAuthentication::new(
        this.spot_api_endpoint.as_ref().clone(),
        auth.api_key.clone(),
        auth.secret_key.clone(),
    );
    let user_data_stream_output = spot_client_with_auth.create_user_data_stream().await?;
    tracing::debug!(
        "Listen key created: {}",
        &user_data_stream_output.listen_key
    );

    let endpoint_str = this.ws_endpoint.to_string();
    let ws_url = format!(
        "{}?listenKey={}",
        endpoint_str, &user_data_stream_output.listen_key
    );

    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (ws_tx, ws_rx) = ws_stream.split();
    let (tx, rx) = async_channel::unbounded();

    let cancellation_token = CancellationToken::new();

    let websocket_id = Uuid::new_v4();

    // Spawn all necessary tasks for this websocket...
    spawn_websocket_sender_task(
        this.clone(),
        ws_tx,
        rx,
        cancellation_token.clone(),
        websocket_id,
    );
    spawn_websocket_receiver_task(
        this.clone(),
        ws_rx,
        cancellation_token.clone(),
        websocket_id,
    );
    spawn_websocket_ping_task(
        this.clone(),
        tx.clone(),
        cancellation_token.clone(),
        websocket_id,
    );
    spawn_websocket_keepalive_task(
        this.clone(),
        spot_client_with_auth,
        user_data_stream_output.listen_key.clone(),
        cancellation_token.clone(),
        websocket_id,
    );

    inner
        .auth_to_listen_key_map
        .insert(auth.clone(), user_data_stream_output.listen_key.clone());

    let websocket_entry = WebsocketEntry {
        id: websocket_id,
        auth: Some(auth),
        listen_key: Some(user_data_stream_output.listen_key),
        topics: Arc::new(RwLock::new(vec![])),
        message_tx: Arc::new(RwLock::new(tx)),
    };
    let websocket_entry = Arc::new(websocket_entry);
    inner.websockets.push(websocket_entry.clone());

    Ok(websocket_entry)
}

#[derive(Debug, thiserror::Error)]
pub enum CreatePublicWebsocketError {
    #[error("Tungestenite error: {0}")]
    TungesteniteError(#[from] tokio_tungstenite::tungstenite::Error),
}

async fn create_public_websocket(
    this: Arc<MexcSpotWebsocketClient>,
    inner: &mut Inner,
) -> Result<Arc<WebsocketEntry>, CreatePublicWebsocketError> {
    let endpoint_str = this.ws_endpoint.to_string();

    let (ws_stream, _) = tokio_tungstenite::connect_async(&endpoint_str).await?;
    let (ws_tx, ws_rx) = ws_stream.split();
    let (tx, rx) = async_channel::unbounded();

    let cancellation_token = CancellationToken::new();

    let websocket_id = Uuid::new_v4();

    // Spawn all necessary tasks for this websocket...
    spawn_websocket_sender_task(
        this.clone(),
        ws_tx,
        rx,
        cancellation_token.clone(),
        websocket_id,
    );
    spawn_websocket_receiver_task(
        this.clone(),
        ws_rx,
        cancellation_token.clone(),
        websocket_id,
    );
    spawn_websocket_ping_task(
        this.clone(),
        tx.clone(),
        cancellation_token.clone(),
        websocket_id,
    );

    let websocket_entry = WebsocketEntry {
        id: websocket_id,
        auth: None,
        listen_key: None,
        topics: Arc::new(RwLock::new(vec![])),
        message_tx: Arc::new(RwLock::new(tx)),
    };
    let websocket_entry = Arc::new(websocket_entry);
    inner.websockets.push(websocket_entry.clone());

    Ok(websocket_entry)
}

#[derive(Debug, thiserror::Error)]
pub enum ReconnectWebsocketError {
    #[error("Unknown websocket")]
    UnknownWebsocket,

    #[error("Tungestenite error: {0}")]
    TungesteniteError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Resubscribe send error")]
    ResubscribeSendError(#[from] async_channel::SendError<SendableMessage>),
}

async fn reconnect_websocket(
    this: Arc<MexcSpotWebsocketClient>,
    websocket_id: Uuid,
) -> Result<(), ReconnectWebsocketError> {
    tracing::debug!("Reconnecting websocket with id...: {}", websocket_id);
    let inner = this.inner.read().await;
    let websocket = inner
        .websockets
        .iter()
        .find(|ws| ws.id == websocket_id)
        .ok_or(ReconnectWebsocketError::UnknownWebsocket)?;

    let endpoint_str = this.ws_endpoint.to_string();
    let ws_url = match &websocket.listen_key {
        Some(listen_key) => {
            format!("{}?listenKey={}", endpoint_str, listen_key)
        }
        None => endpoint_str,
    };

    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;

    tracing::debug!("Reconnected websocket with id: {}", websocket_id);

    let (ws_tx, ws_rx) = ws_stream.split();
    let (tx, rx) = async_channel::unbounded();

    let cancellation_token = CancellationToken::new();

    // Spawn all necessary tasks for this websocket...
    spawn_websocket_sender_task(
        this.clone(),
        ws_tx,
        rx,
        cancellation_token.clone(),
        websocket_id,
    );
    spawn_websocket_receiver_task(
        this.clone(),
        ws_rx,
        cancellation_token.clone(),
        websocket_id,
    );
    spawn_websocket_ping_task(
        this.clone(),
        tx.clone(),
        cancellation_token.clone(),
        websocket_id,
    );
    if let Some(listen_key) = &websocket.listen_key {
        let spot_client_with_auth = MexcSpotApiClientWithAuthentication::new(
            this.spot_api_endpoint.as_ref().clone(),
            websocket
                .auth
                .as_ref()
                .expect("Listen key set but not auth?")
                .api_key
                .clone(),
            websocket
                .auth
                .as_ref()
                .expect("Listen key set but not auth?")
                .secret_key
                .clone(),
        );
        spawn_websocket_keepalive_task(
            this.clone(),
            spot_client_with_auth,
            listen_key.clone(),
            cancellation_token.clone(),
            websocket_id,
        );
    }

    let mut message_tx = websocket.message_tx.write().await;
    *message_tx = tx;

    let topics = websocket.topics.read().await;
    if !topics.is_empty() {
        let topic_strs = topics
            .iter()
            .map(|topic| topic.to_topic_subscription_string())
            .collect();
        let sendable_message = SendableMessage::Subscription(topic_strs);
        message_tx.send(sendable_message).await?;

        tracing::debug!(
            "Resubscribed to all topics for websocket with id: {}",
            websocket_id
        );
    }

    Ok(())
}

fn spawn_websocket_sender_task(
    this: Arc<MexcSpotWebsocketClient>,
    mut ws_tx: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    rx: async_channel::Receiver<SendableMessage>,
    cancellation_token: CancellationToken,
    websocket_id: Uuid,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }
                message_result = rx.recv() => {
                    let message = match message_result {
                        Ok(x) => x,
                        Err(err) => {
                            cancellation_token.cancel();
                            tracing::error!("Error receiving message from channel: {}", err);
                            break;
                        }
                    };
                    let json = serde_json::to_string(&message).expect("Failed to serialize message");
                    let message = Message::Text(json);

                    match ws_tx.send(message).await {
                        Ok(_) => {}
                        Err(err) => match err {
                            Error::ConnectionClosed => {
                                cancellation_token.cancel();
                                tracing::error!("Failed to send message to websocket because the connection was closed");
                                if let Err(err) = reconnect_websocket(this.clone(), websocket_id).await {
                                    tracing::error!("Failed to reconnect websocket: {}", err);
                                }
                                break;
                            }
                            Error::AlreadyClosed => {
                                cancellation_token.cancel();
                                tracing::error!("Failed to send message to websocket because the connection was already closed");
                                if let Err(err) = reconnect_websocket(this.clone(), websocket_id).await {
                                    tracing::error!("Failed to reconnect websocket: {}", err);
                                }
                                break;
                            }
                            Error::Protocol(protocol_err) => match protocol_err {
                                ProtocolError::ResetWithoutClosingHandshake => {
                                    cancellation_token.cancel();
                                    tracing::error!("Failed to send message to websocket because the connection was reset without closing handshake");
                                    if let Err(err) = reconnect_websocket(this.clone(), websocket_id).await {
                                        tracing::error!("Failed to reconnect websocket: {}", err);
                                    }
                                    break;
                                }
                                _ => {
                                    cancellation_token.cancel();
                                    tracing::error!(
                                        "Protocol error sending message to websocket: {}",
                                        protocol_err
                                    );
                                    break;
                                }
                            },
                            _ => {
                                cancellation_token.cancel();
                                tracing::error!("Error sending message to websocket: {}", err);
                                break;
                            }
                        },
                    }
                }
            }
        }
    });
}

fn spawn_websocket_receiver_task(
    this: Arc<MexcSpotWebsocketClient>,
    mut ws_rx: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    cancellation_token: CancellationToken,
    websocket_id: Uuid,
) {
    let broadcast_tx = this.broadcast_tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }
                message_result_opt = ws_rx.next() => {
                    let message_result = match message_result_opt {
                        Some(x) => x,
                        None => {
                            cancellation_token.cancel();
                            break;
                        }
                    };
                    let message = match message_result {
                        Ok(message) => message,
                        Err(err) => match err {
                            Error::ConnectionClosed => {
                                cancellation_token.cancel();
                                tracing::error!("Failed to receive message from websocket because the connection was closed");
                                if let Err(err) = reconnect_websocket(this.clone(), websocket_id).await {
                                    tracing::error!("Failed to reconnect websocket: {}", err);
                                }
                                break;
                            }
                            Error::AlreadyClosed => {
                                cancellation_token.cancel();
                                tracing::error!("Failed to receive message from websocket because the connection was already closed");
                                if let Err(err) = reconnect_websocket(this.clone(), websocket_id).await {
                                    tracing::error!("Failed to reconnect websocket: {}", err);
                                }
                                break;
                            }
                            Error::Protocol(protocol_err) => match protocol_err {
                                ProtocolError::ResetWithoutClosingHandshake => {
                                    cancellation_token.cancel();
                                    tracing::error!("Failed to receive message from websocket because the connection was reset without closing handshake");
                                    if let Err(err) = reconnect_websocket(this.clone(), websocket_id).await {
                                        tracing::error!("Failed to reconnect websocket: {}", err);
                                    }
                                    break;
                                }
                                _ => {
                                    cancellation_token.cancel();
                                    tracing::error!(
                                        "Protocol error receiving message from websocket: {}",
                                        protocol_err
                                    );
                                    break;
                                }
                            },
                            _ => {
                                cancellation_token.cancel();
                                tracing::error!("Error receiving message from websocket: {}", err);
                                break;
                            }
                        },
                    };

                    let text = match message {
                        Message::Text(text) => text,
                        _ => {
                            tracing::debug!("Received non-text message: {:?}", message);
                            continue;
                        }
                    };

                    let raw_message = match serde_json::from_str::<message::RawMessage>(&text) {
                        Ok(x) => x,
                        Err(err) => {
                            cancellation_token.cancel();
                            tracing::error!("Failed to deserialize message: {}\njson: {}", err, &text);
                            break;
                        }
                    };
                    let mexc_message_result: Result<message::Message, ()> = (&raw_message).try_into();
                    let Ok(mexc_message) = mexc_message_result else {
                        continue;
                    };

                    match broadcast_tx.broadcast(Arc::new(mexc_message)).await {
                        Ok(_) => {}
                        Err(err) => {
                            cancellation_token.cancel();
                            tracing::error!("Failed to broadcast message: {}", err);
                            break;
                        }
                    }
                }
            }
        }
    });
}

fn spawn_websocket_ping_task(
    this: Arc<MexcSpotWebsocketClient>,
    sender: Sender<SendableMessage>,
    cancellation_token: CancellationToken,
    websocket_id: Uuid,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                    match sender.send(SendableMessage::Ping).await {
                        Ok(_) => {}
                        Err(err) => {
                            cancellation_token.cancel();
                            tracing::error!("Failed to send ping: {}", err);
                            break;
                        }
                    }
                }
            }
        }
    });
}

fn spawn_websocket_keepalive_task(
    this: Arc<MexcSpotWebsocketClient>,
    spot_client_with_auth: MexcSpotApiClientWithAuthentication,
    listen_key: String,
    cancellation_token: CancellationToken,
    websocket_id: Uuid,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(60 * 30)) => {
                    match spot_client_with_auth
                        .keep_alive_user_data_stream(KeepAliveUserDataStreamParams {
                            listen_key: &listen_key,
                        })
                        .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            cancellation_token.cancel();
                            tracing::error!("Failed to keep alive user data stream: {}", err);
                            break;
                        }
                    }
                }
            }
        }
    });
}
