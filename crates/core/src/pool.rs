use crate::{Asset, StroopAmount};
use serde::{Deserialize, Serialize};

/// Stellar AMM Constant Product Liquidity Pool (x * y = k)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub pool_id: String,
    pub asset_a: Asset,
    pub asset_b: Asset,
    pub reserve_a: StroopAmount,
    pub reserve_b: StroopAmount,
    /// Fee in basis points (e.g. 30 = 0.30%)
    pub fee_bp: u32,
}

impl LiquidityPool {
    pub fn new(
        pool_id: String,
        asset_a: Asset,
        asset_b: Asset,
        reserve_a: StroopAmount,
        reserve_b: StroopAmount,
        fee_bp: u32,
    ) -> Self {
        Self {
            pool_id,
            asset_a,
            asset_b,
            reserve_a,
            reserve_b,
            fee_bp,
        }
    }

    /// Calculate output amount for a swap using constant product formula with fee:
    /// (x + delta_x * (1 - fee)) * (y - delta_y) = x * y
    pub fn get_amount_out(&self, amount_in: StroopAmount, asset_in: &Asset) -> StroopAmount {
        if amount_in.is_zero() {
            return StroopAmount::zero();
        }

        let (r_in, r_out) = if asset_in == &self.asset_a {
            (self.reserve_a.as_stroops(), self.reserve_b.as_stroops())
        } else if asset_in == &self.asset_b {
            (self.reserve_b.as_stroops(), self.reserve_a.as_stroops())
        } else {
            return StroopAmount::zero();
        };

        if r_in == 0 || r_out == 0 {
            return StroopAmount::zero();
        }

        let fee_multiplier = 10_000 - self.fee_bp as i128;
        let amount_in_with_fee = amount_in.as_stroops() as i128 * fee_multiplier;
        let numerator = amount_in_with_fee * r_out as i128;
        let denominator = (r_in as i128 * 10_000) + amount_in_with_fee;

        let amount_out_stroops = (numerator / denominator) as i64;
        StroopAmount::from_stroops(amount_out_stroops)
    }

    /// Spot price of asset_in in terms of asset_out
    pub fn get_spot_price(&self, asset_in: &Asset) -> f64 {
        let (r_in, r_out) = if asset_in == &self.asset_a {
            (self.reserve_a.as_units(), self.reserve_b.as_units())
        } else {
            (self.reserve_b.as_units(), self.reserve_a.as_units())
        };

        if r_in == 0.0 {
            0.0
        } else {
            r_out / r_in
        }
    }

    /// Update pool reserves after a swap
    pub fn apply_swap(&mut self, amount_in: StroopAmount, asset_in: &Asset) -> StroopAmount {
        let amount_out = self.get_amount_out(amount_in, asset_in);
        if asset_in == &self.asset_a {
            self.reserve_a += amount_in;
            self.reserve_b -= amount_out;
        } else {
            self.reserve_b += amount_in;
            self.reserve_a -= amount_out;
        }
        amount_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_product_swap() {
        let pool = LiquidityPool::new(
            "test_pool".to_string(),
            Asset::native(),
            Asset::usdc("ISSUER"),
            StroopAmount::from_units(1_000_000.0),
            StroopAmount::from_units(100_000.0),
            30, // 0.3%
        );

        let input = StroopAmount::from_units(1_000.0);
        let output = pool.get_amount_out(input, &Asset::native());

        // Expected output ~ 99.60 USDC (accounting for 0.3% fee and slippage)
        assert!(output.as_units() > 99.0);
        assert!(output.as_units() < 100.0);
    }
}
