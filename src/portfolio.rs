use crate::{Call, Exchange, KiteConnect, Product, RawResponse, Request, Result, TransactionType};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

/// Holding returned by GET /portfolio/holdings.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Holding {
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Exchange the instrument trades on.
    pub exchange: String,
    /// Numeric instrument token.
    pub instrument_token: u32,
    /// ISIN of the instrument.
    pub isin: String,
    /// Product type of the holding.
    pub product: String,
    /// Price reported for the holding.
    pub price: f64,
    /// Quantity of the holding in use.
    #[serde(deserialize_with = "whole_number")]
    pub used_quantity: i64,
    /// Quantity held.
    #[serde(deserialize_with = "whole_number")]
    pub quantity: i64,
    /// Quantity bought and pending settlement (T1).
    #[serde(deserialize_with = "whole_number")]
    pub t1_quantity: i64,
    /// Realised (settled) quantity.
    #[serde(deserialize_with = "whole_number")]
    pub realised_quantity: i64,
    /// Quantity authorised for sale.
    #[serde(deserialize_with = "whole_number")]
    pub authorised_quantity: i64,
    /// Date and time of the sale authorisation, if any.
    #[serde(default, with = "crate::time::optional_datetime")]
    pub authorised_date: Option<NaiveDateTime>,
    /// Quantity at the start of the day.
    #[serde(deserialize_with = "whole_number")]
    pub opening_quantity: i64,
    /// Quantity pledged as collateral.
    #[serde(deserialize_with = "whole_number")]
    pub collateral_quantity: i64,
    /// Type of collateral.
    pub collateral_type: String,
    /// Whether the holding has a discrepancy.
    pub discrepancy: bool,
    /// Average buy price.
    pub average_price: f64,
    /// Last traded price.
    pub last_price: f64,
    /// Previous close price.
    pub close_price: f64,
    /// Profit and loss.
    pub pnl: f64,
    /// Change in last price from the previous close.
    pub day_change: Option<f64>,
    /// Day change as a percentage.
    pub day_change_percentage: f64,
    /// MTF (margin trading facility) details, if any.
    pub mtf: Option<MtfHolding>,
}

/// `MtfHolding` returned by GET /portfolio/holdings.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MtfHolding {
    /// Quantity held under MTF.
    #[serde(deserialize_with = "whole_number")]
    pub quantity: i64,
    /// MTF quantity in use.
    #[serde(deserialize_with = "whole_number")]
    pub used_quantity: i64,
    /// Average price of the MTF holding.
    pub average_price: f64,
    /// Value of the MTF holding.
    pub value: f64,
    /// Initial margin for the MTF holding.
    pub initial_margin: f64,
}

/// `HoldingCompact` returned by GET /portfolio/holdings/compact.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct HoldingCompact {
    /// Exchange the instrument trades on.
    pub exchange: String,
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Numeric instrument token.
    pub instrument_token: u32,
    /// Quantity bought and pending settlement (T1).
    pub t1_quantity: i64,
    /// Quantity held.
    pub quantity: i64,
    /// Exchanges the instrument is listed on.
    #[serde(default)]
    pub exchanges: Vec<String>,
}

/// `HoldingSummary` returned by GET /portfolio/holdings/summary.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct HoldingSummary {
    /// Total profit and loss across holdings.
    pub total_pnl: f64,
    /// Total profit and loss as a percentage.
    pub total_pnl_percent: f64,
    /// Profit and loss for the day.
    pub today_pnl: f64,
    /// Profit and loss for the day as a percentage.
    pub today_pnl_percent: f64,
    /// Total invested amount.
    pub invested_amount: f64,
    /// Current value of all holdings.
    pub current_value: f64,
}

