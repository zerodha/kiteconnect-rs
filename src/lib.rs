// #![deny(missing_docs,
//         missing_debug_implementations, missing_copy_implementations,
//         trivial_casts, trivial_numeric_casts,
//         unsafe_code,
//         unstable_features,
//         unused_import_braces, unused_qualifications)]
//
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate hyper;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json as json;
extern crate crypto;
#[cfg(test)]
extern crate mockito;

use std::collections::HashMap;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use hyper::header::{Accept, Headers};
header! { (XKiteVersion, "X-Kite-Version") => [String] }
header! { (UserAgent, "User-Agent") => [String] }
header! { (Authorization, "Authorization") => [String] }

#[cfg(not(test))]
const URL: &'static str = "https://api.kite.trade";

#[cfg(test)]
const URL: &'static str = mockito::SERVER_URL;

#[allow(unused_variables)]
trait RequestHandler {
    fn send_request(
        &self,
        url: reqwest::Url,
        method: &str,
        data: Option<HashMap<&str, &str>>
    ) -> Result<reqwest::Response> {
        unimplemented!()
    }
}

error_chain! {
    foreign_links {
        Network(reqwest::Error);
        Io(::std::io::Error);
        Json(json::Error);
    }
    errors {
        KiteException(e: String){
            description("Gateway error"),
            display("{}", e),
        }
    }

}

pub struct KiteConnect {
    api_key: String,
    access_token: String,
}


impl KiteConnect {

    /// Constructs url for the given path and query params
    fn build_url(&self, path: &str, param: Option<Vec<(&str, &str)>>) -> reqwest::Url {
        let url: &str = &format!("{}/{}", URL, &path[1..]);
        let mut url = reqwest::Url::parse(url).unwrap();

        if let Some(data) = param {
            url.query_pairs_mut().extend_pairs(data.iter());
        }
        url
    }

