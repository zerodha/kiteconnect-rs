//! Kite's server-side price alerts (`/alerts`).
//!
//! | operation | method   | path                     |
//! |-----------|----------|--------------------------|
//! | list      | `GET`    | `/alerts`                |
//! | get one   | `GET`    | `/alerts/{uuid}`         |
//! | history   | `GET`    | `/alerts/{uuid}/history` |
//! | create    | `POST`   | `/alerts`                |
//! | modify    | `PUT`    | `/alerts/{uuid}`         |
//! | delete    | `DELETE` | `/alerts?uuid=…`         |
//!
//! Create/modify bodies are form-encoded. The form encoding round-trips
//! exactly: every field sent comes back verbatim on the following
//! `GET /alerts`, including a fractional `rhs_constant` and a name with spaces.

use crate::{Call, Error, KiteConnect, Request};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// `type` of the basket an ATO links to. Distinct from the alert's own `ato`
/// type: the basket service also understands `default` and `sip`, and an ATO's
/// basket must be `alert`.
const BASKET_TYPE_ALERT: &str = "alert";

/// The `gtt` object inside a basket item's order params: two percentage legs.
///
/// Zero disables a leg. Nonzero values become the order's `meta.gttp` at
/// trigger time, so they are order instructions and not decoration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BasketGtt {
    #[serde(default)]
    /// Target leg as a percentage of the trigger price; zero disables it.
    pub target: f64,
    #[serde(default)]
    /// Stoploss leg as a percentage of the trigger price; zero disables it.
    pub stoploss: f64,
}

/// The complete order parameters one basket item carries.
///
/// Every field is serialized, because an `update` operation **replaces** the
/// stored params object: a partial object would erase the fields it omitted.
/// Numbers are JSON numbers, not numeric strings, which the service demands.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[must_use]
pub struct BasketOrderParams {
    #[serde(default)]
    /// Order variety, such as `regular`.
    pub variety: String,
    #[serde(default)]
    /// Transaction type, `BUY` or `SELL`.
    pub transaction_type: String,
    #[serde(default)]
    /// Product type, such as `CNC`, `MIS` or `NRML`.
    pub product: String,
    #[serde(default)]
    /// Order type, such as `MARKET`, `LIMIT`, `SL` or `SL-M`.
    pub order_type: String,
    #[serde(default)]
    /// Order validity, such as `DAY`, `IOC` or `TTL`.
    pub validity: String,
    #[serde(default)]
    /// Minutes a `TTL` order stays open.
    pub validity_ttl: u32,
    #[serde(default)]
    /// Order quantity.
    pub quantity: u32,
    #[serde(default)]
    /// Quantity disclosed to the market.
    pub disclosed_quantity: u32,
    #[serde(default)]
    /// Limit price of the order.
    pub price: f64,
    #[serde(default)]
    /// Trigger price of an `SL` or `SL-M` order.
    pub trigger_price: f64,
    #[serde(default)]
    /// Square-off value for the order.
    pub squareoff: f64,
    #[serde(default)]
    /// Stoploss value for the order.
    pub stoploss: f64,
    #[serde(default)]
    /// Trailing stoploss value for the order.
    pub trailing_stoploss: f64,
    #[serde(default)]
    /// Percentage target and stoploss legs.
    pub gtt: BasketGtt,
    #[serde(default)]
    /// Tags attached to the order.
    pub tags: Vec<String>,
}

impl BasketOrderParams {
    /// Required order fields; every other field is zero or empty.
    pub fn new(
        variety: impl Into<String>,
        transaction_type: impl Into<String>,
        product: impl Into<String>,
        order_type: impl Into<String>,
        validity: impl Into<String>,
        quantity: u32,
        price: f64,
    ) -> Self {
        Self {
            variety: variety.into(),
            transaction_type: transaction_type.into(),
            product: product.into(),
            order_type: order_type.into(),
            validity: validity.into(),
            quantity,
            price,
            ..Self::default()
        }
    }
    /// Minutes a `TTL` order stays open.
    pub fn validity_ttl(mut self, minutes: u32) -> Self {
        self.validity_ttl = minutes;
        self
    }
    /// Trigger price of an `SL` or `SL-M` order.
    pub fn trigger_price(mut self, price: f64) -> Self {
        self.trigger_price = price;
        self
    }

    fn numeric_values_are_finite(&self) -> bool {
        [
            self.price,
            self.trigger_price,
            self.squareoff,
            self.stoploss,
            self.trailing_stoploss,
            self.gtt.target,
            self.gtt.stoploss,
        ]
        .into_iter()
        .all(f64::is_finite)
    }
}

