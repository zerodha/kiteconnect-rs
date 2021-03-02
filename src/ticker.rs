// #![deny(missing_docs,
//         missing_debug_implementations, missing_copy_implementations,
//         trivial_casts, trivial_numeric_casts,
//         unsafe_code,
//         unstable_features,
//         unused_import_braces, unused_qualifications)]
//
use std::thread;
use std::io::{Cursor, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::debug;
use ws::{
    Handler, Handshake, Message, Sender, CloseCode, Result, Error,
    Request, Factory, WebSocket
};
use byteorder::{BigEndian, ReadBytesExt};
use url;
use serde_json::{json, Value as JsonValue};

/// KiteTickerHandler lets the user write the business logic inside
/// the corresponding callbacks which are basically proxied from the
/// Handler callbacks
pub trait KiteTickerHandler {

    fn on_open(&mut self) {
        debug!("Connection opened");
    }

    fn on_ticks(&mut self, tick: Vec<JsonValue>) {
        debug!("{:?}", tick);
    }

    fn on_close(&mut self) {
        debug!("Connection closed");
    }

    fn on_error(&mut self) {
        debug!("Error");
    }
}


struct WebSocketFactory<T> where T: KiteTickerHandler {
    handler: Arc<Mutex<Box<T>>>
}


/// Implements Factory trait on KiteTickerFactory which essentialy
/// sets the Handler type
impl<T> Factory for WebSocketFactory<T> where T: KiteTickerHandler {
    type Handler = WebSocketHandler<T>;

    fn connection_made(&mut self, _ws: Sender) -> WebSocketHandler<T> {
        WebSocketHandler {
            handler: self.handler.clone(),
        }
    }

    fn client_connected(&mut self, _ws: Sender) -> WebSocketHandler<T> {
        WebSocketHandler {
            handler: self.handler.clone(),
        }
    }
}


pub struct WebSocketHandler<T> where T: KiteTickerHandler {
    handler: Arc<Mutex<Box<T>>>,
}



/// Implements the Handler trait on KiteTicker which provides all the
/// callbacks methods ws-rs library
impl<T> Handler for WebSocketHandler<T> where T: KiteTickerHandler {

    fn build_request(&mut self, url: &url::Url) -> Result<Request> {
        let mut req = Request::from_url(url)?;
        req.headers_mut().push(("X-Kite-Version".into(), "3".into()));
        Ok(req)
    }

    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        let cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_open();
        debug!("Connection opened!");
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        if msg.is_binary() && msg.len() > 2 {
            let mut reader = Cursor::new(msg.clone().into_data());
            let number_of_packets = reader.read_i16::<BigEndian>().unwrap();

            let mut tick_data: Vec<JsonValue> = Vec::new();
            let mut packet_length: i16;

            let mut j: u64 = 2;
            for _ in 0..number_of_packets {
                packet_length = reader.read_i16::<BigEndian>().unwrap();

                let instrument_token = reader.read_i32::<BigEndian>().unwrap();
                let segment = instrument_token & 0xFF;
                let mut divisor: f64 = 100.0;
                if segment == 3 {  // cds
                    divisor = 10000000.0;
                }

                let mut tradable = true;
                if segment == 9 {  // indices
                    tradable = false;
                }

                match packet_length {
                    // LTP
                    8 => {
                        tick_data.push(json!({
                            "tradable": tradable,
                            "mode": "ltp",
                            "instrument_token": instrument_token,
                            "last_price": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                        }));
                    },

                    // Index quote/full
                    28 | 32 => {
                        let mut data: JsonValue  = json!({
                            "tradable": tradable,
                            "mode": if packet_length == 28 {"quote"} else {"full"},
                            "instrument_token": instrument_token,
                            "last_price": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                            "ohlc": {
                                "high": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                                "low": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                                "open": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                                "close": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                            },
                            "change": 0
                        });

                        if data["ohlc"]["close"] != 0 {
                            let last_price: f64 = data["last_price"].as_f64().unwrap();
                            let ohlc_close: f64 = data["ohlc"]["close"].as_f64().unwrap();
                            data["change"] = json!((last_price - ohlc_close) * 100 as f64 / ohlc_close);
                        }

                        if packet_length == 32 {  // timestamp incase of full
                            data["timestamp"] = json!(reader.read_i32::<BigEndian>().unwrap() as f64 / divisor);
                        }

                        tick_data.push(data);
                    },

                    // Quote/Full
                    44 | 184 => {
                        let mut data: JsonValue = json!({
                            "tradable": tradable,
                            "mode": if packet_length == 44 {"quote"} else {"full"},
                            "instrument_token": instrument_token,
                            "last_price": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                            "last_quantity": reader.read_i32::<BigEndian>().unwrap() as f64,
                            "average_price": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                            "volume": reader.read_i32::<BigEndian>().unwrap() as f64,
                            "buy_quantity": reader.read_i32::<BigEndian>().unwrap() as f64,
                            "sell_quantity": reader.read_i32::<BigEndian>().unwrap() as f64,
                            "ohlc": {
                                "open": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                                "high": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                                "low": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                                "close": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor
                            }
                        });

                        if data["ohlc"]["close"] != 0 {
                            let last_price: f64 = data["last_price"].as_f64().unwrap();
                            let ohlc_close: f64 = data["ohlc"]["close"].as_f64().unwrap();
                            data["change"] = json!((last_price - ohlc_close) * 100 as f64 / ohlc_close);
                        }

                        if packet_length == 184 {
                            data["last_trade_time"] = json!(reader.read_i32::<BigEndian>().unwrap() as f64);
                            data["oi"] = json!(reader.read_i32::<BigEndian>().unwrap() as f64);
                            data["oi_day_high"] = json!(reader.read_i32::<BigEndian>().unwrap() as f64);
                            data["oi_day_low"] = json!(reader.read_i32::<BigEndian>().unwrap() as f64);
                            data["timestamp"] = json!(reader.read_i32::<BigEndian>().unwrap() as f64);

                            // XXX We have already read 64 bytes now, Remaining 184-64/12 = 10
                            let mut buy_depth_data: Vec<JsonValue> = Vec::with_capacity(5);
                            let mut sell_depth_data: Vec<JsonValue> = Vec::with_capacity(5);
                            for index in 0..10 {
                                let depth_data = json!({
                                    "quantity": reader.read_i32::<BigEndian>().unwrap() as f64,
                                    "price": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                                    "orders": reader.read_i16::<BigEndian>().unwrap() as f64
                                });

                                if index < 5 {
                                    buy_depth_data.push(depth_data);
                                } else {
                                    sell_depth_data.push(depth_data);
                                }

                                // Dont care 2 bytes padding
                                reader.read_i16::<BigEndian>().unwrap();
                            }
                            data["sell"] = json!(sell_depth_data);
                            data["buy"] = json!(buy_depth_data);
                        }

                        tick_data.push(data);
                    }

                    _ => {
                        debug!("undefined packet length received: {}", packet_length)
                    }
                }

                // Place reader in the position after the packet length
                reader.seek(SeekFrom::Start(j+2+packet_length as u64))?;

                j = j+2+packet_length as u64;
            }

            let cloned_handler = self.handler.clone();
            cloned_handler.lock().unwrap().on_ticks(tick_data);
        } else if msg.is_text() {
            // TODO: Handle text messages
            println!("text message received {:?}", msg)
        }

        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, _reason: &str) {
        let cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_close();
        debug!("Connection closed {:?}", code);
    }

    fn on_error(&mut self, err: Error) {
        let cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_error();
        debug!("Error {:?}", err);
    }

}


