use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct AggTrade {
    pub symbol: String,
    pub trade_id: u64,
    pub price: f64,
    pub quantity: f64,
    pub first_trade_id: u64,
    pub last_trade_id: u64,
    pub timestamp: DateTime<Utc>,
    pub is_buyer_maker: bool,
}

// Deserialize AggTrade from JSON
impl<'de> Deserialize<'de> for AggTrade {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        let timestamp = v
            .get("T")
            .and_then(|t| t.as_i64())
            .ok_or_else(|| serde::de::Error::custom("Missing timestamp"))?;
        let datetime = Utc
            .timestamp_millis_opt(timestamp)
            .single()
            .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))?;

        Ok(AggTrade {
            symbol: v
                .get("s")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string(),
            trade_id: v.get("a").and_then(|a| a.as_u64()).unwrap_or_default(),
            price: v
                .get("p")
                .and_then(|p| p.as_str())
                .and_then(|p| p.parse().ok())
                .unwrap_or_default(),
            quantity: v
                .get("q")
                .and_then(|q| q.as_str())
                .and_then(|q| q.parse().ok())
                .unwrap_or_default(),
            first_trade_id: v.get("f").and_then(|f| f.as_u64()).unwrap_or_default(),
            last_trade_id: v.get("l").and_then(|l| l.as_u64()).unwrap_or_default(),
            timestamp: datetime,
            is_buyer_maker: v.get("m").and_then(|m| m.as_bool()).unwrap_or_default(),
        })
    }
}

pub struct AggTradeStorage {
    trades: VecDeque<AggTrade>,
    capacity: usize,
}

impl AggTradeStorage {
    // Create a new AggTradeStorage with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            trades: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    // Add a trade to the storage
    pub fn add_trade(&mut self, trade: AggTrade) {
        if self.trades.len() == self.capacity {
            self.trades.pop_front();
        }
        self.trades.push_back(trade);
    }

    // Get all trades
    pub fn get_trades(&self) -> Vec<AggTrade> {
        self.trades.iter().cloned().collect()
    }

    // Calculate the average price of trades
    pub fn calculate_average_price(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }
        let total_price: f64 = self.trades.iter().map(|trade| trade.price).sum();
        Some(total_price / self.trades.len() as f64)
    }

    // Calculate the total volume of trades
    pub fn total_volume(&self) -> f64 {
        self.trades.iter().map(|trade| trade.quantity).sum()
    }

    // Calculate the median price of trades
    pub fn calculate_median_price(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }
        let mut prices: Vec<f64> = self.trades.iter().map(|trade| trade.price).collect();
        prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = prices.len() / 2;
        if prices.len() % 2 == 0 {
            Some((prices[mid - 1] + prices[mid]) / 2.0)
        } else {
            Some(prices[mid])
        }
    }

    // Calculate the standard deviation of trade prices
    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }
        let mean = self.calculate_average_price()?;
        let variance: f64 = self
            .trades
            .iter()
            .map(|trade| {
                let diff = trade.price - mean;
                diff * diff
            })
            .sum::<f64>()
            / self.trades.len() as f64;
        Some(variance.sqrt())
    }

    // Calculate the volume-weighted average price (VWAP)
    pub fn calculate_vwap(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }
        let total_price_volume: f64 = self
            .trades
            .iter()
            .map(|trade| trade.price * trade.quantity)
            .sum();
        let total_volume: f64 = self.total_volume();
        Some(total_price_volume / total_volume)
    }
}
