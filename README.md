# mexc-rs

[![Crates.io](https://img.shields.io/crates/v/mexc-rs)](https://crates.io/crates/mexc-rs)
[![Documentation](https://docs.rs/mexc-rs/badge.svg)](https://docs.rs/mexc-rs)
[![License](https://img.shields.io/crates/l/mexc-rs)](./LICENSE)

**mexc-rs** is a Rust client library for the [MEXC](https://www.mexc.com/) cryptocurrency exchange API.

It provides a lightweight and easy-to-use interface for interacting with market data, account information, and trading operations.

## Features

- Connect securely using API keys
- Retrieve public market data (tickers, order books, etc.)
- Access private account endpoints (balances, order history)
- Place and manage spot orders and futures
- Easy environment variable support with `.env` files

## Quick Start

To use your MEXC API keys securely, you can store them in an `.env` file. Here’s how to get started:

1. Create a `.env` file in the root of your project.

2. Add your MEXC API credentials to the `.env` file:

```
MEXC_API_KEY=your_api_key_here
MEXC_SECRET_KEY=your_secret_key_here
```

3. Example to fetch the KAS/USDT lasts 5 klines of 15 minutes:

```rust
use mexc_rs::futures::v1::endpoints::get_kline::{GetKline, GetKlineParams};
use mexc_rs::futures::v1::models::KlineInterval;
use mexc_rs::futures::MexcFuturesApiClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
```

More examples are available in the `examples/` directory.

## Testing

Testing all the inputs of the API is somewhat dangerous : the API is real and deals with real orders.

Four of the tests should fail :
 * spot::v3::cancel_order::tests::cancel_order
 * spot::v3::get_order::tests::get_order
 * spot::v3::order::tests::test_order
 * spot::v3::query_order::tests::query_order

 The `test_order` is an invalid price and 3 others fails to catch a real order that should be in your account. If you really need to test, change the orders values, and create the expected orders in your account, at your own risk.