    /// Constructor
    pub fn new(api_key: &str, access_token: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            access_token: access_token.to_string()
        }
    }

    /// Raise or return json response for a given response
    fn _raise_or_return_json(&self, resp: &mut reqwest::Response) -> Result<json::Value> {
        if resp.status().as_u16() == 200 {
            let jsn: json::Value = resp.json()?;
            Ok(jsn)
        } else {
            Err(ErrorKind::KiteException(format!("{}", resp.text()?)).into())
        }
    }

    /// Sets an access token for this instance
    pub fn set_access_token(&mut self, access_token: &str) {
        self.access_token = access_token.to_string();
    }

    /// Returns the login url
    pub fn login_url(&self) -> String {
        format!("https://kite.trade/connect/login?api_key={}", self.api_key)
    }

    /// Request for access token
    pub fn request_access_token(
        &mut self,
        request_token: &str,
        api_secret: &str
    ) -> Result<json::Value> {
        // Create a hex digest from api key, request token, api secret
        let mut sha = Sha256::new();
        sha.input_str(
            format!("{}{}{}", self.api_key, request_token, api_secret).as_str()
        );
        let checksum = sha.result_str();

        let mut data = HashMap::new();
        data.insert("request_token", request_token);
        data.insert("checksum", checksum.as_str());

        let url = self.build_url("/session/token", None);

        let mut resp = self.send_request(url, "POST", Some(data))?;

        if resp.status().as_u16() == 200 {
            let jsn: json::Value = resp.json()?;
            self.set_access_token(jsn["access_token"].as_str().unwrap());
            Ok(jsn)
        } else {
            Err(ErrorKind::KiteException(format!("{}", resp.text()?)).into())
        }
    }

    /// Invalidates the access token
    pub fn invalidate_token(&self, access_token: &str) -> Result<reqwest::Response> {
        let url = self.build_url("/session/token", None);
        let mut data = HashMap::new();
        data.insert("access_token", access_token);

        self.send_request(url, "DELETE", Some(data))
    }

    /// Return the account balance and cash margin details for
    /// a particular segment
    pub fn margins(&self, segment: Option<String>) -> Result<json::Value> {
        let url: reqwest::Url;
        if segment.is_some() {
            url = self.build_url(format!("/user/margins/{}", segment.unwrap().as_str()).as_str(), None)
        } else {
            url = self.build_url("/user/margins", None);
        }

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get all holdings
    pub fn holdings(&self) -> Result<json::Value> {
        let url = self.build_url("/portfolio/holdings", None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get all positions
    pub fn positions(&self) -> Result<json::Value> {
        let url = self.build_url("/portfolio/positions", None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get user profile details
    pub fn profile(&self) -> Result<json::Value> {
        let url = self.build_url("/user/profile", None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    pub fn place_order(
        &self,
        exchange: &str,
        tradingsymbol: &str,
        transaction_type: &str,
        quantity: &str,
        variety: &str,
        price: Option<&str>,
        product: Option<&str>,
        order_type: Option<&str>,
        validity: Option<&str>,
        disclosed_quantity: Option<&str>,
        trigger_price: Option<&str>,
        squareoff: Option<&str>,
        stoploss: Option<&str>,
        trailing_stoploss: Option<&str>,
        tag: Option<&str>,
    ) {
        unimplemented!()
    }

    pub fn modify_order(
        &self,
        order_id: &str,
        variety: &str,
        parent_order_id: Option<&str>,
        exchange: Option<&str>,
        tradingsymbol: Option<&str>,
        transaction_type: Option<&str>,
        quantity: Option<&str>,
        price: Option<&str>,
        order_type: Option<&str>,
        product: Option<&str>,
        trigger_price: Option<&str>,
        validity: Option<&str>,
        disclosed_quantity: Option<&str>,
    ) {
        unimplemented!()
    }

    pub fn cancel_order(
        &self,
        order_id: &str,
        variety: &str,
        parent_order_id: Option<&str>,
    ) {
        unimplemented!()
    }

    pub fn exit_order(
        &self,
        order_id: &str,
        variety: &str,
        parent_order_id: Option<&str>,
    ) {
        unimplemented!()
    }

    pub fn orders(&self) -> Result<json::Value> {
        let url = self.build_url("/orders", None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    pub fn order_history(&self, order_id: &str) -> Result<json::Value> {
        let mut params: Vec<(&str, &str)> = Vec::new();
        params.push(("order_id", order_id));

        let url = self.build_url("/orders", Some(params));

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get all trades
    pub fn trades(&self) -> Result<json::Value> {
        let url = self.build_url("/trades", None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get all trades
    pub fn order_trades(&self, order_id: &str) -> Result<json::Value> {
        let url = self.build_url(format!("/orders/{}/trades", order_id).as_str(), None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    pub fn convert_position(
        &self,
        exchange: &str,
        tradingsymbol: &str,
        transaction_type: &str,
        position_type: &str,
        quantity: &str,
        old_product: &str,
        new_product: &str,
    ) {
        unimplemented!()
    }

    pub fn mf_orders(&self, order_id: Option<&str>) -> Result<json::Value> {
        let url: reqwest::Url;
        if order_id.is_some() {
            url = self.build_url(format!("/mf/orders/{}", order_id.unwrap()).as_str(), None);
        } else {
            url = self.build_url("/mf/orders", None);
        }

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    pub fn place_mf_order(
        &self,
        tradingsymbol: &str,
        transaction_type: &str,
        quantity: Option<&str>,
        amount: Option<&str>,
        tag: Option<&str>
    ) {
        unimplemented!()
    }

    pub fn cancel_mf_order(&self, order_id: &str) {
        unimplemented!()
    }

    pub fn mf_sips(&self, sip_id: Option<&str>) {
        unimplemented!()
    }

    pub fn place_mf_sip(
        &self,
        tradingsymbol: &str,
        amount: &str,
        instalments: &str,
        frequency: &str,
        initial_amount: Option<&str>,
        instalment_day: Option<&str>,
        tag: Option<&str>
    ) {
        unimplemented!()
    }

    pub fn modify_mf_sip(
        &self,
        sip_id: &str,
        amount: &str,
        status: &str,
        instalments: &str,
        frequency: &str,
        instalment_day: Option<&str>,
    ) {
        unimplemented!()
    }

    pub fn cancel_mf_sip(&self, sip_id: &str) {
        unimplemented!()
    }

    pub fn mf_holdings(&self) {
        unimplemented!()
    }

    pub fn mf_instruments(&self) {
        unimplemented!()
    }

    pub fn instruments(&self, exchange: Option<&str>) {
        unimplemented!()
    }

    pub fn quote(&self, instruments: Vec<&str>) {
        unimplemented!()
    }

    pub fn ohlc(&self, instruments: Vec<&str>) {
        unimplemented!()
    }

    pub fn ltp(&self, instruments: Vec<&str>) {
        unimplemented!()
    }

    pub fn instruments_margins(&self, segment: &str) {
        unimplemented!()
    }

    pub fn historical_data(
        &self,
        instrument_token: &str,
        from_date: &str,
        to_date: &str,
        interval: &str,
        continuos: bool,
    ) {
        unimplemented!()
    }
}

/// Implement the request handler for kiteconnect struct
impl RequestHandler for KiteConnect {
    // Generic request builder
    fn send_request(
        &self,
        url: reqwest::Url,
        method: &str,
        data: Option<HashMap<&str, &str>>,
    ) -> Result<reqwest::Response> {
        let mut headers = Headers::new();
        headers.set(XKiteVersion("3".to_string()));
        headers.set(Authorization(format!("token {}:{}", self.api_key, self.access_token)));
        headers.set(UserAgent("Rust".to_string()));
        headers.set(Accept::json());
        let client = reqwest::Client::new();

        let resp = match method {
            "GET" => client.get(url).headers(headers).send()?,
            "POST" => client.post(url).headers(headers).json(&data).send()?,
            "DELETE" => client.delete(url).headers(headers).send()?,
            "PUT" => client.put(url).headers(headers).send()?,
            _ => client.get(url).headers(headers).send()?,
        };

        Ok(resp)
    }
}


// Mock tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_build_url() {
        let kiteconnect = KiteConnect::new("key", "token");
        let url = kiteconnect.build_url("/my-holdings", None);
        assert_eq!(url.as_str(), format!("{}/my-holdings", URL).as_str());

        let mut params: Vec<(&str, &str)> = Vec::new();
        params.push(("one", "1"));
        let url = kiteconnect.build_url("/my-holdings", Some(params));
        assert_eq!(url.as_str(), format!("{}/my-holdings?one=1", URL).as_str());
    }

    #[test]
    fn test_set_access_token() {
        let mut kiteconnect = KiteConnect::new("key", "token");
        assert_eq!(kiteconnect.access_token, "token");
        kiteconnect.set_access_token("my_token");
        assert_eq!(kiteconnect.access_token, "my_token");
    }

    #[test]
    fn test_login_url() {
        let kiteconnect = KiteConnect::new("key", "token");
        assert_eq!(kiteconnect.login_url(), "https://kite.trade/connect/login?api_key=key");
    }

    #[test]
    fn test_margins() {
        let api_key: &str = &env::var("API_KEY").unwrap();
        let access_token: &str = &env::var("ACCESS_TOKEN").unwrap();
        let kiteconnect = KiteConnect::new(api_key, access_token);

        let _mock1 = mockito::mock("GET", mockito::Matcher::Regex(r"^/user/margins".to_string()))
        .match_header("Accept", "application/json")
        .with_body_from_file("mocks/margins.json")
        .create();
        let _mock1 = mockito::mock("GET", mockito::Matcher::Regex(r"^/user/margins/commodity".to_string()))
        .match_header("Accept", "application/json")
        .with_body_from_file("mocks/margins.json")
        .create();

        let data: json::Value = kiteconnect.margins(None).unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
        let data: json::Value = kiteconnect.margins(Some("commodity".to_string())).unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }

    #[test]
    fn test_holdings() {
        let api_key: &str = &env::var("API_KEY").unwrap();
        let access_token: &str = &env::var("ACCESS_TOKEN").unwrap();
        let kiteconnect = KiteConnect::new(api_key, access_token);

        let _mock = mockito::mock("GET", mockito::Matcher::Regex(r"^/portfolio/holdings".to_string()))
        .match_header("Accept", "application/json")
        .with_body_from_file("mocks/holdings.json")
        .create();

        let data: json::Value = kiteconnect.holdings().unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }

    #[test]
    fn test_positions() {
        let api_key: &str = &env::var("API_KEY").unwrap();
        let access_token: &str = &env::var("ACCESS_TOKEN").unwrap();
        let kiteconnect = KiteConnect::new(api_key, access_token);

        let _mock = mockito::mock("GET", mockito::Matcher::Regex(r"^/portfolio/positions".to_string()))
        .match_header("Accept", "application/json")
        .with_body_from_file("mocks/positions.json")
        .create();

        let data: json::Value = kiteconnect.positions().unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }

    #[test]
    fn test_order_trades() {
        let api_key: &str = &env::var("API_KEY").unwrap();
        let access_token: &str = &env::var("ACCESS_TOKEN").unwrap();
        let kiteconnect = KiteConnect::new(api_key, access_token);

        let _mock2 = mockito::mock(
            "GET", mockito::Matcher::Regex(r"^/orders/171229000724687/trades".to_string())
        )
        .match_header("Accept", "application/json")
        .with_body_from_file("mocks/order_trades.json")
        .create();

        let data: json::Value = kiteconnect.order_trades("171229000724687").unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }

    #[test]
    fn test_trades() {
        let api_key: &str = &env::var("API_KEY").unwrap();
        let access_token: &str = &env::var("ACCESS_TOKEN").unwrap();
        let kiteconnect = KiteConnect::new(api_key, access_token);

        let _mock1 = mockito::mock("GET", mockito::Matcher::Regex(r"^/trades".to_string()))
        .match_header("Accept", "application/json")
        .with_body_from_file("mocks/trades.json")
        .create();

        let data: json::Value = kiteconnect.trades().unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }
}
