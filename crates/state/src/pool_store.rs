use stellar_mev_core::{Asset, LiquidityPool, StroopAmount};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PoolStore {
    pools: HashMap<String, LiquidityPool>,
    pair_index: HashMap<(Asset, Asset), Vec<String>>,
}

impl PoolStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, pool: LiquidityPool) {
        let pair1 = (pool.asset_a.clone(), pool.asset_b.clone());
        let pair2 = (pool.asset_b.clone(), pool.asset_a.clone());

        self.pair_index.entry(pair1).or_default().push(pool.pool_id.clone());
        self.pair_index.entry(pair2).or_default().push(pool.pool_id.clone());
        self.pools.insert(pool.pool_id.clone(), pool);
    }

    pub fn update_reserves(&mut self, pool_id: &str, reserve_a: StroopAmount, reserve_b: StroopAmount) -> bool {
        if let Some(pool) = self.pools.get_mut(pool_id) {
            pool.reserve_a = reserve_a;
            pool.reserve_b = reserve_b;
            true
        } else {
            false
        }
    }

    pub fn get(&self, pool_id: &str) -> Option<&LiquidityPool> {
        self.pools.get(pool_id)
    }

    pub fn get_pools_for_pair(&self, asset_a: &Asset, asset_b: &Asset) -> Vec<LiquidityPool> {
        let pair = (asset_a.clone(), asset_b.clone());
        if let Some(ids) = self.pair_index.get(&pair) {
            ids.iter().filter_map(|id| self.pools.get(id).cloned()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn all_pools(&self) -> Vec<LiquidityPool> {
        self.pools.values().cloned().collect()
    }
}
