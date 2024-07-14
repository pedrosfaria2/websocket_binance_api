use crate::storage::aggtrade_storage::AggTradeStorage;
use crate::websocket::client::run::run;
use crate::websocket::client::{BINANCE_WS_COMBINED_URL, BINANCE_WS_URL};
use inquire::{MultiSelect, Select};
use tokio::time::Sleep;
use std::io::{self, Write};
use std::sync::{Arc, RwLock};

/// Displays the main menu and processes user selections
pub async fn show_menu(symbols: &Vec<String>) {
    const OPTIONS: [&str; 7] = [
        "Subscribe to aggTrade",
        "Subscribe to trade",
        "Subscribe to kline",
        "Custom Subscribe",
        "List Symbols",
        "List Subscriptions",
        "Exit",
    ];

    let storage = Arc::new(RwLock::new(AggTradeStorage::new(1000)));

    loop {
        clear_screen();
        println!("==============================");
        println!("       Binance WebSocket      ");
        println!("==============================");

        let choice = Select::new("Choose an option:", OPTIONS.to_vec()).prompt();

        match choice {
            Ok(option) => match option {
                "Subscribe to aggTrade" => subscribe("aggTrade", &storage, symbols).await,
                "Subscribe to trade" => subscribe("trade", &storage, symbols).await,
                "Subscribe to kline" => subscribe_with_interval("kline", &storage, symbols).await,
                "Custom Subscribe" => custom_subscribe(&storage, symbols).await,
                "List Symbols" => list_symbols(symbols),
                "List Subscriptions" => list_subscriptions(&storage),
                "Exit" => break,
                _ => unreachable!(),
            },
            Err(_) => break,
        }

        // Ensure all tasks complete before returning to the menu
        wait_for_shutdown().await;
    }
}

/// Subscribes to a single stream type (aggTrade, trade)
async fn subscribe(stream_type: &str, storage: &Arc<RwLock<AggTradeStorage>>, symbols: &Vec<String>) {
    if let Some(symbol) = select_symbol(symbols) {
        let url = format!("{}{}@{}", BINANCE_WS_URL, symbol, stream_type);
        process_subscription(&url, &vec![format!("{}@{}", symbol, stream_type)], storage).await;
    }
}

/// Subscribes to a stream type with interval (kline)
async fn subscribe_with_interval(stream_type: &str, storage: &Arc<RwLock<AggTradeStorage>>, symbols: &Vec<String>) {
    let symbol_selection = select_symbol(symbols);
    if let Some(symbol) = symbol_selection {
        const INTERVALS: [&str; 16] = [
            "1s", "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d",
            "3d", "1w", "1M",
        ];
        if let Some(interval) = select_interval(INTERVALS.to_vec()) {
            let url = format!("{}{}@{}_{}", BINANCE_WS_URL, symbol, stream_type, interval);
            process_subscription(
                &url,
                &vec![format!("{}@{}_{}", symbol, stream_type, interval)],
                storage,
            )
            .await;
        }
    }
}

/// Subscribes to multiple custom streams
async fn custom_subscribe(storage: &Arc<RwLock<AggTradeStorage>>, symbols: &Vec<String>) {
    let selected_symbols = MultiSelect::new("Choose symbols:", symbols.to_vec())
        .prompt()
        .unwrap_or_default();
    let stream_types = vec!["aggTrade", "trade", "kline"];
    let selected_streams = MultiSelect::new("Choose stream types:", stream_types)
        .prompt()
        .unwrap_or_default();

    let mut streams = Vec::new();
    for stream in &selected_streams {
        if *stream == "kline" {
            const INTERVALS: [&str; 16] = [
                "1s", "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h",
                "1d", "3d", "1w", "1M",
            ];
            if let Some(interval) = select_interval(INTERVALS.to_vec()) {
                for symbol in &selected_symbols {
                    streams.push(format!("{}@{}_{}", symbol, stream, interval));
                }
            }
        } else {
            for symbol in &selected_symbols {
                streams.push(format!("{}@{}", symbol, stream));
            }
        }
    }

    let combined_streams = streams.join("/");
    let url = format!("{}{}", BINANCE_WS_COMBINED_URL, combined_streams);
    process_subscription(&url, &streams, storage).await;
}

/// Processes the WebSocket subscription
async fn process_subscription(
    url: &str,
    streams: &Vec<String>,
    storage: &Arc<RwLock<AggTradeStorage>>,
) {
    clear_screen();
    println!("Subscribing to streams...");
    println!("Streams: {:?}", streams);
    println!("Combined URL: {}", url);

    if let Err(e) = run(url, streams, 1, storage).await {
        eprintln!("Error: {}", e);
    }
}

/// Waits for shutdown signal and completes all tasks
fn wait_for_shutdown() -> Sleep {
    // Wait for a small amount of time to ensure all tasks complete
    return tokio::time::sleep(tokio::time::Duration::from_millis(100));
}

/// Selects a symbol from the list of symbols
fn select_symbol(symbols: &Vec<String>) -> Option<String> {
    Select::new("Choose a symbol:", symbols.to_vec()).prompt().ok()
}

/// Selects an interval from the list of intervals
fn select_interval(intervals: Vec<&str>) -> Option<String> {
    Select::new("Choose an interval:", intervals)
        .prompt()
        .ok()
        .map(|s| s.to_string())
}

/// Lists all available symbols
fn list_symbols(symbols: &Vec<String>) {
    clear_screen();
    println!("Available symbols:");
    for symbol in symbols {
        println!("{}", symbol);
    }
    pause();
}

/// Lists current subscriptions (placeholder)
fn list_subscriptions(storage: &Arc<RwLock<AggTradeStorage>>) {
    clear_screen();
    println!("Listing subscriptions...");
    let read = storage.read().unwrap();
    let trades = read.get_trades();
    for trade in trades {
        println!("{:?}", trade);
    }
    pause();
}

/// Clears the terminal screen
fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    io::stdout().flush().unwrap();
}

/// Pauses execution until the user presses Enter
fn pause() {
    println!("\nPress Enter to continue...");
    let _ = io::stdin().read_line(&mut String::new());
}
