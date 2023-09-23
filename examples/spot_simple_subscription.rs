use futures::StreamExt;
use mexc_rs::spot::ws::public::subscription::{
    PublicSpotDealsSubscriptionRequest, Subscribe, SubscribeParams, PublicSubscriptionRequest,
};
use mexc_rs::spot::ws::public::{MexcSpotPublicWsClient, PublicMexcSpotWsMessage};
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "mexc_rs=trace,spot_simple_subscription=trace");
    tracing_subscriber::fmt::init();

    let ws_client = MexcSpotPublicWsClient::default();
    let subscribe_params = SubscribeParams {
        subscription_requests: vec![
            PublicSubscriptionRequest::SpotDeals(PublicSpotDealsSubscriptionRequest {
                symbol: "BTCUSDT".to_string(),
            }),
            PublicSubscriptionRequest::SpotDeals(PublicSpotDealsSubscriptionRequest {
                symbol: "KASUSDT".to_string(),
            }),
        ],
        wait_for_confirmation: None,
    };
    ws_client
        .subscribe(subscribe_params)
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
        }
    }
}
