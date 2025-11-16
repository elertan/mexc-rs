use dotenv::dotenv;
use futures::StreamExt;
use mexc_rs::spot::ws::stream::Stream;
use mexc_rs::spot::ws::subscribe::{Subscribe, SubscribeParams};
use mexc_rs::spot::ws::topic::{DepthTopic, Topic};
use mexc_rs::spot::ws::MexcSpotWebsocketClient;

#[tokio::main]
async fn main() {
    std::env::set_var(
        "RUST_LOG",
        "mexc_rs=debug,spot_simple_private_subscription=trace",
    );
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let ws_client = MexcSpotWebsocketClient::default().into_arc();
    ws_client
        .clone()
        .subscribe(
            SubscribeParams::default()
                .with_topics(vec![Topic::Depth(DepthTopic::new("BTCUSDT".to_string()))]),
        )
        .await
        .expect("Failed to subscribe");

    let mut stream = ws_client.stream();
    while let Some(message) = stream.next().await {
        dbg!(&message);
    }
}
