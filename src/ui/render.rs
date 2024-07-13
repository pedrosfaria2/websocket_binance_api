use tui::backend::Backend;
use tui::widgets::{Block, Borders, Table, Row, Cell, Paragraph, Chart, Dataset, Axis};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Style, Color};
use tui::text::{Span, Spans};
use crate::storage::aggtrade_storage::AggTrade;

pub struct RenderData {
    pub trades: Vec<AggTrade>,
    pub avg_price: f64,
    pub median_price: f64,
    pub std_dev: f64,
    pub total_volume: f64,
    pub volume_weighted_avg_price: f64,
    pub prices: Vec<(f64, f64)>,
}

pub fn render_ui<B: Backend>(f: &mut tui::Frame<B>, data: RenderData) {
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
    ])
        .style(Style::default().fg(Color::Yellow).bg(Color::Blue));

    let trades: Vec<Row> = data.trades.iter().map(|trade| {
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
        Spans::from(vec![Span::raw(format!("Average Price: {:.2}", data.avg_price))]),
        Spans::from(vec![Span::raw(format!("Median Price: {:.2}", data.median_price))]),
        Spans::from(vec![Span::raw(format!("Standard Deviation: {:.2}", data.std_dev))]),
        Spans::from(vec![Span::raw(format!("Total Volume: {:.4}", data.total_volume))]),
        Spans::from(vec![Span::raw(format!("VWAP: {:.2}", data.volume_weighted_avg_price))]),
    ])
        .block(Block::default().borders(Borders::ALL).title("Statistics"));

    f.render_widget(stats, chunks[1]);

    let price_min = data.prices.iter().map(|&(_, y)| y).fold(f64::INFINITY, f64::min);
    let price_max = data.prices.iter().map(|&(_, y)| y).fold(f64::NEG_INFINITY, f64::max);

    let datasets = vec![
        Dataset::default()
            .name("Prices")
            .marker(tui::symbols::Marker::Dot)
            .style(Style::default().fg(Color::Cyan))
            .data(&data.prices),
    ];

    let chart = Chart::new(datasets)
        .block(Block::default().borders(Borders::ALL).title("Price Chart"))
        .x_axis(
            Axis::default()
                .title(Span::styled("Timestamp", Style::default().fg(Color::Gray)))
                .style(Style::default().fg(Color::Gray))
                .bounds([data.prices.first().map(|&(x, _)| x).unwrap_or(0.0), data.prices.last().map(|&(x, _)| x).unwrap_or(0.0)])
                .labels(vec![
                    Span::styled(format!("{}", data.prices.first().map(|&(x, _)| x).unwrap_or(0.0)), Style::default().add_modifier(tui::style::Modifier::BOLD)),
                    Span::styled(format!("{}", data.prices.last().map(|&(x, _)| x).unwrap_or(0.0)), Style::default().add_modifier(tui::style::Modifier::BOLD)),
                ])
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
}
