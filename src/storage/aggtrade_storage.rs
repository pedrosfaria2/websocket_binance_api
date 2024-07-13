use std::collections::VecDeque;
use serde::{Deserialize, Deserializer};
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};

#[derive(Debug, Clone, Deserialize)]
pub struct AggTrade {
    pub symbol: String,
    pub trade_id: u64,
    pub price: f64,
    pub quantity: f64,
    pub first_trade_id: u64,
    pub last_trade_id: u64,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: DateTime<Utc>,
    pub is_buyer_maker: bool,
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = u64::deserialize(deserializer)?;
    let naive = NaiveDateTime::from_timestamp((timestamp / 1000) as i64, ((timestamp % 1000) * 1_000_000) as u32);
    Ok(DateTime::<Utc>::from_utc(naive, Utc))
}

impl AggTrade {
    pub fn from_json(json: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(json.clone()).ok()
    }
}

pub struct AggTradeStorage {
    trades: VecDeque<AggTrade>,
    capacity: usize,
}

impl AggTradeStorage {
    pub fn new(capacity: usize) -> Self {
        Self {
            trades: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add_trade(&mut self, trade: AggTrade) {
        if self.trades.len() == self.capacity {
            self.trades.pop_front();
        }
        self.trades.push_back(trade);
    }

    pub fn get_trades(&self) -> Vec<AggTrade> {
        self.trades.iter().cloned().collect()
    }

    pub fn calculate_average_price(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }
        let total_price: f64 = self.trades.iter().map(|trade| trade.price).sum();
        Some(total_price / self.trades.len() as f64)
    }

    pub fn total_volume(&self) -> f64 {
        self.trades.iter().map(|trade| trade.quantity).sum()
    }

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

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }
        let mean = self.calculate_average_price()?;
        let variance: f64 = self.trades.iter().map(|trade| {
            let diff = trade.price - mean;
            diff * diff
        }).sum::<f64>() / self.trades.len() as f64;
        Some(variance.sqrt())
    }

    pub fn calculate_vwap(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }
        let total_price_volume: f64 = self.trades.iter().map(|trade| trade.price * trade.quantity).sum();
        let total_volume: f64 = self.total_volume();
        Some(total_price_volume / total_volume)
    }
}
