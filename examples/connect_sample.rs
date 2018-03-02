extern crate kiteconnect;
extern crate serde_json as json;

use kiteconnect::connect::KiteConnect;
use kiteconnect::serializers::{Data, Holding};


fn main() {
    let kiteconnect = KiteConnect::new("<API-KEY>", "<ACCESS-TOKEN>");
    let holdings: Data<Vec<Holding>> = kiteconnect.holdings().unwrap();
    println!("{:?}", holdings);
}
