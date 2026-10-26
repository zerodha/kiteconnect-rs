# kiteconnect-rs

[![crates.io](https://img.shields.io/crates/v/kiteconnect.svg)](https://crates.io/crates/kiteconnect)
[![docs.rs](https://docs.rs/kiteconnect/badge.svg)](https://docs.rs/kiteconnect)

A Rust client for the [Kite Connect API](https://kite.trade/docs/connect/v3/) by Zerodha.

- **Typed REST calls**: Covers Kite Connect v3 endpoints with typed request parameters and responses.
- **Lazy `Call<T>` requests**: Every endpoint returns a `Call<T>` that sends nothing until awaited, with `.raw()`, `.request()`, and `.response()` escape hatches.
- **WebSocket ticker**: Connects to `wss://ws.kite.trade` in a background Tokio task, reconnects with exponential backoff, and restores subscriptions.
- **Redacted secrets**: Redacts tokens, API secrets, and secret query or form fields from `Debug` output and strips query strings from error paths.

Version 1.0 is a rewrite and does not share an API with 0.2.x.

## Install

```sh
cargo add kiteconnect
cargo add tokio --features macros,rt-multi-thread
```

Or in `Cargo.toml`:

```toml
[dependencies]
kiteconnect = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Development and tests

The contract tests use the pinned `mock_responses` git submodule. Clone with
submodules enabled, or initialize it before running the test suite:

```sh
git submodule update --init --recursive
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

## Log in and read holdings

Kite Connect login is a browser redirect. Open `login_url()`, log in, and take the `request_token` from the redirect URL. Then exchange it with your API secret:

```rust,no_run
use kiteconnect::{ApiSecret, KiteConnect};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut kite = KiteConnect::builder("your_api_key").build()?;
    println!("Log in at {}", kite.login_url());

    let api_secret = ApiSecret::new(std::env::var("KITE_API_SECRET")?)?;
    let session = kite
        .generate_session("request_token_from_redirect", &api_secret)
        .await?;
    kite.set_access_token(session.access_token);

    for holding in kite.holdings().await? {
        println!(
            "{} {} @ {}",
            holding.tradingsymbol, holding.quantity, holding.last_price
        );
    }
    Ok(())
}
```

If you already have an access token, set it on the builder with `.access_token(AccessToken::new(token)?)`.

## `Call<T>`

Every endpoint method returns a `Call<T>`. It sends nothing until you use it:

```rust,no_run
# async fn example(kite: kiteconnect::KiteConnect) -> kiteconnect::Result<()> {
let holdings = kite.holdings().await?; // typed: Vec<Holding>
let data = kite.holdings().raw().await?; // `data` as serde_json::Value, no typed decoding
println!("{:?}", kite.holdings().request()); // the request, not sent (secrets redacted)
# Ok(())
# }
```

`.response()` sends the request and returns the HTTP status, content type, and body as is. `kite.call(Request::get("/path"))` reaches an endpoint the crate does not wrap.

## Ticker

```rust,no_run
use kiteconnect::{AccessToken, KiteConnect, Mode, TickerEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kite = KiteConnect::builder("your_api_key")
        .access_token(AccessToken::new(std::env::var("KITE_ACCESS_TOKEN")?)?)
        .build()?;
    let (ticker, mut events) = kite.ticker().spawn()?;
    ticker.subscribe(&[408065], Mode::Full)?;

    while let Some(event) = events.recv().await {
        match event {
            TickerEvent::Ticks(ticks) => {
                for tick in ticks {
                    println!("{} {:?}", tick.instrument_token, tick.last_price);
                }
            }
            TickerEvent::OrderUpdate(order) => println!("{} {}", order.order_id, order.status),
            TickerEvent::Closed(reason) => {
                eprintln!("ticker stopped: {reason}");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
```

The ticker runs in its own Tokio task and connects to `wss://ws.kite.trade`. After a dropped connection it reconnects with exponential backoff (at most 30 seconds) and subscribes again to the current set. A handshake that Kite rejects with HTTP 400, 401, or 403 ends the ticker instead. `KiteTicker` is a cloneable handle, so several tasks can change subscriptions while one task reads events. The event channel is bounded: a slow reader pauses socket reads, so no tick or order update is dropped. `Closed` is always the last event.

## Errors

Every call returns `kiteconnect::Result<T>`. A Kite error envelope becomes `Error::Api`, with Kite's `error_type`, message, and HTTP status.

```rust,no_run
# async fn example(kite: kiteconnect::KiteConnect) -> kiteconnect::Result<()> {
match kite.orders().await {
    Ok(orders) => println!("{} orders", orders.len()),
    Err(err) if err.is_session_expired() => {
        // TokenException: renew with `renew_access_token`, or log in again.
    }
    Err(kiteconnect::Error::Api(api)) => eprintln!("{}: {}", api.kind, api.message),
    Err(err) => return Err(err),
}
# Ok(())
# }
```

`is_session_expired()` is true only for `TokenException`. Kite rejects such a request before it acts on it, so it is safe to renew the token and send the request again. Retry nothing else, an order write least of all.

Tokens and the API secret are newtypes whose `Debug` output is redacted. `Request` and `Body` redact secret query and form values in `Debug`. Decode errors name the path without its query string.

## MSRV

Rust 1.92.

## License

The crate uses the MIT license. See [LICENSE](https://github.com/zerodha/kiteconnect-rs/blob/master/LICENSE).
