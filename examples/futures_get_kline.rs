use mexc_rs::futures::MexcFuturesApiClient;
use mexc_rs::futures::v1::endpoints::get_kline::{GetKline, GetKlineParams};
use mexc_rs::futures::v1::endpoints::get_server_time::GetServerTime;
use mexc_rs::futures::v1::models::KlineInterval;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "mexc_rs=trace,futures_get_kline=trace");
    tracing_subscriber::fmt::init();

    let client = MexcFuturesApiClient::default();
    let params = GetKlineParams {
        symbol: "KAS_USDT",
        interval: KlineInterval::FifteenMinutes,
        start: None,
        end: None,
    };
    let output = client.get_kline(params).await?;
    // first 5 klines
    tracing::info!("Output: {:#?}", &output.klines[0..5]);

    Ok(())
}
