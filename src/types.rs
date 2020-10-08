use serde::{de, Serialize, Deserialize, Deserializer};
use chrono::{NaiveDateTime, DateTime, Utc};
use std::collections::HashMap;
#[derive(Serialize, Deserialize, Debug)]
enum OrderVariety {
    #[serde(rename = "regular")]
    REGULAR,
    #[serde(rename = "amo")]
    AMO,
    #[serde(rename = "bo")]
    BO,
    #[serde(rename = "co")]
    CO
}
#[derive(Serialize, Deserialize, Debug)]
enum OrderType {
    MARKET,
    LIMIT,
    SL,
    #[serde(rename = "SL-M")]
    SLM
}
#[derive(Serialize, Deserialize, Debug)]
enum Product {
    CNC,
    NRML,
    MIS
}
#[derive(Serialize, Deserialize, Debug)]
enum OrderValidity {
    DAY,
    IOC
}
#[derive(Serialize, Deserialize, Debug)]
enum OrderStatus {
    COMPLETE,
    REJECTED,
    CANCELLED,
    OPEN
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
    parent_order_id: String,
    exchange_order_id: String,
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
    #[serde(deserialize_with = "naive_date_time_from_str")]    
    order_timestamp: NaiveDateTime,
    #[serde(deserialize_with = "naive_date_time_from_str")]    
    exchange_timestamp: NaiveDateTime,
    status_message: String,
    tag: String
}

fn naive_date_time_from_str<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(de::Error::custom)
}