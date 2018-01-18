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

use ws::{
    Handler, Handshake, Message, Sender, CloseCode, Result, Error,
    Request, Builder, Factory
};

use byteorder::{BigEndian, ReadBytesExt};


struct KiteTickerFactory;


/// Implements Factory trait on KiteTickerFactory which essentialy
/// sets the Handler type
impl Factory for KiteTickerFactory {
    type Handler = KiteTicker;

    fn connection_made(&mut self, ws: Sender) -> KiteTicker {
        KiteTicker {
            sender: Some(ws),
            ..Default::default()
        }
    }

    fn client_connected(&mut self, ws: Sender) -> KiteTicker {
        KiteTicker {
            sender: Some(ws),
            ..Default::default()
        }
    }
}


/// Implements the Handler trait on KiteTicker which provides all the
/// callbacks methods ws-rs library
#[cfg(feature="ssl")]
impl Handler for KiteTicker {

    fn build_request(&mut self, url: &url::Url) -> Result<Request> {
        let mut req = Request::from_url(url)?;
        req.headers_mut().push(("X-Kite-Version".into(), "3".into()));
        println!("REQUEST: {:?}", req);
        Ok(req)
    }

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        // TODO Subscribe to the initial instrument_tokens
        KiteTicker::on_open_cb(self.sender.clone());
        println!("Connection opened {:?}", shake);
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        KiteTicker::on_message_cb(self.sender.clone(), msg.clone());
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
        KiteTicker::on_close_cb(self.sender.clone(), code.clone(), reason.clone());
        println!("Connection closed {:?}", code);
    }

    fn on_error(&mut self, err: Error) {
        KiteTicker::on_error_cb(self.sender.clone(), &err);
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


/// Default trait implementation
impl Default for KiteTicker {
    fn default() -> KiteTicker {
        KiteTicker {
            sender: None,
            api_key: "".to_string(),
            access_token: "".to_string(),
            user_id: "".to_string(),
        }
    }
}


/// Display trait implmentation
impl fmt::Display for KiteTicker {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "KiteTicker( api_key={}, access_token={}, user_id={})",
            self.api_key,
            self.access_token,
            self.user_id
        )
    }
}


/// KiteTickerHandler lets the user write the business logic inside
/// the corresponding callbacks which are basically proxied from the
/// Handler callbacks
trait KiteTickerHandler {

    fn on_open_cb(sender: Option<Sender>) -> Result<()> {
        Ok(())
    }

    fn on_message_cb(sender: Option<Sender>, msg: Message) -> Result<()> {
        Ok(())
    }

    fn on_close_cb(sender: Option<Sender>, code: CloseCode, reason: &str ) -> Result<()> {
        Ok(())
    }

    fn on_error_cb(sender: Option<Sender>, err: &Error) -> Result<()> {
        Ok(())
    }

}


/// Implments the apis exposed from KiteTicker struct
#[cfg(feature="ssl")]
impl KiteTicker {

    /// Constructor
    pub fn new(api_key: String, access_token: String, user_id: String) -> KiteTicker {
        KiteTicker {
            api_key: api_key,
            access_token: access_token,
            user_id: user_id,
            sender: None
        }
    }

    /// Creates a websocket and delegates to it to child thread. Also sets the
    /// broadcaster so that other methods can easily send message on this socket
    pub fn connect(&mut self) -> Result<()> {
        let mut ws = Builder::new().build(
            KiteTickerFactory{}
        ).unwrap();
        let sender = ws.broadcaster();
        let socket_url = format!(
            "wss://websocket.kite.trade/v3?api_key={}&user_id={}&access_token={}",
            self.api_key,
            self.user_id,
            self.access_token
        );
        let url = url::Url::parse(socket_url.as_str()).unwrap();

        ws.connect(url.clone()).unwrap();
        thread::spawn(|| ws.run().unwrap());

        self.sender = Some(sender);

        Ok(())
    }

    /// Subscribe to a list of instrument_tokens
    pub fn subscribe(&self, instrument_tokens: Vec<u32>) -> Result<()> {
        let data = json!({
            "a": "subscribe",
            "v": instrument_tokens
        });
        match self.sender {
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
        match self.sender {
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
        match self.sender {
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


#[cfg(feature="ssl")]
fn main() {

    use std::env;

    let api_key = env::var("API_KEY").unwrap();
    let access_token = env::var("ACCESS_TOKEN").unwrap();
    let user_id = env::var("USER_ID").unwrap();

    impl KiteTickerHandler for KiteTicker {

        fn on_message_cb(sender: Option<Sender>, msg: Message) -> Result<()> {
            if sender.is_some() { println!("I am fellow on_message callback");}
            Ok(())
        }

        fn on_open_cb(sender: Option<Sender>) -> Result<()> {
            if sender.is_some() { println!("I am fellow on_open callback");}
            Ok(())
        }

        fn on_close_cb(sender: Option<Sender>, code: CloseCode, reason: &str ) -> Result<()> {
            if sender.is_some() { println!("I am fellow on_close callback");}
            Ok(())
        }

        fn on_error_cb(sender: Option<Sender>, err: &Error) -> Result<()> {
            if sender.is_some() { println!("I am fellow on_error callback");}
            Ok(())
        }

    }

    let mut ticker = KiteTicker::new(api_key, access_token, user_id);
    ticker.connect();


    ticker.subscribe(vec![256265]);

    loop {}

}

#[cfg(not(feature="ssl"))]
fn main() {
    println!("SSL feature is not enabled.")
}
