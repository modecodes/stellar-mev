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

    /// Pre-flight simulate candidate arbitrage opportunity against current market state
    pub fn simulate_opportunity(
        &self,
        path: &ArbitragePath,
        state: &MarketState,
    ) -> Result<StroopAmount, String> {
        debug!("Simulating execution for opportunity: {}", path.id);

        let mut current_amount = path.input_amount;

        for hop in &path.hops {
            match &hop.venue {
                VenueType::AmmPool { pool_id } => {
                    let pools = state.get_pools_for_pair(&hop.source, &hop.destination);
                    let pool = pools
                        .into_iter()
                        .find(|p| p.pool_id == *pool_id)
                        .ok_or_else(|| format!("Pool {} not found in market state", pool_id))?;

                    let output_amount = pool.get_amount_out(current_amount, &hop.source);
                    if output_amount.is_zero() {
                        return Err(format!("Zero output on pool hop {}", pool_id));
                    }
                    current_amount = output_amount;
                }
                VenueType::SdexOrderbook => {
                    let ob = state
                        .get_orderbook(&hop.source, &hop.destination)
                        .ok_or_else(|| {
                            format!("Orderbook {:?} -> {:?} not found", hop.source, hop.destination)
                        })?;

                    let (output_amount, _) = ob.simulate_sell_base(current_amount);
                    if output_amount.is_zero() {
                        return Err("Zero output on SDEX orderbook hop".to_string());
                    }
                    current_amount = output_amount;
                }
                VenueType::SorobanDex { contract_id } => {
                    info!("Soroban contract pre-flight simulation for {}", contract_id);
                    let simulated_units = current_amount.as_units() * hop.rate;
                    current_amount = StroopAmount::from_units(simulated_units);
                }
            }
        }

        let simulated_profit = current_amount - path.input_amount;
        let expected_min_profit = StroopAmount::from_units(
            path.expected_profit.as_units() * (1.0 - self.max_slippage_tolerance_pct / 100.0),
        );

        if simulated_profit >= expected_min_profit && simulated_profit.is_positive() {
            info!(
                "Pre-flight simulation SUCCESS for {}: Net Profit = {} Stroops ({:.2} XLM)",
                path.id,
                simulated_profit.as_stroops(),
                simulated_profit.as_units()
            );
            Ok(simulated_profit)
        } else {
            Err(format!(
                "Pre-flight simulation failed: Simulated profit ({}) < Minimum threshold ({})",
                simulated_profit, expected_min_profit
            ))
        }
    }
}
