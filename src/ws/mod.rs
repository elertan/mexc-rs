use std::pin::Pin;
use std::task::{Context, Poll};
use futures::{Stream, StreamExt};
use futures::stream::BoxStream;
use crate::ws::spot_deals::SpotDealsMessage;

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

pub struct MexcWsClient {
    endpoint: MexcWsEndpoint,
    message_sender: async_broadcast::Sender<MexcWsMessage>,
    message_receiver: async_broadcast::Receiver<MexcWsMessage>,
}

impl MexcWsClient {
    pub fn new(endpoint: MexcWsEndpoint) -> Self {
        let (message_sender, message_receiver) = async_broadcast::broadcast(1000000);

        Self { endpoint, message_sender, message_receiver }
    }

    pub fn stream(&self) -> BoxStream<MexcWsMessage> {
        let mut mr = self.message_receiver.clone();
        let stream = async_stream::stream! {
            while let Ok(message) = mr.recv().await {
                yield message;
            }
        };
        stream.boxed()
    }
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

#[derive(Debug, Clone)]
pub enum MexcWsMessage {
    SpotDeals(SpotDealsMessage),
}
