use serde::Deserialize;
use serde_json::Value;
use std::fmt;

#[derive(Debug, Deserialize)]
pub(crate) struct ApiEnvelope<T> {
    pub status: String,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error_type: Option<String>,
}

impl ApiEnvelope<Value> {
    pub(crate) fn into_error(self, status: u16, default: impl FnOnce() -> String) -> Error {
        ApiError {
            kind: ErrorKind::from(self.error_type.as_deref().unwrap_or("Unknown")),
            message: self.message.unwrap_or_else(default),
            status,
            data: self.data,
        }
        .into()
    }
}

/// Result type of every fallible call in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Error returned by the client, a [`crate::Call`] or the ticker.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The ticker was spawned from a client with no access token.
    #[error("ticker requires an access token")]
    NoAccessToken,
    /// Kite answered with an error envelope.
    #[error(transparent)]
    Api(#[from] ApiError),
    /// Cloudflare blocked or challenged the request.
    #[error(
        "Cloudflare blocked the request (HTTP {status}, Ray {})",
        .ray.as_deref().unwrap_or("unknown")
    )]
    Blocked {
        /// HTTP status code.
        status: u16,
        /// Cloudflare Ray ID, if the response had one.
        ray: Option<String>,
    },
    /// Kite returned an unsuccessful HTTP status without a JSON error envelope.
    #[error("HTTP {status} from {path} (not a Kite JSON error response)")]
    Http {
        /// HTTP status code.
        status: u16,
        /// Request path, without query values.
        path: String,
    },
    /// The HTTP request failed before a response arrived. The source has no URL.
    #[error("request failed")]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// The response body did not decode into the expected type.
    #[error("could not decode the response from {path}")]
    Decode {
        /// Request path, without the query string.
        path: String,
        /// Decoding error.
        source: serde_json::Error,
    },
    /// The ticker WebSocket failed.
    #[error(transparent)]
    WebSocket(Box<dyn std::error::Error + Send + Sync>),
    /// A URL did not parse.
    #[error(transparent)]
    InvalidUrl(Box<dyn std::error::Error + Send + Sync>),
    /// An argument was rejected before any request was sent.
    #[error("{0}")]
    InvalidInput(&'static str),
}

/// Error envelope from Kite: `{"status":"error","error_type":..,"message":..}`.
#[non_exhaustive]
pub struct ApiError {
    /// Kite's `error_type`.
    pub kind: ErrorKind,
    /// Kite's `message`.
    pub message: String,
    /// HTTP status code.
    pub status: u16,
    /// Kite's `data`, if the envelope had one.
    pub data: Option<Value>,
}

impl fmt::Debug for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiError")
            .field("kind", &self.kind)
            .field("message", &sanitize_error(&self.message))
            .field("status", &self.status)
            .field("data", &self.data.as_ref().map(redact_value))
            .finish()
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "kite API error {} (HTTP {}): {}",
            self.kind,
            self.status,
            sanitize_error(&self.message)
        )
    }
}

impl std::error::Error for ApiError {}

/*
 * Keep the raw API data available through the public field, but make derived
 * diagnostics safe when an upstream error echoes a secret-shaped field.
 */
fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        if is_secret_key(key) {
                            Value::String("<redacted>".to_string())
                        } else {
                            redact_value(value)
                        },
                    )
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(redact_value).collect()),
        _ => value.clone(),
    }
}

impl ApiError {
    /// Creates an API error.
    pub fn new(
        kind: ErrorKind,
        message: impl Into<String>,
        status: u16,
        data: Option<Value>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            status,
            data,
        }
    }

    /// Kite rejected the access token. Kite rejects such a request before
    /// acting on it, so a caller may refresh the token and retry it.
    #[must_use]
    pub fn is_session_expired(&self) -> bool {
        self.kind == ErrorKind::Token
    }
}

impl Error {
    pub(crate) fn decode(path: &str, source: serde_json::Error) -> Self {
        Self::Decode {
            path: path
                .split_once('?')
                .map_or(path, |(path, _)| path)
                .to_string(),
            source,
        }
    }

    /// Returns true when Kite rejected the access token (`TokenException`).
    ///
    /// Only such a rejection is safe to refresh and retry, because Kite rejects the
    /// token before it acts on the request.
    #[must_use]
    pub fn is_session_expired(&self) -> bool {
        matches!(self, Self::Api(error) if error.is_session_expired())
    }
}

