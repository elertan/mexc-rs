use rust_decimal::Decimal;
use dotenv::dotenv;
use mexc_rs::spot::v3::cancel_order::{CancelOrderEndpoint, CancelOrderParams};
use mexc_rs::spot::v3::enums::{OrderSide, OrderType};
use mexc_rs::spot::v3::order::{OrderEndpoint, OrderParams};
use mexc_rs::spot::{MexcSpotApiClientWithAuthentication, MexcSpotApiEndpoint};
use std::str::FromStr;
use mexc_rs::spot::v3::create_user_data_stream::CreateUserDataStreamEndpoint;
use mexc_rs::spot::v3::keep_alive_user_data_stream::{KeepAliveUserDataStreamEndpoint, KeepAliveUserDataStreamParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "mexc_rs=debug,spot_keep_alive_user_data_stream=trace");
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let client =
        MexcSpotApiClientWithAuthentication::new(MexcSpotApiEndpoint::Base, api_key, secret_key);

    let output = client.create_user_data_stream().await?;
    tracing::info!("Listen key: {}", &output.listen_key);

    tracing::info!("Waiting a bit... (usually should be like 30mins)");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    tracing::info!("Sending keep alive...");
    let output = client.keep_alive_user_data_stream(KeepAliveUserDataStreamParams {
        listen_key: &output.listen_key
    }).await?;
    tracing::info!("Keep alive response: {:?}", &output);

    tracing::info!("Done!");

    Ok(())
}
