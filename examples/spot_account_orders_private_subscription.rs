use dotenv::dotenv;
use futures::StreamExt;
use tracing_subscriber::util::SubscriberInitExt;
use mexc_rs::spot::{MexcSpotApiClientWithAuthentication, MexcSpotApiEndpoint};
use mexc_rs::spot::ws::MexcSpotWsEndpoint;
use mexc_rs::spot::ws::private::{MexcSpotPrivateWsClient, PrivateMexcSpotWsMessage};
use mexc_rs::spot::ws::private::subscription::{PrivateSubscriptionTopic, PrivateSubscribe, PrivateSubscribeParams};

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "mexc_rs=debug,spot_account_orders_private_subscription=trace");
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let client =
        MexcSpotApiClientWithAuthentication::new(MexcSpotApiEndpoint::Base, api_key, secret_key);
    let private_ws_client = MexcSpotPrivateWsClient::new(MexcSpotWsEndpoint::Base, client);
    let subscribe_params = PrivateSubscribeParams {
        subscription_topics: vec![
            PrivateSubscriptionTopic::AccountDeals,
            PrivateSubscriptionTopic::AccountOrders,
        ],
        wait_for_confirmation: None,
    };
    private_ws_client
        .private_subscribe(subscribe_params)
        .await
        .expect("Failed to subscribe");

    let mut stream = private_ws_client.stream();
    while let Some(message) = stream.next().await {
        match message.as_ref() {
            PrivateMexcSpotWsMessage::AccountDeals(account_deals) => {
                tracing::info!("Account deals: {:#?}", account_deals);
            }
            PrivateMexcSpotWsMessage::AccountOrders(account_orders) => {
                tracing::info!("Account orders: {:#?}", account_orders);
            }
            _ => {},
        }
    }
}
