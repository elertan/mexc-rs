use dotenv::dotenv;
use mexc_rs::spot::v3::cancel_order::{CancelOrderEndpoint, CancelOrderParams};
use mexc_rs::spot::v3::enums::{OrderSide, OrderType};
use mexc_rs::spot::v3::order::{OrderEndpoint, OrderParams};
use mexc_rs::spot::{MexcSpotApiClientWithAuthentication, MexcSpotApiEndpoint};
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "mexc_rs=debug,spot_create_order=trace");
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let client =
        MexcSpotApiClientWithAuthentication::new(MexcSpotApiEndpoint::Base, api_key, secret_key);

    // Order needs to be at least 5 USDT
    // Order low enough to never be filled
    let order_params = OrderParams {
        symbol: "KASUSDT",
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: Some(Decimal::from_str("5000")?),
        quote_order_quantity: None,
        price: Some(Decimal::from_str("0.001")?),
        new_client_order_id: None,
    };
    let order_output = client.order(order_params).await?;
    tracing::info!("{:?}", &order_output);

    tracing::info!("Waiting for 3 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let cancel_order_params = CancelOrderParams {
        symbol: "KASUSDT",
        original_client_order_id: None,
        order_id: Some(order_output.order_id.as_str()),
        new_client_order_id: None,
    };
    client.cancel_order(cancel_order_params).await?;

    tracing::info!("Done!");

    Ok(())
}
