use crate::storage::aggtrade_storage::AggTradeStorage;
use crate::ui::render::{render_ui, RenderData};
use crate::websocket::handler::input::handle_input;
use crate::websocket::message::parse_agg_trade;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::StreamExt;
use serde_json::Value;
use std::io::stdout;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use tui::backend::CrosstermBackend;
use tui::Terminal;

pub async fn handle_aggtrade_messages<S>(
    mut read: S,
    storage: Arc<RwLock<AggTradeStorage>>,
    mut shutdown_rx: mpsc::Receiver<()>,
) where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    // Initialize terminal
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();

    // Channel for user input shutdown signal
    let (input_tx, mut input_rx) = mpsc::channel(1);
    tokio::spawn(async move {
        handle_input(input_tx).await;
    });

    'main_loop: loop {
        tokio::select! {
            // Handle incoming messages
            Some(message) = read.next() => {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if let Some(agg_trade) = parse_agg_trade(&json) {
                                {
                                    let mut storage = storage.write().unwrap();
                                    storage.add_trade(agg_trade.clone());
                                }

                                // Calculate statistics
                                let (avg_price, median_price, std_dev, total_volume, volume_weighted_avg_price, max_price, min_price, ema, sma, rsi, buyer_maker_true, buyer_maker_false, last_price, trades, prices) = {
                                    let storage = storage.read().unwrap();
                                    (
                                        storage.calculate_average_price().unwrap_or(0.0),
                                        storage.calculate_median_price().unwrap_or(0.0),
                                        storage.calculate_standard_deviation().unwrap_or(0.0),
                                        storage.total_volume(),
                                        storage.calculate_vwap().unwrap_or(0.0),
                                        storage.calculate_max_price().unwrap_or(0.0),
                                        storage.calculate_min_price().unwrap_or(0.0),
                                        storage.calculate_ema(10).unwrap_or(0.0),
                                        storage.calculate_sma(10).unwrap_or(0.0),
                                        storage.calculate_rsi(14).unwrap_or(0.0),
                                        storage.calculate_buyer_maker_count().0,
                                        storage.calculate_buyer_maker_count().1,
                                        agg_trade.price,
                                        storage.get_trades().iter().rev().take(20).cloned().collect::<Vec<_>>(),
                                        storage.get_trades().iter().map(|trade| (trade.timestamp.timestamp_millis() as f64, trade.price)).collect::<Vec<_>>()
                                    )
                                };

                                // Create RenderData
                                let render_data = RenderData {
                                    trades: &trades,
                                    avg_price,
                                    median_price,
                                    std_dev,
                                    total_volume,
                                    volume_weighted_avg_price,
                                    max_price,
                                    min_price,
                                    ema,
                                    sma,
                                    rsi,
                                    last_price,
                                    prices: &prices,
                                    buyer_maker_count: (buyer_maker_true, buyer_maker_false),
                                };

                                // Draw UI
                                terminal.draw(|f| {
                                    render_ui(f, render_data);
                                }).unwrap();
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("WebSocket connection closed.");
                        break 'main_loop;
                    }
                    _ => {}
                }
            },
            // Handle shutdown signal
            _ = shutdown_rx.recv() => {
                println!("Received shutdown signal.");
                break 'main_loop;
            },
            // Handle input shutdown signal
            _ = input_rx.recv() => {
                println!("Received input shutdown signal.");
                break 'main_loop;
            },
        }
    }

    // Restore terminal
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}
