use crate::{Call, KiteConnect, Request};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Metadata from GET /user/profile.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UserMeta {
    /// Demat consent status.
    pub demat_consent: String,
}

/// Metadata from GET /user/profile/full.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FullUserMeta {
    /// Proof of address status, if any.
    pub poa: Option<String>,
    /// Silo identifier of the account.
    pub silo: String,
    /// Account blocks on the user, if any.
    #[serde(default)]
    pub account_blocks: Vec<String>,
}

/// Bank account from GET /user/profile/full.
#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Bank {
    /// Bank name.
    pub name: String,
    /// Bank branch.
    pub branch: String,
    /// Bank account number as returned by the API.
    pub account: String,
}

/// User profile from GET /user/profile.
#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UserProfile {
    /// User ID.
    pub user_id: String,
    /// Full name of the user.
    pub user_name: String,
    /// Short name of the user.
    pub user_shortname: String,
    /// Avatar image URL, if any.
    pub avatar_url: Option<String>,
    /// User type.
    pub user_type: String,
    /// Email address of the user.
    pub email: String,
    /// Broker ID.
    pub broker: String,
    /// Extra order metadata, such as iceberg details.
    pub meta: UserMeta,
    /// Products enabled for the user.
    #[serde(default)]
    pub products: Vec<String>,
    /// Order types enabled for the user.
    #[serde(default)]
    pub order_types: Vec<String>,
    /// Exchanges enabled for the user.
    #[serde(default)]
    pub exchanges: Vec<String>,
}

/// Full user profile from GET /user/profile/full.
#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FullUserProfile {
    /// User ID.
    pub user_id: String,
    /// Full name of the user.
    pub user_name: String,
    /// Short name of the user.
    pub user_shortname: String,
    /// Avatar image URL, if any.
    pub avatar_url: Option<String>,
    /// User type.
    pub user_type: String,
    /// Email address of the user.
    pub email: String,
    /// Broker ID.
    pub broker: String,
    /// Phone number as returned by the API.
    pub phone: String,
    /// Two-factor authentication type.
    pub twofa_type: String,
    /// Bank accounts linked to the user.
    #[serde(default)]
    pub bank_accounts: Vec<Bank>,
    /// Depository participant IDs of the user.
    #[serde(default)]
    pub dp_ids: Vec<String>,
    /// Products enabled for the user.
    #[serde(default)]
    pub products: Vec<String>,
    /// Order types enabled for the user.
    #[serde(default)]
    pub order_types: Vec<String>,
    /// Exchanges enabled for the user.
    #[serde(default)]
    pub exchanges: Vec<String>,
    /// PAN as returned by the API.
    pub pan: String,
    /// Tags on the user account.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Time of the last password update.
    #[serde(default, with = "crate::time::optional_datetime")]
    pub password_timestamp: Option<NaiveDateTime>,
    /// Time of the last two-factor authentication update.
    #[serde(default, with = "crate::time::optional_datetime")]
    pub twofa_timestamp: Option<NaiveDateTime>,
    /// Extra order metadata, such as iceberg details.
    pub meta: FullUserMeta,
}

impl fmt::Debug for Bank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bank")
            .field("name", &self.name)
            .field("branch", &self.branch)
            .field("account", &"<redacted>")
            .finish()
    }
}

impl fmt::Debug for UserProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserProfile")
            .field("user_id", &"<redacted>")
            .field("user_name", &"<redacted>")
            .field("user_shortname", &"<redacted>")
            .field("avatar_url", &"<redacted>")
            .field("user_type", &self.user_type)
            .field("email", &"<redacted>")
            .field("broker", &self.broker)
            .field("meta", &self.meta)
            .field("products", &self.products)
            .field("order_types", &self.order_types)
            .field("exchanges", &self.exchanges)
            .finish()
    }
}

impl fmt::Debug for FullUserProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FullUserProfile")
            .field("user_id", &"<redacted>")
            .field("user_name", &"<redacted>")
            .field("user_shortname", &"<redacted>")
            .field("avatar_url", &"<redacted>")
            .field("user_type", &self.user_type)
            .field("email", &"<redacted>")
            .field("broker", &self.broker)
            .field("phone", &"<redacted>")
            .field("twofa_type", &self.twofa_type)
            .field("bank_accounts", &self.bank_accounts)
            .field("dp_ids", &"<redacted>")
            .field("products", &self.products)
            .field("order_types", &self.order_types)
            .field("exchanges", &self.exchanges)
            .field("pan", &"<redacted>")
            .field("tags", &self.tags)
            .field("password_timestamp", &self.password_timestamp)
            .field("twofa_timestamp", &self.twofa_timestamp)
            .field("meta", &self.meta)
            .finish()
    }
}

