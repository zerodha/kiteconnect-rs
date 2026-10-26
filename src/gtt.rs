use crate::{Call, Error, Exchange, KiteConnect, Product, Request, TransactionType};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// One trigger value, limit price and quantity for the Kite GTT endpoints.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct GttTriggerParams {
    /// Price at which the leg triggers.
    pub trigger_value: f64,
    /// Limit price of the order placed on trigger.
    pub limit_price: f64,
    /// Quantity of the order placed on trigger.
    pub quantity: i64,
}

impl GttTriggerParams {
    /// Build a trigger leg for the Kite GTT endpoints.
    #[must_use]
    pub fn new(trigger_value: f64, limit_price: f64, quantity: i64) -> Self {
        Self {
            trigger_value,
            limit_price,
            quantity,
        }
    }
}

/// Single or paired trigger legs for the Kite GTT endpoints.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum GttTrigger {
    /// One trigger leg (`single`).
    Single(GttTriggerParams),
    /// Two trigger legs (`two-leg`).
    TwoLeg {
        /// Lower trigger leg.
        lower: GttTriggerParams,
        /// Upper trigger leg.
        upper: GttTriggerParams,
    },
    /// One trailing trigger leg (`trailing-single`).
    TrailingSingle {
        /// Trigger leg.
        leg: GttTriggerParams,
        /// Trailing distance in points.
        trailing_points: f64,
    },
    /// Two trailing trigger legs (`trailing-two-leg`).
    TrailingTwoLeg {
        /// Lower trigger leg.
        lower: GttTriggerParams,
        /// Upper trigger leg.
        upper: GttTriggerParams,
        /// Trailing distance in points.
        trailing_points: f64,
    },
}

/// Form parameters for `POST /gtt/triggers` and `PUT /gtt/triggers/{id}`.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use]
pub struct GttParams {
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Exchange of the instrument.
    pub exchange: Exchange,
    /// Last traded price of the instrument.
    pub last_price: f64,
    /// Transaction type of the orders.
    pub transaction_type: TransactionType,
    /// Product of the limit orders.
    pub product: Product,
    /// Trigger legs.
    pub trigger: GttTrigger,
}

impl GttParams {
    /// Build a GTT whose limit orders use the CNC product.
    pub fn new(
        tradingsymbol: impl Into<String>,
        exchange: Exchange,
        last_price: f64,
        transaction_type: TransactionType,
        trigger: GttTrigger,
    ) -> Self {
        Self {
            tradingsymbol: tradingsymbol.into(),
            exchange,
            last_price,
            transaction_type,
            product: Product::Cnc,
            trigger,
        }
    }

    /// Set the product of every limit order.
    pub fn product(mut self, product: Product) -> Self {
        self.product = product;
        self
    }

    fn into_form(self) -> crate::Result<[(&'static str, String); 3]> {
        let (trigger_type, legs, trailing_points) = match self.trigger {
            GttTrigger::Single(leg) => ("single", vec![leg], None),
            GttTrigger::TwoLeg { lower, upper } => ("two-leg", vec![lower, upper], None),
            GttTrigger::TrailingSingle {
                leg,
                trailing_points,
            } => ("trailing-single", vec![leg], Some(trailing_points)),
            GttTrigger::TrailingTwoLeg {
                lower,
                upper,
                trailing_points,
            } => (
                "trailing-two-leg",
                vec![lower, upper],
                Some(trailing_points),
            ),
        };
        if !self.last_price.is_finite()
            || legs
                .iter()
                .any(|leg| !leg.trigger_value.is_finite() || !leg.limit_price.is_finite())
            || trailing_points.is_some_and(|points| !points.is_finite())
        {
            return Err(Error::InvalidInput("GTT numeric values must be finite"));
        }
        let mut condition = json!({
            "exchange": self.exchange,
            "tradingsymbol": self.tradingsymbol,
            "last_price": self.last_price,
            "trigger_values": legs.iter().map(|leg| leg.trigger_value).collect::<Vec<_>>()
        });
        if let Some(points) = trailing_points {
            condition["trailing_points"] = json!([points]);
        }
        let orders: Vec<_> = legs
            .iter()
            .map(|leg| {
                json!({
                    "exchange": self.exchange,
                    "tradingsymbol": self.tradingsymbol,
                    "transaction_type": self.transaction_type,
                    "quantity": leg.quantity,
                    "price": leg.limit_price,
                    "order_type": "LIMIT",
                    "product": self.product,
                })
            })
            .collect();
        Ok([
            ("type", trigger_type.to_string()),
            ("condition", condition.to_string()),
            ("orders", json!(orders).to_string()),
        ])
    }
}

/// Trigger ID returned by POST, PUT and DELETE /gtt/triggers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GttResponse {
    /// Id of the trigger.
    pub trigger_id: i64,
}

