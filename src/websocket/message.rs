use futures_util::StreamExt;
use serde_json::Value;
use tokio_tungstenite::tungstenite::protocol::Message;
use crate::storage::aggtrade_storage::{AggTrade, AggTradeStorage};
use std::sync::{Arc, Mutex};
use std::io::{self, stdout};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode};
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui::widgets::{Block, Borders, Table, Row, Cell, Paragraph, Chart, Dataset, Axis};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Style, Color};
use tui::text::{Span, Spans};
use chrono::{DateTime, Utc, NaiveDateTime};

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

    loop {
        while let Some(message) = read.next().await {
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
                            let trades: Vec<_> = storage.get_trades().iter().rev().take(20).collect::<Vec<_>>().iter().rev().map(|trade| {
                                Row::new(vec![
                                    Cell::from(trade.symbol.clone()),
                                    Cell::from(trade.trade_id.to_string()),
                                    Cell::from(format!("{:.2}", trade.price)),
                                    Cell::from(format!("{:.4}", trade.quantity)),
                                    Cell::from(trade.first_trade_id.to_string()),
                                    Cell::from(trade.last_trade_id.to_string()),
                                    Cell::from(trade.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
                                    Cell::from(trade.is_buyer_maker.to_string()),
                                ])
                            }).collect();

                            // Prepare data for the chart
                            let prices: Vec<(f64, f64)> = storage.get_trades().iter().map(|trade| (trade.timestamp.timestamp() as f64, trade.price)).collect();

                            // Draw UI
                            terminal.draw(|f| {
                                let chunks = Layout::default()
                                    .direction(Direction::Vertical)
                                    .margin(1)
                                    .constraints(
                                        [
                                            Constraint::Percentage(50),
                                            Constraint::Percentage(25),
                                            Constraint::Percentage(25),
                                        ]
                                            .as_ref(),
                                    )
                                    .split(f.size());

                                let header = Row::new(vec![
                                    Cell::from("Symbol"),
                                    Cell::from("Trade ID"),
                                    Cell::from("Price"),
                                    Cell::from("Quantity"),
                                    Cell::from("First Trade ID"),
                                    Cell::from("Last Trade ID"),
                                    Cell::from("Timestamp"),
                                    Cell::from("Buyer Maker"),
                                ]).style(Style::default().fg(Color::Yellow).bg(Color::Blue));

                                let table = Table::new(trades)
                                    .header(header)
                                    .block(Block::default().borders(Borders::ALL).title("Trades"))
                                    .widths(&[
                                        Constraint::Length(10),
                                        Constraint::Length(15),
                                        Constraint::Length(10),
                                        Constraint::Length(10),
                                        Constraint::Length(15),
                                        Constraint::Length(15),
                                        Constraint::Length(20),
                                        Constraint::Length(15),
                                    ]);

                                f.render_widget(table, chunks[0]);

                                let stats = Paragraph::new(vec![
                                    Spans::from(vec![Span::raw(format!("Average Price: {:.2}", avg_price))]),
                                    Spans::from(vec![Span::raw(format!("Median Price: {:.2}", median_price))]),
                                    Spans::from(vec![Span::raw(format!("Standard Deviation: {:.2}", std_dev))]),
                                    Spans::from(vec![Span::raw(format!("Total Volume: {:.4}", total_volume))]),
                                    Spans::from(vec![Span::raw(format!("VWAP: {:.2}", volume_weighted_avg_price))]),
                                ])
                                    .block(Block::default().borders(Borders::ALL).title("Statistics"));

                                f.render_widget(stats, chunks[1]);

                                let price_min = prices.iter().map(|&(_, y)| y).fold(f64::INFINITY, f64::min);
                                let price_max = prices.iter().map(|&(_, y)| y).fold(f64::NEG_INFINITY, f64::max);

                                let datasets = vec![
                                    Dataset::default()
                                        .name("Prices")
                                        .marker(tui::symbols::Marker::Dot)
                                        .style(Style::default().fg(Color::Cyan))
                                        .data(&prices),
                                ];

                                let x_labels = if !prices.is_empty() {
                                    let first = prices.first().unwrap().0;
                                    let last = prices.last().unwrap().0;
                                    vec![
                                        Span::styled(NaiveDateTime::from_timestamp(first as i64, 0).format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().add_modifier(tui::style::Modifier::BOLD)),
                                        Span::styled(NaiveDateTime::from_timestamp(last as i64, 0).format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().add_modifier(tui::style::Modifier::BOLD)),
                                    ]
                                } else {
                                    vec![]
                                };

                                let chart = Chart::new(datasets)
                                    .block(Block::default().borders(Borders::ALL).title("Price Chart"))
                                    .x_axis(
                                        Axis::default()
                                            .title(Span::styled("Timestamp", Style::default().fg(Color::Gray)))
                                            .style(Style::default().fg(Color::Gray))
                                            .bounds([prices.first().map(|&(x, _)| x).unwrap_or(0.0), prices.last().map(|&(x, _)| x).unwrap_or(0.0)])
                                            .labels(x_labels)
                                    )
                                    .y_axis(
                                        Axis::default()
                                            .title(Span::styled("Price", Style::default().fg(Color::Gray)))
                                            .style(Style::default().fg(Color::Gray))
                                            .bounds([price_min, price_max])
                                            .labels(vec![
                                                Span::styled(format!("{:.2}", price_min), Style::default().add_modifier(tui::style::Modifier::BOLD)),
                                                Span::styled(format!("{:.2}", price_max), Style::default().add_modifier(tui::style::Modifier::BOLD)),
                                            ]),
                                    );

                                f.render_widget(chart, chunks[2]);
                            }).unwrap();
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("WebSocket connection closed.");
                    break;
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}

fn parse_agg_trade(data: &Value) -> Option<AggTrade> {
    let timestamp = data.get("T")?.as_u64()?;
    let naive = NaiveDateTime::from_timestamp((timestamp / 1000) as i64, ((timestamp % 1000) * 1_000_000) as u32);
    let datetime = DateTime::<Utc>::from_utc(naive, Utc);

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
