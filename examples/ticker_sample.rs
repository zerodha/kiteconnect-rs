extern crate kiteconnect;
extern crate ws;
extern crate serde_json as json;

use kiteconnect::ticker::{KiteTicker, KiteTickerHandler, WebSocketHandler};
use ws::Message;

#[derive(Debug)]
struct CustomHandler {
    count: u32
}

impl KiteTickerHandler for CustomHandler {
    fn on_open(&mut self) {
        // Subscribe to a list of tokens on opening the websocket connection
        //ws.subscribe(vec![53511431]);
        //ws.set_mode("full", vec![53511431]);
        println!("Fellow on_open callback");
    }

    fn on_ticks(&mut self, tick: Vec<json::Value>) {
        println!("{:?}", tick);
        println!("Fellow on_ticks callback");
    }

    fn on_close(&mut self) {
        println!("Fellow on_close callback");
    }

    fn on_error(&mut self) {
        println!("Fellow on_error callback");
    }
}

fn main() {
    // Assumes you have generated the access token beforehand.
    let mut ticker = KiteTicker::new("<API-KEY>", "<ACCESS-TOKEN>");

    let custom_handler = CustomHandler {
        count: 0
    };

    ticker.connect(custom_handler, None);

    ticker.subscribe(vec![3693569]);

    ticker.set_mode("full", vec![3693569]);

    loop {}
}
