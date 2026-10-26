use crate::{
    Call, Error, Exchange, KiteConnect, OrderType, Product, Request, TransactionType, Variety,
};
use serde::{Deserialize, Serialize};

/// Order in the `POST /margins/orders` and `POST /margins/basket` body.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
#[must_use]
pub struct OrderMarginParam {
    /// Exchange of the order.
    pub exchange: Exchange,
    /// Trading symbol of the order.
    pub tradingsymbol: String,
    /// Transaction type (buy or sell).
    pub transaction_type: TransactionType,
    /// Order variety.
    pub variety: Variety,
    /// Product type.
    pub product: Product,
    /// Order type.
    pub order_type: OrderType,
    /// Order quantity.
    pub quantity: i64,
    /// Order price, omitted when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// Trigger price, omitted when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<f64>,
    /// Market protection percentage, omitted when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_protection: Option<f64>,
}

impl OrderMarginParam {
    /// Creates an order with the required fields.
    pub fn new(
        exchange: Exchange,
        tradingsymbol: impl Into<String>,
        transaction_type: TransactionType,
        variety: Variety,
        product: Product,
        order_type: OrderType,
        quantity: i64,
    ) -> Self {
        Self {
            exchange,
            tradingsymbol: tradingsymbol.into(),
            transaction_type,
            variety,
            product,
            order_type,
            quantity,
            price: None,
            trigger_price: None,
            market_protection: None,
        }
    }
    /// Sets the order price.
    pub fn price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }
    /// Sets the trigger price.
    pub fn trigger_price(mut self, price: f64) -> Self {
        self.trigger_price = Some(price);
        self
    }
    /// Set the market protection percentage for the margin request.
    pub fn market_protection(mut self, protection: f64) -> Self {
        self.market_protection = Some(protection);
        self
    }

    fn numeric_values_are_finite(&self) -> bool {
        [self.price, self.trigger_price, self.market_protection]
            .into_iter()
            .flatten()
            .all(f64::is_finite)
    }
}

/// Order in the `POST /charges/orders` body.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct OrderChargesParam {
    /// ID of the order.
    pub order_id: String,
    /// Exchange of the order.
    pub exchange: Exchange,
    /// Trading symbol of the order.
    pub tradingsymbol: String,
    /// Transaction type (buy or sell).
    pub transaction_type: TransactionType,
    /// Order variety.
    pub variety: Variety,
    /// Product type.
    pub product: Product,
    /// Order type.
    pub order_type: OrderType,
    /// Order quantity.
    pub quantity: i64,
    /// Average price of the order.
    pub average_price: f64,
}

impl OrderChargesParam {
    /// Creates an order with the required fields.
    #[expect(
        clippy::too_many_arguments,
        reason = "Required fields of the Kite charges endpoint"
    )]
    pub fn new(
        order_id: impl Into<String>,
        exchange: Exchange,
        tradingsymbol: impl Into<String>,
        transaction_type: TransactionType,
        variety: Variety,
        product: Product,
        order_type: OrderType,
        quantity: i64,
        average_price: f64,
    ) -> Self {
        Self {
            order_id: order_id.into(),
            exchange,
            tradingsymbol: tradingsymbol.into(),
            transaction_type,
            variety,
            product,
            order_type,
            quantity,
            average_price,
        }
    }

    fn numeric_values_are_finite(&self) -> bool {
        self.average_price.is_finite()
    }
}

/// Parameters for `POST /margins/orders`.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use]
pub struct MarginParams {
    /// Orders to compute margins for.
    pub order_params: Vec<OrderMarginParam>,
    /// Whether to request compact output (`mode=compact`).
    pub compact: bool,
}

impl MarginParams {
    /// Creates parameters for the given orders.
    pub fn new(order_params: Vec<OrderMarginParam>) -> Self {
        Self {
            order_params,
            compact: false,
        }
    }
    /// Sets compact output.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
}

/// Parameters for `POST /margins/basket`.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use]
pub struct BasketParams {
    /// Orders to compute basket margins for.
    pub order_params: Vec<OrderMarginParam>,
    /// Whether to request compact output (`mode=compact`).
    pub compact: bool,
    /// Whether to consider existing positions.
    pub consider_positions: bool,
}

