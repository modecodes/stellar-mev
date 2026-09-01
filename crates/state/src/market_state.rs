use crate::orderbook::OrderBook;
use crate::pool_store::PoolStore;
use parking_lot::RwLock;
use stellar_mev_core::{Asset, LedgerDelta, LedgerHeader, LiquidityPool};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct MarketStateInner {
    pub current_ledger: Option<LedgerHeader>,
    pub orderbooks: HashMap<(Asset, Asset), OrderBook>,
    pub pools: PoolStore,
    pub assets: HashSet<Asset>,
    pub offer_index: HashMap<u64, (Asset, Asset)>, // maps offer_id to pair
}

#[derive(Clone, Default)]
pub struct MarketState {
    inner: Arc<RwLock<MarketStateInner>>,
}

impl MarketState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MarketStateInner::default())),
        }
    }
}