pub struct KiteTicker {
    sender: Option<Sender>,
    api_key: String,
    access_token: String,
    subscribed_tokens: HashMap<u32, String>
}


/// Implments the apis exposed from KiteTicker struct
impl KiteTicker {

    /// Constructor
    pub fn new(api_key: &str, access_token: &str) -> KiteTicker {
        KiteTicker {
            sender: None,
            api_key: api_key.to_string(),
            access_token: access_token.to_string(),
            subscribed_tokens: HashMap::new()
        }
    }

    /// Creates a websocket and delegates to it to child thread. Also sets the
    /// broadcaster so that other methods can easily send message on this socket
    pub fn connect<F>(&mut self, handler: F, uri: Option<&str>) -> Result<()>
        where F: KiteTickerHandler + Send + 'static {
        let factory = WebSocketFactory {
            handler: Arc::new(Mutex::new(Box::new(handler)))
        };
        let mut ws = WebSocket::new(factory).unwrap();
        let sender = ws.broadcaster();
        let socket_url = format!(
            "wss://{}?api_key={}&access_token={}",
            match uri {
                Some(uri) => uri,
                None => "ws.kite.trade"
            },
            self.api_key,
            self.access_token
        );
        let url = url::Url::parse(socket_url.as_str()).unwrap();

