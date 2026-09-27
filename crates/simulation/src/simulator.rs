use stellar_mev_core::{ArbitragePath, StroopAmount, VenueType};
use stellar_mev_state::MarketState;
use tracing::{debug, info};

pub struct TransactionSimulator {
    pub max_slippage_tolerance_pct: f64,
}

impl TransactionSimulator {
    pub fn new(max_slippage_tolerance_pct: f64) -> Self {
        Self {
            max_slippage_tolerance_pct,
        }
    }
}
