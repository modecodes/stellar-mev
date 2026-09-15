use stellar_mev_core::{Asset, VenueType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: Asset,
    pub to: Asset,
    pub venue: VenueType,
    pub rate: f64,
    pub weight: f64,
}

#[derive(Debug, Clone, Default)]
pub struct MarketGraph {
    pub assets: Vec<Asset>,
    pub asset_index: HashMap<Asset, usize>,
    pub edges: Vec<Edge>,
}

impl MarketGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_add_node(&mut self, asset: &Asset) -> usize {
        if let Some(&idx) = self.asset_index.get(asset) {
            idx
        } else {
            let idx = self.assets.len();
            self.assets.push(asset.clone());
            self.asset_index.insert(asset.clone(), idx);
            idx
        }
    }
}
