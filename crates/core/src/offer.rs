use crate::{Asset, Price, StroopAmount};
use serde::{Deserialize, Serialize};

/// Stellar SDEX Limit Offer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub offer_id: u64,
    pub seller: String,
    pub selling: Asset,
    pub buying: Asset,
    pub amount: StroopAmount,
    pub price: Price,
    pub last_modified_ledger: u32,
}

impl Offer {
    pub fn new(
        offer_id: u64,
        seller: String,
        selling: Asset,
        buying: Asset,
        amount: StroopAmount,
        price: Price,
        last_modified_ledger: u32,
    ) -> Self {
        Self {
            offer_id,
            seller,
            selling,
            buying,
            amount,
            price,
            last_modified_ledger,
        }
    }

    /// Calculate the amount of `buying` asset required to buy all `selling` asset in this offer
    pub fn buying_amount(&self) -> StroopAmount {
        let buying_units = self.amount.as_units() * self.price.as_f64();
        StroopAmount::from_units(buying_units)
    }

    /// Calculate effective price of buying asset per unit of selling asset
    pub fn price_f64(&self) -> f64 {
        self.price.as_f64()
    }
}
