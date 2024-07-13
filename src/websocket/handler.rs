use crate::websocket::message::parse_agg_trade;
use crate::websocket::input::handle_input;
use crate::storage::aggtrade_storage::{AggTradeStorage};
use std::sync::{Arc, Mutex};
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::protocol::Message;
use std::io::stdout;
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}};
use serde_json::Value;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use crate::ui::render::{render_ui, RenderData};

pub async fn handle_messages<S>(
    mut read: S,
    storage: Arc<Mutex<AggTradeStorage>>,
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

    // Create a channel to receive the shutdown signal
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);

    // Clone the shutdown_tx for use in the ctrl_c handler
    let shutdown_tx_clone = shutdown_tx.clone();

    // Spawn a task to listen for Ctrl+C and send the shutdown signal
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("failed to listen for event");
        shutdown_tx_clone.send(()).await.expect("failed to send shutdown signal");
    });

    // Create a separate thread to handle user input
    let input_handle = handle_input(shutdown_tx);

    'main_loop: loop {
        tokio::select! {
            Some(message) = read.next() => {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if let Some(agg_trade) = parse_agg_trade(&json) {
                                let mut storage = storage.lock().unwrap();
                                storage.add_trade(agg_trade.clone());

                                // Calculate statistics
                                let avg_price = storage.calculate_average_price().unwrap_or(0.0);
                                let median_price = storage.calculate_median_price().unwrap_or(0.0);
                                let std_dev = storage.calculate_standard_deviation().unwrap_or(0.0);
                                let total_volume = storage.total_volume();
                                let volume_weighted_avg_price = storage.calculate_vwap().unwrap_or(0.0);

                                // Prepare data for display
                                let trades: Vec<_> = storage.get_trades().iter().rev().take(20).cloned().collect();

                                // Prepare data for the chart
                                let prices: Vec<(f64, f64)> = storage.get_trades().iter().map(|trade| (trade.timestamp.timestamp_millis() as f64, trade.price)).collect();

                                // Create RenderData
                                let render_data = RenderData {
                                    trades,
                                    avg_price,
                                    median_price,
                                    std_dev,
                                    total_volume,
                                    volume_weighted_avg_price,
                                    prices,
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
            _ = shutdown_rx.recv() => {
                println!("Received shutdown signal.");
                break 'main_loop;
            },
        }
    }

    // Wait for the input handling thread to finish
    input_handle.await.unwrap();

    // Restore terminal
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}
