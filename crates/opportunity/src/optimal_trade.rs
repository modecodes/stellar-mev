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

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_mev_core::{Asset, Offer, Price};

    #[test]
    fn test_compute_optimal_amm_input() {
        let xlm = Asset::native();
        let usdc = Asset::usdc("GBBD47IF6LWK7P7MDEVTESTISSUER1234567890");
        let pool = LiquidityPool::new(
            "pool_xlm_usdc_test".to_string(),
            xlm,
            usdc,
            StroopAmount::from_units(10_000.0), // 10,000 XLM
            StroopAmount::from_units(1_000.0),  // 1,000 USDC -> 1 USDC = 10 XLM
            30,                                 // 0.3% fee (30 bp)
        );

        // Counter market offers 15 XLM per USDC (> 10 AMM price) -> arbitrage exists
        let optimal = compute_optimal_amm_input(&pool, true, 15.0);
        assert!(optimal.is_positive());
        assert!(optimal.as_units() < 10_000.0);

        // Counter market offers 5 XLM per USDC (< 10 AMM price) -> no profitable input
        let unprofitable = compute_optimal_amm_input(&pool, true, 5.0);
        assert_eq!(unprofitable, StroopAmount::zero());
    }

    #[test]
    fn test_compute_optimal_orderbook_fill() {
        let mut ob = OrderBook::new();

        ob.add_ask(Offer {
            offer_id: 1,
            seller: "SELLER1".to_string(),
            selling: Asset::native(),
            buying: Asset::usdc("GBBD47IF6LWK7P7MDEVTESTISSUER1234567890"),
            amount: StroopAmount::from_units(500.0),
            price: Price::from_f64(1.2),
            last_modified_ledger: 100,
        });

        let best_fill = compute_optimal_orderbook_fill(&ob, 1.05, StroopAmount::from_units(100.0));
        assert!(best_fill.is_positive());
    }
}
