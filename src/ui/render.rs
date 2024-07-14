use crate::storage::aggtrade_storage::AggTrade;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Cell, Chart, Dataset, Gauge, Paragraph, Row, Table};

pub struct RenderData<'a> {
    pub trades: &'a [AggTrade],
    pub avg_price: f64,
    pub median_price: f64,
    pub std_dev: f64,
    pub total_volume: f64,
    pub volume_weighted_avg_price: f64,
    pub max_price: f64,
    pub min_price: f64,
    pub ema: f64,
    pub sma: f64,
    pub rsi: f64,
    pub last_price: f64,
    pub prices: &'a [(f64, f64)],
    pub buyer_maker_count: (usize, usize),
    pub message_count: u64,
    pub avg_arrival_interval: f64,
    pub avg_processing_time: f64,
    pub arrival_intervals: &'a [(f64, f64)],
    pub processing_times: &'a [(f64, f64)],
}

macro_rules! create_span {
    ($($arg:tt)*) => {
        Span::raw(format!($($arg)*))
    };
}

pub fn render_ui(f: &mut ratatui::Frame<>, data: &RenderData) {
    // Layout with four vertical chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Table header
    let header = Row::new(vec![
        Cell::from("Symbol"),
        Cell::from("Trade ID"),
        Cell::from("Price"),
        Cell::from("Quantity"),
        Cell::from("First Trade ID"),
        Cell::from("Last Trade ID"),
        Cell::from("Timestamp"),
        Cell::from("Buyer Maker"),
    ])
    .style(Style::default().fg(Color::Yellow).bg(Color::Blue));

    // Table rows
    let trades: Vec<Row> = data
        .trades
        .iter()
        .map(|trade| {
            Row::new(vec![
                Cell::from(trade.symbol.clone()),
                Cell::from(trade.trade_id.to_string()),
                Cell::from(format!("{:.2}", trade.price)),
                Cell::from(format!("{:.4}", trade.quantity)),
                Cell::from(trade.first_trade_id.to_string()),
                Cell::from(trade.last_trade_id.to_string()),
                Cell::from(format!(
                    "{}.{}",
                    trade.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    trade.timestamp.timestamp_subsec_millis()
                )),
                Cell::from(if trade.is_buyer_maker { "Buy" } else { "Sell" }.to_string()),
            ])
        })
        .collect();

    const WIDTHS: [Constraint; 8] = [
        Constraint::Length(10),
        Constraint::Length(15),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(15),
        Constraint::Length(15),
        Constraint::Length(24),
        Constraint::Length(15),
    ];
    // Table widget
    let table = Table::new(trades, WIDTHS)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Trades"));

    // Render table
    f.render_widget(table, chunks[0]);

    // Layout for statistics and additional charts
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    // Statistics paragraph column 1
    let stats_column_1 = Paragraph::new(Line::from(vec![
        create_span!("Last Price: {:.2}", data.last_price),
        create_span!("Average Price: {:.2}", data.avg_price),
        create_span!("Median Price: {:.2}", data.median_price),
        create_span!("Max Price: {:.2}", data.max_price),
        create_span!("Min Price: {:.2}", data.min_price),
        create_span!("EMA: {:.2}", data.ema),
        create_span!("SMA: {:.2}", data.sma),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Statistics"));

    // Render statistics column 1
    f.render_widget(stats_column_1, stats_chunks[0]);

    // Statistics paragraph column 2
    let stats_column_2 = Paragraph::new(Line::from(vec![
        create_span!("VWAP: {:.2}", data.volume_weighted_avg_price),
        create_span!("Total Volume: {:.4}", data.total_volume),
        create_span!("Standard Deviation: {:.2}", data.std_dev),
        create_span!("RSI: {:.2}", data.rsi),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Statistics (contd.)"),
    );

    // Render statistics column 2
    f.render_widget(stats_column_2, stats_chunks[1]);

    // Calculate buyer maker percentages
    let (buyer_maker_true, buyer_maker_false) = data.buyer_maker_count;
    let total_buyer_maker = buyer_maker_true + buyer_maker_false;
    let buyer_maker_true_percent = if total_buyer_maker > 0 {
        (buyer_maker_true as f64 / total_buyer_maker as f64) * 100.0
    } else {
        0.0
    };
    let buyer_maker_false_percent = 100.0 - buyer_maker_true_percent;

    // Gauge for buyer maker percentages
    let buyer_maker_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Buyer Maker Distribution"),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Black)
                .add_modifier(ratatui::style::Modifier::ITALIC),
        )
        .percent(buyer_maker_true_percent as u16)
        .label(Span::styled(
            format!(
                "Buy: {:.1}%, Sell: {:.1}%",
                buyer_maker_true_percent, buyer_maker_false_percent
            ),
            Style::default().add_modifier(ratatui::style::Modifier::BOLD),
        ));

    // Render the buyer maker gauge
    f.render_widget(buyer_maker_gauge, stats_chunks[2]);

    // Performance statistics paragraph
    let performance_stats = Paragraph::new(Line::from(vec![
        create_span!("Messages Processed: {}", data.message_count),
        create_span!("Avg Arrival Interval: {:.2} ms", data.avg_arrival_interval),
        create_span!("Avg Processing Time: {:.2} ms", data.avg_processing_time),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Performance Stats"),
    );

    // Render performance statistics
    f.render_widget(performance_stats, stats_chunks[3]);

    // Calculate price bounds
    let price_min = data
        .prices
        .iter()
        .map(|&(_, y)| y)
        .fold(f64::INFINITY, f64::min);
    let price_max = data
        .prices
        .iter()
        .map(|&(_, y)| y)
        .fold(f64::NEG_INFINITY, f64::max);

    // Dataset for chart
    let price_dataset = vec![Dataset::default()
        .name("Prices")
        .marker(ratatui::symbols::Marker::Block)
        .style(Style::default().fg(Color::Cyan))
        .data(&data.prices)];

    // Chart widget
    let price_chart = Chart::new(price_dataset)
        .block(Block::default().borders(Borders::ALL).title("Price Chart"))
        .x_axis(
            Axis::default()
                .title(Span::styled("Timestamp", Style::default().fg(Color::Gray)))
                .style(Style::default().fg(Color::Gray))
                .bounds([
                    data.prices.first().map(|&(x, _)| x).unwrap_or(0.0),
                    data.prices.last().map(|&(x, _)| x).unwrap_or(0.0),
                ])
                .labels(vec![
                    Span::styled(
                        format!("{}", data.prices.first().map(|&(x, _)| x).unwrap_or(0.0)),
                        Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{}", data.prices.last().map(|&(x, _)| x).unwrap_or(0.0)),
                        Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Price", Style::default().fg(Color::Gray)))
                .style(Style::default().fg(Color::Gray))
                .bounds([price_min, price_max])
                .labels(vec![
                    Span::styled(
                        format!("{:.2}", price_min),
                        Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:.2}", price_max),
                        Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                ]),
        );

    // Render price chart
    f.render_widget(price_chart, chunks[2]);

    // Dataset for performance chart
    let performance_dataset = vec![
        Dataset::default()
            .name("Arrival Interval")
            .marker(ratatui::symbols::Marker::Braille)
            .style(Style::default().fg(Color::Yellow))
            .data(&data.arrival_intervals),
        Dataset::default()
            .name("Processing Time")
            .marker(ratatui::symbols::Marker::Braille)
            .style(Style::default().fg(Color::Green))
            .data(&data.processing_times),
    ];

    // Performance chart widget
    let performance_chart = Chart::new(performance_dataset)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Performance Chart"),
        )
        .x_axis(
            Axis::default()
                .title(Span::styled(
                    "Message Count",
                    Style::default().fg(Color::Gray),
                ))
                .style(Style::default().fg(Color::Gray))
                .bounds([
                    data.arrival_intervals
                        .first()
                        .map(|&(x, _)| x)
                        .unwrap_or(0.0),
                    data.arrival_intervals
                        .last()
                        .map(|&(x, _)| x)
                        .unwrap_or(0.0),
                ])
                .labels(vec![
                    Span::styled(
                        format!(
                            "{}",
                            data.arrival_intervals
                                .first()
                                .map(|&(x, _)| x)
                                .unwrap_or(0.0)
                        ),
                        Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(
                            "{}",
                            data.arrival_intervals
                                .last()
                                .map(|&(x, _)| x)
                                .unwrap_or(0.0)
                        ),
                        Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Time (ms)", Style::default().fg(Color::Gray)))
                .style(Style::default().fg(Color::Gray))
                .bounds([
                    data.arrival_intervals
                        .iter()
                        .map(|&(_, y)| y)
                        .fold(f64::INFINITY, f64::min),
                    data.arrival_intervals
                        .iter()
                        .map(|&(_, y)| y)
                        .fold(f64::NEG_INFINITY, f64::max),
                ])
                .labels(vec![
                    Span::styled(
                        format!(
                            "{:.2}",
                            data.arrival_intervals
                                .iter()
                                .map(|&(_, y)| y)
                                .fold(f64::INFINITY, f64::min)
                        ),
                        Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(
                            "{:.2}",
                            data.arrival_intervals
                                .iter()
                                .map(|&(_, y)| y)
                                .fold(f64::NEG_INFINITY, f64::max)
                        ),
                        Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                ]),
        );

    // Render performance chart
    f.render_widget(performance_chart, chunks[3]);
}
