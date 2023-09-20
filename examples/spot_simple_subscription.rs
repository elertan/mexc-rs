use futures::StreamExt;
use mexc_rs::spot::ws::subscription::{
    SpotDealsSubscriptionRequest, Subscribe, SubscribeParams, SubscriptionRequest,
};
use mexc_rs::spot::ws::{MexcWsClient, MexcWsMessage};
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "mexc_rs=trace,spot_simple_subscription=trace");
    tracing_subscriber::fmt::init();

    let ws_client = MexcWsClient::default();
    let subscribe_params = SubscribeParams {
        subscription_requests: vec![
            SubscriptionRequest::SpotDeals(SpotDealsSubscriptionRequest {
                symbol: "BTCUSDT".to_string(),
            }),
            SubscriptionRequest::SpotDeals(SpotDealsSubscriptionRequest {
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
            MexcWsMessage::SpotDeals(spot_deals_message) => {
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
