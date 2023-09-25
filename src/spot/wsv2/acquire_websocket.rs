use crate::spot::v3::create_user_data_stream::CreateUserDataStreamEndpoint;
use crate::spot::v3::ApiError;
use crate::spot::wsv2::auth::WebsocketAuth;
use crate::spot::wsv2::endpoint::MexcWebsocketEndpoint;
use crate::spot::wsv2::topic::Topic;
use crate::spot::wsv2::{Inner, MexcWebsocketClient, WebsocketEntry};
use crate::spot::{MexcSpotApiClientWithAuthentication, MexcSpotApiEndpoint};
use async_trait::async_trait;
use std::sync::Arc;

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
}

#[async_trait]
pub(crate) trait AcquireWebsocketsForTopics {
    async fn acquire_websockets_for_topics(
        self: Arc<Self>,
        params: AcquireWebsocketsForTopicsParams,
    ) -> Result<AcquireWebsocketsForTopicsOutput, AcquireWebsocketForTopicsError>;
}

#[async_trait]
impl AcquireWebsocketsForTopics for MexcWebsocketClient {
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

        if let Some(auth) = params.auth {
            acquire_websockets_for_private_topics(self.clone(), &mut inner, &auth, private_topics)
                .await;
        }

        // if let Some(auth) = params.auth {
        //     let listen_key = inner.auth_to_listen_key_map.get(&auth);
        // } else {}

        todo!()
    }
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
    this: Arc<MexcWebsocketClient>,
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
    let matching_websockets = inner
        .websockets
        .iter()
        .filter_map(|websocket_entry| {
            // If this websocket is not for authenticated topics, we can ignore it.
            if websocket_entry.auth != *auth {
                return None;
            }
            // If this websocket is for the same user (via auth)
            // see what topics are facilitated by this websocket.
            let topics_facilitated_by_websocket = websocket_entry
                .topics
                .iter()
                .filter_map(|t| private_topics.contains(t).then(|| t.clone()))
                .collect::<Vec<_>>();
            Some((websocket_entry.clone(), topics_facilitated_by_websocket))
        })
        .collect::<Vec<_>>();

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
    let websocket_that_can_accommodate = inner.websockets.iter().find(|websocket| {
        websocket.auth == *auth && websocket.topics.len() + private_topics.len() <= 30
    });

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
                        for_topics.extend(
                            topics_not_covered_matching_websockets
                                .iter()
                                .map(|t| t.clone()),
                        );
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
    this: Arc<MexcWebsocketClient>,
    inner: &mut Inner,
    auth: WebsocketAuth,
) -> Result<Arc<WebsocketEntry>, CreatePrivateWebsocketError> {
    // let (ws, _) = match auth {
    //     None => {
    //         tokio_tungstenite::connect_async(ws_endpoint.as_ref()).await?
    //     }
    //     Some(auth) => {
    //     }
    // };
    //
    // Check whether we can create a new websocket for the topics
    let amount_of_websockets_for_auth = inner
        .websockets
        .iter()
        .filter(|websocket| websocket.auth == auth)
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

    // Spawn all necessary tasks for this websocket...

    inner
        .auth_to_listen_key_map
        .insert(auth.clone(), user_data_stream_output.listen_key.clone());

    let websocket_entry = WebsocketEntry {
        id: uuid::Uuid::new_v4(),
        auth,
        listen_key: Some(user_data_stream_output.listen_key),
        topics: vec![],
    };
    let websocket_entry = Arc::new(websocket_entry);
    inner.websockets.push(websocket_entry.clone());

    Ok(websocket_entry)
}