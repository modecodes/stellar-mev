use stellar_mev_core::{Asset, VenueType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: Asset,
    pub to: Asset,
    pub venue: VenueType,
    pub rate: f64,    // Units of 'to' received per 1 unit of 'from'
    pub weight: f64,  // -ln(rate) for negative log cycle detection
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

    pub fn add_edge(&mut self, from: Asset, to: Asset, venue: VenueType, rate: f64) {
        if rate <= 0.0 {
            return;
        }
        let _u = self.get_or_add_node(&from);
        let _v = self.get_or_add_node(&to);
        let weight = -rate.ln();

        self.edges.push(Edge {
            from,
            to,
            venue,
            rate,
            weight,
        });
    }
}
