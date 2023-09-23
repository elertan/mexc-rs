use futures::StreamExt;
use mexc_rs::spot::ws::public::subscription::{PublicSpotDealsSubscriptionTopic, PublicSubscribe, PublicSubscribeParams, PublicSubscriptionTopic, PublicUnsubscribe, PublicUnsubscribeParams};
use mexc_rs::spot::ws::public::{MexcSpotPublicWsClient};
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "mexc_rs=debug,spot_subscription_and_unsubscribe=trace");
    tracing_subscriber::fmt::init();

    let topics = vec![
        PublicSubscriptionTopic::SpotDeals(PublicSpotDealsSubscriptionTopic {
            symbol: "BTCUSDT".to_string(),
        }),
        PublicSubscriptionTopic::SpotDeals(PublicSpotDealsSubscriptionTopic {
            symbol: "KASUSDT".to_string(),
        }),
    ];

    tracing::info!("Subscribing to topics...");
    let ws_client = MexcSpotPublicWsClient::default();
    let subscribe_params = PublicSubscribeParams {
        subscription_topics: topics.clone(),
        wait_for_confirmation: None,
    };
    ws_client
        .public_subscribe(subscribe_params)
        .await
        .expect("Failed to subscribe");

    let topics = ws_client.get_subscription_topics().await;
    tracing::info!("Subscribed to topics: {:#?}", topics);

    // wait 5 seconds
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    tracing::info!("Unsubscribing from topics...");
    let unsubscribe_params = PublicUnsubscribeParams {
        subscription_topics: topics,
        wait_for_confirmation: None,
    };
    ws_client
        .public_unsubscribe(unsubscribe_params)
        .await
        .expect("Failed to unsubscribe");

    let topics = ws_client.get_subscription_topics().await;
    tracing::info!("Subscribed to topics: {:#?}", topics);

    tracing::info!("Done!");
}
