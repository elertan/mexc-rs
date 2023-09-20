use futures::StreamExt;
use mexc_rs::ws::subscription::{
    SpotDealsSubscriptionRequest, Subscribe, SubscribeParams, SubscriptionRequest,
};
use mexc_rs::ws::MexcWsClient;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "mexc_rs=trace");
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
    };
    let subscribe_output = ws_client
        .subscribe(subscribe_params)
        .await
        .expect("Failed to subscribe");
    tracing::info!("{:?}", subscribe_output);

    let message_result = ws_client.stream().next().await;
    tracing::info!("{:?}", message_result);

    assert!(message_result.is_some());
    todo!()
}
