use crate::spot::wsv2::auth::WebsocketAuth;
use crate::spot::wsv2::endpoint::MexcWebsocketEndpoint;
use crate::spot::wsv2::topic::Topic;
use crate::spot::MexcSpotApiEndpoint;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod acquire_websocket;
pub mod auth;
pub mod endpoint;
pub mod subscribe;
pub mod topic;
pub mod unsubscribe;

#[derive(Debug)]
pub struct WebsocketEntry {
    pub id: Uuid,
    pub auth: Option<WebsocketAuth>,
    pub listen_key: Option<String>,
    pub topics: Vec<Topic>,
    pub message_tx: async_channel::Sender<SendableMessage>,
}

#[derive(Debug)]
struct Inner {
    pub auth_to_listen_key_map: HashMap<WebsocketAuth, String>,
    pub websockets: Vec<Arc<WebsocketEntry>>,
}

#[derive(Debug, Clone)]
pub struct MexcSpotWebsocketClient {
    ws_endpoint: Arc<MexcWebsocketEndpoint>,
    spot_api_endpoint: Arc<MexcSpotApiEndpoint>,
    inner: Arc<RwLock<Inner>>,
}

impl MexcSpotWebsocketClient {
    pub fn new_with_endpoints(
        ws_endpoint: MexcWebsocketEndpoint,
        spot_api_endpoint: MexcSpotApiEndpoint,
    ) -> Self {
        Self {
            ws_endpoint: Arc::new(ws_endpoint),
            spot_api_endpoint: Arc::new(spot_api_endpoint),
            inner: Arc::new(RwLock::new(Inner {
                auth_to_listen_key_map: HashMap::new(),
                websockets: Vec::new(),
            })),
        }
    }

    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

impl Default for MexcSpotWebsocketClient {
    fn default() -> Self {
        Self::new_with_endpoints(MexcWebsocketEndpoint::Base, MexcSpotApiEndpoint::Base)
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(
    tag = "method",
    content = "params",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum SendableMessage {
    Subscription(String),
    Unsubscription(String),
    Ping,
}