/// Kite's `error_type` values.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// `GeneralException`.
    General,
    /// `TokenException`: the session expired or the token is invalid.
    Token,
    /// `PermissionException`.
    Permission,
    /// `UserException`.
    User,
    /// `TwoFAException`.
    TwoFA,
    /// `OrderException`.
    Order,
    /// `InputException`.
    Input,
    /// `DataException`.
    Data,
    /// `NetworkException`.
    Network,
    /// Any other `error_type` string.
    Other(String),
}

impl From<&str> for ErrorKind {
    fn from(value: &str) -> Self {
        match value {
            "GeneralException" => Self::General,
            "TokenException" => Self::Token,
            "PermissionException" => Self::Permission,
            "UserException" => Self::User,
            "TwoFAException" => Self::TwoFA,
            "OrderException" => Self::Order,
            "InputException" => Self::Input,
            "DataException" => Self::Data,
            "NetworkException" => Self::Network,
            other => Self::Other(other.to_string()),
        }
    }
}

impl ErrorKind {
    /// Returns the `error_type` string Kite sends, such as `TokenException`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::General => "GeneralException",
            Self::Token => "TokenException",
            Self::Permission => "PermissionException",
            Self::User => "UserException",
            Self::TwoFA => "TwoFAException",
            Self::Order => "OrderException",
            Self::Input => "InputException",
            Self::Data => "DataException",
            Self::Network => "NetworkException",
            Self::Other(value) => value,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Cloudflare blocks become a typed error so their HTML never reaches a toast.
pub(crate) async fn error_from_response(response: reqwest::Response) -> Error {
    let status = response.status().as_u16();
    let path = response.url().path().to_string();
    let mitigated = response.headers().contains_key("cf-mitigated");
    let ray = response
        .headers()
        .get("cf-ray")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let text = match response.text().await {
        Ok(text) => text,
        Err(err) => return crate::client::transport_error(err),
    };
    if mitigated || is_cloudflare_block_page(&text) {
        return Error::Blocked { status, ray };
    }
    api_error_from_text(status, &path, &text)
}

/// Cloudflare's interstitial challenge page, its "Attention Required" block
/// page, and the bare `error code: 1010` browser-signature ban.
fn is_cloudflare_block_page(text: &str) -> bool {
    text.contains("<title>Just a moment...</title>")
        || text.contains("<title>Attention Required! | Cloudflare</title>")
        || text.trim() == "error code: 1010"
}

pub(crate) fn api_error_from_text(status: u16, path: &str, text: &str) -> Error {
    match serde_json::from_str::<ApiEnvelope<Value>>(text) {
        Ok(envelope) => envelope.into_error(status, || format!("HTTP {status}")),
        Err(source) if (200..300).contains(&status) => Error::decode(path, source),
        Err(_) => Error::Http {
            status,
            path: path.split('?').next().unwrap_or_default().to_owned(),
        },
    }
}

/// Returns true only when an API error has the requested HTTP status and hint.
pub fn api_error_has_hint(err: &Error, status: u16, hint: &str) -> bool {
    let Error::Api(ApiError {
        status: actual_status,
        data: Some(data),
        ..
    }) = err
    else {
        return false;
    };
    *actual_status == status
        && data
            .get("hints")
            .and_then(Value::as_array)
            .is_some_and(|hints| hints.iter().any(|value| value.as_str() == Some(hint)))
}

pub(crate) fn snippet(text: &str) -> String {
    const MAX: usize = 300;
    let mut out: String = text.chars().take(MAX).collect();
    if text.chars().count() > MAX {
        out.push('…');
    }
    out
}

/// Keys whose values are never printed, in a query, a form or error text.
const SECRET_KEYS: [&str; 7] = [
    "api_secret",
    "access_token",
    "refresh_token",
    "request_token",
    "password",
    "checksum",
    "twofa_value",
];

/// Returns true when values of this query, form or JSON key must never be printed.
#[must_use]
pub fn is_secret_key(key: &str) -> bool {
    SECRET_KEYS
        .iter()
        .any(|secret| key.eq_ignore_ascii_case(secret))
}

/// Redacts token and password values in query, form, and JSON error text.
#[must_use]
pub fn sanitize_error(message: &str) -> String {
    SECRET_KEYS
        .into_iter()
        .chain(["authorization: token"])
        .fold(message.to_string(), |message, key| redact_kv(&message, key))
}

/// Replaces the value following a key with `<redacted>`, case-insensitive on
/// the key. Recognizes three separator shapes:
///
/// - `key=value` (form fields, URL query params): the value runs until the next
///   whitespace, `;`, or `&`.
/// - `key value` (the `Authorization: token <token>` header): same value
///   termination as above.
/// - `key: value` / `key:value` / `"key": "value"` / `"key" : "value"` (JSON
///   and colon-separated shapes): optional whitespace and an optional
///   closing quote may sit between the key and the colon, and optional
///   whitespace and an optional opening quote may sit between the colon and
///   the value. A quoted value is redacted up to its closing quote (kept
///   in place); an unquoted value runs until whitespace, `;`, `&`, `,`, or
///   `}`.
fn redact_kv(input: &str, key: &str) -> String {
    let lower_input = input.to_ascii_lowercase();
    let lower_key = key.to_ascii_lowercase();
    let mut out = String::with_capacity(input.len());
    let mut idx = 0;

    while idx < input.len() {
        let Some(rel) = lower_input[idx..].find(&lower_key) else {
            out.push_str(&input[idx..]);
            return out;
        };
        let key_start = idx + rel;
        let pos = key_start + key.len();
        if let Some(sep @ ('=' | ' ')) = lower_input[pos..].chars().next() {
            out.push_str(&input[idx..pos]);
            out.push(sep);
            out.push_str("<redacted>");
            let value_start = pos + sep.len_utf8();
            let mut end = value_start;
            for (offset, ch) in input[value_start..].char_indices() {
                if ch.is_whitespace() || ch == ';' || ch == '&' {
                    break;
                }
                end = value_start + offset + ch.len_utf8();
            }
            idx = end;
        } else {
            let mut p = pos;
            if input[p..].starts_with('"') {
                p += 1;
            }
            p += input[p..].chars().take_while(|c| *c == ' ').count();

            if input[p..].starts_with(':') {
                out.push_str(&input[idx..p]);
                out.push(':');
                let mut value_start = p + 1;
                while input[value_start..].starts_with(' ') {
                    out.push(' ');
                    value_start += 1;
                }
                let quoted = input[value_start..].starts_with('"');
                if quoted {
                    out.push('"');
                    value_start += 1;
                }
                out.push_str("<redacted>");
                if quoted {
                    if let Some(rel_end) = input[value_start..].find('"') {
                        out.push('"');
                        idx = value_start + rel_end + 1;
                    } else {
                        idx = input.len();
                    }
                } else {
                    let mut end = value_start;
                    for (offset, ch) in input[value_start..].char_indices() {
                        if ch.is_whitespace() || matches!(ch, ';' | '&' | ',' | '}') {
                            break;
                        }
                        end = value_start + offset + ch.len_utf8();
                    }
                    idx = end;
                }
            } else {
                out.push_str(&input[idx..pos]);
                idx = pos;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn non_json_http_errors_keep_status_without_response_body_or_query() {
        let error = super::api_error_from_text(
            502,
            "/orders?access_token=dummy",
            "<html>upstream failed</html>",
        );
        assert_eq!(
            error.to_string(),
            "HTTP 502 from /orders (not a Kite JSON error response)"
        );
    }

    use super::{ApiError, Error, ErrorKind, error_from_response, sanitize_error};
    use serde_json::json;

    #[test]
    fn error_kind_round_trips_documented_and_unknown_names() {
        for (wire, expected) in [
            ("GeneralException", ErrorKind::General),
            ("TokenException", ErrorKind::Token),
            ("PermissionException", ErrorKind::Permission),
            ("UserException", ErrorKind::User),
            ("TwoFAException", ErrorKind::TwoFA),
            ("OrderException", ErrorKind::Order),
            ("InputException", ErrorKind::Input),
            ("DataException", ErrorKind::Data),
            ("NetworkException", ErrorKind::Network),
            (
                "FutureException",
                ErrorKind::Other("FutureException".to_string()),
            ),
            ("", ErrorKind::Other(String::new())),
        ] {
            let kind = ErrorKind::from(wire);
            assert_eq!(kind, expected);
            assert_eq!(kind.as_str(), wire);
            assert_eq!(kind.to_string(), wire);
        }
    }

    #[test]
    fn session_expiry_matches_only_token_exceptions() {
        for (kind, status, expected) in [
            (ErrorKind::Token, 400, true),
            (ErrorKind::Token, 403, true),
            (ErrorKind::Permission, 403, false),
            (ErrorKind::Other("Forbidden".to_string()), 403, false),
            (ErrorKind::Input, 400, false),
            (ErrorKind::General, 500, false),
        ] {
            let error = ApiError {
                kind,
                message: "boom".to_string(),
                status,
                data: None,
            };
            assert_eq!(error.is_session_expired(), expected);
            assert_eq!(Error::Api(error).is_session_expired(), expected);
        }
        assert!(
            !Error::Blocked {
                status: 403,
                ray: None
            }
            .is_session_expired()
        );
        assert!(!Error::InvalidInput("invalid input").is_session_expired());
        assert!(
            !Error::Decode {
                path: "/user/profile".to_string(),
                source: serde_json::from_str::<serde_json::Value>("{").unwrap_err(),
            }
            .is_session_expired()
        );
        assert!(
            !Error::WebSocket(Box::new(
                tokio_tungstenite::tungstenite::Error::ConnectionClosed
            ))
            .is_session_expired()
        );
        assert!(
            !Error::Transport(Box::new(
                reqwest::Client::new()
                    .get("invalid URL")
                    .build()
                    .unwrap_err()
            ))
            .is_session_expired()
        );
    }

    async fn error_for(status: u16, headers: &[(&str, &str)], body: &str) -> Error {
        let server = httpmock::MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.path("/");
                let mut then = then.status(status).body(body);
                for (name, value) in headers {
                    then = then.header(*name, *value);
                }
            })
            .await;
        let response = reqwest::get(server.url("/")).await.unwrap();
        error_from_response(response).await
    }

    #[tokio::test]
    async fn a_challenge_page_without_the_mitigated_header_is_still_cloudflare() {
        let err = error_for(
            503,
            &[("cf-ray", "8c1f2a3b4c5d6e7f-BOM")],
            "<html><head><title>Just a moment...</title></head></html>",
        )
        .await;
        assert_eq!(
            err.to_string(),
            "Cloudflare blocked the request (HTTP 503, Ray 8c1f2a3b4c5d6e7f-BOM)"
        );
    }

    #[tokio::test]
    async fn error_1010_without_a_ray_header_is_cloudflare() {
        let err = error_for(403, &[], "error code: 1010").await;
        assert!(matches!(
            err,
            Error::Blocked {
                status: 403,
                ray: None
            }
        ));
        assert_eq!(
            err.to_string(),
            "Cloudflare blocked the request (HTTP 403, Ray unknown)"
        );
    }

    /// A real Kite 403 carries its own envelope and must stay an API error: the
    /// session-expiry path keys on it.
    #[tokio::test]
    async fn a_kite_error_envelope_is_not_mistaken_for_cloudflare() {
        let err = error_for(
            403,
            &[("cf-ray", "8c1f2a3b4c5d6e7f-BOM")],
            r#"{"status":"error","message":"Invalid session","error_type":"TokenException"}"#,
        )
        .await;
        assert_eq!(
            err.to_string(),
            "kite API error TokenException (HTTP 403): Invalid session"
        );
    }

    #[test]
    fn redacts_access_token_in_query_string() {
        let msg = "request failed for ...&access_token=SECRETVAL123&uid=42";
        let out = sanitize_error(msg);
        assert!(
            !out.contains("SECRETVAL123"),
            "access_token value leaked: {out}"
        );
        assert!(out.contains("access_token=<redacted>"));
        assert!(out.contains("uid=42"));
    }

    #[test]
    fn redacts_access_token_in_authorization_header_shape() {
        let msg = "Authorization: token ABC123HEADER rejected";
        let out = sanitize_error(msg);
        assert!(!out.contains("ABC123HEADER"), "header token leaked: {out}");
        assert!(out.contains("token <redacted>"));
        assert!(out.contains("rejected"));
    }

    #[test]
    fn redacts_password_value() {
        let out = sanitize_error("login form password=hunter2 was refused");
        assert!(!out.contains("hunter2"), "password leaked: {out}");
        assert!(out.contains("password=<redacted>"));
        assert!(out.contains("refused"));
    }

    #[test]
    fn redacts_every_request_secret_key_in_error_text() {
        for key in [
            "api_secret",
            "access_token",
            "refresh_token",
            "request_token",
            "password",
            "checksum",
            "twofa_value",
        ] {
            for (message, expected) in [
                (
                    format!("{key}=SENTINEL-SECRET&status=failed"),
                    format!("{key}=<redacted>&status=failed"),
                ),
                (
                    format!("\"{key}\": \"SENTINEL-SECRET\""),
                    format!("\"{key}\": \"<redacted>\""),
                ),
                (
                    format!("{}=SENTINEL-SECRET", key.to_ascii_uppercase()),
                    format!("{}=<redacted>", key.to_ascii_uppercase()),
                ),
            ] {
                assert_eq!(sanitize_error(&message), expected);
            }
        }
    }

    #[test]
    fn keeps_api_key_and_redacts_access_token() {
        let out = sanitize_error("api_key=test-key access_token=xyz");
        assert_eq!(out, "api_key=test-key access_token=<redacted>");
    }

    #[test]
    fn leaves_prose_mentioning_a_token_unchanged() {
        let msg = "instrument token is required";
        assert_eq!(sanitize_error(msg), msg);
    }

    #[test]
    fn passthrough_when_no_secret_present() {
        let msg = "kite API error NetworkException (HTTP 502): bad gateway";
        assert_eq!(sanitize_error(msg), msg);
    }

    #[test]
    fn api_error_diagnostics_redact_secret_values() {
        let error = Error::Api(ApiError::new(
            ErrorKind::Other("FutureException".to_string()),
            "access_token=SENTINEL-MESSAGE",
            403,
            Some(json!({
                "access_token": "SENTINEL-DATA",
                "nested": {"password": "SENTINEL-NESTED"},
                "hint": "keep"
            })),
        ));
        let output = format!("{error} {error:?}");
        assert!(!output.contains("SENTINEL"), "diagnostics leaked: {output}");
        assert!(output.contains("<redacted>"), "not redacted: {output}");
        assert!(
            output.contains("keep"),
            "useful non-secret data disappeared: {output}"
        );
    }

    #[test]
    fn redacts_json_colon_shape_no_space() {
        let out = sanitize_error(r#"{"access_token":"SECRETVAL123","uid":"42"}"#);
        assert!(!out.contains("SECRETVAL123"), "value leaked: {out}");
        assert_eq!(out, r#"{"access_token":"<redacted>","uid":"42"}"#);
    }

    #[test]
    fn redacts_json_colon_shape_with_space_after_colon() {
        let out = sanitize_error(r#"{"access_token": "SECRETVAL123"}"#);
        assert!(!out.contains("SECRETVAL123"), "value leaked: {out}");
        assert_eq!(out, r#"{"access_token": "<redacted>"}"#);
    }

    #[test]
    fn redacts_json_colon_shape_with_space_before_colon() {
        let out = sanitize_error(r#"{"access_token" : "SECRETVAL123"}"#);
        assert!(!out.contains("SECRETVAL123"), "value leaked: {out}");
        assert_eq!(out, r#"{"access_token" : "<redacted>"}"#);
    }

    #[test]
    fn redacts_bare_colon_shape_without_quotes() {
        let out = sanitize_error("access_token:SECRETVAL123 rejected");
        assert!(!out.contains("SECRETVAL123"), "value leaked: {out}");
        assert_eq!(out, "access_token:<redacted> rejected");
    }

    #[test]
    fn redacts_bare_colon_shape_with_space_after() {
        let out = sanitize_error("access_token: SECRETVAL123, uid: 42");
        assert!(!out.contains("SECRETVAL123"), "value leaked: {out}");
        assert_eq!(out, "access_token: <redacted>, uid: 42");
    }

    #[test]
    fn equals_and_space_shapes_still_work_after_colon_support_added() {
        let eq = sanitize_error("access_token=SECRETVAL123&uid=42");
        assert_eq!(eq, "access_token=<redacted>&uid=42");

        let space = sanitize_error("Authorization: token ABC123HEADER rejected");
        assert_eq!(space, "Authorization: token <redacted> rejected");
    }

    #[test]
    fn colon_shape_does_not_touch_non_secret_key() {
        let msg = r#"{"user_id":"AB1234","status":"error"}"#;
        assert_eq!(sanitize_error(msg), msg);
    }

    #[test]
    fn redacts_refresh_token() {
        let out = sanitize_error(r#"{"refresh_token":"PUBSECRET1","access_token":"ENCSECRET1"}"#);
        assert!(!out.contains("PUBSECRET1"), "refresh_token leaked: {out}");
        assert!(!out.contains("ENCSECRET1"), "access_token leaked: {out}");
        assert_eq!(
            out,
            r#"{"refresh_token":"<redacted>","access_token":"<redacted>"}"#
        );
    }

    #[test]
    fn refresh_token_redacted_in_eq_shape() {
        let out = sanitize_error("refresh_token=SECRETPUB rejected");
        assert!(!out.contains("SECRETPUB"), "value leaked: {out}");
        assert_eq!(out, "refresh_token=<redacted> rejected");
    }
}
