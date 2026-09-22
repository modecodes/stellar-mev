use stellar_mev_core::{LiquidityPool, StroopAmount};

/// Computes the optimal input amount for arbitrage against an AMM Constant Product pool (xy=k)
/// given an external target counter-price P (amount of output asset per 1 unit of input asset).
/// Formula: dx* = (sqrt(Rx * Ry * fee * P) - Rx) / fee
pub fn compute_optimal_amm_input(
    pool: &LiquidityPool,
    asset_in_a: bool,
    counter_price_out_per_in: f64,
) -> StroopAmount {
    let (r_in, r_out) = if asset_in_a {
        (pool.reserve_a.as_units(), pool.reserve_b.as_units())
    } else {
        (pool.reserve_b.as_units(), pool.reserve_a.as_units())
    };

    let fee_factor = (10_000 - pool.fee_bp) as f64 / 10_000.0;
    if r_in <= 0.0 || r_out <= 0.0 || counter_price_out_per_in <= 0.0 {
        return StroopAmount::zero();
    }

    let numerator = (r_in * r_out * fee_factor * counter_price_out_per_in).sqrt();
    let dx = (numerator - r_in) / fee_factor;

    if dx > 0.0 {
        StroopAmount::from_units(dx)
    } else {
        StroopAmount::zero()
    }
}
