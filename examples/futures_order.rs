use std::str::FromStr;
use rust_decimal::Decimal;
use dotenv::dotenv;
use mexc_rs::futures::{MexcFuturesApiClientWithAuthentication, MexcFuturesApiEndpoint};
use mexc_rs::futures::v1::endpoints::order::{Order, OrderParams};
use mexc_rs::futures::v1::models::{OpenType, OrderSide, OrderType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "mexc_rs=trace,futures_order=trace");
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let client = MexcFuturesApiClientWithAuthentication::new(MexcFuturesApiEndpoint::Base, api_key, secret_key);
    let params = OrderParams {
        symbol: "KAS_USDT",
        price: Decimal::from_str("0.001").unwrap(),
        volume: Decimal::from_str("50000").unwrap(),
        leverage: None,
        side: OrderSide::OpenLong,
        order_type: OrderType::PriceLimitedOrder,
        open_type: OpenType::Isolated,
        position_id: None,
        external_order_id: None,
        stop_loss_price: None,
        take_profit_price: None,
        position_mode: None,
        reduce_only: None,
    };
    let order_output = client.order(params).await?;
    tracing::info!("{:#?}", order_output);

    Ok(())
}
