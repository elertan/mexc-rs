use dotenv::dotenv;
use futures::StreamExt;
use tracing_subscriber::util::SubscriberInitExt;
use mexc_rs::spot::{MexcSpotApiClientWithAuthentication, MexcSpotApiEndpoint};
use mexc_rs::spot::ws::MexcSpotWsEndpoint;
use mexc_rs::spot::ws::private::{MexcSpotPrivateWsClient, PrivateMexcSpotWsMessage};
use mexc_rs::spot::ws::private::subscription::{PrivateSubscriptionTopic, PrivateSubscribe, PrivateSubscribeParams, PrivateUnsubscribeParams, PrivateUnsubscribe};

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "mexc_rs=debug,spot_private_subscription_and_unsubscribe=trace");
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let topics = vec![
        PrivateSubscriptionTopic::AccountUpdate,
        PrivateSubscriptionTopic::AccountDeals,
        PrivateSubscriptionTopic::AccountOrders,
    ];

    let client =
        MexcSpotApiClientWithAuthentication::new(MexcSpotApiEndpoint::Base, api_key, secret_key);
    let private_ws_client = MexcSpotPrivateWsClient::new(MexcSpotWsEndpoint::Base, client);

    tracing::info!("Subscribing to topics...");
    let subscribe_params = PrivateSubscribeParams {
        subscription_topics: topics.clone(),
        wait_for_confirmation: None,
    };
    private_ws_client
        .private_subscribe(subscribe_params)
        .await
        .expect("Failed to subscribe");

    let topics = private_ws_client.get_subscription_topics().await;
    tracing::info!("Subscribed to topics: {:#?}", topics);

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    tracing::info!("Unsubscribing from topics...");

    let unsubscribe_params = PrivateUnsubscribeParams {
        subscription_topics: topics.clone(),
        wait_for_confirmation: None,
    };
    private_ws_client
        .private_unsubscribe(unsubscribe_params)
        .await
        .expect("Failed to unsubscribe");

    let topics = private_ws_client.get_subscription_topics().await;
    tracing::info!("Subscribed to topics: {:#?}", topics);

    tracing::info!("Done!");
}
