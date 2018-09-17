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
    fn on_open<T>(&mut self, ws: &mut WebSocketHandler<T>)
    where T: KiteTickerHandler {
        // Subscribe to a list of tokens on opening the websocket connection
        ws.subscribe(vec![53511431]);
        ws.set_mode("full", vec![53511431]);
        println!("Fellow on_open callback");
    }

    fn on_ticks<T>(&mut self, ws: &mut WebSocketHandler<T>, tick: Vec<json::Value>)
    where T: KiteTickerHandler {
        println!("{:?}", tick);
        println!("Fellow on_ticks callback");
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
    let mut ticker = KiteTicker::new("<API-KEY>", "<ACCESS-TOKEN>");

    let custom_handler = CustomHandler {
        count: 0
    };

    ticker.connect(custom_handler, None);

    loop {}
}
