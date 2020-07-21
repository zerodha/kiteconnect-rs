extern crate kiteconnect;
extern crate serde_json as json;

use kiteconnect::connect::KiteConnect;

fn main() {
    let mut kiteconnect = KiteConnect::new("<API-KEY>", "");

    // Open browser with this URL and get the request token from the callback
    let loginurl = kiteconnect.login_url();
    println!("{:?}", loginurl);

    // Generate access token with the above request token
    let resp = kiteconnect.generate_session("<REQUEST-TOKEN>", "<API-SECRET>");
    // `generate_session` internally sets the access token from the response
    println!("{:?}", resp);

    let holdings: json::Value = kiteconnect.holdings().unwrap();
    println!("{:?}", holdings);
}