/// Position returned by GET /portfolio/positions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Position {
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Exchange the instrument trades on.
    pub exchange: String,
    /// Numeric instrument token.
    pub instrument_token: u32,
    /// Product type of the position.
    pub product: String,
    /// Net quantity of the position.
    pub quantity: i64,
    /// Quantity carried over from the previous day.
    pub overnight_quantity: i64,
    /// Contract multiplier.
    pub multiplier: f64,
    /// Average buy price.
    pub average_price: f64,
    /// Previous close price.
    pub close_price: f64,
    /// Last traded price.
    pub last_price: f64,
    /// Value of the position.
    pub value: f64,
    /// Profit and loss.
    pub pnl: f64,
    /// Mark-to-market profit and loss.
    pub m2m: f64,
    /// Unrealised profit and loss.
    pub unrealised: f64,
    /// Realised profit and loss.
    pub realised: f64,
    /// Total quantity bought.
    pub buy_quantity: i64,
    /// Average buy price.
    pub buy_price: f64,
    /// Total buy value.
    pub buy_value: f64,
    /// Mark-to-market profit and loss on buys.
    pub buy_m2m: f64,
    /// Total quantity sold.
    pub sell_quantity: i64,
    /// Average sell price.
    pub sell_price: f64,
    /// Total sell value.
    pub sell_value: f64,
    /// Mark-to-market profit and loss on sells.
    pub sell_m2m: f64,
    /// Quantity bought during the day.
    pub day_buy_quantity: i64,
    /// Average buy price for the day.
    pub day_buy_price: f64,
    /// Buy value for the day.
    pub day_buy_value: f64,
    /// Quantity sold during the day.
    pub day_sell_quantity: i64,
    /// Average sell price for the day.
    pub day_sell_price: f64,
    /// Sell value for the day.
    pub day_sell_value: f64,
}

/// Positions returned by GET /portfolio/positions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Positions {
    /// Net positions.
    #[serde(default)]
    pub net: Vec<Position>,
    /// Day positions.
    #[serde(default)]
    pub day: Vec<Position>,
}

/// `AuctionInstrument` returned by GET /portfolio/holdings/auctions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AuctionInstrument {
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Exchange the instrument trades on.
    pub exchange: String,
    /// Numeric instrument token.
    pub instrument_token: u32,
    /// ISIN of the instrument.
    pub isin: String,
    /// Product type of the holding.
    pub product: String,
    /// Price reported for the holding.
    pub price: f64,
    /// Quantity held.
    pub quantity: i64,
    /// Quantity bought and pending settlement (T1).
    pub t1_quantity: i64,
    /// Realised (settled) quantity.
    pub realised_quantity: i64,
    /// Quantity authorised for sale.
    pub authorised_quantity: i64,
    /// Date and time of the sale authorisation, if any.
    #[serde(default, with = "crate::time::optional_datetime")]
    pub authorised_date: Option<NaiveDateTime>,
    /// Quantity at the start of the day.
    pub opening_quantity: i64,
    /// Quantity pledged as collateral.
    pub collateral_quantity: i64,
    /// Type of collateral.
    pub collateral_type: String,
    /// Whether the holding has a discrepancy.
    pub discrepancy: bool,
    /// Average buy price.
    pub average_price: f64,
    /// Last traded price.
    pub last_price: f64,
    /// Previous close price.
    pub close_price: f64,
    /// Profit and loss.
    pub pnl: f64,
    /// Change in last price from the previous close.
    pub day_change: f64,
    /// Day change as a percentage.
    pub day_change_percentage: f64,
    /// Auction number.
    pub auction_number: String,
}

/// `PositionType` sent to the Kite portfolio endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum PositionType {
    /// Wire value `day`.
    Day,
    /// Wire value `overnight`.
    Overnight,
}

impl std::fmt::Display for PositionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Day => "day",
            Self::Overnight => "overnight",
        })
    }
}

/// `HoldingAuthType` sent to the Kite portfolio endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum HoldingAuthType {
    /// Wire value `equity`.
    Equity,
    /// Wire value `mf`.
    Mf,
}

impl std::fmt::Display for HoldingAuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Equity => "equity",
            Self::Mf => "mf",
        })
    }
}

/// `HoldingAuthTransferType` sent to the Kite portfolio endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum HoldingAuthTransferType {
    /// Wire value `pre`.
    Pre,
    /// Wire value `post`.
    Post,
    /// Wire value `off`.
    Off,
    /// Wire value `gift`.
    Gift,
}

impl std::fmt::Display for HoldingAuthTransferType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Pre => "pre",
            Self::Post => "post",
            Self::Off => "off",
            Self::Gift => "gift",
        })
    }
}

/// Parameters for PUT /portfolio/positions.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConvertPositionParams {
    /// Exchange of the position.
    pub exchange: Exchange,
    /// Trading symbol of the instrument.
    pub tradingsymbol: String,
    /// Current product of the position.
    pub old_product: Product,
    /// Product to convert the position to.
    pub new_product: Product,
    /// Position type (day or overnight).
    pub position_type: PositionType,
    /// Transaction type (buy or sell).
    pub transaction_type: TransactionType,
    /// Quantity to convert.
    pub quantity: i64,
}

