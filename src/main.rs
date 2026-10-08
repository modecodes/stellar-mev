use clap::Parser;
use colored::*;
use stellar_mev_core::StroopAmount;
use stellar_mev_dashboard::{DashboardApp, render_dashboard};
use stellar_mev_opportunity::OpportunityEngine;
use stellar_mev_state::MarketState;
use stellar_mev_streamer::SyntheticReplayStreamer;
use stellar_mev_transaction::TransactionCoordinator;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

/// Stellar MEV Execution Engine CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about = "High-Throughput Stellar MEV & DEX Execution Engine in Rust", long_about = None)]
struct Args {
    /// Operating mode: 'replay', 'live', 'testnet'
    #[arg(short = 'm', long, default_value = "replay")]
    mode: String,

    /// Number of ledgers to process in replay mode
    #[arg(short = 'l', long, default_value_t = 100)]
    ledgers: u32,

    /// Inter-ledger delay in milliseconds for replay mode
    #[arg(short = 'd', long, default_value_t = 150)]
    delay: u64,

    /// Minimum profit threshold in XLM
    #[arg(short = 'p', long, default_value_t = 0.1)]
    min_profit: f64,

    /// Default trade capital size in XLM
    #[arg(short = 's', long, default_value_t = 1000.0)]
    trade_size: f64,

    /// Launch interactive Ratatui TUI dashboard
    #[arg(long, default_value_t = false)]
    dashboard: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.dashboard {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
            .init();
    }

    println!("{}", "=========================================================".cyan().bold());
    println!("{}", "   ⚡ STELLAR MEV & DEX EXECUTION ENGINE (RUST) ⚡   ".yellow().bold());
    println!("{}", "=========================================================".cyan().bold());
    println!(" Mode: {}", args.mode.green().bold());
    println!(" Min Profit Target: {:.4} XLM", args.min_profit);
    println!(" Capital Trade Size: {:.2} XLM", args.trade_size);
    println!(" Processing Stream...\n");

    let state = MarketState::new();
    let opp_engine = OpportunityEngine::new(
        StroopAmount::from_units(args.min_profit),
        StroopAmount::from_units(args.trade_size),
    );
    let coordinator = TransactionCoordinator::new(
        "GABOTEXECUTORADDRESS9876543210STELLARMEV",
        100_000,
        1.5,
    );

    let (tx, mut rx) = mpsc::channel(10_000);

    let mode = args.mode.clone();
    let num_ledgers = args.ledgers;
    let delay_ms = args.delay;

    tokio::spawn(async move {
        if mode == "replay" || mode == "testnet" {
            let streamer = SyntheticReplayStreamer::new(num_ledgers, delay_ms);
            if let Err(e) = streamer.start_stream(tx).await {
                eprintln!("Streamer error: {}", e);
            }
        }
    });

    let mut dashboard_app = DashboardApp::new();

    if args.dashboard {
        // Run Ratatui Dashboard Loop
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let mut delta_count = 0u64;

        while let Some(delta) = rx.recv().await {
            delta_count += 1;
            state.apply_delta(delta);

            dashboard_app.current_ledger = state.current_ledger_sequence();
            dashboard_app.delta_count = delta_count;
            dashboard_app.active_assets_count = state.all_assets().len();
            dashboard_app.active_pools_count = state.all_pools().len();
            dashboard_app.ops_per_sec = 250.0;

            let opportunities = opp_engine.scan_opportunities(&state);
            for opp in opportunities {
                dashboard_app.add_opportunity(opp.clone());
                dashboard_app.add_log(
                    "DISCOV",
                    &format!(
                        "Found {:?} Opportunity: +{:.4} XLM ({:.2}%)",
                        opp.strategy_type,
                        opp.net_profit.as_units(),
                        opp.profit_margin_pct
                    ),
                );

                if let Ok(res) = coordinator.execute_opportunity(&opp, &state).await {
                    dashboard_app.total_transactions_executed += 1;
                    dashboard_app.cumulative_profit_xlm += opp.net_profit.as_units();
                    dashboard_app.add_log(
                        "EXEC",
                        &format!(
                            "Executed Tx {} | Profit: +{:.4} XLM",
                            &res.tx_hash[..16],
                            res.actual_profit_stroops as f64 / 10_000_000.0
                        ),
                    );
                }
            }

            terminal.draw(|f| render_dashboard(f, &dashboard_app))?;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;
    } else {
        // Headless CLI execution mode
        let mut delta_count = 0u64;
        let mut total_profit = 0.0;
        let mut executed_txs = 0;

        while let Some(delta) = rx.recv().await {
            delta_count += 1;
            state.apply_delta(delta);

            let current_seq = state.current_ledger_sequence();
            let opportunities = opp_engine.scan_opportunities(&state);

            for opp in opportunities {
                println!(
                    "{} [Ledger #{}] Discovered {:?} cycle: Hops={} | Est. Profit: +{:.4} XLM ({:.2}%)",
                    "🎯 OPPORTUNITY DETECTED:".green().bold(),
                    current_seq,
                    opp.strategy_type,
                    opp.hops.len(),
                    opp.net_profit.as_units(),
                    opp.profit_margin_pct
                );

                match coordinator.execute_opportunity(&opp, &state).await {
                    Ok(res) => {
                        executed_txs += 1;
                        let profit_xlm = res.actual_profit_stroops as f64 / 10_000_000.0;
                        total_profit += profit_xlm;
                        println!(
                            "{} Tx Hash: {} | Net Realized Profit: +{:.4} XLM | Total Cumulative PnL: +{:.4} XLM\n",
                            "🚀 TX SUBMITTED & EXECUTED:".yellow().bold(),
                            res.tx_hash,
                            profit_xlm,
                            total_profit
                        );
                    }
                    Err(e) => {
                        println!("{} Simulation / Execution failed: {}\n", "❌ REJECTED:".red(), e);
                    }
                }
            }
        }

        println!("\n{}", "=========================================================".cyan().bold());
        println!("{}", " SUMMARY AUDIT REPORT ".yellow().bold());
        println!(" Total Deltas Ingested: {}", delta_count);
        println!(" Total MEV Executions: {}", executed_txs);
        println!(" Total Realized PnL: +{:.4} XLM", total_profit);
        println!("{}", "=========================================================".cyan().bold());
    }

    Ok(())
}