/// Rejection metadata returned by GET /gtt/triggers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GttMeta {
    /// Reason the trigger was rejected, if any.
    pub rejection_reason: Option<String>,
}

/// Trigger condition returned by GET /gtt/triggers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GttCondition {
    /// Exchange of the instrument.
    pub exchange: String,
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Last traded price when the trigger was set.
    pub last_price: f64,
    #[serde(default)]
    /// Trigger prices, one per leg.
    pub trigger_values: Vec<f64>,
    /// Instrument token, if returned.
    pub instrument_token: Option<u32>,
    #[serde(default)]
    /// Trailing distance in points, if any.
    pub trailing_points: Vec<f64>,
}

/// Order execution status returned by GET /gtt/triggers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GttOrderResult {
    /// Order status.
    pub status: String,
    /// Id of the placed order, if any.
    pub order_id: Option<String>,
    /// Reason the order was rejected, if any.
    pub rejection_reason: Option<String>,
}

/// Triggered order details returned by GET /gtt/triggers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GttResult {
    /// Account id.
    pub account_id: String,
    /// Exchange of the instrument.
    pub exchange: String,
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Order validity.
    pub validity: String,
    /// Product type.
    pub product: String,
    /// Order type.
    pub order_type: String,
    /// Transaction type.
    pub transaction_type: String,
    /// Order quantity.
    pub quantity: f64,
    /// Order price.
    pub price: f64,
    /// Order metadata, if any.
    pub meta: Option<String>,
    #[serde(default, with = "crate::time::optional_datetime")]
    /// Time of the result, if available.
    pub timestamp: Option<NaiveDateTime>,
    /// Price at which the trigger fired.
    pub triggered_at: f64,
    /// Status of the placed order, if any.
    pub order_result: Option<GttOrderResult>,
}

/// Order leg returned by GET /gtt/triggers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GttOrder {
    /// Exchange of the instrument.
    pub exchange: String,
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Product type.
    pub product: String,
    /// Order type.
    pub order_type: String,
    /// Transaction type.
    pub transaction_type: String,
    /// Order quantity.
    pub quantity: f64,
    /// Order price.
    pub price: f64,
    /// Result of the triggered order, if any.
    pub result: Option<GttResult>,
}

/// Trigger returned by GET /gtt/triggers and /gtt/triggers/{id}.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Gtt {
    /// Trigger id.
    pub id: i64,
    /// Owner's user id.
    pub user_id: String,
    /// Id of the parent trigger, if any.
    pub parent_trigger: Option<i64>,
    #[serde(rename = "type")]
    /// Trigger type, such as `single` or `two-leg`.
    pub trigger_type: String,
    #[serde(default, with = "crate::time::optional_datetime")]
    /// Creation time, if returned.
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, with = "crate::time::optional_datetime")]
    /// Last update time, if returned.
    pub updated_at: Option<NaiveDateTime>,
    #[serde(default, with = "crate::time::optional_datetime")]
    /// Expiry time, if returned.
    pub expires_at: Option<NaiveDateTime>,
    /// Trigger status.
    pub status: String,
    /// Trigger condition.
    pub condition: GttCondition,
    #[serde(default)]
    /// Orders placed when the trigger fires.
    pub orders: Vec<GttOrder>,
    /// Rejection metadata, if any.
    pub meta: Option<GttMeta>,
}