/// The `params` of a basket write: either typed, or a verbatim JSON object.
///
/// Untagged, so both serialize as the bare params object the service expects.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum BasketItemParams {
    /// Typed order params built by this client.
    Authored(BasketOrderParams),
    /// Echoed back byte-for-byte. An `update` replaces the stored params, so
    /// resending a fetched object verbatim is the only lossless way to move an
    /// item whose params [`BasketOrderParams`] cannot represent.
    Preserved(Value),
}

impl BasketItemParams {
    fn numeric_values_are_finite(&self) -> bool {
        match self {
            Self::Authored(params) => params.numeric_values_are_finite(),
            Self::Preserved(_) => true,
        }
    }
}

/// One entry in a basket write's `items` array.
///
/// `type` is the **operation**, not the instrument's or the alert's type. The
/// alerts read model has no such field, which is why a fetched basket cannot
/// be sent back as a write: the server rejects an item whose operation type is
/// missing.
///
/// The variants carry precisely the fields their operation needs, so an
/// `update` or `delete` without a server item id, or a `delete` that smuggles
/// params, cannot be constructed.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum BasketOperation {
    /// Adds a new item to the basket.
    Insert {
        /// Exchange of the instrument.
        exchange: String,
        /// Trading symbol of the instrument.
        tradingsymbol: String,
        /// One-based order position. The basket service's SQL reads an `update`
        /// weight of 0 as "unchanged", so a position is never zero.
        weight: u32,
        /// Order params of the item.
        params: BasketItemParams,
    },
    /// Replaces an existing basket item.
    Update {
        /// Server id of the item to replace.
        id: i64,
        /// Exchange of the instrument.
        exchange: String,
        /// Trading symbol of the instrument.
        tradingsymbol: String,
        /// One-based order position.
        weight: u32,
        /// Order params that replace the stored ones.
        params: BasketItemParams,
    },
    /// The whole operation. The service needs nothing else, and sending an
    /// instrument or params alongside would suggest this edits the item.
    Delete {
        /// Server id of the item to delete.
        id: i64,
    },
}

impl BasketOperation {
    fn numeric_values_are_finite(&self) -> bool {
        match self {
            Self::Insert { params, .. } | Self::Update { params, .. } => {
                params.numeric_values_are_finite()
            }
            Self::Delete { .. } => true,
        }
    }
}

impl BasketWrite {
    fn numeric_values_are_finite(&self) -> bool {
        let items = match self {
            Self::Create(basket) => &basket.items,
            Self::Modify(basket) => &basket.items,
        };
        items.iter().all(BasketOperation::numeric_values_are_finite)
    }
}

/// The `basket` field of a `POST /alerts` that creates an ATO.
///
/// Carries the basket-level metadata, which only a creation may set: the alerts
/// service does not return a basket's name, type or tags, so an update that
/// sent them would overwrite values the caller cannot read back.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct BasketCreate {
    /// Basket name.
    pub name: String,
    #[serde(rename = "type")]
    /// Basket type; `alert` for an ATO.
    pub basket_type: String,
    /// Basket tags.
    pub tags: Vec<String>,
    /// Operations that build the basket.
    pub items: Vec<BasketOperation>,
}

impl BasketCreate {
    /// A new `alert` basket named `name` (1-50 bytes) holding `items`.
    pub fn new(name: impl Into<String>, items: Vec<BasketOperation>) -> Self {
        Self {
            name: name.into(),
            basket_type: BASKET_TYPE_ALERT.to_string(),
            tags: Vec::new(),
            items,
        }
    }
}

/// The `basket` field of a `PUT /alerts/{uuid}` that changes an existing ATO's
/// basket: operations only, and deliberately no `name`, `type` or `tags`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct BasketModify {
    /// Operations applied to the stored basket.
    pub items: Vec<BasketOperation>,
}

impl BasketModify {
    /// Operations applied to the stored basket.
    #[must_use]
    pub fn new(items: Vec<BasketOperation>) -> Self {
        Self { items }
    }
}

/// The `basket` an ATO write sends. An ATO write without one keeps the stored
/// basket, which is how a condition is edited without touching the orders.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum BasketWrite {
    /// Creates a basket along with its metadata.
    Create(BasketCreate),
    /// Changes the items of an existing basket.
    Modify(BasketModify),
}

