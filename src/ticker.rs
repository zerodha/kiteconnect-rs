// #![deny(missing_docs,
//         missing_debug_implementations, missing_copy_implementations,
//         trivial_casts, trivial_numeric_casts,
//         unsafe_code,
//         unstable_features,
//         unused_import_braces, unused_qualifications)]
//
use std::thread;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

use ws::{
    Handler, Handshake, Message, Sender, CloseCode, Result, Error,
    Request, Factory, WebSocket
};
use byteorder::{BigEndian, ReadBytesExt};
use url;
use serde_json as json;


/// KiteTickerHandler lets the user write the business logic inside
/// the corresponding callbacks which are basically proxied from the
/// Handler callbacks
pub trait KiteTickerHandler {

    fn on_open<T>(&mut self, ws: &mut WebSocketHandler<T>)
    where T: KiteTickerHandler {
        println!("Connection opened");
    }

    fn on_message<T>(&mut self, ws: &mut WebSocketHandler<T>, msg: Message)
    where T: KiteTickerHandler {
        println!("{:?}", msg);
    }

    fn on_close<T>(&mut self, ws: &mut WebSocketHandler<T>)
    where T: KiteTickerHandler {
        println!("Connection closed");
    }

    fn on_error<T>(&mut self, ws: &mut WebSocketHandler<T>)
    where T: KiteTickerHandler {
        println!("Error");
    }
}


struct WebSocketFactory<T> where T: KiteTickerHandler {
    handler: Arc<Mutex<Box<T>>>
}


/// Implements Factory trait on KiteTickerFactory which essentialy
/// sets the Handler type
impl<T> Factory for WebSocketFactory<T> where T: KiteTickerHandler {
    type Handler = WebSocketHandler<T>;

    fn connection_made(&mut self, ws: Sender) -> WebSocketHandler<T> {
        WebSocketHandler {
            ws: Some(ws),
            handler: self.handler.clone()
        }
    }

    fn client_connected(&mut self, ws: Sender) -> WebSocketHandler<T> {
        WebSocketHandler {
            ws: Some(ws),
            handler: self.handler.clone()
        }
    }
}


pub struct WebSocketHandler<T> where T: KiteTickerHandler {
    handler: Arc<Mutex<Box<T>>>,
    ws: Option<Sender>
}


impl<T> WebSocketHandler<T> where T: KiteTickerHandler {
    /// Subscribe to a list of instrument_tokens
    pub fn subscribe(&self, instrument_tokens: Vec<u32>) -> Result<()> {
        let data = json!({
            "a": "subscribe",
            "v": instrument_tokens
        });
        match self.ws {
            Some(ref s) => {
                s.send(data.to_string());
                Ok(())
            },
            None => {
                Ok(println!("Sender not bound to the instance"))
            }
        }
    }

    /// Unsubscribe the given list of instrument_tokens
    pub fn unsubscribe(&self, instrument_tokens: Vec<u32>) -> Result<()> {
        let data = json!({
            "a": "unsubscribe",
            "v": instrument_tokens
        });
        match self.ws {
            Some(ref s) => {
                s.send(data.to_string());
                Ok(())
            },
            None => {
                Ok(println!("Sender not bound to the instance"))
            }
        }
    }

    /// Resubscribe to all current subscribed tokens
    pub fn resubscribe(&self) {
        unimplemented!()
    }

    /// Set streaming mode for the given list of tokens.
    pub fn set_mode(&self, mode: &str, instrument_tokens: Vec<u32>) -> Result<()> {
        let data = json!({
            "a": "mode",
            "v": [mode.to_string(), instrument_tokens]
        });
        match self.ws {
            Some(ref s) => {
                s.send(data.to_string());
                Ok(())
            }
            None => {
                Ok(println!("Sender not bound to the instance"))
            }
        }
    }

    /// Parses binary message to a json
    fn _parse_binary(&self, msg: &Message) {
        println!(">>>>>>{:?}", msg);
    }
}