impl KiteConnect {
    /// `GET /gtt/triggers`.
    pub fn gtts(&self) -> Call<Vec<Gtt>> {
        Call::new(self.clone(), Request::get("/gtt/triggers"))
    }
    /// `GET /gtt/triggers/{id}`.
    pub fn gtt(&self, trigger_id: i64) -> Call<Gtt> {
        Call::new(
            self.clone(),
            Request::get(format!("/gtt/triggers/{trigger_id}")),
        )
    }
    /// `POST /gtt/triggers`.
    pub fn place_gtt(&self, params: GttParams) -> Call<GttResponse> {
        let request = Request::post("/gtt/triggers");
        match params.into_form() {
            Ok(form) => Call::new(self.clone(), request.form(form)),
            Err(error) => Call::failed(self.clone(), request, error),
        }
    }
    /// `PUT /gtt/triggers/{id}`.
    pub fn modify_gtt(&self, trigger_id: i64, params: GttParams) -> Call<GttResponse> {
        let request = Request::put(format!("/gtt/triggers/{trigger_id}"));
        match params.into_form() {
            Ok(form) => Call::new(self.clone(), request.form(form)),
            Err(error) => Call::failed(self.clone(), request, error),
        }
    }
    /// `DELETE /gtt/triggers/{id}`.
    pub fn delete_gtt(&self, trigger_id: i64) -> Call<GttResponse> {
        Call::new(
            self.clone(),
            Request::delete(format!("/gtt/triggers/{trigger_id}")),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::Value;

    fn client(server: &MockServer) -> KiteConnect {
        KiteConnect::builder("key")
            .base_url(server.base_url())
            .build()
            .unwrap()
    }
    fn params() -> GttParams {
        GttParams::new(
            "RAIN",
            Exchange::Nse,
            102.6,
            TransactionType::Sell,
            GttTrigger::TwoLeg {
                lower: GttTriggerParams::new(102.0, 101.5, 1),
                upper: GttTriggerParams::new(103.7, 104.0, 2),
            },
        )
    }
    fn wire(params: GttParams) -> (String, Value, Value) {
        let [(_, kind), (_, condition), (_, orders)] = params.into_form().unwrap();
        (
            kind,
            serde_json::from_str(&condition).unwrap(),
            serde_json::from_str(&orders).unwrap(),
        )
    }
    fn fixture<T: serde::de::DeserializeOwned>(body: &str) -> T {
        let value: Value = serde_json::from_str(body).unwrap();
        serde_json::from_value(value["data"].clone()).unwrap()
    }
    fn assert_gtt(gtt: &Gtt, id: i64) {
        assert_eq!(gtt.id, id);
        assert_eq!(gtt.user_id, "XX0000");
        assert_eq!(gtt.trigger_type, "two-leg");
        assert_eq!(gtt.status, "triggered");
        assert_eq!(gtt.created_at.unwrap().to_string(), "2019-09-09 15:13:22");
        assert_eq!(gtt.updated_at.unwrap().to_string(), "2019-09-09 15:15:08");
        assert_eq!(gtt.expires_at.unwrap().to_string(), "2020-01-01 12:00:00");
        assert_eq!(gtt.parent_trigger, None);
        assert!(gtt.meta.is_none());
        assert_eq!(gtt.condition.exchange, "NSE");
        assert_eq!(gtt.condition.tradingsymbol, "RAIN");
        assert_eq!(gtt.condition.last_price.to_bits(), f64::to_bits(102.6));
        assert_eq!(gtt.condition.trigger_values, vec![102.0, 103.7]);
        assert_eq!(gtt.condition.instrument_token, Some(3_926_273));
        assert!(gtt.orders[0].result.is_none());
        assert_eq!(gtt.orders[1].quantity.to_bits(), f64::to_bits(1.0));
        assert_eq!(gtt.orders[1].price.to_bits(), f64::to_bits(1.0));
        let result = gtt.orders[1].result.as_ref().unwrap();
        assert_eq!(result.account_id, "XX0000");
        assert_eq!(result.timestamp.unwrap().to_string(), "2019-09-09 15:15:08");
        assert_eq!(result.triggered_at.to_bits(), f64::to_bits(103.7));
        let order_result = result.order_result.as_ref().unwrap();
        assert_eq!(order_result.status, "failed");
        assert_eq!(order_result.order_id.as_deref(), Some(""));
        assert_eq!(
            order_result.rejection_reason.as_deref(),
            Some(
                "Your order price is lower than the current lower circuit limit of 70.65. Place an order within the daily range."
            )
        );
    }

    #[test]
    fn fixtures_decode() {
        assert_gtt(
            &fixture(include_str!("../mock_responses/gtt_get_order.json")),
            123,
        );
        let list: Vec<Gtt> = fixture(include_str!("../mock_responses/gtt_get_orders.json"));
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, 112_127);
        assert_eq!(list[0].status, "active");
        assert_eq!(list[0].condition.tradingsymbol, "INFY");
        assert_eq!(list[0].condition.trigger_values, vec![702.0]);
        assert!(list[0].meta.as_ref().unwrap().rejection_reason.is_none());
        assert_gtt(&list[1], 105_099);
        for body in [
            include_str!("../mock_responses/gtt_place_order.json"),
            include_str!("../mock_responses/gtt_modify_order.json"),
            include_str!("../mock_responses/gtt_delete_order.json"),
        ] {
            let response: GttResponse = fixture(body);
            assert_eq!(response.trigger_id, 123);
        }
    }

    #[test]
    fn nullable_fields_and_future_response_strings_decode() {
        let mut value: Value =
            serde_json::from_str(include_str!("../mock_responses/gtt_get_order.json")).unwrap();
        value["data"]["type"] = json!("future-type");
        value["data"]["status"] = json!("future-status");
        value["data"]["created_at"] = json!("");
        value["data"]["updated_at"] = Value::Null;
        value["data"].as_object_mut().unwrap().remove("expires_at");
        value["data"]["meta"] = json!({"rejection_reason": "rejected"});
        let gtt: Gtt = serde_json::from_value(value["data"].clone()).unwrap();
        assert_eq!(gtt.trigger_type, "future-type");
        assert_eq!(gtt.status, "future-status");
        assert!(gtt.created_at.is_none());
        assert!(gtt.updated_at.is_none());
        assert!(gtt.expires_at.is_none());
        assert_eq!(
            gtt.meta.unwrap().rejection_reason.as_deref(),
            Some("rejected")
        );
        for field in ["id", "condition"] {
            let mut missing = value["data"].clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<Gtt>(missing).is_err());
        }
        assert!(serde_json::from_value::<GttResponse>(json!({})).is_err());
        assert!(serde_json::from_value::<GttResponse>(json!({"trigger_id": "123"})).is_err());
    }

