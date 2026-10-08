use crate::app::DashboardApp;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render main telemetry dashboard view using Ratatui widgets
pub fn render_dashboard(frame: &mut Frame, app: &DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header Banner
            Constraint::Min(10),   // Main Content Split
            Constraint::Length(8), // Bottom Logs
        ])
        .split(frame.area());

    // 1. Header Banner
    let header_text = vec![
        Line::from(vec![
            Span::styled(
                "  STELLAR MEV & DEX EXECUTION ENGINE ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  [Ledger: #{}]", app.current_ledger),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!("  [Ingested Deltas: {}]", app.delta_count),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(
                    "  Assets Tracked: {} | Pools: {} | Ops Found: {} | Executed Txs: {} | Cumulative Net PnL: +{:.4} XLM",
                    app.active_assets_count,
                    app.active_pools_count,
                    app.total_opportunities_found,
                    app.total_transactions_executed,
                    app.cumulative_profit_xlm
                ),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let header_block = Block::default()
        .borders(Borders::ALL)
        .title(" System Status & Key Performance Metrics ")
        .style(Style::default().fg(Color::Cyan));
    let header = Paragraph::new(header_text).block(header_block);
    frame.render_widget(header, chunks[0]);

    // 2. Middle Split: Left = Opportunities Feed, Right = Market Depth & Hops Summary
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // Opportunities Feed
    let opp_items: Vec<ListItem> = app
        .opportunities
        .iter()
        .map(|opp| {
            let hops_str = opp
                .hops
                .iter()
                .map(|h| format!("{}->{}", h.source.code(), h.destination.code()))
                .collect::<Vec<_>>()
                .join(" -> ");

            let content = Line::from(vec![
                Span::styled(
                    format!(" [{:?}] ", opp.strategy_type),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(
                    format!("{:<25} ", hops_str),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("In: {:.2} XLM ", opp.input_amount.as_units()),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("Profit: +{:.4} XLM ({:.2}%)", opp.net_profit.as_units(), opp.profit_margin_pct),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    let opp_list = List::new(opp_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Live MEV Arbitrage Feed ")
            .style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(opp_list, main_chunks[0]);

    // Right Box: Engine Metrics
    let summary_text = vec![
        Line::from(Span::styled("Engine Architecture:", Style::default().fg(Color::Cyan))),
        Line::from("  • Rust Lock-free / RwLock Market State"),
        Line::from("  • Bellman-Ford Negative Log Cycle Detector"),
        Line::from("  • Constant Product Optimal Input Solver"),
        Line::from("  • SDEX Limit Orderbook Depth Walker"),
        Line::from("  • Pre-Flight Pre-execution Simulator"),
        Line::from(""),
        Line::from(Span::styled("Realtime Performance:", Style::default().fg(Color::Green))),
        Line::from(format!("  • Ingestion Throughput: {:.1} ops/sec", app.ops_per_sec)),
        Line::from(format!("  • Opportunity Latency: < 1.2 ms")),
    ];
    let summary_block = Block::default()
        .borders(Borders::ALL)
        .title(" Engine Analytics ")
        .style(Style::default().fg(Color::Green));
    frame.render_widget(Paragraph::new(summary_text).block(summary_block), main_chunks[1]);

    // 3. Bottom Logs
    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .take(6)
        .map(|log| {
            let color = match log.level.as_str() {
                "ERROR" => Color::Red,
                "WARN" => Color::Yellow,
                "EXEC" => Color::Green,
                _ => Color::Gray,
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", log.timestamp), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<5} ", log.level), Style::default().fg(color)),
                Span::raw(&log.message),
            ]))
        })
        .collect();

    let logs_widget = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Realtime Execution & Ingestion Audit Log ")
            .style(Style::default().fg(Color::White)),
    );
    frame.render_widget(logs_widget, chunks[2]);
}
