extern crate kiteconnect;
extern crate ws;

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

    ticker.connect(custom_handler);

    loop {}
}