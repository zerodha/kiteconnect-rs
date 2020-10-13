//! kiteconnect-rs is a rust implementation of the KiteConnect library.
//! 
//! The crate is called `kiteconnect` and you can depend of it via cargo:
//! 
//! ```ini
//! [dependencies.kiteconnect]
//! version = "*"
//! ```
//! 
//! If you want to use the git version:
//! 
//! ```ini
//! [dependencies.kiteconnect]
//! git = "https://github.com/zerodhatech/kiteconnect-rs"
//! ```
//! 
//! # Basic Operation
//! 
//! kiteconnect-rs had both the connect and ticker implementation of the KiteConnect API
//! 
//! 
//! # HTTP API
//! 
//! 
//! 
//! ```rust,no_run
//! # extern crate kiteconnect;
//! extern crate serde_json as json;
//! 
//! use kiteconnect::connect::KiteConnect;
//! 
//! fn main() {
//!     let mut kiteconnect = KiteConnect::new("<API-KEY>", "");
//! 
//!     // Open browser with this URL and get the request token from the callback
//!     let loginurl = kiteconnect.login_url();
//!     println!("{:?}", loginurl);
//! 
//!     // Generate access token with the above request token
//!     let resp = kiteconnect.generate_session("<REQUEST-TOKEN>", "<API-SECRET>");
//!     // `generate_session` internally sets the access token from the response
//!     println!("{:?}", resp);
//! 
//!     let holdings: json::Value = kiteconnect.holdings().unwrap();
//!     println!("{:?}", holdings);
//! # }
//! ```
//! 
//! # Ticker
//! ```rust, no_run
//! extern crate kiteconnect;
//! extern crate serde_json as json;
//! 
//! use kiteconnect::ticker::{KiteTicker, KiteTickerHandler, WebSocketHandler};
//! 
//! #[derive(Debug)]
//! struct CustomHandler {
//!     count: u32
//! }
//! 
//! impl KiteTickerHandler for CustomHandler {
//!     fn on_open<T>(&mut self, ws: &mut WebSocketHandler<T>)
//!     where T: KiteTickerHandler {
//!         // Subscribe to a list of tokens on opening the websocket connection
//!         ws.subscribe(vec![123456]);
//!         println!("Fellow on_open callback");
//!     }
//!     fn on_ticks<T>(&mut self, ws: &mut WebSocketHandler<T>, tick: Vec<json::Value>)
//!     where T: KiteTickerHandler {
//!         println!("{:?}", tick);
//!         println!("Fellow on_ticks callback");
//!     }
//! 
//!     fn on_close<T>(&mut self, ws: &mut WebSocketHandler<T>)
//!     where T: KiteTickerHandler {
//!         println!("Fellow on_close callback");
//!     }
//! 
//!     fn on_error<T>(&mut self, ws: &mut WebSocketHandler<T>)
//!     where T: KiteTickerHandler {
//!         println!("Fellow on_error callback");
//!     }
//! }
//! 
//! fn main() {
//!     let mut ticker = KiteTicker::new("<API-KEY>", "<ACCESS-TOKEN>");
//! 
//!     let custom_handler = CustomHandler {
//!         count: 0
//!     };
//! 
//!     ticker.connect(custom_handler, None);
//! 
//!     loop {}
//! # }
//! ```
//!
#[cfg(test)]
extern crate mockito;
extern crate csv;
extern crate ws;
extern crate url;
extern crate byteorder;

pub mod connect;
pub mod ticker;
pub mod types;