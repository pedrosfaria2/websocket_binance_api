use crate::storage::aggtrade_storage::AggTrade;
use chrono::{TimeZone, Utc};
use serde_json::Value;

pub fn parse_agg_trade(data: &Value) -> Option<AggTrade> {
    // Extract timestamp and convert to DateTime
    let timestamp = data.get("T")?.as_u64()?;
    let datetime = Utc.timestamp_millis_opt(timestamp as i64).single()?;

    // Create and return AggTrade struct
    Some(AggTrade {
        symbol: data.get("s")?.as_str()?.to_string(),  // Symbol
        trade_id: data.get("a")?.as_u64()?,            // Trade ID
        price: data.get("p")?.as_str()?.parse().ok()?, // Price
        quantity: data.get("q")?.as_str()?.parse().ok()?, // Quantity
        first_trade_id: data.get("f")?.as_u64()?,      // First trade ID
        last_trade_id: data.get("l")?.as_u64()?,       // Last trade ID
        timestamp: datetime,                           // Timestamp
        is_buyer_maker: data.get("m")?.as_bool()?,     // Buyer maker flag
    })
}