/// Alert returned by `/alerts`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Alert {
    #[serde(rename = "type")]
    /// Alert type.
    pub alert_type: AlertType,
    /// Owner's user id.
    pub user_id: String,
    /// Alert UUID.
    pub uuid: String,
    /// Alert name.
    pub name: String,
    /// Alert status.
    pub status: AlertStatus,
    /// Reason the alert was disabled, if any.
    pub disabled_reason: Option<String>,
    /// Left-hand side attribute, such as `LastTradedPrice`.
    pub lhs_attribute: String,
    /// Left-hand side exchange.
    pub lhs_exchange: String,
    /// Left-hand side trading symbol.
    pub lhs_tradingsymbol: String,
    /// Comparison operator as sent on the wire.
    pub operator: String,
    /// Right-hand side type, `constant` or `instrument`.
    pub rhs_type: String,
    /// Right-hand side attribute for an instrument condition.
    pub rhs_attribute: Option<String>,
    /// Right-hand side exchange for an instrument condition.
    pub rhs_exchange: Option<String>,
    /// Right-hand side trading symbol for an instrument condition.
    pub rhs_tradingsymbol: Option<String>,
    /// Right-hand side constant value.
    pub rhs_constant: f64,
    #[serde(default)]
    /// Number of times the alert has triggered.
    pub alert_count: i64,
    #[serde(with = "crate::time::datetime")]
    /// Creation time.
    pub created_at: NaiveDateTime,
    #[serde(with = "crate::time::datetime")]
    /// Last update time.
    pub updated_at: NaiveDateTime,
    /// Linked basket for an ATO.
    pub basket: Option<AlertBasket>,
}

/// ATO basket returned by `/alerts`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AlertBasket {
    /// Basket name, if returned.
    pub name: Option<String>,
    #[serde(rename = "type")]
    /// Basket type, if returned.
    pub basket_type: Option<String>,
    #[serde(default)]
    /// Basket tags.
    pub tags: Vec<String>,
    #[serde(default)]
    /// Basket items.
    pub items: Vec<AlertBasketItem>,
}

/// ATO basket item returned by `/alerts`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AlertBasketItem {
    #[serde(rename = "type")]
    /// Operation type of the item, if returned.
    pub item_type: Option<String>,
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Exchange of the instrument.
    pub exchange: String,
    /// One-based order position.
    pub weight: i64,
    /// Order params as returned by the server.
    pub params: Value,
    /// Server item id, if returned.
    pub id: Option<i64>,
    /// Instrument token, if returned.
    pub instrument_token: Option<u32>,
}

/// Trigger entry returned by `/alerts/{uuid}/history`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AlertHistory {
    /// Alert UUID.
    pub uuid: String,
    #[serde(rename = "type")]
    /// Alert type.
    pub alert_type: String,
    #[serde(default)]
    /// Quote metadata captured at trigger time.
    pub meta: Vec<AlertHistoryMeta>,
    /// Condition that triggered.
    pub condition: String,
    #[serde(with = "crate::time::datetime")]
    /// Time the alert triggered.
    pub created_at: NaiveDateTime,
    /// Order metadata for an ATO trigger, if any.
    pub order_meta: Option<Value>,
}

/// Quote metadata returned by `/alerts/{uuid}/history`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AlertHistoryMeta {
    /// Instrument token.
    pub instrument_token: u32,
    /// Trading symbol.
    pub tradingsymbol: String,
    #[serde(with = "crate::time::datetime")]
    /// Quote timestamp.
    pub timestamp: NaiveDateTime,
    /// Last traded price.
    pub last_price: f64,
    /// Day OHLC prices.
    pub ohlc: AlertOhlc,
    /// Net change of the price.
    pub net_change: f64,
    /// Exchange.
    pub exchange: String,
    #[serde(default, with = "crate::time::optional_datetime")]
    /// Last trade time, if available.
    pub last_trade_time: Option<NaiveDateTime>,
    /// Quantity of the last trade.
    pub last_quantity: i64,
    /// Total buy quantity.
    pub buy_quantity: i64,
    /// Total sell quantity.
    pub sell_quantity: i64,
    /// Volume traded.
    pub volume: i64,
    /// Volume tick.
    pub volume_tick: i64,
    /// Average traded price.
    pub average_price: f64,
    /// Open interest.
    pub oi: i64,
    /// Day high of open interest.
    pub oi_day_high: i64,
    /// Day low of open interest.
    pub oi_day_low: i64,
    /// Lower circuit limit.
    pub lower_circuit_limit: f64,
    /// Upper circuit limit.
    pub upper_circuit_limit: f64,
}

