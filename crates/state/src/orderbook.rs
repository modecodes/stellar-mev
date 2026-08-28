use stellar_mev_core::{Offer, Price};
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
}
