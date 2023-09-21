use dotenv::dotenv;
use mexc_rs::futures::{MexcFuturesApiClientWithAuthentication, MexcFuturesApiEndpoint};
use mexc_rs::futures::v1::endpoints::get_open_positions::GetOpenPositions;
use mexc_rs::futures::v1::endpoints::get_server_time::GetServerTime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "mexc_rs=trace,futures_get_open_positions=trace");
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let client = MexcFuturesApiClientWithAuthentication::new(MexcFuturesApiEndpoint::Base, api_key, secret_key);
    let open_positions = client.get_open_positions(None).await?;
    tracing::info!("{:#?}", open_positions);

    Ok(())
}
