use serde_derive;

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSession {
    pub api_key: String,
    pub public_token: String,
    pub login_time: String,
    pub user_name: String,
    pub user_shortname: String,
    pub avatar_url: Option<String>,
    pub user_type: String,
    pub email: String,
    pub phone: Option<String>,
    pub broker: String,
    pub products: Vec<String>,
    pub order_types: Vec<String>,
    pub exchanges: Vec<String>,
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionTokens {
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub user_name: String,
    pub user_shortname: String,
    pub avatar_url: Option<String>,
    pub user_type: String,
    pub email: String,
    pub phone: String,
    pub broker: String,
    pub products: Vec<String>,
    pub order_types: Vec<String>,
    pub exchanges: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Margins {
	pub category: Option<String>,
	pub enabled: bool,
	pub net: f64,
	pub available: AvailableMargins,
	pub utilised: Option<UsedMargins>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AvailableMargins {
	pub adhoc_margin: f64,
	pub cash: f64,
	pub collateral: f64,
	pub intraday_payin: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsedMargins {
	pub debits: f64,
	pub exposure: f64,
	pub m2m_realised: f64,
	pub m2m_unrealised: f64,
	pub option_premium: f64,
	pub payout: f64,
	pub span: f64,
	pub holding_sales: f64,
	pub turnover: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllMargins {
    pub equity: Option<Margins>,
    pub commodity: Option<Margins>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Holding {
    pub tradingsymbol: String,
    pub exchange: String,
    pub instrument_token: u32,
    pub isin: String,
    pub product: String,
    pub price: f64,
    pub quantity: i32,
    pub t1_quantity: i32,
    pub realised_quantity: i32,
    pub collateral_quantity: i32,
    pub collateral_type: String,
    pub average_price: f64,
    pub last_price: f64,
    pub close_price: f64,
    pub pnl: f64,
    pub day_change: f64,
    pub day_change_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub tradingsymbol: String,
	pub exchange: String,
	pub instrument_token: u32,
	pub product: String,
	pub quantity: i32,
	pub overnight_quantity: i32,
	pub multiplier: f64,
	pub average_price: f64,
	pub close_price: f64,
	pub last_price: f64,
	pub value: f64,
	pub pnl: f64,
	pub m2m: f64,
	pub unrealised: f64,
	pub realised: f64,
	pub buy_quantity: i32,
	pub buy_price: f64,
	pub buy_value: f64,
	pub buy_m2m: f64,
	pub sell_quantity: i32,
	pub sell_price: f64,
	pub sell_value: f64,
	pub sell_m2m: f64,
	pub day_buy_quantity: i32,
	pub day_buy_price: f64,
	pub day_buy_value: f64,
	pub day_sell_quantity: i32,
	pub day_sell_price: f64,
	pub day_sell_value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Positions {
    pub net: Vec<Position>,
    pub day: Vec<Position>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub account_id: String,
	pub placed_by: String,
	pub order_id: String,
	pub exchange_order_id: String,
	pub parent_order_id: String,
	pub status: String,
	pub status_message: String,
	pub order_timestamp: String,
	pub exchange_update_timestamp: String,
	pub exchange_timestamp: String,
	pub meta: String,
	pub rejected_by: String,
	pub variety: String,
	pub exchange: String,
	pub tradingsymbol: String,
	pub instrument_token: i32,
	pub order_type: String,
	pub transaction_type: String,
	pub validity: String,
	pub product: String,
	pub quantity: f64,
	pub disclosed_quantity: f64,
	pub price: f64,
	pub trigger_price: f64,
	pub average_price: f64,
	pub filled_quantity: f64,
	pub pending_quantity: f64,
	pub cancelled_quantity: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderResponse {
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
	pub average_price: f64,
	pub quantity: f64,
	pub trade_id: String,
	pub product: String,
	pub fill_timestamp: String,
	pub exchange_timestamp: String,
	pub exchange_order_id: String,
	pub order_id: String,
	pub transaction_type: String,
	pub tradingsymbol: String,
	pub exchange: String,
	pub instrument_token: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MFHolding {
    pub folio: String,
	pub fund: String,
	pub tradingsymbol: String,
	pub average_price: f64,
	pub last_price: f64,
	pub pnl: f64,
	pub quantity: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MFOrder {
    pub order_id: String,
	pub exchange_order_id: String,
	pub tradingsymbol: String,
	pub status: String,
	pub status_message: String,
	pub folio: String,
	pub fund: String,
	pub order_timestamp: String,
	pub exchange_timestamp: String,
	pub settlement_id: String,
	pub transaction_type: String,
	pub variety: String,
	pub purchase_type: String,
	pub quantity: f64,
	pub amount: f64,
	pub last_price: f64,
	pub average_price: f64,
	pub placed_by: String,
	pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MFOrderResponse {
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MFSIP {
    pub id: String,
	pub tradingsymbol: String,
	pub fund_name: String,
	pub dividend_type: String,
	pub transaction_type: String,
	pub status: String,
	pub created: String,
	pub frequency: String,
	pub instalment_amount: f64,
	pub instalments: i32,
	pub last_instalment: String,
	pub pending_instalments: i32,
	pub instalment_day: i32,
	pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MFSIPResponse {
    pub order_id: String,
    pub sip_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quote<U> {
	pub quote: HashMap<String, U>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OHLC {
	pub open: f64,
	pub high: f64,
	pub low: f64,
	pub close: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Depth {
	pub buy: Vec<Buy>,
	pub sell: Vec<Sell>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Buy {
	pub price: f64,
	pub quantity: i32,
	pub orders: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sell {
	pub price: f64,
	pub quantity: i32,
	pub orders: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuoteData {
	pub instrument_token: i32,
	pub timestamp: String,
	pub last_price: f64,
	pub last_quantity: i32,
	pub last_trade_time: String,
	pub average_price: f64,
	pub volume: i32,
	pub buy_quantity: i32,
	pub sell_quantity: i32,
	pub ohlc: OHLC,
	pub net_change: f64,
	pub oi: f64,
	pub oi_day_high: f64,
	pub oi_day_low: f64,
	pub depth: Depth,
}
