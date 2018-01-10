# kiteconnect-rust
API wrapper for kiteconnect in rust


## Usage

```rust
extern crate kiteconnect
extern crate serde_json as json;

use kiteconnect::KiteConnect;

let kiteconnect = KiteConnect::new("<API-KEY>", "<ACCESS-TOKEN>");
let holdings: json::Value = kiteconnect.holdings()?;
println!("{:?}", holdings);
```