impl ConvertPositionParams {
    /// Creates parameters for converting a position.
    pub fn new(
        exchange: Exchange,
        tradingsymbol: impl Into<String>,
        old_product: Product,
        new_product: Product,
        position_type: PositionType,
        transaction_type: TransactionType,
        quantity: i64,
    ) -> Self {
        Self {
            exchange,
            tradingsymbol: tradingsymbol.into(),
            old_product,
            new_product,
            position_type,
            transaction_type,
            quantity,
        }
    }
}

/// Instrument and quantity sent to POST /portfolio/holdings/authorise.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HoldingsAuthInstruments {
    /// ISIN of the instrument.
    pub isin: String,
    /// Quantity to authorise.
    pub quantity: f64,
}

impl HoldingsAuthInstruments {
    /// Creates an instrument entry from an ISIN and quantity.
    pub fn new(isin: impl Into<String>, quantity: f64) -> Self {
        Self {
            isin: isin.into(),
            quantity,
        }
    }
}

/// Parameters for POST /portfolio/holdings/authorise.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
#[must_use]
pub struct HoldingAuthParams {
    /// Authorisation type (sent as `type`).
    pub auth_type: Option<HoldingAuthType>,
    /// Transfer type of the authorisation.
    pub transfer_type: Option<HoldingAuthTransferType>,
    /// Execution date of the transfer.
    pub exec_date: Option<NaiveDate>,
    /// Instruments to authorise.
    pub instruments: Vec<HoldingsAuthInstruments>,
}

impl HoldingAuthParams {
    /// Creates empty parameters.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the authorisation type.
    pub fn auth_type(mut self, value: HoldingAuthType) -> Self {
        self.auth_type = Some(value);
        self
    }
    /// Sets the transfer type.
    pub fn transfer_type(mut self, value: HoldingAuthTransferType) -> Self {
        self.transfer_type = Some(value);
        self
    }
    /// Sets the execution date.
    pub fn exec_date(mut self, value: NaiveDate) -> Self {
        self.exec_date = Some(value);
        self
    }
    /// Sets the instruments to authorise.
    pub fn instruments(mut self, value: Vec<HoldingsAuthInstruments>) -> Self {
        self.instruments = value;
        self
    }
}

/// Authorization request and redirect from POST /portfolio/holdings/authorise.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct HoldingsAuthResp {
    /// Authorisation request ID.
    pub request_id: String,
    /// URL to send the user to for authorisation (built locally, not in the response).
    #[serde(skip_deserializing)]
    pub redirect_url: String,
}