/// Implements the Handler trait on KiteTicker which provides all the
/// callbacks methods ws-rs library
#[cfg(feature="ssl")]
impl<T> Handler for WebSocketHandler<T> where T: KiteTickerHandler {

    fn build_request(&mut self, url: &url::Url) -> Result<Request> {
        let mut req = Request::from_url(url)?;
        req.headers_mut().push(("X-Kite-Version".into(), "3".into()));
        println!("REQUEST: {:?}", req);
        Ok(req)
    }

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        let cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_open(self);
        println!("Connection opened {:?}", shake);
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_message(self, msg.clone());
        println!("MESSAGE : {:?}", msg);
        if msg.is_binary() && msg.len() > 2 {
            println!("MESSAGE LENGTH: {}", msg.len());
            let mut reader = Cursor::new(msg.clone().into_data());
            let number_of_packets = reader.read_i16::<BigEndian>().unwrap();
            println!("NUMBER OF PACKETS : {}", number_of_packets);
            for packet_index in 0..number_of_packets {
                let packet_length = reader.read_i16::<BigEndian>().unwrap();
                println!("PACKET_LENGTH : {}", packet_length);
                reader.seek(SeekFrom::Start(4 * (packet_index + 1) as u64));
                println!("POSITION : {}", reader.position());
                let instrument_token = reader.read_i32::<BigEndian>().unwrap();
                let segment = instrument_token & 0xff;
                let mut divisor: f64 = 100.0;
                if segment == 3 {  // cds
                    divisor = 10000000.0;
                }
                let mut tradable = true;
                if segment == 9 {  // indices
                    tradable = false;
                }
                if packet_length == 8 {
                    let mut data = json!({
                        "tradable": tradable,
                        "mode": "ltp",
                        "instrument_token": instrument_token,
                        "last_price": reader.read_i32::<BigEndian>().unwrap() as f64 / divisor,
                    });
                    println!("{:?}", data);
                } else if packet_length == 28 || packet_length == 32 {
                    let mut mode = "quote";
                    if packet_length == 28 {
                        mode = "full";
                    } else {
                        mode = "quote";
                    }
                    let mut data = json!({
                        "tradable": tradable,
                        "mode": mode,
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
                    println!("{:?}", data);
                } else if packet_length == 44 || packet_length == 184 {
                    let mut mode = "quote";
                    if packet_length == 184 {
                        mode = "full";
                    } else {
                        mode = "quote";
                    }
                    let mut data = json!({
                        "tradable": tradable,
                        "mode": mode,
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

                        // TODO Parse depth :(
                    }
                }
            }
            // TODO Iter through packets and construct json
            // self._parse_binary(&msg);
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        let cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_close(self);
        println!("Connection closed {:?}", code);
    }

    fn on_error(&mut self, err: Error) {
        let cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_error(self);
        println!("Error {:?}", err);
    }

}


#[cfg(feature="ssl")]
pub struct KiteTicker {
    sender: Option<Sender>,
    api_key: String,
    access_token: String,
}


/// Implments the apis exposed from KiteTicker struct
#[cfg(feature="ssl")]
impl KiteTicker {

    /// Constructor
    pub fn new(api_key: String, access_token: String) -> KiteTicker {
        KiteTicker {
            sender: None,
            api_key: api_key,
            access_token: access_token,
        }
    }

    /// Creates a websocket and delegates to it to child thread. Also sets the
    /// broadcaster so that other methods can easily send message on this socket
    pub fn connect<F>(&mut self, handler: F) -> Result<()>
        where F: KiteTickerHandler + Send + 'static {
        let factory = WebSocketFactory {
            handler: Arc::new(Mutex::new(Box::new(handler)))
        };
        let mut ws = WebSocket::new(factory).unwrap();
        let sender = ws.broadcaster();
        let socket_url = format!(
            "wss://ws.kite.trade?api_key={}&access_token={}",
            self.api_key,
            self.access_token
        );
        let url = url::Url::parse(socket_url.as_str()).unwrap();

        ws.connect(url.clone()).unwrap();
        thread::spawn(|| ws.run().unwrap());

        self.sender = Some(sender);

        Ok(())
    }

}
