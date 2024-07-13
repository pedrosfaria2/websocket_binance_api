use crate::storage::aggtrade_storage::AggTrade;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Axis, Block, Borders, Cell, Chart, Dataset, Gauge, Paragraph, Row, Table};

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
}

pub fn render_ui<B: Backend>(f: &mut tui::Frame<B>, data: RenderData) {
    // Layout with three vertical chunks
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
                Cell::from(trade.symbol.as_str()),
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
                Cell::from(if trade.is_buyer_maker { "Buy" } else { "Sell" }),
            ])
        })
        .collect();

    // Table widget
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
            Constraint::Length(24),
            Constraint::Length(15),
        ]);

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
                Constraint::Percentage(50),
            ]
                .as_ref(),
        )
        .split(chunks[1]);

    // Statistics paragraph column 1
    let stats_column_1 = Paragraph::new(vec![
        Spans::from(vec![Span::raw(format!(
            "Last Price: {:.2}",
            data.last_price
        ))]),
        Spans::from(vec![Span::raw(format!(
            "Average Price: {:.2}",
            data.avg_price
        ))]),
        Spans::from(vec![Span::raw(format!(
            "Median Price: {:.2}",
            data.median_price
        ))]),
        Spans::from(vec![Span::raw(format!("Max Price: {:.2}", data.max_price))]),
        Spans::from(vec![Span::raw(format!("Min Price: {:.2}", data.min_price))]),
        Spans::from(vec![Span::raw(format!("EMA: {:.2}", data.ema))]),
        Spans::from(vec![Span::raw(format!("SMA: {:.2}", data.sma))]),
    ])
        .block(Block::default().borders(Borders::ALL).title("Statistics"));

    // Render statistics column 1
    f.render_widget(stats_column_1, stats_chunks[0]);

    // Statistics paragraph column 2
    let stats_column_2 = Paragraph::new(vec![
        Spans::from(vec![Span::raw(format!(
            "VWAP: {:.2}",
            data.volume_weighted_avg_price
        ))]),
        Spans::from(vec![Span::raw(format!(
            "Total Volume: {:.4}",
            data.total_volume
        ))]),
        Spans::from(vec![Span::raw(format!(
            "Standard Deviation: {:.2}",
            data.std_dev
        ))]),
        Spans::from(vec![Span::raw(format!("RSI: {:.2}", data.rsi))]),
    ])
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
                .add_modifier(tui::style::Modifier::ITALIC),
        )
        .percent(buyer_maker_true_percent as u16)
        .label(Span::styled(
            format!(
                "Buy: {:.1}%, Sell: {:.1}%",
                buyer_maker_true_percent, buyer_maker_false_percent
            ),
            Style::default().add_modifier(tui::style::Modifier::BOLD),
        ));

    // Render the buyer maker gauge
    f.render_widget(buyer_maker_gauge, stats_chunks[2]);

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
    let datasets = vec![Dataset::default()
        .name("Prices")
        .marker(tui::symbols::Marker::Block)
        .style(Style::default().fg(Color::Cyan))
        .data(data.prices)];

    // Chart widget
    let chart = Chart::new(datasets)
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
                        Style::default().add_modifier(tui::style::Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{}", data.prices.last().map(|&(x, _)| x).unwrap_or(0.0)),
                        Style::default().add_modifier(tui::style::Modifier::BOLD),
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
                        Style::default().add_modifier(tui::style::Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:.2}", price_max),
                        Style::default().add_modifier(tui::style::Modifier::BOLD),
                    ),
                ]),
        );

    // Render chart
    f.render_widget(chart, chunks[2]);
}
