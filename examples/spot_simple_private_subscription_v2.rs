use dotenv::dotenv;
use mexc_rs::spot::wsv2::auth::WebsocketAuth;
use mexc_rs::spot::wsv2::subscribe::{Subscribe, SubscribeParams};
use mexc_rs::spot::wsv2::topic::Topic;
use mexc_rs::spot::wsv2::MexcSpotWebsocketClient;

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
        .subscribe(
            SubscribeParams::default()
                .with_auth(websocket_auth)
                .with_topics(vec![Topic::AccountUpdate]),
        )
        .await
        .expect("Failed to subscribe");

    // let mut stream = private_ws_client.stream();
    // while let Some(message) = stream.next().await {
    //     match message.as_ref() {
    //         PrivateMexcSpotWsMessage::AccountUpdate(account_update_message) => {
    //             tracing::info!("Account update: {:#?}", account_update_message);
    //         }
    //         _ => {},
    //     }
    // }
}