/// Available funds from GET /user/margins/{segment}.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AvailableMargins {
    /// Adhoc margin added to the account.
    pub adhoc_margin: f64,
    /// Available cash.
    pub cash: f64,
    /// Collateral available.
    pub collateral: f64,
    /// Intraday pay-in amount.
    pub intraday_payin: f64,
    /// Live balance, if returned.
    pub live_balance: Option<f64>,
    /// Opening balance.
    pub opening_balance: f64,
}

/// Used funds from GET /user/margins/{segment}.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UsedMargins {
    /// Total debits.
    pub debits: f64,
    /// Exposure margin used.
    pub exposure: f64,
    /// Realised mark-to-market profit or loss.
    pub m2m_realised: f64,
    /// Unrealised mark-to-market profit or loss.
    pub m2m_unrealised: f64,
    /// Option premium used.
    pub option_premium: f64,
    /// Payout amount.
    pub payout: f64,
    /// SPAN margin used.
    pub span: f64,
    /// Holding sales amount.
    pub holding_sales: f64,
    /// Turnover margin used.
    pub turnover: f64,
    /// Liquid collateral used.
    pub liquid_collateral: f64,
    /// Stock collateral used.
    pub stock_collateral: f64,
    /// Delivery margin used.
    pub delivery: f64,
}

/// Segment margins from GET /user/margins/{segment}.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Margins {
    /// Whether the segment is enabled.
    pub enabled: bool,
    /// Net available margin.
    pub net: f64,
    /// Available funds.
    pub available: AvailableMargins,
    /// Utilised funds.
    pub utilised: UsedMargins,
}

/// All segment margins from GET /user/margins.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AllMargins {
    /// Equity segment margins.
    pub equity: Margins,
    /// Commodity segment margins, if enabled.
    pub commodity: Option<Margins>,
}