    #[test]
    fn typed_params_preserve_leg_order_product_and_trailing_types() {
        let (kind, condition, orders) = wire(params().product(Product::Nrml));
        assert_eq!(kind, "two-leg");
        assert_eq!(condition["trigger_values"], json!([102.0, 103.7]));
        assert_eq!(orders[0]["price"], 101.5);
        assert_eq!(orders[1]["quantity"], json!(2));
        assert_eq!(orders[0]["product"], "NRML");
        assert_eq!(orders[1]["product"], "NRML");
        assert_eq!(orders[1]["order_type"], "LIMIT");
        for (trigger, kind, values) in [
            (
                GttTrigger::Single(GttTriggerParams::new(100.0, 99.0, 1)),
                "single",
                json!([100.0]),
            ),
            (
                GttTrigger::TrailingSingle {
                    leg: GttTriggerParams::new(100.0, 99.0, 1),
                    trailing_points: 2.0,
                },
                "trailing-single",
                json!([100.0]),
            ),
            (
                GttTrigger::TrailingTwoLeg {
                    lower: GttTriggerParams::new(100.0, 99.0, 1),
                    upper: GttTriggerParams::new(110.0, 111.0, 2),
                    trailing_points: 2.0,
                },
                "trailing-two-leg",
                json!([100.0, 110.0]),
            ),
        ] {
            let (wire_kind, condition, orders) = wire(GttParams::new(
                "RAIN",
                Exchange::Nse,
                102.6,
                TransactionType::Sell,
                trigger,
            ));
            assert_eq!(wire_kind, kind);
            assert_eq!(condition["trigger_values"], values);
            assert_eq!(orders[0]["product"], "CNC");
            if kind != "single" {
                assert_eq!(condition["trailing_points"], json!([2.0]));
            }
        }
    }

