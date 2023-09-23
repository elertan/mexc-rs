use futures::StreamExt;
use mexc_rs::spot::ws::public::subscription::{PublicSpotDealsSubscriptionTopic, PublicSpotKlineSubscriptionTopic, PublicSubscribe, PublicSubscribeParams, PublicSubscriptionTopic};
use mexc_rs::spot::ws::public::{MexcSpotPublicWsClient, PublicMexcSpotWsMessage};
use tracing_subscriber::util::SubscriberInitExt;
use mexc_rs::spot::ws::public::kline::KlineIntervalSubscription;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "mexc_rs=debug,spot_simple_subscription=trace");
    tracing_subscriber::fmt::init();

    let ws_client = MexcSpotPublicWsClient::default();
    let subscribe_params = PublicSubscribeParams {
        subscription_topics: vec![
            PublicSubscriptionTopic::SpotDeals(PublicSpotDealsSubscriptionTopic {
                symbol: "BTCUSDT".to_string(),
            }),
            PublicSubscriptionTopic::SpotDeals(PublicSpotDealsSubscriptionTopic {
                symbol: "KASUSDT".to_string(),
            }),
            PublicSubscriptionTopic::SpotKline(PublicSpotKlineSubscriptionTopic {
                symbol: "KASUSDT".to_string(),
                interval: KlineIntervalSubscription::OneMinute,
            }),
        ],
        wait_for_confirmation: None,
    };
    ws_client
        .public_subscribe(subscribe_params)
        .await
        .expect("Failed to subscribe");

    let mut stream = ws_client.stream();
    while let Some(message) = stream.next().await {
        match message.as_ref() {
            PublicMexcSpotWsMessage::SpotDeals(spot_deals_message) => {
                for deal in &spot_deals_message.deals {
                    tracing::info!(
                        "Spot deal for '{}' at price {} with quantity {} at {}",
                        deal.symbol,
                        deal.price,
                        deal.quantity,
                        deal.timestamp.format("%a %b %e %T %Y")
                    )
                }
            }
            PublicMexcSpotWsMessage::SpotKline(spot_kline_message) => {
                tracing::info!(
                    "Spot kline for '{}' with closing price {} at {}",
                    spot_kline_message.symbol,
                    spot_kline_message.close,
                    spot_kline_message.start_time.format("%a %b %e %T %Y")
                );
            }
            _ => {}
        }
    }
}
