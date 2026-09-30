use crate::builder::TransactionBuilder;
use stellar_mev_core::ArbitragePath;
use stellar_mev_simulation::TransactionSimulator;
use stellar_mev_state::MarketState;
use tracing::info;

pub struct ExecutionResult {
    pub tx_hash: String,
    pub status: String,
    pub actual_profit_stroops: i64,
    pub ledger_sequence: u32,
}

pub struct TransactionCoordinator {
    pub builder: TransactionBuilder,
    pub simulator: TransactionSimulator,
    pub sequence_counter: std::sync::atomic::AtomicU64,
}

impl TransactionCoordinator {
    pub fn new(source_account: &str, initial_sequence: u64, max_slippage_pct: f64) -> Self {
        Self {
            builder: TransactionBuilder::new(source_account, 100),
            simulator: TransactionSimulator::new(max_slippage_pct),
            sequence_counter: std::sync::atomic::AtomicU64::new(initial_sequence),
        }
    }
}
