use std::sync::Arc;
use async_trait::async_trait;
use crate::spot::wsv2::acquire_websocket::{AcquireWebsocketsForTopics, AcquireWebsocketForTopicsError, AcquireWebsocketsForTopicsOutput, AcquireWebsocketsForTopicsParams};
use crate::spot::wsv2::auth::WebsocketAuth;
use crate::spot::wsv2::MexcWebsocketClient;
use crate::spot::wsv2::topic::Topic;

#[derive(Debug)]
pub struct SubscribeParams {
    pub auth: Option<WebsocketAuth>,
    pub topics: Vec<Topic>,
    pub wait_for_confirmation: bool,
}

impl Default for SubscribeParams {
    fn default() -> Self {
        Self::new(None, Vec::new(), true)
    }
}

impl SubscribeParams {
    pub fn new(auth: Option<WebsocketAuth>, topics: Vec<Topic>, wait_for_confirmation: bool) -> Self {
        Self {
            auth,
            topics,
            wait_for_confirmation,
        }
    }

    pub fn with_auth(mut self, auth: WebsocketAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn with_topic(mut self, topic: Topic) -> Self {
        self.topics.push(topic);
        self
    }

    pub fn with_topics(mut self, topics: Vec<Topic>) -> Self {
        self.topics.extend(topics);
        self
    }

    pub fn with_wait_for_confirmation(mut self, wait_for_confirmation: bool) -> Self {
        self.wait_for_confirmation = wait_for_confirmation;
        self
    }
}

#[derive(Debug)]
pub struct SubscribeOutput {}

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
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
pub trait Subscribe {
    async fn subscribe(self: Arc<Self>, params: SubscribeParams) -> Result<SubscribeOutput, SubscribeError>;
}

#[async_trait]
impl Subscribe for MexcWebsocketClient {
    async fn subscribe(self: Arc<Self>, params: SubscribeParams) -> Result<SubscribeOutput, SubscribeError> {
        let mut acquire_websocket_params = AcquireWebsocketsForTopicsParams::default()
            .for_topics(params.topics);
        if let Some(auth) = params.auth {
            acquire_websocket_params = acquire_websocket_params.with_auth(auth);
        }
        let acquire_output = match self.acquire_websockets_for_topics(acquire_websocket_params).await {
            Ok(x) => x,
            Err(err) => match err {
                AcquireWebsocketForTopicsError::MaximumAmountOfTopicsForUserWillBeExceeded => {
                    return Err(SubscribeError::MaximumAmountOfTopicsForUserWillBeExceeded);
                }
                AcquireWebsocketForTopicsError::RequestedTopicsRequireAuthentication => {
                    return Err(SubscribeError::RequestedTopicsRequireAuthentication);
                }
            }
        };

        todo!()
    }
}
