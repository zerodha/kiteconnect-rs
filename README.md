# kiteconnect-rs
API wrapper for kiteconnect in rust


## Docs

https://docs.rs/kiteconnect

## Usage

Head on to https://crates.io/crates/kiteconnect

Copy `kiteconnect = "<VERSION>"` dependency to Cargo.toml file


### KiteConnect REST APIs

```rust
extern crate kiteconnect;
extern crate serde_json as json;

use kiteconnect::connect::KiteConnect;

fn main() {
    let kiteconnect = KiteConnect::new("<API-KEY>", "<ACCESS-TOKEN>");
    let holdings: json::Value = kiteconnect.holdings().unwrap();
    println!("{:?}", holdings);
}
```

### Kite Ticker Websocket

```rust
extern crate kiteconnect;

use kiteconnect::ticker::{KiteTicker, KiteTickerHandler, WebSocketHandler}

#[derive(Debug)]
struct CustomHandler {
    count: u32
}

impl KiteTickerHandler for CustomHandler {
    fn on_open<T>(&mut self, ws: &mut WebSocketHandler<T>)
    where T: KiteTickerHandler {
        // Subscribe to a list of tokens on opening the websocket connection
        ws.subscribe(vec![123456]);
        println!("Fellow on_open callback");
    }
    fn on_message<T>(&mut self, ws: &mut WebSocketHandler<T>, msg: Message)
    where T: KiteTickerHandler {
        println!("Fellow on_message callback");
    }

    fn on_close<T>(&mut self, ws: &mut WebSocketHandler<T>)
    where T: KiteTickerHandler {
        println!("Fellow on_close callback");
    }

    fn on_error<T>(&mut self, ws: &mut WebSocketHandler<T>)
    where T: KiteTickerHandler {
        println!("Fellow on_error callback");
    }
}

fn main() {
    let mut ticker = KiteTicker::new("<API-KEY>".to_string(), "<ACCESS-TOKEN>".to_string());

    let custom_handler = CustomHandler {
        count: 0
    };

    ticket.connect(custom_handler);

    loop {}
}

```

## TODO
- [ ] Parsing binary to json
- [ ] Add serializer structs for all kiteconnect returning datastructures
- [ ] Reconnection mechanism
