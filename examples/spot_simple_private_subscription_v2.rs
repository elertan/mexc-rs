use dotenv::dotenv;
use futures::StreamExt;
use mexc_rs::spot::ws::auth::WebsocketAuth;
use mexc_rs::spot::ws::message::kline::KlineIntervalTopic;
use mexc_rs::spot::ws::stream::Stream;
use mexc_rs::spot::ws::subscribe::{Subscribe, SubscribeParams};
use mexc_rs::spot::ws::topic::{DealsTopic, KlineTopic, Topic};
use mexc_rs::spot::ws::MexcSpotWebsocketClient;

#[tokio::main]
async fn main() {
    std::env::set_var(
        "RUST_LOG",
        "mexc_rs=debug,spot_simple_private_subscription=trace",
    );
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let websocket_auth = WebsocketAuth::new(api_key, secret_key);

    let ws_client = MexcSpotWebsocketClient::default().into_arc();
    ws_client
        .clone()
        .subscribe(
            SubscribeParams::default()
                .with_auth(websocket_auth)
                .with_topics(vec![
                    Topic::AccountUpdate,
                    Topic::AccountOrders,
                    Topic::AccountDeals,
                    Topic::Kline(KlineTopic::new(
                        "BTCUSDT".to_string(),
                        KlineIntervalTopic::OneMinute,
                    )),
                    Topic::Kline(KlineTopic::new(
                        "KASUSDT".to_string(),
                        KlineIntervalTopic::OneMinute,
                    )),
                    Topic::Deals(DealsTopic::new("KASUSDT".to_string())),
                ]),
        )
        .await
        .expect("Failed to subscribe");

    let mut stream = ws_client.stream();
    while let Some(message) = stream.next().await {
        dbg!(&message);
    }
}
