use stellar_mev_core::{LiquidityPool, StroopAmount};
use stellar_mev_state::OrderBook;

/// Computes the optimal input amount for arbitrage against an AMM Constant Product pool (xy=k)
/// given an external target counter-price P (amount of output asset per 1 unit of input asset).
/// Derived by solving d/dx [ (y - (x*y)/(x + dx*(1-f))) * P - dx ] = 0
/// Closed-form formula: dx* = (sqrt(Rx * Ry * fee * P) - Rx) / fee
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

/// Computes optimal trade size walking an SDEX Orderbook to maximize profit against a target rate
pub fn compute_optimal_orderbook_fill(
    orderbook: &OrderBook,
    min_profitable_rate: f64,
    max_capital: StroopAmount,
) -> StroopAmount {
    let mut best_fill = StroopAmount::zero();
    let step = max_capital.as_units() / 20.0;
    if step <= 0.0 {
        return StroopAmount::zero();
    }

    let mut current_input = step;
    while current_input <= max_capital.as_units() {
        let input_stroops = StroopAmount::from_units(current_input);
        let (output_stroops, rate) = orderbook.simulate_sell_base(input_stroops);

        if rate >= min_profitable_rate && output_stroops.as_units() > current_input {
            best_fill = input_stroops;
        } else {
            break;
        }
        current_input += step;
    }

    best_fill
}
