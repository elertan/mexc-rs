use crate::spot::ws::message::Message;
use crate::spot::ws::MexcSpotWebsocketClient;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::sync::Arc;

pub trait Stream {
    fn stream<'a>(self: Arc<Self>) -> BoxStream<'a, Arc<Message>>;
}

impl Stream for MexcSpotWebsocketClient {
    fn stream<'a>(self: Arc<Self>) -> BoxStream<'a, Arc<Message>> {
        let mut rx = self.broadcast_rx.clone();
        let stream = async_stream::stream! {
            while let Ok(message) = rx.recv().await {
                yield message;
            }
        };
        stream.boxed()
    }
}