/// OHLC prices returned by `/alerts/{uuid}/history`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AlertOhlc {
    /// Open price.
    pub open: f64,
    /// High price.
    pub high: f64,
    /// Low price.
    pub low: f64,
    /// Close price.
    pub close: f64,
}

/// Alert type sent to `/alerts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum AlertType {
    /// Plain price alert (`simple`).
    Simple,
    /// Alert-to-order: firing it also places the alert's basket of orders.
    Ato,
    #[serde(other)]
    /// Any value this client does not recognise.
    Unknown,
}

impl fmt::Display for AlertType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unknown => "unknown",
            Self::Simple => "simple",
            Self::Ato => "ato",
        })
    }
}

/// Alert status sent to `/alerts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    /// Alert is deleted (`deleted`).
    Deleted,
    /// Alert is disabled (`disabled`).
    Disabled,
    /// Alert is enabled (`enabled`).
    Enabled,
    #[serde(other)]
    /// Any value this client does not recognise.
    Unknown,
}

impl fmt::Display for AlertStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unknown => "unknown",
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Deleted => "deleted",
        })
    }
}

/// Right-hand side type sent to `/alerts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum AlertRhsType {
    /// Compare against a constant (`constant`).
    Constant,
    /// Compare against another instrument (`instrument`).
    Instrument,
}

impl fmt::Display for AlertRhsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Constant => "constant",
            Self::Instrument => "instrument",
        })
    }
}

/// Comparison operator sent to `/alerts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum AlertOperator {
    #[serde(rename = "<=")]
    /// `<=` operator.
    Le,
    #[serde(rename = ">=")]
    /// `>=` operator.
    Ge,
    #[serde(rename = "<")]
    /// `<` operator.
    Lt,
    #[serde(rename = ">")]
    /// `>` operator.
    Gt,
    #[serde(rename = "==")]
    /// `==` operator.
    Eq,
}

impl fmt::Display for AlertOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Le => "<=",
            Self::Ge => ">=",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Eq => "==",
        })
    }
}

/// Parameters for `POST /alerts` and `PUT /alerts/{uuid}`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[must_use]
pub struct AlertParams {
    /// Alert name.
    pub name: String,
    /// Alert type.
    pub alert_type: AlertType,
    /// Left-hand side exchange.
    pub lhs_exchange: String,
    /// Left-hand side trading symbol.
    pub lhs_tradingsymbol: String,
    /// Left-hand side attribute, such as `LastTradedPrice`.
    pub lhs_attribute: String,
    /// Comparison operator.
    pub operator: AlertOperator,
    /// Right-hand side type.
    pub rhs_type: AlertRhsType,
    /// Right-hand side constant, used when `rhs_type` is `constant`.
    pub rhs_constant: f64,
    /// Right-hand side exchange, used for an instrument condition.
    pub rhs_exchange: String,
    /// Right-hand side trading symbol, used for an instrument condition.
    pub rhs_tradingsymbol: String,
    /// Right-hand side attribute, used for an instrument condition.
    pub rhs_attribute: String,
    /// Sent only when `alert_type` is [`AlertType::Ato`].
    pub basket: Option<BasketWrite>,
    /// A `PUT` without a status re-enables a disabled alert.
    pub status: Option<AlertStatus>,
}