impl KiteConnect {
    /// `GET /user/profile`.
    pub fn profile(&self) -> Call<UserProfile> {
        Call::new(self.clone(), Request::get("/user/profile"))
    }
    /// `GET /user/profile/full`.
    pub fn full_profile(&self) -> Call<FullUserProfile> {
        Call::new(self.clone(), Request::get("/user/profile/full"))
    }
    /// `GET /user/margins`.
    pub fn margins(&self) -> Call<AllMargins> {
        Call::new(self.clone(), Request::get("/user/margins"))
    }
    /// `GET /user/margins/{segment}`.
    pub fn segment_margins(&self, segment: &str) -> Call<Margins> {
        Call::new(
            self.clone(),
            Request::get(format!(
                "/user/margins/{}",
                crate::client::path_segment(segment)
            )),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::{Value, json};

    fn data(fixture: &str) -> Value {
        serde_json::from_str::<Value>(fixture).unwrap()["data"].clone()
    }

    #[test]
    fn profile_fixtures_decode_nested_types() {
        let profile: UserProfile =
            serde_json::from_value(data(include_str!("../mock_responses/profile.json"))).unwrap();
        assert_eq!(profile.user_id, "AB1234");
        assert_eq!(profile.user_shortname, "AxAx");
        assert_eq!(profile.avatar_url, None);
        assert_eq!(profile.meta.demat_consent, "physical");
        let full: FullUserProfile =
            serde_json::from_value(data(include_str!("../mock_responses/full_profile.json")))
                .unwrap();
        assert_eq!(full.phone, "*9999");
        assert_eq!(full.bank_accounts[0].name, "HDFC BANK");
        assert_eq!(full.bank_accounts[0].branch, "");
        assert_eq!(full.bank_accounts[1].account, "*9999");
        assert_eq!(full.meta.poa.as_deref(), Some("physical"));
        assert_eq!(full.meta.silo, "x");
        assert_eq!(full.meta.account_blocks.len(), 0);
        assert_eq!(
            full.password_timestamp.unwrap().to_string(),
            "1970-01-01 00:00:00"
        );
        assert_eq!(
            full.twofa_timestamp.unwrap().to_string(),
            "1970-01-01 00:00:00"
        );
    }

    #[test]
    fn profile_debug_redacts_personal_fields() {
        let mut profile: UserProfile =
            serde_json::from_value(data(include_str!("../mock_responses/profile.json"))).unwrap();
        profile.user_id = "SENTINEL-USER-ID".to_string();
        profile.email = "SENTINEL-EMAIL".to_string();
        let mut full: FullUserProfile =
            serde_json::from_value(data(include_str!("../mock_responses/full_profile.json")))
                .unwrap();
        full.user_id = "SENTINEL-FULL-USER-ID".to_string();
        full.phone = "SENTINEL-PHONE".to_string();
        full.pan = "SENTINEL-PAN".to_string();
        full.bank_accounts[0].account = "SENTINEL-ACCOUNT".to_string();
        full.dp_ids[0] = "SENTINEL-DP-ID".to_string();
        let output = format!("{profile:?} {full:?}");
        assert!(
            !output.contains("SENTINEL"),
            "profile diagnostics leaked: {output}"
        );
        assert!(
            output.contains("<redacted>"),
            "profile diagnostics were not redacted"
        );
    }

    #[test]
    fn margin_fixtures_decode_required_totals() {
        let all: AllMargins =
            serde_json::from_value(data(include_str!("../mock_responses/margins.json"))).unwrap();
        assert_eq!(
            all.equity.net.to_bits(),
            f64::to_bits(99_725.050_000_000_02)
        );
        assert_eq!(all.equity.available.cash.to_bits(), f64::to_bits(245_431.6));
        assert_eq!(all.equity.utilised.span.to_bits(), f64::to_bits(101_989.0));
        assert_eq!(
            all.commodity.as_ref().unwrap().net.to_bits(),
            f64::to_bits(100_661.7)
        );
        let equity: Margins =
            serde_json::from_value(data(include_str!("../mock_responses/margins_equity.json")))
                .unwrap();
        assert_eq!(
            equity.available.opening_balance.to_bits(),
            f64::to_bits(245_431.6)
        );
        assert_eq!(equity.utilised.debits.to_bits(), f64::to_bits(145_706.55));
        let commodity: Margins = serde_json::from_value(data(include_str!(
            "../mock_responses/margin_commodity.json"
        )))
        .unwrap();
        assert_eq!(commodity.available.live_balance, Some(96426.33));
        assert_eq!(commodity.utilised.exposure.to_bits(), f64::to_bits(117.35));
        let mut missing = data(include_str!("../mock_responses/margins_equity.json"));
        missing.as_object_mut().unwrap().remove("net");
        assert!(serde_json::from_value::<Margins>(missing).is_err());
        let mut missing = data(include_str!("../mock_responses/margins_equity.json"));
        missing["available"].as_object_mut().unwrap().remove("cash");
        assert!(serde_json::from_value::<Margins>(missing).is_err());
        assert!(serde_json::from_value::<UserProfile>(json!({})).is_err());
    }

    #[tokio::test]
    async fn profile_http_contract() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/user/profile")
                    .query_param_count(".*", ".*", 0)
                    .header_missing("content-type");
                then.body(include_str!("../mock_responses/profile.json"));
            })
            .await;
        let client = KiteConnect::builder("key")
            .base_url(server.base_url())
            .build()
            .unwrap();
        assert_eq!(client.profile().await.unwrap().user_id, "AB1234");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn full_profile_http_contract() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/user/profile/full")
                    .query_param_count(".*", ".*", 0)
                    .header_missing("content-type");
                then.body(include_str!("../mock_responses/full_profile.json"));
            })
            .await;
        let client = KiteConnect::builder("key")
            .base_url(server.base_url())
            .build()
            .unwrap();
        assert_eq!(client.full_profile().await.unwrap().twofa_type, "totp");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn margins_http_contract() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/user/margins")
                    .query_param_count(".*", ".*", 0)
                    .header_missing("content-type");
                then.body(include_str!("../mock_responses/margins.json"));
            })
            .await;
        let client = KiteConnect::builder("key")
            .base_url(server.base_url())
            .build()
            .unwrap();
        assert_eq!(
            client
                .margins()
                .await
                .unwrap()
                .commodity
                .unwrap()
                .net
                .to_bits(),
            f64::to_bits(100_661.7)
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn segment_margins_http_contract() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/user/margins/equity")
                    .query_param_count(".*", ".*", 0)
                    .header_missing("content-type");
                then.body(include_str!("../mock_responses/margins_equity.json"));
            })
            .await;
        let client = KiteConnect::builder("key")
            .base_url(server.base_url())
            .build()
            .unwrap();
        assert_eq!(
            client
                .segment_margins("equity")
                .await
                .unwrap()
                .utilised
                .debits
                .to_bits(),
            f64::to_bits(145_706.55)
        );
        mock.assert_async().await;
    }

    #[test]
    fn segment_margins_escapes_the_path_component() {
        let client = KiteConnect::builder("key").build().unwrap();
        let call = client.segment_margins("a/b?x#y");
        let request = call.request();
        assert_eq!(request.path(), "/user/margins/a%2Fb%3Fx%23y");
        assert_eq!(request.query_pairs(), []);
    }
}
