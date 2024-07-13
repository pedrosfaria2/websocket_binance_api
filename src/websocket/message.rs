use chrono::{Utc, TimeZone};
use serde_json::Value;
use crate::storage::aggtrade_storage::AggTrade;

pub fn parse_agg_trade(data: &Value) -> Option<AggTrade> {
    let timestamp = data.get("T")?.as_u64()?;
    let datetime = Utc.timestamp_millis_opt(timestamp as i64).single()?;

    Some(AggTrade {
        symbol: data.get("s")?.as_str()?.to_string(),
        trade_id: data.get("a")?.as_u64()?,
        price: data.get("p")?.as_str()?.parse().ok()?,
        quantity: data.get("q")?.as_str()?.parse().ok()?,
        first_trade_id: data.get("f")?.as_u64()?,
        last_trade_id: data.get("l")?.as_u64()?,
        timestamp: datetime,
        is_buyer_maker: data.get("m")?.as_bool()?,
    })
}
