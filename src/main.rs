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

    // Constructs url for the given path
    fn build_url(&self, path: &str, param: Option<Vec<(&str, &str)>>) -> reqwest::Url {
        let url: &str = &format!("{}/{}", URL, &path[1..]);
        let mut url = reqwest::Url::parse(url).unwrap();

        if let Some(data) = param {
            url.query_pairs_mut().extend_pairs(data.iter());
        }
        url
    }

    // Constructor
    pub fn new(api_key: &str, access_token: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            access_token: access_token.to_string()
        }
    }

    // Raise or return json response
    fn _raise_or_return_json(&self, resp: &mut reqwest::Response) -> Result<json::Value> {
        if resp.status().as_u16() == 200 {
            let jsn: json::Value = resp.json()?;
            Ok(jsn)
        } else {
            Err(ErrorKind::KiteException(format!("{}", resp.text()?)).into())
        }
    }

    // Set an access token
    pub fn set_access_token(&mut self, access_token: String) {
        self.access_token = access_token;
    }

    // Returns the login url
    pub fn login_url(&self) -> String {
        format!("https://kite.trade/connect/login?api_key={}", self.api_key)
    }

    // Request for access token
    pub fn request_access_token(
        &mut self,
        request_token: &str,
        api_secret: &str
    ) -> Result<json::Value> {
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
            self.set_access_token(jsn["access_token"].to_string());
            Ok(jsn)
        } else {
            Err(ErrorKind::KiteException(format!("{}", resp.text()?)).into())
        }
    }

    // Invalidates the access token
    pub fn invalidate_token(&self, access_token: &str) -> Result<reqwest::Response> {
        let url = self.build_url("/session/token", None);
        let mut data = HashMap::new();
        data.insert("access_token", access_token);

        self.send_request(url, "DELETE", Some(data))
    }

    // Return the account balance and cash margin details for
    // a particular segment
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

    // Get all holdings
    pub fn holdings(&self) -> Result<json::Value> {
        let mut params: Vec<(&str, &str)> = Vec::new();
        params.push(("api_key", self.api_key.as_str()));
        params.push(("access_token", self.access_token.as_str()));

        let url = self.build_url("/portfolio/holdings", Some(params));

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    // Get all positions
    pub fn positions(&self) -> Result<json::Value> {
        let mut params: Vec<(&str, &str)> = Vec::new();
        params.push(("api_key", self.api_key.as_str()));
        params.push(("access_token", self.access_token.as_str()));

        let url = self.build_url("/portfolio/positions", Some(params));

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

    // Get all trades
    pub fn trades(&self, order_id: Option<String>) -> Result<json::Value> {
        let mut params: Vec<(&str, &str)> = Vec::new();
        params.push(("api_key", self.api_key.as_str()));
        params.push(("access_token", self.access_token.as_str()));

        let url: reqwest::Url;
        if order_id.is_some() {
            url = self.build_url(format!("/orders/{}/trades", order_id.unwrap().as_str()).as_str(), None)
        } else {
            url = self.build_url("/trades", None);
        }

        let mut resp = self.send_request(url, "GET", None)?;
        self._raise_or_return_json(&mut resp)
    }

}

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
}