        ws.connect(url.clone()).unwrap();
        thread::spawn(|| ws.run().unwrap());

        self.sender = Some(sender);

        Ok(())
    }

    /// Subscribe to a list of instrument_tokens
    pub fn subscribe(&mut self, instrument_tokens: Vec<u32>) -> Result<()> {
        let data = json!({
            "a": "subscribe",
            "v": instrument_tokens
        });

        for token in &instrument_tokens {
            self.subscribed_tokens.insert(*token, "quote".to_string());
        }

        match self.sender {
            Some(ref s) => {
                s.send(data.to_string())?;
                Ok(())
            },
            None => {
                Ok(debug!("Sender not bound to the instance"))
            }
        }
    }

    /// Unsubscribe the given list of instrument_tokens
    pub fn unsubscribe(&mut self, instrument_tokens: Vec<u32>) -> Result<()> {
        let data = json!({
            "a": "unsubscribe",
            "v": instrument_tokens
        });

        for token in &instrument_tokens {
            self.subscribed_tokens.remove(token);
        }

        match self.sender {
            Some(ref s) => {
                s.send(data.to_string())?;
                Ok(())
            },
            None => {
                Ok(debug!("Sender not bound to the instance"))
            }
        }
    }

    /// Resubscribe to all current subscribed tokens
    pub fn resubscribe(&mut self) -> Result<()> {
        let mut modes: HashMap<String, Vec<u32>> = HashMap::new();

        for (token, mode) in self.subscribed_tokens.iter() {
            modes.entry(mode.clone()).or_insert(vec![]).push(token.clone());
        }

        for (mode, tokens) in modes.iter() {
            debug!("Resubscribing and set mode: {} - {:?}", mode, tokens);
            self.subscribe(tokens.clone())?;
            self.set_mode(mode.as_str(), tokens.clone())?;
        }
        Ok(())
    }

    /// Set streaming mode for the given list of tokens.
    pub fn set_mode(&mut self, mode: &str, instrument_tokens: Vec<u32>) -> Result<()> {
        let data = json!({
            "a": "mode",
            "v": [mode.to_string(), instrument_tokens]
        });

        for token in &instrument_tokens {
            *self.subscribed_tokens.entry(*token).or_insert("".to_string()) = mode.to_string();
        }

        match self.sender {
            Some(ref s) => {
                s.send(data.to_string())?;
                Ok(())
            }
            None => {
                Ok(debug!("Sender not bound to the instance"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ws::listen;

    struct Server {
        out: Sender,
    }

    impl Handler for Server {
        fn on_message(&mut self, msg: Message) -> Result<()> {
            let message = msg.clone();
            assert_eq!(message.into_text().unwrap(), "PING".to_string());
            self.out.send(msg)
        }
    }

    #[test]
    fn test_kite_ticker() {
        thread::spawn(move || {
            listen("127.0.0.1:3012", |out| {
                Server { out: out }
            }).unwrap()
        });

        struct MyHandler;
        impl KiteTickerHandler for MyHandler {}
        let mut kiteticker = KiteTicker::new("<API-KEY>", "<ACCESS-TOKEN>");
        kiteticker.connect(MyHandler{}, Some("127.0.0.1:3012"));
        kiteticker.sender.unwrap().send("PING");
    }
}
