# kiteconnect-rs
API wrapper for kiteconnect in rust


## Docs

https://docs.rs/kiteconnect

## Usage

Head on to https://crates.io/crates/kiteconnect

Copy `kiteconnect = "<VERSION>"` dependency to Cargo.toml file


```rust
extern crate kiteconnect;
extern crate serde_json as json;

use kiteconnect::KiteConnect;

let kiteconnect = KiteConnect::new("<API-KEY>", "<ACCESS-TOKEN>");
let holdings: json::Value = kiteconnect.holdings()?;
println!("{:?}", holdings);
```
