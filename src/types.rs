use serde::{de, Serialize, Deserialize, Deserializer};
use chrono::{NaiveDateTime};

#[derive(Serialize, Deserialize, Debug)]
enum OrderVariety {
    #[serde(rename = "regular")]
    Regular,
    #[serde(rename = "amo")]
    AfterMarketOrder,
    #[serde(rename = "bo")]
    BracketOrder,
    #[serde(rename = "co")]
    CoverOrder
}
#[derive(Serialize, Deserialize, Debug)]
enum OrderType {
    #[serde(rename = "MARKET")]
    Market,
    #[serde(rename = "LIMIT")]
    Limit,
    #[serde(rename = "SL")]
    StopLoss,
    #[serde(rename = "SL-M")]
    StopLossMarket
}
#[derive(Serialize, Deserialize, Debug)]
enum Product {
    #[serde(rename = "CNC")]
    CashAndCarry,
    #[serde(rename = "NRML")]
    Normal,
    #[serde(rename = "MIS")]
    MarginIntradaySqareoff
}
#[derive(Serialize, Deserialize, Debug)]
enum OrderValidity {
    #[serde(rename = "DAY")]
    Day,
    #[serde(rename = "IOC")]
    ImmediateOrCancel
}
#[derive(Serialize, Deserialize, Debug)]
enum OrderStatus {
    #[serde(rename = "VALIDATION PENDING")]
    ValidationPending,
    #[serde(rename = "PUT ORDER REQ RECEIVED")]
    PutOrderReqReceived,
    #[serde(rename = "OPEN PENDING")]
    OpenPending,
    #[serde(rename = "MODIFY VALIDATION PENDING")]
    ModifyValidationPending,
    #[serde(rename = "MODIFY PENDING")]
    ModifyPending,
    #[serde(rename = "MODIFIED")]
    Modified,
    #[serde(rename = "COMPLETE")]
    Complete,
    #[serde(rename = "REJECTED")]
    Rejected,
    #[serde(rename = "CANCELLED")]
    Cancelled,
    #[serde(rename = "OPEN")]
    Open
}
#[derive(Serialize, Deserialize, Debug)]
enum TransactionType {
    BUY,
    SELL
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    user_type: String,
    email: String,
    user_name: String,
    user_shortname: String,
    broker: String,
    exchanges: Vec<String>,
    products: Vec<String>,
    order_types: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    order_id: String,
    parent_order_id: Option<String>,
    exchange_order_id: Option<String>,
    placed_by: String,
    variety: String,
    status: OrderStatus,
    tradingsymbol: String,
    exchange: String,
    instrument_token: isize,
    transaction_type: TransactionType,
    order_type: OrderType,
    product: Product,
    validity: OrderValidity,
    price: f64,
    quantity: f64,
    trigger_price: f64,
    average_price: f64,
    pending_quantity: f64,
    filled_quantity: f64,
    disclosed_quantity: f64,
    market_protection: f64,
    #[serde(deserialize_with = "optional_naive_date_time_from_str")]    
    order_timestamp: Option<NaiveDateTime>,
    #[serde(deserialize_with = "optional_naive_date_time_from_str")]    
    exchange_timestamp: Option<NaiveDateTime>,
    status_message: Option<String>,
    tag: Option<String>
}

fn optional_naive_date_time_from_str<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let maybe_naive_date_string: Option<String> = match Deserialize::deserialize(deserializer) {
        Ok(naive_date_string) => Some(naive_date_string),
        Err(_) => None
    };

    match maybe_naive_date_string {
        Some(naive_date_string) => {
            NaiveDateTime::parse_from_str(&naive_date_string, "%Y-%m-%d %H:%M:%S")
            .map(Some)
            .map_err(de::Error::custom)            
        },
        None => Ok(None)
    }
}