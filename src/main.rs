// #![deny(missing_docs,
//         missing_debug_implementations, missing_copy_implementations,
//         trivial_casts, trivial_numeric_casts,
//         unsafe_code,
//         unstable_features,
//         unused_import_braces, unused_qualifications)]
//
extern crate ws;
extern crate url;
#[macro_use]
extern crate serde_json as json;
extern crate byteorder;


use std::fmt;
use std::thread;
use std::io::Cursor;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use ws::{
    Handler, Handshake, Message, Sender, CloseCode, Result, Error,
    Request, Builder, Factory, WebSocket
};

use byteorder::{BigEndian, ReadBytesExt};


/// KiteTickerHandler lets the user write the business logic inside
/// the corresponding callbacks which are basically proxied from the
/// Handler callbacks
trait KiteTickerHandler {

    fn on_open<T>(&mut self, ws: &mut WebSocketHandler<T>) -> Result<()>
    where T: KiteTickerHandler {
        println!("Connection opened");
        Ok(())
    }

    fn on_message<T>(&mut self, ws: &mut WebSocketHandler<T>, msg: Message) -> Result<()>
    where T: KiteTickerHandler {
        println!("{:?}", msg);
        Ok(())
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


struct WebSocketHandler<T> where T: KiteTickerHandler {
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
        let mut cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_open(self);
        println!("Connection opened {:?}", shake);
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let mut cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_message(self, msg.clone());
        if msg.is_binary() && msg.len() > 2 {
            // TODO Split packet logic
            let mut reader = Cursor::new(msg.clone().into_data());
            let number_of_packets = reader.read_i16::<BigEndian>().unwrap();
            for packet_index in 0..number_of_packets {
                let packet_length = reader.read_i16::<BigEndian>().unwrap();
                println!("PACKET_LENGTH : {}", packet_length);
            }
            // TODO Iter through packets and construct json
            // self._parse_binary(&msg);
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        let mut cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_close(self);
        println!("Connection closed {:?}", code);
    }

    fn on_error(&mut self, err: Error) {
        let mut cloned_handler = self.handler.clone();
        cloned_handler.lock().unwrap().on_error(self);
        println!("Error {:?}", err);
    }

}


#[cfg(feature="ssl")]
struct KiteTicker {
    sender: Option<Sender>,
    api_key: String,
    access_token: String,
    user_id: String,
}


/// Implments the apis exposed from KiteTicker struct
#[cfg(feature="ssl")]
impl KiteTicker {

    /// Constructor
    pub fn new(api_key: String, access_token: String, user_id: String) -> KiteTicker {
        KiteTicker {
            sender: None,
            api_key: api_key,
            access_token: access_token,
            user_id: user_id,
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
        // ws.run().unwrap();

        self.sender = Some(sender);

        Ok(())
    }

}


#[cfg(feature="ssl")]
fn main() {

    use std::env;

    let api_key = env::var("API_KEY").unwrap();
    let access_token = env::var("ACCESS_TOKEN").unwrap();
    let user_id = env::var("USER_ID").unwrap();

    #[derive(Debug)]
    struct MyStruct;

    impl KiteTickerHandler for MyStruct {
        fn on_open<T>(&mut self, ws: &mut WebSocketHandler<T>) -> Result<()> where T: KiteTickerHandler {
            ws.subscribe(vec![256265]);
            println!(">>>>>>>>>OYE");
            Ok(())
        }
        fn on_message<T>(&mut self, ws: &mut WebSocketHandler<T>, msg: Message) -> Result<()> where T: KiteTickerHandler {
            println!("I am fellow on_message callback");
            println!("{:?}", msg);
            Ok(())
        }
    }

    let mut ticker = KiteTicker::new(api_key, access_token, user_id);
    let closure_struct = MyStruct{};
    ticker.connect(closure_struct);

    loop {}

}

#[cfg(not(feature="ssl"))]
fn main() {
    println!("SSL feature is not enabled.")
}
