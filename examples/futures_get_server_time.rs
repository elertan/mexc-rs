use mexc_rs::futures::v1::endpoints::get_server_time::GetServerTime;
use mexc_rs::futures::MexcFuturesApiClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "mexc_rs=debug,futures_get_server_time=trace");
    tracing_subscriber::fmt::init();

    let client = MexcFuturesApiClient::default();
    let server_time = client.get_server_time().await?;
    tracing::info!("Server time: {:?}", server_time);

    Ok(())
}