impl BasketParams {
    /// Creates parameters for the given orders.
    pub fn new(order_params: Vec<OrderMarginParam>) -> Self {
        Self {
            order_params,
            compact: false,
            consider_positions: false,
        }
    }
    /// Sets compact output.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    /// Sets whether existing positions are considered.
    pub fn consider_positions(mut self, consider_positions: bool) -> Self {
        self.consider_positions = consider_positions;
        self
    }
}

/// Parameters for `POST /charges/orders`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ChargesParams {
    /// Orders to compute charges for.
    pub order_params: Vec<OrderChargesParam>,
}

impl ChargesParams {
    /// Creates parameters for the given orders.
    #[must_use]
    pub fn new(order_params: Vec<OrderChargesParam>) -> Self {
        Self { order_params }
    }
}

/// Profit and loss returned by the Kite margin endpoints.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MarginPnl {
    /// Realised profit and loss.
    pub realised: f64,
    /// Unrealised profit and loss.
    pub unrealised: f64,
}

/// GST breakdown returned by the Kite margin and charges endpoints.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Gst {
    /// Integrated GST.
    pub igst: f64,
    /// Central GST.
    pub cgst: f64,
    /// State GST.
    pub sgst: f64,
    /// Total GST.
    pub total: f64,
}

/// Charges breakdown returned by the Kite margin and charges endpoints.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Charges {
    /// Transaction tax (for example STT).
    pub transaction_tax: f64,
    /// Type of transaction tax.
    pub transaction_tax_type: String,
    /// Exchange turnover charge.
    pub exchange_turnover_charge: f64,
    /// SEBI turnover charge.
    pub sebi_turnover_charge: f64,
    /// Brokerage charge.
    pub brokerage: f64,
    /// Stamp duty.
    pub stamp_duty: f64,
    /// GST breakdown.
    pub gst: Gst,
    /// Total charges.
    pub total: f64,
}

/// Order or basket totals returned by POST /margins/orders and /margins/basket.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OrderMargins {
    /// Margin type (`type` in the response).
    #[serde(rename = "type")]
    pub margin_type: Option<String>,
    /// Trading symbol of the order.
    pub tradingsymbol: Option<String>,
    /// Exchange of the order.
    pub exchange: Option<String>,
    /// SPAN margin.
    pub span: Option<f64>,
    /// Exposure margin.
    pub exposure: Option<f64>,
    /// Option premium.
    pub option_premium: Option<f64>,
    /// Additional margin.
    pub additional: Option<f64>,
    /// Bracket order margin.
    pub bo: Option<f64>,
    /// Cash margin.
    pub cash: Option<f64>,
    /// VAR margin.
    pub var: Option<f64>,
    /// Profit and loss figures.
    pub pnl: Option<MarginPnl>,
    /// Leverage applied.
    pub leverage: Option<f64>,
    /// Charges breakdown, if any.
    pub charges: Option<Charges>,
    /// Total margin required.
    pub total: f64,
}

/// Initial, final and per-order totals returned by POST /margins/basket.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BasketMargins {
    /// Initial margin totals.
    pub initial: OrderMargins,
    /// Final margin totals (`final` in the response).
    #[serde(rename = "final")]
    pub final_margin: OrderMargins,
    /// Per-order margin totals.
    #[serde(default)]
    pub orders: Vec<OrderMargins>,
    /// Charges breakdown for the basket, if any.
    pub charges: Option<Charges>,
}

/// Order charges returned by POST /charges/orders.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OrderCharges {
    /// Exchange of the order.
    pub exchange: String,
    /// Trading symbol of the order.
    pub tradingsymbol: String,
    /// Transaction type (buy or sell).
    pub transaction_type: String,
    /// Order variety.
    pub variety: String,
    /// Product type.
    pub product: String,
    /// Order type.
    pub order_type: String,
    /// Order quantity.
    pub quantity: f64,
    /// Order price.
    pub price: f64,
    /// Charges breakdown for the order.
    pub charges: Charges,
}

