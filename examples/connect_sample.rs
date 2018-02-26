extern crate kiteconnect;
extern crate serde_json as json;

use kiteconnect::connect::KiteConnect;

fn main() {
    let kiteconnect = KiteConnect::new("<API-KEY>", "<ACCESS-TOKEN>");
    let holdings: json::Value = kiteconnect.holdings().unwrap();
    println!("{:?}", holdings);
}