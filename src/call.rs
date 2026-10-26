use crate::error::ApiEnvelope;
use crate::{Error, KiteConnect, RawResponse, Request, Result};
use serde::de::{DeserializeOwned, Error as _};
use serde_json::Value;
use std::{
    future::{Future, IntoFuture},
    pin::Pin,
};

/// A Kite API request that is sent when you await it or call one of its methods.
///
/// - `.await` sends the request and decodes `data` into `T`.
/// - [`Call::raw`] sends it and returns `data` as untyped JSON.
/// - [`Call::request`] returns the request without sending it.
/// - [`Call::response`] sends it and returns the HTTP response unchanged.
#[must_use = "a Call sends nothing until awaited"]
pub struct Call<T> {
    client: KiteConnect,
    request: Request,
    decoder: fn(&KiteConnect, &RawResponse, &str) -> Result<T>,
    preflight_error: Option<Error>,
}

impl<T> Call<T> {
    pub(crate) fn with_decoder(
        client: KiteConnect,
        request: Request,
        decoder: fn(&KiteConnect, &RawResponse, &str) -> Result<T>,
    ) -> Self {
        Self {
            client,
            request,
            decoder,
            preflight_error: None,
        }
    }
    pub(crate) fn failed(client: KiteConnect, request: Request, error: Error) -> Self
    where
        T: DeserializeOwned,
    {
        Self {
            client,
            request,
            decoder: decode::<T>,
            preflight_error: Some(error),
        }
    }
    /// Returns the request this call sends, without sending it.
    #[must_use = "the request is returned for inspection without sending it"]
    pub fn request(&self) -> &Request {
        &self.request
    }
    /// Sends the request and returns the envelope's `data` as JSON, with no typed decoding.
    ///
    /// # Errors
    /// Returns an error if the request is invalid, transport fails, Kite rejects
    /// the request, or the response is not a valid JSON envelope.
    pub async fn raw(self) -> Result<Value> {
        let Self {
            client,
            request,
            preflight_error,
            ..
        } = self;
        if let Some(error) = preflight_error {
            return Err(error);
        }
        envelope_data(&client.send(&request).await?, request.path())
    }
    /// Sends the request and returns the HTTP response without reading the envelope.
    ///
    /// # Errors
    /// Returns an error if the request is invalid, transport fails, or Kite
    /// returns an unsuccessful HTTP status.
    pub async fn response(self) -> Result<RawResponse> {
        let Self {
            client,
            request,
            preflight_error,
            ..
        } = self;
        if let Some(error) = preflight_error {
            return Err(error);
        }
        client.send(&request).await
    }
}

impl<T: DeserializeOwned> Call<T> {
    pub(crate) fn new(client: KiteConnect, request: Request) -> Self {
        Self::with_decoder(client, request, decode::<T>)
    }
}

impl<T: Send + 'static> IntoFuture for Call<T> {
    type Output = Result<T>;
    type IntoFuture = Pin<Box<dyn Future<Output = Result<T>> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            client,
            request,
            decoder,
            preflight_error,
        } = self;
        Box::pin(async move {
            if let Some(error) = preflight_error {
                return Err(error);
            }
            decoder(&client, &client.send(&request).await?, request.path())
        })
    }
}

pub(crate) fn decode<T: DeserializeOwned>(
    _: &KiteConnect,
    response: &RawResponse,
    path: &str,
) -> Result<T> {
    serde_json::from_value(envelope_data(response, path)?)
        .map_err(|source| Error::decode(path, source))
}

pub(crate) fn envelope_data(response: &RawResponse, path: &str) -> Result<Value> {
    let envelope: ApiEnvelope<Value> =
        serde_json::from_slice(response.body()).map_err(|source| Error::decode(path, source))?;
    if envelope.status == "success" {
        Ok(envelope.data.unwrap_or(Value::Null))
    } else {
        Err(envelope.into_error(response.status(), || "Unknown Kite API error".to_string()))
    }
}

/// Kite can answer a CSV endpoint with a JSON error envelope and HTTP 200.
pub(crate) fn decode_csv<T: DeserializeOwned>(
    _: &KiteConnect,
    response: &RawResponse,
    path: &str,
) -> Result<Vec<T>> {
    if response.body().trim_ascii_start().starts_with(b"{") {
        envelope_data(response, path)?;
    }
    csv::Reader::from_reader(response.body())
        .deserialize()
        .collect::<std::result::Result<_, _>>()
        .map_err(|source| Error::decode(path, serde_json::Error::custom(source)))
}

pub(crate) fn success_bool(_: &KiteConnect, response: &RawResponse, path: &str) -> Result<bool> {
    envelope_data(response, path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_decode_failure_omits_query_values_from_the_path() {
        #[derive(serde::Deserialize)]
        struct Row {
            #[serde(rename = "quantity")]
            _quantity: i64,
        }

        let client = KiteConnect::builder("k").build().unwrap();
        let path = "/orders?access_token=SENTINEL-QUERY";
        for (body, csv) in [
            ("invalid json", false),
            (r#"{"status":"success","data":{}}"#, false),
            ("quantity\ninvalid\n", true),
        ] {
            let response = RawResponse {
                status: 200,
                content_type: None,
                body: body.as_bytes().to_vec(),
            };
            let error = if csv {
                decode_csv::<Row>(&client, &response, path).err().unwrap()
            } else {
                decode::<Vec<Row>>(&client, &response, path).err().unwrap()
            };
            assert_eq!(
                error.to_string(),
                "could not decode the response from /orders"
            );
            assert!(matches!(error, Error::Decode { path, .. } if path == "/orders"));
        }
        let error = crate::error::api_error_from_text(500, path, "invalid json");
        assert_eq!(
            error.to_string(),
            "HTTP 500 from /orders (not a Kite JSON error response)"
        );
    }

    #[test]
    fn success_boolean_matches_go_envelope_semantics() {
        let client = KiteConnect::builder("k").build().unwrap();
        for body in [
            include_str!("../mock_responses/session_logout.json"),
            r#"{"status":"success","data":{"status":true}}"#,
            r#"{"status":"success","data":false}"#,
            r#"{"status":"success","data":null}"#,
        ] {
            let response = RawResponse {
                status: 200,
                content_type: None,
                body: body.as_bytes().to_vec(),
            };
            assert!(success_bool(&client, &response, "/session/token").unwrap());
        }
        let response = RawResponse {
            status: 403,
            content_type: None,
            body: br#"{"status":"error","message":"Token expired","error_type":"TokenException"}"#
                .to_vec(),
        };
        assert!(matches!(
            success_bool(&client, &response, "/session/token"),
            Err(Error::Api(_))
        ));
        let response = RawResponse {
            status: 200,
            content_type: None,
            body: b"invalid json".to_vec(),
        };
        assert!(matches!(
            success_bool(&client, &response, "/session/token"),
            Err(Error::Decode { .. })
        ));
    }
}
