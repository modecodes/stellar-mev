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

    /// Process incoming real-time ledger deltas and update internal orderbooks/pools
    pub fn apply_delta(&self, delta: LedgerDelta) {
        let mut inner = self.inner.write();
        match delta {
            LedgerDelta::LedgerClosed(header) => {
                inner.current_ledger = Some(header);
            }
            LedgerDelta::OfferCreated(offer) => {
                let pair = (offer.selling.clone(), offer.buying.clone());
                inner.assets.insert(offer.selling.clone());
                inner.assets.insert(offer.buying.clone());
                inner.offer_index.insert(offer.offer_id, pair.clone());

                let ob = inner.orderbooks.entry(pair).or_default();
                ob.add_ask(offer);
            }
            LedgerDelta::OfferUpdated(offer) => {
                let pair = (offer.selling.clone(), offer.buying.clone());
                inner.assets.insert(offer.selling.clone());
                inner.assets.insert(offer.buying.clone());

                let old_pair = inner.offer_index.get(&offer.offer_id).cloned();
                if let Some(prev_pair) = old_pair {
                    if let Some(ob) = inner.orderbooks.get_mut(&prev_pair) {
                        ob.remove_offer(offer.offer_id);
                    }
                }

                inner.offer_index.insert(offer.offer_id, pair.clone());
                let ob = inner.orderbooks.entry(pair).or_default();
                ob.add_ask(offer);
            }
            LedgerDelta::OfferRemoved { offer_id } => {
                if let Some(pair) = inner.offer_index.remove(&offer_id) {
                    if let Some(ob) = inner.orderbooks.get_mut(&pair) {
                        ob.remove_offer(offer_id);
                    }
                }
            }
            LedgerDelta::PoolReserveUpdated {
                pool_id,
                reserve_a,
                reserve_b,
            } => {
                let updated = inner.pools.update_reserves(&pool_id, reserve_a, reserve_b);
                if !updated {
                    let xlm = Asset::native();
                    let usdc = Asset::usdc("GA5ZSEJYB37JRC5AVCIA5XYGEVYCH3ZSH4BVW6M5S3LX46B45W55E4KP");
                    let aqua = Asset::aqua("GBNYLNC5A3G4HKTX4BMBNTXDQCHV4EHSVYDEM26W2W2T5LWWNWEXAQUA");

                    let (asset_a, asset_b) = if pool_id.contains("usdc_aqua") {
                        (usdc, aqua)
                    } else {
                        (xlm, usdc)
                    };

                    let pool = LiquidityPool::new(
                        pool_id,
                        asset_a.clone(),
                        asset_b.clone(),
                        reserve_a,
                        reserve_b,
                        30,
                    );
                    inner.assets.insert(asset_a);
                    inner.assets.insert(asset_b);
                    inner.pools.insert(pool);
                }
            }
            LedgerDelta::SorobanContractEvent { .. } => {}
        }
    }

    pub fn add_pool(&self, pool: LiquidityPool) {
        let mut inner = self.inner.write();
        inner.assets.insert(pool.asset_a.clone());
        inner.assets.insert(pool.asset_b.clone());
        inner.pools.insert(pool);
    }

    pub fn get_orderbook(&self, selling: &Asset, buying: &Asset) -> Option<OrderBook> {
        let inner = self.inner.read();
        inner.orderbooks.get(&(selling.clone(), buying.clone())).cloned()
    }

    pub fn get_pools_for_pair(&self, asset_a: &Asset, asset_b: &Asset) -> Vec<LiquidityPool> {
        let inner = self.inner.read();
        inner.pools.get_pools_for_pair(asset_a, asset_b)
    }

    pub fn all_assets(&self) -> Vec<Asset> {
        let inner = self.inner.read();
        inner.assets.iter().cloned().collect()
    }

    pub fn all_pools(&self) -> Vec<LiquidityPool> {
        let inner = self.inner.read();
        inner.pools.all_pools()
    }

    pub fn current_ledger_sequence(&self) -> u32 {
        let inner = self.inner.read();
        inner.current_ledger.as_ref().map(|h| h.sequence).unwrap_or(0)
    }
}
