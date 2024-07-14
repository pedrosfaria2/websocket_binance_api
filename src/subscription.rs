use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Symbol {
    symbol: String,
}

#[derive(Deserialize, Debug)]
struct ExchangeInfo {
    symbols: Vec<Symbol>,
}

pub async fn fetch_symbols() -> Result<Vec<String>, Error> {
    const URL: &str = "https://api.binance.com/api/v3/exchangeInfo";
    let response = reqwest::get(URL).await?.json::<ExchangeInfo>().await?;
    Ok(response
        .symbols
        .into_iter()
        .map(|s| s.symbol.to_lowercase())
        .collect())
}
