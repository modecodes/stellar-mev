use stellar_mev_core::{Offer, Price, StroopAmount};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct OrderBookSide {
    /// Price -> list of offers at this price level (FIFO matching order)
    pub levels: BTreeMap<Price, Vec<Offer>>,
}

impl OrderBookSide {
    pub fn insert(&mut self, offer: Offer) {
        self.levels.entry(offer.price).or_default().push(offer);
    }

    pub fn remove(&mut self, offer_id: u64) -> bool {
        let mut empty_price = None;
        let mut removed = false;

        for (price, level) in self.levels.iter_mut() {
            if let Some(pos) = level.iter().position(|o| o.offer_id == offer_id) {
                level.remove(pos);
                removed = true;
                if level.is_empty() {
                    empty_price = Some(*price);
                }
                break;
            }
        }

        if let Some(price) = empty_price {
            self.levels.remove(&price);
        }

        removed
    }
}

/// Orderbook for a specific asset pair (Selling -> Buying)
#[derive(Debug, Clone, Default)]
pub struct OrderBook {
    pub asks: OrderBookSide, // Offers selling base for quote
    pub bids: OrderBookSide, // Offers buying base for quote
}

impl OrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_ask(&mut self, offer: Offer) {
        self.asks.insert(offer);
    }

    pub fn add_bid(&mut self, offer: Offer) {
        self.bids.insert(offer);
    }

    pub fn remove_offer(&mut self, offer_id: u64) -> bool {
        self.asks.remove(offer_id) || self.bids.remove(offer_id)
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.levels.keys().next().copied()
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.levels.keys().next_back().copied()
    }

    /// Walk the orderbook asks to calculate how much quote asset is obtained by spending `amount_in` of base asset
    pub fn simulate_sell_base(&self, amount_in: StroopAmount) -> (StroopAmount, f64) {
        let mut remaining = amount_in.as_units();
        let mut total_out = 0.0;

        for (price, level) in self.asks.levels.iter() {
            for offer in level {
                let offer_capacity = offer.amount.as_units();
                let fill_base = remaining.min(offer_capacity);
                let fill_quote = fill_base * price.as_f64();

                total_out += fill_quote;
                remaining -= fill_base;

                if remaining <= 0.0000001 {
                    break;
                }
            }
            if remaining <= 0.0000001 {
                break;
            }
        }

        let filled_amount = amount_in.as_units() - remaining;
        let effective_rate = if filled_amount > 0.0 { total_out / filled_amount } else { 0.0 };
        (StroopAmount::from_units(total_out), effective_rate)
    }
}