    #[tokio::test]
    async fn non_finite_gtt_values_return_an_error() {
        let client = KiteConnect::builder("key").build().unwrap();
        let error = client
            .place_gtt(GttParams::new(
                "RAIN",
                Exchange::Nse,
                f64::NAN,
                TransactionType::Sell,
                GttTrigger::Single(GttTriggerParams::new(100.0, 99.0, 1)),
            ))
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput("GTT numeric values must be finite")
        ));
    }

    #[tokio::test]
    async fn gtts_request() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/gtt/triggers");
                then.status(200)
                    .body(include_str!("../mock_responses/gtt_get_orders.json"));
            })
            .await;
        let gtts = client(&server).gtts().await.unwrap();
        assert_eq!(gtts.len(), 2);
        assert_eq!(gtts[0].id, 112_127);
        assert_gtt(&gtts[1], 105_099);
        mock.assert_async().await;
    }
    #[tokio::test]
    async fn gtt_request() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/gtt/triggers/123");
                then.status(200)
                    .body(include_str!("../mock_responses/gtt_get_order.json"));
            })
            .await;
        assert_gtt(&client(&server).gtt(123).await.unwrap(), 123);
        mock.assert_async().await;
    }
    #[tokio::test]
    async fn place_gtt_request() {
        let server = MockServer::start_async().await;
        let mock = server.mock_async(|when, then| {
            when.method(POST).path("/gtt/triggers")
                .form_urlencoded_tuple("type", "two-leg")
                .form_urlencoded_tuple("condition", r#"{"exchange":"NSE","last_price":102.6,"tradingsymbol":"RAIN","trigger_values":[102.0,103.7]}"#)
                .form_urlencoded_tuple("orders", r#"[{"exchange":"NSE","order_type":"LIMIT","price":101.5,"product":"CNC","quantity":1,"tradingsymbol":"RAIN","transaction_type":"SELL"},{"exchange":"NSE","order_type":"LIMIT","price":104.0,"product":"CNC","quantity":2,"tradingsymbol":"RAIN","transaction_type":"SELL"}]"#);
            then.status(200).body(include_str!("../mock_responses/gtt_place_order.json"));
        }).await;
        assert_eq!(
            client(&server)
                .place_gtt(params())
                .await
                .unwrap()
                .trigger_id,
            123
        );
        mock.assert_async().await;
    }
    #[tokio::test]
    async fn modify_gtt_request() {
        let server = MockServer::start_async().await;
        let mock = server.mock_async(|when, then| {
            when.method(PUT).path("/gtt/triggers/123")
                .form_urlencoded_tuple("type", "trailing-single")
                .form_urlencoded_tuple("condition", r#"{"exchange":"NSE","last_price":102.6,"tradingsymbol":"RAIN","trailing_points":[2.0],"trigger_values":[100.0]}"#)
                .form_urlencoded_tuple("orders", r#"[{"exchange":"NSE","order_type":"LIMIT","price":99.0,"product":"MIS","quantity":1,"tradingsymbol":"RAIN","transaction_type":"BUY"}]"#);
            then.status(200).body(include_str!("../mock_responses/gtt_modify_order.json"));
        }).await;
        let params = GttParams::new(
            "RAIN",
            Exchange::Nse,
            102.6,
            TransactionType::Buy,
            GttTrigger::TrailingSingle {
                leg: GttTriggerParams::new(100.0, 99.0, 1),
                trailing_points: 2.0,
            },
        )
        .product(Product::Mis);
        assert_eq!(
            client(&server)
                .modify_gtt(123, params)
                .await
                .unwrap()
                .trigger_id,
            123
        );
        mock.assert_async().await;
    }
    #[tokio::test]
    async fn delete_gtt_request() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(DELETE).path("/gtt/triggers/123");
                then.status(200)
                    .body(include_str!("../mock_responses/gtt_delete_order.json"));
            })
            .await;
        assert_eq!(
            client(&server).delete_gtt(123).await.unwrap().trigger_id,
            123
        );
        mock.assert_async().await;
    }
}
