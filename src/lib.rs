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

use hyper::header::Headers;
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

        let api_key: &str = &self.api_key.clone();
        let mut data = HashMap::new();
        data.insert("api_key", api_key);
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

    /// Place an order
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
    ) -> Result<json::Value> {
        let mut params = HashMap::new();
        params.insert("exchange", exchange);
        params.insert("tradingsymbol", tradingsymbol);
        params.insert("transaction_type", transaction_type);
        params.insert("quantity", quantity);
        params.insert("variety", variety);
        if price.is_some() { params.insert("price", price.unwrap()); }
        if product.is_some() { params.insert("product", product.unwrap()); }
        if order_type.is_some() { params.insert("order_type", order_type.unwrap()); }
        if validity.is_some() { params.insert("validity", validity.unwrap()); }
        if disclosed_quantity.is_some() { params.insert("disclosed_quantity", disclosed_quantity.unwrap()); }
        if trigger_price.is_some() { params.insert("trigger_price", trigger_price.unwrap()); }
        if squareoff.is_some() { params.insert("squareoff", squareoff.unwrap()); }
        if stoploss.is_some() { params.insert("stoploss", stoploss.unwrap()); }
        if trailing_stoploss.is_some() { params.insert("trailing_stoploss", trailing_stoploss.unwrap()); }
        if tag.is_some() { params.insert("tag", tag.unwrap()); }

        let url = self.build_url(format!("/orders/{}", variety).as_str(), None);

        let mut resp = self.send_request(url, "POST", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Modify an open order
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
    ) -> Result<json::Value> {
        let mut params = HashMap::new();
        params.insert("order_id", order_id);
        params.insert("variety", variety);
        if parent_order_id.is_some() { params.insert("parent_order_id", parent_order_id.unwrap()); }
        if exchange.is_some() { params.insert("exchange", exchange.unwrap()); }
        if tradingsymbol.is_some() { params.insert("tradingsymbol", tradingsymbol.unwrap()); }
        if transaction_type.is_some() { params.insert("transaction_type", transaction_type.unwrap()); }
        if quantity.is_some() { params.insert("quantity", quantity.unwrap()); }
        if price.is_some() { params.insert("price", price.unwrap()); }
        if order_type.is_some() { params.insert("order_type", order_type.unwrap()); }
        if product.is_some() { params.insert("product", product.unwrap()); }
        if trigger_price.is_some() { params.insert("trigger_price", trigger_price.unwrap()); }
        if validity.is_some() { params.insert("validity", validity.unwrap()); }
        if disclosed_quantity.is_some() { params.insert("disclosed_quantity", disclosed_quantity.unwrap()); }

        let url = self.build_url(format!("/orders/{}/{}", variety, order_id).as_str(), None);

        let mut resp = self.send_request(url, "POST", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Cancel an order
    pub fn cancel_order(
        &self,
        order_id: &str,
        variety: &str,
        parent_order_id: Option<&str>,
    ) -> Result<json::Value> {
        let mut params = HashMap::new();
        params.insert("order_id", order_id);
        params.insert("variety", variety);
        if parent_order_id.is_some() { params.insert("parent_order_id", parent_order_id.unwrap()); }
        let url = self.build_url(format!("/orders/{}/{}", variety, order_id).as_str(), None);

        let mut resp = self.send_request(url, "DELETE", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Exit a BO/CO order
    pub fn exit_order(
        &self,
        order_id: &str,
        variety: &str,
        parent_order_id: Option<&str>,
    ) -> Result<json::Value> {
        self.cancel_order(order_id, variety, parent_order_id)
    }

    /// Get a list of orders
    pub fn orders(&self) -> Result<json::Value> {
        let url = self.build_url("/orders", None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get the list of order history
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

    /// Modify an open position product type
    pub fn convert_position(
        &self,
        exchange: &str,
        tradingsymbol: &str,
        transaction_type: &str,
        position_type: &str,
        quantity: &str,
        old_product: &str,
        new_product: &str,
    ) -> Result<json::Value> {
        let mut params = HashMap::new();
        params.insert("exchange", exchange);
        params.insert("tradingsymbol", tradingsymbol);
        params.insert("transaction_type", transaction_type);
        params.insert("position_type", position_type);
        params.insert("quantity", quantity);
        params.insert("old_product", old_product);
        params.insert("new_product", new_product);

        let url = self.build_url("/portfolio/positions", None);

        let mut resp = self.send_request(url, "PUT", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get all mutual fund orders or individual order info
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

    /// Place a mutual fund order
    pub fn place_mf_order(
        &self,
        tradingsymbol: &str,
        transaction_type: &str,
        quantity: Option<&str>,
        amount: Option<&str>,
        tag: Option<&str>
    ) -> Result<json::Value> {
        let mut params = HashMap::new();
        params.insert("tradingsymbol", tradingsymbol);
        params.insert("transaction_type", transaction_type);
        if quantity.is_some() { params.insert("quantity", quantity.unwrap()); }
        if amount.is_some() { params.insert("amount", amount.unwrap()); }
        if tag.is_some() { params.insert("tag", tag.unwrap()); }

        let url = self.build_url("/mf/orders", None);

        let mut resp = self.send_request(url, "POST", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Cancel a mutual fund order
    pub fn cancel_mf_order(&self, order_id: &str) -> Result<json::Value> {
        let url = self.build_url(format!("/mf/orders/{}", order_id).as_str(), None);

        let mut resp = self.send_request(url, "DELETE", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get list of mutual fund SIP's or individual SIP info
    pub fn mf_sips(&self, sip_id: Option<&str>) -> Result<json::Value> {
        let url: reqwest::Url;
        if sip_id.is_some() {
            url = self.build_url(format!("/mf/sips/{}", sip_id.unwrap()).as_str(), None);
        } else {
            url = self.build_url("/mf/sips", None);
        }

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Place a mutual fund SIP
    pub fn place_mf_sip(
        &self,
        tradingsymbol: &str,
        amount: &str,
        instalments: &str,
        frequency: &str,
        initial_amount: Option<&str>,
        instalment_day: Option<&str>,
        tag: Option<&str>
    ) -> Result<json::Value> {
        let mut params = HashMap::new();
        params.insert("tradingsymbol", tradingsymbol);
        params.insert("amount", amount);
        params.insert("instalments", instalments);
        params.insert("frequency", frequency);
        if initial_amount.is_some() { params.insert("initial_amount", initial_amount.unwrap()); }
        if instalment_day.is_some() { params.insert("instalment_day", instalment_day.unwrap()); }
        if tag.is_some() { params.insert("tag", tag.unwrap()); }

        let url = self.build_url("/mf/sips", None);

        let mut resp = self.send_request(url, "POST", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Modify a mutual fund SIP
    pub fn modify_mf_sip(
        &self,
        sip_id: &str,
        amount: &str,
        status: &str,
        instalments: &str,
        frequency: &str,
        instalment_day: Option<&str>,
    ) -> Result<json::Value> {
        let mut params = HashMap::new();
        params.insert("sip_id", sip_id);
        params.insert("amount", amount);
        params.insert("status", status);
        params.insert("instalments", instalments);
        params.insert("frequency", frequency);
        if instalment_day.is_some() { params.insert("instalment_day", instalment_day.unwrap()); }

        let url = self.build_url(format!("/mf/sips/{}", sip_id).as_str(), None);

        let mut resp = self.send_request(url, "POST", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Cancel a mutual fund SIP
    pub fn cancel_mf_sip(&self, sip_id: &str) -> Result<json::Value> {
        let url = self.build_url(format!("/mf/sips/{}", sip_id).as_str(), None);

        let mut resp = self.send_request(url, "DELETE", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get a list of mutual fund holdings
    pub fn mf_holdings(&self) -> Result<json::Value> {
        let url = self.build_url("/mf/holdings", None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Get list of mutual fund instruments
    pub fn mf_instruments(&self) -> Result<reqwest::Response> {
        let url = self.build_url("/mf/instruments", None);

        self.send_request(url, "GET", None)
    }

    /// Retrieve the list of market instruments available to trade
    pub fn instruments(&self, exchange: Option<&str>) -> Result<reqwest::Response> {
        let url: reqwest::Url;
        if exchange.is_some() {
            url = self.build_url(format!("/instruments{}", exchange.unwrap()).as_str(), None);
        } else {
            url = self.build_url("/instruments", None);
        }

        self.send_request(url, "GET", None)
    }

    /// Retrieve quote for list of instruments
    pub fn quote(&self, instruments: Vec<&str>) -> Result<json::Value> {
        let instruments: String = instruments.join("");
        let mut params = HashMap::new();
        // FIXME
        params.insert("i", instruments.as_str());
        let url = self.build_url("/quote", None);

        let mut resp = self.send_request(url, "GET", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Retreive OHLC and market depth for list of instruments
    pub fn ohlc(&self, instruments: Vec<&str>) -> Result<json::Value> {
        let instruments: String = instruments.join("");
        let mut params = HashMap::new();
        // FIXME
        params.insert("i", instruments.as_str());
        let url = self.build_url("/quote/ohlc", None);

        let mut resp = self.send_request(url, "GET", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Retreive last price for list of instuments
    pub fn ltp(&self, instruments: Vec<&str>) -> Result<json::Value> {
        let instruments: String = instruments.join("");
        let mut params = HashMap::new();
        // FIXME
        params.insert("i", instruments.as_str());
        let url = self.build_url("/quote/ltp", None);

        let mut resp = self.send_request(url, "GET", Some(params))?;
        self._raise_or_return_json(&mut resp)
    }

    /// Retreive margins provided for individual segments
    pub fn instruments_margins(&self, segment: &str) -> Result<json::Value> {
        let url = self.build_url(format!("/margins/{}", segment).as_str(), None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    /// Retreive historical data (candles) for an instument
    pub fn historical_data(
        &self,
        instrument_token: &str,
        from_date: &str,
        to_date: &str,
        interval: &str,
        continuos: &str,
    ) -> Result<json::Value> {
        let mut params = HashMap::new();
        params.insert("instrument_token", instrument_token);
        params.insert("from", from_date);
        params.insert("to", to_date);
        params.insert("interval", interval);
        params.insert("continuos", continuos);
        let url = self.build_url(format!("/instruments/historical/{}/{}", instrument_token, interval).as_str(), None);

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
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
        let client = reqwest::Client::new();

        match method {
            "GET" => Ok(client.get(url).headers(headers).json(&data).send()?),
            "POST" => Ok(client.post(url).headers(headers).json(&data).send()?),
            "DELETE" => Ok(client.delete(url).headers(headers).json(&data).send()?),
            "PUT" => Ok(client.put(url).headers(headers).json(&data).send()?),
            _ => Err(ErrorKind::KiteException("Unknown method".to_string()).into()),
        }
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
        .with_body_from_file("mocks/margins.json")
        .create();
        let _mock1 = mockito::mock("GET", mockito::Matcher::Regex(r"^/user/margins/commodity".to_string()))
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
        .with_body_from_file("mocks/order_trades.json")
        .create();

        let data: json::Value = kiteconnect.order_trades("171229000724687").unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }

    #[test]
    fn test_orders() {
        let api_key: &str = &env::var("API_KEY").unwrap();
        let access_token: &str = &env::var("ACCESS_TOKEN").unwrap();
        let kiteconnect = KiteConnect::new(api_key, access_token);

        let _mock2 = mockito::mock(
            "GET", mockito::Matcher::Regex(r"^/orders".to_string())
        )
        .with_body_from_file("mocks/orders.json")
        .create();

        let data: json::Value = kiteconnect.orders().unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }

    #[test]
    fn test_order_history() {
        let api_key: &str = &env::var("API_KEY").unwrap();
        let access_token: &str = &env::var("ACCESS_TOKEN").unwrap();
        let kiteconnect = KiteConnect::new(api_key, access_token);

        let _mock2 = mockito::mock(
            "GET", mockito::Matcher::Regex(r"^/orders".to_string())
        )
        .with_body_from_file("mocks/order_info.json")
        .create();

        let data: json::Value = kiteconnect.order_history("171229000724687").unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }

    #[test]
    fn test_trades() {
        let api_key: &str = &env::var("API_KEY").unwrap();
        let access_token: &str = &env::var("ACCESS_TOKEN").unwrap();
        let kiteconnect = KiteConnect::new(api_key, access_token);

        let _mock1 = mockito::mock("GET", mockito::Matcher::Regex(r"^/trades".to_string()))
        .with_body_from_file("mocks/trades.json")
        .create();

        let data: json::Value = kiteconnect.trades().unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }

    #[test]
    fn test_mf_orders() {
        let api_key: &str = &env::var("API_KEY").unwrap();
        let access_token: &str = &env::var("ACCESS_TOKEN").unwrap();
        let kiteconnect = KiteConnect::new(api_key, access_token);

        let _mock1 = mockito::mock(
            "GET", mockito::Matcher::Regex(r"^/mf/orders$".to_string())
        )
        .with_body_from_file("mocks/mf_orders.json")
        .create();

        let _mock2 = mockito::mock(
            "GET", mockito::Matcher::Regex(r"^/mf/orders".to_string())
        )
        .with_body_from_file("mocks/mf_orders_info.json")
        .create();

        let data: json::Value = kiteconnect.mf_orders(None).unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
        let data: json::Value = kiteconnect.mf_orders(Some("171229000724687")).unwrap();
        println!("{:?}", data);
        assert!(data.is_object());
    }
}