impl AlertParams {
    /// Required condition fields for `/alerts`.
    pub fn new(
        name: impl Into<String>,
        alert_type: AlertType,
        lhs_exchange: impl Into<String>,
        lhs_tradingsymbol: impl Into<String>,
        lhs_attribute: impl Into<String>,
        operator: AlertOperator,
        rhs_type: AlertRhsType,
    ) -> Self {
        Self {
            name: name.into(),
            alert_type,
            lhs_exchange: lhs_exchange.into(),
            lhs_tradingsymbol: lhs_tradingsymbol.into(),
            lhs_attribute: lhs_attribute.into(),
            operator,
            rhs_type,
            rhs_constant: 0.0,
            rhs_exchange: String::new(),
            rhs_tradingsymbol: String::new(),
            rhs_attribute: String::new(),
            basket: None,
            status: None,
        }
    }
    /// Constant value sent to `/alerts`.
    pub fn rhs_constant(mut self, value: f64) -> Self {
        self.rhs_type = AlertRhsType::Constant;
        self.rhs_constant = value;
        self
    }
    /// Instrument condition sent to `/alerts`.
    pub fn rhs_instrument(
        mut self,
        exchange: impl Into<String>,
        symbol: impl Into<String>,
        attribute: impl Into<String>,
    ) -> Self {
        self.rhs_type = AlertRhsType::Instrument;
        self.rhs_exchange = exchange.into();
        self.rhs_tradingsymbol = symbol.into();
        self.rhs_attribute = attribute.into();
        self
    }
    /// Basket JSON sent to `/alerts` for an ATO.
    pub fn basket(mut self, basket: BasketWrite) -> Self {
        self.basket = Some(basket);
        self
    }
    /// Status sent to `/alerts`.
    pub fn status(mut self, status: AlertStatus) -> Self {
        self.status = Some(status);
        self
    }
    fn into_form(self) -> crate::Result<Vec<(&'static str, String)>> {
        let mut form = vec![
            ("name", self.name),
            ("type", self.alert_type.to_string()),
            ("lhs_exchange", self.lhs_exchange),
            ("lhs_tradingsymbol", self.lhs_tradingsymbol),
            ("lhs_attribute", self.lhs_attribute),
            ("operator", self.operator.to_string()),
            ("rhs_type", self.rhs_type.to_string()),
        ];
        match self.rhs_type {
            AlertRhsType::Constant => {
                if !self.rhs_constant.is_finite() {
                    return Err(Error::InvalidInput("alert numeric values must be finite"));
                }
                form.push(("rhs_constant", self.rhs_constant.to_string()));
            }
            AlertRhsType::Instrument => form.extend([
                ("rhs_exchange", self.rhs_exchange),
                ("rhs_tradingsymbol", self.rhs_tradingsymbol),
                ("rhs_attribute", self.rhs_attribute),
            ]),
        }
        if self.alert_type == AlertType::Ato
            && let Some(basket) = self.basket
        {
            if !basket.numeric_values_are_finite() {
                return Err(Error::InvalidInput("alert numeric values must be finite"));
            }
            let basket = serde_json::to_string(&basket)
                .map_err(|_| Error::InvalidInput("alert numeric values must be finite"))?;
            form.push(("basket", basket));
        }
        if let Some(status) = self.status {
            form.push(("status", status.to_string()));
        }
        Ok(form)
    }
}