impl KiteConnect {
    /// `POST /margins/orders`.
    pub fn order_margins(&self, params: MarginParams) -> Call<Vec<OrderMargins>> {
        let MarginParams {
            order_params,
            compact,
        } = params;
        let mut query = Vec::new();
        if compact {
            query.push(("mode", "compact"));
        }
        let request = Request::post("/margins/orders").query(query);
        if !order_params
            .iter()
            .all(OrderMarginParam::numeric_values_are_finite)
        {
            return Call::failed(
                self.clone(),
                request,
                Error::InvalidInput("margin numeric values must be finite"),
            );
        }
        match serde_json::to_value(order_params) {
            Ok(value) => Call::new(self.clone(), request.json(value)),
            Err(_) => Call::failed(
                self.clone(),
                request,
                Error::InvalidInput("margin numeric values must be finite"),
            ),
        }
    }
    /// `POST /margins/basket`.
    pub fn basket_margins(&self, params: BasketParams) -> Call<BasketMargins> {
        let BasketParams {
            order_params,
            compact,
            consider_positions,
        } = params;
        let mut query = Vec::new();
        if consider_positions {
            query.push(("consider_positions", "true"));
        }
        if compact {
            query.push(("mode", "compact"));
        }
        let request = Request::post("/margins/basket").query(query);
        if !order_params
            .iter()
            .all(OrderMarginParam::numeric_values_are_finite)
        {
            return Call::failed(
                self.clone(),
                request,
                Error::InvalidInput("margin numeric values must be finite"),
            );
        }
        match serde_json::to_value(order_params) {
            Ok(value) => Call::new(self.clone(), request.json(value)),
            Err(_) => Call::failed(
                self.clone(),
                request,
                Error::InvalidInput("margin numeric values must be finite"),
            ),
        }
    }
    /// `POST /charges/orders`.
    pub fn order_charges(&self, params: ChargesParams) -> Call<Vec<OrderCharges>> {
        let ChargesParams { order_params } = params;
        let request = Request::post("/charges/orders");
        if !order_params
            .iter()
            .all(OrderChargesParam::numeric_values_are_finite)
        {
            return Call::failed(
                self.clone(),
                request,
                Error::InvalidInput("margin numeric values must be finite"),
            );
        }
        match serde_json::to_value(order_params) {
            Ok(value) => Call::new(self.clone(), request.json(value)),
            Err(_) => Call::failed(
                self.clone(),
                request,
                Error::InvalidInput("margin numeric values must be finite"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::{Value, json};

    fn fixture<T: serde::de::DeserializeOwned>(body: &str) -> T {
        let value: Value = serde_json::from_str(body).unwrap();
        serde_json::from_value(value["data"].clone()).unwrap()
    }
    fn client(server: &MockServer) -> KiteConnect {
        KiteConnect::builder("key")
            .base_url(server.base_url())
            .build()
            .unwrap()
    }
    fn order() -> OrderMarginParam {
        OrderMarginParam::new(
            Exchange::Nse,
            "INFY",
            TransactionType::Buy,
            Variety::Regular,
            Product::Cnc,
            OrderType::Market,
            1,
        )
    }
    #[test]
    fn fixtures_decode_all_margin_and_charge_types() {
        let margins: Vec<OrderMargins> =
            fixture(include_str!("../mock_responses/order_margins.json"));
        assert_eq!(margins.len(), 1);
        assert_eq!(margins[0].total.to_bits(), f64::to_bits(1_498.0));
        assert_eq!(margins[0].tradingsymbol.as_deref(), Some("INFY"));
        assert_eq!(margins[0].var, Some(1498.0));
        assert_eq!(
            margins[0].pnl.as_ref().unwrap().realised.to_bits(),
            f64::to_bits(0.0)
        );
        assert_eq!(
            margins[0].pnl.as_ref().unwrap().unrealised.to_bits(),
            f64::to_bits(0.0)
        );
        let charges = margins[0].charges.as_ref().unwrap();
        assert_eq!(charges.transaction_tax.to_bits(), f64::to_bits(1.498));
        assert_eq!(charges.transaction_tax_type, "stt");
        assert_eq!(charges.brokerage.to_bits(), f64::to_bits(0.01));
        assert_eq!(charges.total.to_bits(), f64::to_bits(1.792_551_22));
        assert!((charges.gst.total - 0.011_372_219_999_999_999).abs() < 1e-15);
        assert_eq!(charges.gst.cgst.to_bits(), f64::to_bits(0.0));
        assert_eq!(charges.gst.sgst.to_bits(), f64::to_bits(0.0));
        let basket: BasketMargins = fixture(include_str!("../mock_responses/basket_margins.json"));
        assert_eq!(basket.initial.total.to_bits(), f64::to_bits(96_504.975));
        assert!((basket.final_margin.total - 34_786.725_000_000_006).abs() < 1e-9);
        assert_eq!(basket.orders.len(), 2);
        assert_eq!(basket.orders[0].total.to_bits(), f64::to_bits(95_983.725));
        assert_eq!(basket.orders[1].total.to_bits(), f64::to_bits(521.25));
        assert_eq!(
            basket.charges.as_ref().unwrap().brokerage.to_bits(),
            f64::to_bits(40.0)
        );
        let charges: Vec<OrderCharges> =
            fixture(include_str!("../mock_responses/virtual_contract_note.json"));
        assert_eq!(charges.len(), 3);
        assert_eq!(charges[0].tradingsymbol, "SBIN");
        assert_eq!(charges[0].price.to_bits(), f64::to_bits(560.0));
        assert_eq!(
            charges[0].charges.total.to_bits(),
            f64::to_bits(0.582_696_8)
        );
        assert_eq!(charges[1].exchange, "MCX");
        assert_eq!(charges[1].charges.transaction_tax_type, "ctt");
        assert_eq!(charges[2].quantity.to_bits(), f64::to_bits(100.0));
    }
    #[test]
    fn compact_totals_decode_and_missing_totals_fail() {
        let margin: OrderMargins = serde_json::from_value(json!({"total":0})).unwrap();
        assert_eq!(margin.total.to_bits(), f64::to_bits(0.0));
        assert!(margin.charges.is_none());
        for value in [json!({}), json!({"total":null}), json!({"total":"1"})] {
            assert!(serde_json::from_value::<OrderMargins>(value).is_err());
        }
        let basket: BasketMargins =
            serde_json::from_value(json!({"initial":{"total":1}, "final":{"total":2}})).unwrap();
        assert_eq!(basket.initial.total.to_bits(), f64::to_bits(1.0));
        assert_eq!(basket.final_margin.total.to_bits(), f64::to_bits(2.0));
        assert!(basket.orders.is_empty());
        let mut data: Value =
            serde_json::from_str(include_str!("../mock_responses/order_margins.json")).unwrap();
        data["data"][0]["charges"]
            .as_object_mut()
            .unwrap()
            .remove("total");
        assert!(serde_json::from_value::<Vec<OrderMargins>>(data["data"].clone()).is_err());
        let mut data: Value =
            serde_json::from_str(include_str!("../mock_responses/order_margins.json")).unwrap();
        data["data"][0]["charges"]["gst"]
            .as_object_mut()
            .unwrap()
            .remove("total");
        assert!(serde_json::from_value::<Vec<OrderMargins>>(data["data"].clone()).is_err());
        for section in ["initial", "final"] {
            let mut data: Value =
                serde_json::from_str(include_str!("../mock_responses/basket_margins.json"))
                    .unwrap();
            data["data"][section]
                .as_object_mut()
                .unwrap()
                .remove("total");
            assert!(serde_json::from_value::<BasketMargins>(data["data"].clone()).is_err());
        }
    }
    #[test]
    fn params_serialize_go_fields_and_omit_unset_prices() {
        assert_eq!(
            serde_json::to_value(order()).unwrap(),
            json!({
                "exchange":"NSE", "tradingsymbol":"INFY", "transaction_type":"BUY", "variety":"regular", "product":"CNC", "order_type":"MARKET", "quantity":1
            })
        );
        let priced = order().price(10.0).trigger_price(9.0);
        let value = serde_json::to_value(priced).unwrap();
        assert_eq!(value["price"], 10.0);
        assert_eq!(value["trigger_price"], 9.0);
        let charge = OrderChargesParam::new(
            "OID-1",
            Exchange::Nse,
            "INFY",
            TransactionType::Sell,
            Variety::Regular,
            Product::Cnc,
            OrderType::Limit,
            2,
            1498.0,
        );
        assert_eq!(
            serde_json::to_value(charge).unwrap(),
            json!({
                "order_id":"OID-1", "exchange":"NSE", "tradingsymbol":"INFY", "transaction_type":"SELL", "variety":"regular", "product":"CNC", "order_type":"LIMIT", "quantity":2,"average_price":1498.0
            })
        );
    }

    #[tokio::test]
    async fn non_finite_json_inputs_return_errors() {
        let client = KiteConnect::builder("key").build().unwrap();
        let error = client
            .order_margins(MarginParams::new(vec![order().price(f64::NAN)]))
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput("margin numeric values must be finite")
        ));

        let error = client
            .basket_margins(BasketParams::new(vec![order().price(f64::INFINITY)]))
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput("margin numeric values must be finite")
        ));

        let error = client
            .order_charges(ChargesParams::new(vec![OrderChargesParam::new(
                "OID-1",
                Exchange::Nse,
                "INFY",
                TransactionType::Buy,
                Variety::Regular,
                Product::Cnc,
                OrderType::Market,
                1,
                f64::NEG_INFINITY,
            )]))
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput("margin numeric values must be finite")
        ));
    }

    #[tokio::test]
    async fn order_margins_request() {
        let server = MockServer::start_async().await;
        let mock = server.mock_async(|when, then| {
            when.method(POST).path("/margins/orders").query_param("mode", "compact")
                .header("content-type", "application/json")
                .json_body(json!([{"exchange":"NSE", "tradingsymbol":"INFY", "transaction_type":"BUY", "variety":"regular", "product":"CNC", "order_type":"MARKET", "quantity":1}]));
            then.status(200).json_body(json!({"status":"success", "data":[{"total":1498}]}));
        }).await;
        let margins = client(&server)
            .order_margins(MarginParams::new(vec![order()]).compact(true))
            .await
            .unwrap();
        assert_eq!(margins[0].total.to_bits(), f64::to_bits(1_498.0));
        mock.assert_async().await;
        assert_eq!(
            client(&server)
                .order_margins(MarginParams::new(vec![]))
                .request()
                .query_pairs()
                .len(),
            0
        );
    }
    #[tokio::test]
    async fn basket_margins_request() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/margins/basket")
                    .query_param("mode", "compact")
                    .query_param("consider_positions", "true")
                    .json_body(json!([{"exchange":"NSE", "tradingsymbol":"INFY", "transaction_type":"BUY", "variety":"regular", "product":"CNC", "order_type":"MARKET", "quantity":1, "price":0.0}]));
                then.status(200)
                    .body(include_str!("../mock_responses/basket_margins.json"));
            })
            .await;
        let basket = client(&server)
            .basket_margins(
                BasketParams::new(vec![order().price(0.0)])
                    .compact(true)
                    .consider_positions(true),
            )
            .await
            .unwrap();
        assert_eq!(basket.initial.total.to_bits(), f64::to_bits(96_504.975));
        mock.assert_async().await;
        assert_eq!(
            client(&server)
                .basket_margins(BasketParams::new(vec![order()]))
                .request()
                .query_pairs()
                .len(),
            0
        );
    }
    #[tokio::test]
    async fn order_charges_request() {
        let server = MockServer::start_async().await;
        let mock = server.mock_async(|when, then| {
            when.method(POST).path("/charges/orders").header("content-type", "application/json")
                .json_body(json!([{"order_id":"OID-1", "exchange":"NSE", "tradingsymbol":"SBIN", "transaction_type":"BUY", "variety":"regular", "product":"CNC", "order_type":"MARKET", "quantity":1,"average_price":560.0}]));
            then.status(200).body(include_str!("../mock_responses/virtual_contract_note.json"));
        }).await;
        let orders = vec![OrderChargesParam::new(
            "OID-1",
            Exchange::Nse,
            "SBIN",
            TransactionType::Buy,
            Variety::Regular,
            Product::Cnc,
            OrderType::Market,
            1,
            560.0,
        )];
        let charges = client(&server)
            .order_charges(ChargesParams::new(orders))
            .await
            .unwrap();
        assert_eq!(
            charges[0].charges.total.to_bits(),
            f64::to_bits(0.582_696_8)
        );
        mock.assert_async().await;
    }
}