impl KiteConnect {
    /// `GET /portfolio/holdings`.
    pub fn holdings(&self) -> Call<Vec<Holding>> {
        Call::new(self.clone(), Request::get("/portfolio/holdings"))
    }
    /// `GET /portfolio/holdings/summary`.
    pub fn holdings_summary(&self) -> Call<HoldingSummary> {
        Call::new(self.clone(), Request::get("/portfolio/holdings/summary"))
    }
    /// `GET /portfolio/holdings/compact`.
    pub fn holdings_compact(&self) -> Call<Vec<HoldingCompact>> {
        Call::new(self.clone(), Request::get("/portfolio/holdings/compact"))
    }
    /// `GET /portfolio/positions`.
    pub fn positions(&self) -> Call<Positions> {
        Call::new(self.clone(), Request::get("/portfolio/positions"))
    }
    /// `GET /portfolio/holdings/auctions`.
    pub fn auction_instruments(&self) -> Call<Vec<AuctionInstrument>> {
        Call::new(self.clone(), Request::get("/portfolio/holdings/auctions"))
    }
    /// `PUT /portfolio/positions`.
    pub fn convert_position(&self, params: ConvertPositionParams) -> Call<bool> {
        let form: Vec<(String, String)> = vec![
            ("exchange".into(), params.exchange.to_string()),
            ("tradingsymbol".into(), params.tradingsymbol),
            ("old_product".into(), params.old_product.to_string()),
            ("new_product".into(), params.new_product.to_string()),
            ("position_type".into(), params.position_type.to_string()),
            (
                "transaction_type".into(),
                params.transaction_type.to_string(),
            ),
            ("quantity".into(), params.quantity.to_string()),
        ];
        Call::with_decoder(
            self.clone(),
            Request::put("/portfolio/positions").form(form),
            crate::call::success_bool,
        )
    }
    /// `POST /portfolio/holdings/authorise`.
    pub fn initiate_holdings_auth(&self, params: HoldingAuthParams) -> Call<HoldingsAuthResp> {
        let mut form: Vec<(&'static str, String)> = Vec::new();
        if let Some(value) = params.auth_type {
            form.push(("type", value.to_string()));
        }
        if let Some(value) = params.transfer_type {
            form.push(("transfer_type", value.to_string()));
        }
        if let Some(value) = params.exec_date {
            form.push(("exec_date", value.to_string()));
        }
        for instrument in params.instruments {
            form.push(("isin", instrument.isin));
            form.push(("quantity", format!("{:.6}", instrument.quantity)));
        }
        Call::with_decoder(
            self.clone(),
            Request::post("/portfolio/holdings/authorise").form(form),
            decode_holdings_auth,
        )
    }
}

fn decode_holdings_auth(
    client: &KiteConnect,
    response: &RawResponse,
    path: &str,
) -> Result<HoldingsAuthResp> {
    let mut auth: HoldingsAuthResp = crate::call::decode(client, response, path)?;
    auth.redirect_url = format!(
        "https://kite.zerodha.com/connect/portfolio/authorise/holdings/{}/{}",
        crate::client::path_segment(client.api_key()),
        crate::client::path_segment(&auth.request_id)
    );
    Ok(auth)
}

/// Decodes an integer that some backends send as a whole float, such as `-0.0`.
fn whole_number<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<i64, D::Error> {
    let number = serde_json::Number::deserialize(deserializer)?;
    if let Some(value) = number.as_i64() {
        return Ok(value);
    }
    match number.as_f64() {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the value is whole and within the exact f64 integer range"
        )]
        Some(value) if value.fract() == 0.0 && value.abs() <= 9_007_199_254_740_992.0 => {
            Ok(value as i64)
        }
        _ => Err(D::Error::custom(format!(
            "expected a whole number, got {number}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{
        Method::{GET, POST, PUT},
        MockServer,
    };
    use serde_json::{Value, json};

    const HOLDINGS: &str = include_str!("../mock_responses/holdings.json");
    const SUMMARY: &str = include_str!("../mock_responses/holdings_summary.json");
    const COMPACT: &str = include_str!("../mock_responses/holdings_compact.json");
    const POSITIONS: &str = include_str!("../mock_responses/positions.json");
    const AUCTIONS: &str = include_str!("../mock_responses/auctions_list.json");
    const AUTH: &str = include_str!("../mock_responses/holdings_auth.json");
    const CONVERT: &str = include_str!("../mock_responses/convert_position.json");

    fn data(fixture: &str) -> Value {
        serde_json::from_str::<Value>(fixture).unwrap()["data"].clone()
    }

    fn client(server: &MockServer) -> KiteConnect {
        KiteConnect::builder("test-api-key")
            .base_url(server.base_url())
            .build()
            .unwrap()
    }

    #[test]
    fn fixtures_decode_literal_values() {
        let holdings: Vec<Holding> = serde_json::from_value(data(HOLDINGS)).unwrap();
        assert_eq!(holdings.len(), 2);
        assert_eq!(holdings[0].tradingsymbol, "AARON");
        assert_eq!(holdings[0].instrument_token, 263_681);
        assert_eq!(holdings[0].last_price.to_bits(), f64::to_bits(352.95));
        assert_eq!(
            holdings[0].authorised_date,
            Some(
                NaiveDate::from_ymd_opt(2025, 1, 17)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
            )
        );
        let mtf = holdings[0].mtf.as_ref().unwrap();
        assert_eq!(mtf.quantity, 1000);
        assert_eq!(mtf.value.to_bits(), f64::to_bits(100_000.0));
        assert_eq!(mtf.average_price.to_bits(), f64::to_bits(100.0));
        let summary: HoldingSummary = serde_json::from_value(data(SUMMARY)).unwrap();
        assert_eq!(
            summary.invested_amount.to_bits(),
            f64::to_bits(41_044.404_05)
        );
        assert_eq!(
            summary.current_value.to_bits(),
            f64::to_bits(47_842.640_000_000_01)
        );
        let compact: Vec<HoldingCompact> = serde_json::from_value(data(COMPACT)).unwrap();
        assert_eq!(compact.len(), 21);
        assert_eq!(compact[0].tradingsymbol, "63MOONS");
        assert_eq!(compact[0].instrument_token, 3_038_209);
        assert_eq!(compact[0].quantity, 1);
        assert_eq!(compact[0].exchanges.len(), 0);
        let positions: Positions = serde_json::from_value(data(POSITIONS)).unwrap();
        assert_eq!(positions.net[0].tradingsymbol, "LEADMINI17DECFUT");
        assert_eq!(positions.net[0].quantity, 1);
        assert_eq!(
            positions.net[0].buy_value.to_bits(),
            f64::to_bits(161_050.0)
        );
        assert_eq!(positions.day[0].instrument_token, 53_505_799);
        let auctions: Vec<AuctionInstrument> = serde_json::from_value(data(AUCTIONS)).unwrap();
        assert_eq!(auctions.len(), 3);
        assert_eq!(auctions[0].tradingsymbol, "ASHOKLEY");
        assert_eq!(auctions[0].auction_number, "20");
        assert_eq!(auctions[0].average_price.to_bits(), f64::to_bits(131.95));
        let auth: HoldingsAuthResp = serde_json::from_value(data(AUTH)).unwrap();
        assert_eq!(auth.request_id, "na8QgCeQm05UHG6NL9sAGRzdfSF64UdB");
    }

    #[test]
    fn required_numbers_fail_and_optional_fields_accept_absence() {
        let mut holding = data(HOLDINGS)[0].clone();
        holding.as_object_mut().unwrap().remove("price");
        assert!(serde_json::from_value::<Holding>(holding).is_err());
        let mut summary = data(SUMMARY);
        summary.as_object_mut().unwrap().remove("current_value");
        assert!(serde_json::from_value::<HoldingSummary>(summary).is_err());
        for empty in [Value::Null, json!("")] {
            let mut holding = data(HOLDINGS)[0].clone();
            holding["authorised_date"] = empty;
            holding.as_object_mut().unwrap().remove("mtf");
            holding["product"] = json!("FUTURE_PRODUCT");
            let holding: Holding = serde_json::from_value(holding).unwrap();
            assert!(holding.authorised_date.is_none());
            assert!(holding.mtf.is_none());
            assert_eq!(holding.product, "FUTURE_PRODUCT");
        }
    }

    #[test]
    fn holding_quantities_accept_whole_floats_and_reject_fractions() {
        let mut holding = data(HOLDINGS)[0].clone();
        holding["used_quantity"] = json!(-0.0);
        holding["quantity"] = json!(2.0);
        holding["mtf"]["used_quantity"] = json!(-0.0);
        let holding: Holding = serde_json::from_value(holding).unwrap();
        assert_eq!(holding.used_quantity, 0);
        assert_eq!(holding.quantity, 2);
        assert_eq!(holding.mtf.unwrap().used_quantity, 0);
        let mut holding = data(HOLDINGS)[0].clone();
        holding["quantity"] = json!(1.5);
        assert!(serde_json::from_value::<Holding>(holding).is_err());
    }

    #[test]
    fn input_enums_match_wire_values() {
        for (value, wire) in [
            (PositionType::Day, "day"),
            (PositionType::Overnight, "overnight"),
        ] {
            assert_eq!(serde_json::to_value(value).unwrap(), json!(wire));
            assert_eq!(value.to_string(), wire);
        }
        for (value, wire) in [
            (HoldingAuthType::Equity, "equity"),
            (HoldingAuthType::Mf, "mf"),
        ] {
            assert_eq!(serde_json::to_value(value).unwrap(), json!(wire));
            assert_eq!(value.to_string(), wire);
        }
        for (value, wire) in [
            (HoldingAuthTransferType::Pre, "pre"),
            (HoldingAuthTransferType::Post, "post"),
            (HoldingAuthTransferType::Off, "off"),
            (HoldingAuthTransferType::Gift, "gift"),
        ] {
            assert_eq!(serde_json::to_value(value).unwrap(), json!(wire));
            assert_eq!(value.to_string(), wire);
        }
        assert!(serde_json::from_value::<PositionType>(json!("invalid")).is_err());
    }

    #[tokio::test]
    async fn holdings_checks_route_and_decodes() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/portfolio/holdings");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(HOLDINGS);
            })
            .await;
        let result = client(&server).holdings().await.unwrap();
        assert_eq!(result[0].tradingsymbol, "AARON");
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn holdings_summary_checks_route_and_decodes() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/portfolio/holdings/summary");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(SUMMARY);
            })
            .await;
        let result = client(&server).holdings_summary().await.unwrap();
        assert_eq!(
            result.invested_amount.to_bits(),
            f64::to_bits(41_044.404_05)
        );
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn holdings_compact_checks_route_and_decodes() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/portfolio/holdings/compact");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(COMPACT);
            })
            .await;
        let result = client(&server).holdings_compact().await.unwrap();
        assert_eq!(result[0].instrument_token, 3_038_209);
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn positions_checks_route_and_decodes() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/portfolio/positions");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(POSITIONS);
            })
            .await;
        let result = client(&server).positions().await.unwrap();
        assert_eq!(result.net[0].quantity, 1);
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn auction_instruments_checks_route_and_decodes() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/portfolio/holdings/auctions");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(AUCTIONS);
            })
            .await;
        let result = client(&server).auction_instruments().await.unwrap();
        assert_eq!(result[0].auction_number, "20");
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn convert_position_checks_form_and_success() {
        let server = MockServer::start_async().await;
        let mock = server.mock_async(|when, then| {
            when.method(PUT).path("/portfolio/positions")
                .body("exchange=NSE&tradingsymbol=INFY&old_product=MIS&new_product=CNC&position_type=day&transaction_type=BUY&quantity=1");
            then.status(200).header("content-type", "application/json").body(CONVERT);
        }).await;
        let params = ConvertPositionParams::new(
            Exchange::Nse,
            "INFY",
            Product::Mis,
            Product::Cnc,
            PositionType::Day,
            TransactionType::Buy,
            1,
        );
        assert!(client(&server).convert_position(params).await.unwrap());
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn holdings_auth_checks_repeated_form_and_computed_redirect() {
        let server = MockServer::start_async().await;
        let mock = server.mock_async(|when, then| {
            when.method(POST).path("/portfolio/holdings/authorise")
                .body("type=equity&transfer_type=pre&exec_date=2025-01-17&isin=INE721Z01010&quantity=1.000000&isin=INE062A01020&quantity=2.500000");
            then.status(200).header("content-type", "application/json").body(AUTH);
        }).await;
        let params = HoldingAuthParams::new()
            .auth_type(HoldingAuthType::Equity)
            .transfer_type(HoldingAuthTransferType::Pre)
            .exec_date(NaiveDate::from_ymd_opt(2025, 1, 17).unwrap())
            .instruments(vec![
                HoldingsAuthInstruments::new("INE721Z01010", 1.0),
                HoldingsAuthInstruments::new("INE062A01020", 2.5),
            ]);
        let auth = client(&server)
            .initiate_holdings_auth(params.clone())
            .await
            .unwrap();
        assert_eq!(auth.request_id, "na8QgCeQm05UHG6NL9sAGRzdfSF64UdB");
        assert_eq!(
            auth.redirect_url,
            "https://kite.zerodha.com/connect/portfolio/authorise/holdings/test-api-key/na8QgCeQm05UHG6NL9sAGRzdfSF64UdB"
        );
        assert_eq!(
            client(&server)
                .initiate_holdings_auth(params)
                .raw()
                .await
                .unwrap(),
            data(AUTH)
        );
        mock.assert_calls_async(2).await;
    }

    #[tokio::test]
    async fn holdings_auth_omits_optional_form_fields() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/portfolio/holdings/authorise")
                    .body("");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(AUTH);
            })
            .await;
        let auth = client(&server)
            .initiate_holdings_auth(HoldingAuthParams::new())
            .await
            .unwrap();
        assert_eq!(auth.request_id, "na8QgCeQm05UHG6NL9sAGRzdfSF64UdB");
        mock.assert_calls_async(1).await;
    }

    #[test]
    fn holdings_auth_redirect_escapes_path_components() {
        let client = KiteConnect::builder("api/key").build().unwrap();
        let response = RawResponse {
            status: 200,
            content_type: None,
            body: br#"{"status":"success","data":{"request_id":"request/id?#"}}"#.to_vec(),
        };
        let auth =
            decode_holdings_auth(&client, &response, "/portfolio/holdings/authorise").unwrap();
        assert_eq!(
            auth.redirect_url,
            "https://kite.zerodha.com/connect/portfolio/authorise/holdings/api%2Fkey/request%2Fid%3F%23"
        );
    }
}