impl KiteConnect {
    /// `GET /alerts` with optional filters.
    pub fn alerts(&self, filters: &HashMap<String, String>) -> Call<Vec<Alert>> {
        Call::new(
            self.clone(),
            Request::get("/alerts").query(
                filters
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str())),
            ),
        )
    }
    /// `GET /alerts/{uuid}`.
    pub fn alert(&self, uuid: &str) -> Call<Alert> {
        let uuid = crate::client::path_segment(uuid);
        Call::new(self.clone(), Request::get(format!("/alerts/{uuid}")))
    }
    /// `GET /alerts/{uuid}/history`.
    pub fn alert_history(&self, uuid: &str) -> Call<Vec<AlertHistory>> {
        let uuid = crate::client::path_segment(uuid);
        Call::new(
            self.clone(),
            Request::get(format!("/alerts/{uuid}/history")),
        )
    }
    /// `POST /alerts`.
    pub fn create_alert(&self, params: AlertParams) -> Call<Alert> {
        let request = Request::post("/alerts");
        match params.into_form() {
            Ok(form) => Call::new(self.clone(), request.form(form)),
            Err(error) => Call::failed(self.clone(), request, error),
        }
    }
    /// `PUT /alerts/{uuid}`.
    pub fn modify_alert(&self, uuid: &str, params: AlertParams) -> Call<Alert> {
        let uuid = crate::client::path_segment(uuid);
        let request = Request::put(format!("/alerts/{uuid}"));
        match params.into_form() {
            Ok(form) => Call::new(self.clone(), request.form(form)),
            Err(error) => Call::failed(self.clone(), request, error),
        }
    }
    /// `DELETE /alerts?uuid=..`.
    ///
    /// The one call that can fail before sending: an empty `uuids` is refused,
    /// because a bare `DELETE /alerts` may delete every alert.
    ///
    /// # Errors
    /// Returns [`crate::Error::InvalidInput`] if `uuids` is empty.
    pub fn delete_alerts(&self, uuids: &[&str]) -> crate::Result<Call<bool>> {
        if uuids.is_empty() {
            return Err(crate::Error::InvalidInput(
                "at least one uuid must be provided",
            ));
        }
        Ok(Call::with_decoder(
            self.clone(),
            Request::delete("/alerts").query(uuids.iter().map(|uuid| ("uuid", *uuid))),
            crate::call::success_bool,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_form(params: AlertParams) -> Vec<(String, String)> {
        let client = KiteConnect::builder("k").build().unwrap();
        let call = client.create_alert(params);
        let crate::Body::Form(form) = call.request().body() else {
            panic!("form")
        };
        form.clone()
    }
    fn params() -> AlertParams {
        AlertParams::new(
            "NIFTY 50",
            AlertType::Simple,
            "INDICES",
            "NIFTY 50",
            "LastTradedPrice",
            AlertOperator::Ge,
            AlertRhsType::Constant,
        )
        .rhs_constant(27000.0)
        .status(AlertStatus::Enabled)
    }
    fn ato(basket: Option<BasketWrite>) -> AlertParams {
        AlertParams {
            alert_type: AlertType::Ato,
            basket,
            ..params()
        }
    }

    fn lookup<'a>(form: &'a [(String, String)], key: &str) -> Option<&'a str> {
        form.iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    fn order_params() -> BasketOrderParams {
        BasketOrderParams::new("regular", "BUY", "CNC", "LIMIT", "DAY", 10, 1500.0)
    }

    fn insert(symbol: &str, weight: u32) -> BasketOperation {
        BasketOperation::Insert {
            exchange: "NSE".to_string(),
            tradingsymbol: symbol.to_string(),
            weight,
            params: BasketItemParams::Authored(order_params()),
        }
    }

    /// The `basket` form field, parsed back into JSON so the assertions are
    /// about the document the service receives rather than about key order.
    fn basket_json(form: &[(String, String)]) -> Value {
        serde_json::from_str(lookup(form, "basket").expect("form carries a basket"))
            .expect("basket field is JSON")
    }

    #[test]
    fn enums_display_their_wire_strings() {
        assert_eq!(AlertType::Simple.to_string(), "simple");
        assert_eq!(AlertType::Ato.to_string(), "ato");
        assert_eq!(AlertStatus::Enabled.to_string(), "enabled");
        assert_eq!(AlertStatus::Disabled.to_string(), "disabled");
        assert_eq!(AlertStatus::Deleted.to_string(), "deleted");
        assert_eq!(AlertRhsType::Constant.to_string(), "constant");
        assert_eq!(AlertRhsType::Instrument.to_string(), "instrument");
        assert_eq!(AlertOperator::Le.to_string(), "<=");
        assert_eq!(AlertOperator::Eq.to_string(), "==");
        for status in [
            AlertStatus::Enabled,
            AlertStatus::Disabled,
            AlertStatus::Deleted,
        ] {
            assert_eq!(
                serde_json::to_value(status).unwrap(),
                Value::String(status.to_string())
            );
        }
    }

    #[test]
    fn alert_form_matches_the_wire_contract() {
        let form = test_form(params());
        let lookup =
            |key: &str| lookup(&form, key).unwrap_or_else(|| panic!("missing form field {key}"));
        assert_eq!(lookup("type"), "simple");
        assert_eq!(lookup("name"), "NIFTY 50");
        assert_eq!(lookup("lhs_exchange"), "INDICES");
        assert_eq!(lookup("lhs_tradingsymbol"), "NIFTY 50");
        assert_eq!(lookup("lhs_attribute"), "LastTradedPrice");
        assert_eq!(lookup("operator"), ">=");
        assert_eq!(lookup("rhs_type"), "constant");
        assert_eq!(lookup("rhs_constant"), "27000");
        assert_eq!(lookup("status"), "enabled");
        assert_eq!(super::tests::lookup(&form, "basket"), None);
    }

    /// `rhs_type` is the service's "revalidate the condition" flag on modify and
    /// `status` stops a level edit from re-arming a disabled alert. Both are
    /// easy to "tidy away" as redundant, so pin them explicitly.
    ///
    /// `operator` is pinned alongside `rhs_type` because the live service rejects
    /// a `PUT` that carries one without the other
    /// (`400 InputException: Invalid operator`): a condition modify is all-or-
    /// nothing, never a delta.
    #[test]
    fn alert_form_always_carries_the_whole_condition_and_status() {
        for status in [AlertStatus::Enabled, AlertStatus::Disabled] {
            let form = test_form(params().status(status));
            assert!(form.iter().any(|(k, v)| k == "rhs_type" && v == "constant"));
            assert!(form.iter().any(|(k, v)| k == "operator" && v == ">="));
            assert!(
                form.iter()
                    .any(|(k, v)| k == "status" && *v == status.to_string())
            );
        }
        let unset = AlertParams {
            status: None,
            ..params()
        };
        assert_eq!(lookup(&test_form(unset), "status"), None);
    }

    #[test]
    fn fractional_levels_survive_form_encoding() {
        let form = test_form(AlertParams {
            rhs_constant: 71.85,
            ..params()
        });
        assert!(
            form.iter()
                .any(|(k, v)| k == "rhs_constant" && v == "71.85")
        );
    }

    /// A creation carries the whole basket, including the metadata only a
    /// creation may set. `type` is `ato` on the alert and `alert` on the basket:
    /// two different fields whose values are easy to cross.
    #[test]
    fn ato_create_form_carries_the_basket_and_its_metadata() {
        let basket = BasketWrite::Create(BasketCreate::new(
            "alerts-basket",
            vec![insert("INFY", 1), insert("TCS", 2)],
        ));
        let form = test_form(ato(Some(basket)));
        assert_eq!(lookup(&form, "type"), Some("ato"));
        let basket = basket_json(&form);
        assert_eq!(basket["name"], "alerts-basket");
        assert_eq!(basket["type"], "alert");
        assert_eq!(basket["tags"], serde_json::json!([]));
        let items = basket["items"].as_array().expect("items array");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["type"], "insert");
        assert_eq!(items[0]["tradingsymbol"], "INFY");
        assert_eq!(items[0]["weight"], 1);
        assert_eq!(items[1]["weight"], 2);
        assert!(items[0].get("id").is_none());
    }

    /// The order parameters go out as JSON **numbers**, not numeric strings, and
    /// every field is present. Both matter: the Go decoder enforces the
    /// structural types, and an `update` replaces the stored params object, so an
    /// omitted field is an erased field rather than an unchanged one.
    #[test]
    fn basket_order_params_serialize_as_numbers_and_carry_every_field() {
        let basket = BasketWrite::Create(BasketCreate::new("b", vec![insert("INFY", 1)]));
        let params_json = basket_json(&test_form(ato(Some(basket))))["items"][0]["params"].clone();
        assert_eq!(params_json["variety"], "regular");
        assert_eq!(params_json["transaction_type"], "BUY");
        assert_eq!(params_json["product"], "CNC");
        assert_eq!(params_json["order_type"], "LIMIT");
        assert_eq!(params_json["validity"], "DAY");
        for key in [
            "validity_ttl",
            "quantity",
            "disclosed_quantity",
            "price",
            "trigger_price",
            "squareoff",
            "stoploss",
            "trailing_stoploss",
        ] {
            assert!(
                params_json[key].is_number(),
                "{key} must be a JSON number, got {}",
                params_json[key]
            );
        }
        assert_eq!(params_json["quantity"], 10);
        assert_eq!(params_json["price"], 1500.0);
        assert_eq!(params_json["gtt"]["target"], 0.0);
        assert_eq!(params_json["gtt"]["stoploss"], 0.0);
        assert_eq!(params_json["tags"], serde_json::json!([]));
        let mut keys: Vec<&str> = params_json
            .as_object()
            .expect("params object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "disclosed_quantity",
                "gtt",
                "order_type",
                "price",
                "product",
                "quantity",
                "squareoff",
                "stoploss",
                "tags",
                "trailing_stoploss",
                "transaction_type",
                "trigger_price",
                "validity",
                "validity_ttl",
                "variety",
            ]
        );
    }

    /// A basket modify carries operations only: no `name`, `type` or `tags`.
    ///
    /// The alerts service does not return a basket's metadata, so a write that
    /// sent any of it would overwrite values the caller cannot read back.
    #[test]
    fn ato_modify_form_sends_operations_without_basket_metadata() {
        let basket = BasketWrite::Modify(BasketModify::new(vec![
            insert("SBIN", 3),
            BasketOperation::Update {
                id: 275_218_517,
                exchange: "NSE".to_string(),
                tradingsymbol: "INFY".to_string(),
                weight: 1,
                params: BasketItemParams::Authored(order_params()),
            },
            BasketOperation::Delete { id: 99 },
        ]));
        let form = test_form(ato(Some(basket)));
        assert_eq!(lookup(&form, "type"), Some("ato"));
        let basket = basket_json(&form);
        let object = basket.as_object().expect("basket object");
        assert_eq!(object.keys().collect::<Vec<_>>(), vec!["items"]);
        let items = basket["items"].as_array().expect("items array");
        assert_eq!(items[0]["type"], "insert");
        assert_eq!(items[1]["type"], "update");
        assert_eq!(items[1]["id"], 275_218_517);
        assert_eq!(items[1]["weight"], 1);
        assert!(items[1]["params"].is_object(), "an update replaces params");
        assert_eq!(
            items[2],
            serde_json::json!({"type": "delete", "id": 99}),
            "a delete carries only its id"
        );
    }

    /// An ATO write without a basket sends `type=ato` and **no** basket field,
    /// which is what makes the service preserve the stored orders.
    #[test]
    fn an_ato_without_a_basket_omits_the_field() {
        let form = test_form(ato(None));
        assert_eq!(lookup(&form, "type"), Some("ato"));
        assert_eq!(lookup(&form, "basket"), None);
        assert_eq!(lookup(&form, "status"), Some("enabled"));
        assert_eq!(lookup(&form, "operator"), Some(">="));
    }

    /// Go sends a basket only for an ATO; a simple alert's form never names one.
    #[test]
    fn a_simple_alert_never_sends_a_basket() {
        let basket = BasketWrite::Create(BasketCreate::new("b", vec![insert("INFY", 1)]));
        let form = test_form(params().basket(basket));
        assert_eq!(lookup(&form, "type"), Some("simple"));
        assert_eq!(lookup(&form, "basket"), None);
    }

    /// A preserved item's params are echoed back byte-for-byte, because an
    /// `update` replaces them.
    #[test]
    fn a_preserved_items_params_go_back_verbatim() {
        let fetched = serde_json::json!({
            "order_type": "SL-M",
            "quantity": 10,
            "price": 0,
            "disclosed_quantity": 4,
            "gtt": {"target": 2.5, "stoploss": -1.5}
        });
        let basket = BasketWrite::Modify(BasketModify::new(vec![BasketOperation::Update {
            id: 7,
            exchange: "NSE".to_string(),
            tradingsymbol: "INFY".to_string(),
            weight: 2,
            params: BasketItemParams::Preserved(fetched.clone()),
        }]));
        let basket = basket_json(&test_form(ato(Some(basket))));
        assert_eq!(basket["items"][0]["params"], fetched);
        assert_eq!(basket["items"][0]["weight"], 2);
    }

    /// The alert `type` and the basket `type` are different fields with
    /// different vocabularies, and crossing them is the mistake this pins.
    #[test]
    fn the_alert_type_and_the_basket_type_are_not_the_same_field() {
        let basket = BasketWrite::Create(BasketCreate::new("b", vec![insert("INFY", 1)]));
        let form = test_form(ato(Some(basket)));
        assert_eq!(lookup(&form, "type"), Some("ato"));
        assert_eq!(basket_json(&form)["type"], "alert");
    }

    /// `basket_id` is declared on the service's params struct but the validators
    /// never copy it into the alert: the handler owns the linkage. Sending one
    /// would be claiming to control something this client does not.
    #[test]
    fn no_write_ever_names_a_basket_id() {
        for params in [
            params(),
            ato(None),
            ato(Some(BasketWrite::Create(BasketCreate::new(
                "b",
                vec![insert("INFY", 1)],
            )))),
            ato(Some(BasketWrite::Modify(BasketModify::new(vec![
                BasketOperation::Delete { id: 5 },
            ])))),
        ] {
            let form = test_form(params);
            assert_eq!(lookup(&form, "basket_id"), None);
            if let Some(basket) = lookup(&form, "basket") {
                assert!(
                    !basket.contains("basket_id") && !basket.contains("\"id\":0"),
                    "a write must not name a basket id: {basket}"
                );
            }
        }
    }

    #[tokio::test]
    async fn non_finite_alert_values_return_errors() {
        let client = KiteConnect::builder("k").build().unwrap();
        let error = client
            .create_alert(params().rhs_constant(f64::NAN))
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput("alert numeric values must be finite")
        ));

        let basket = BasketWrite::Create(BasketCreate::new(
            "b",
            vec![BasketOperation::Insert {
                exchange: "NSE".into(),
                tradingsymbol: "INFY".into(),
                weight: 1,
                params: BasketItemParams::Authored(BasketOrderParams {
                    price: f64::INFINITY,
                    ..order_params()
                }),
            }],
        ));
        let error = client.create_alert(ato(Some(basket))).await.unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput("alert numeric values must be finite")
        ));
    }
}
