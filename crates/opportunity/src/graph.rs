use stellar_mev_core::{Asset, VenueType};
use std::collections::{HashMap, HashSet};

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

    /// Bellman-Ford negative log cycle detector for triangular & multi-hop arbitrage
    pub fn find_arbitrage_cycles(&self) -> Vec<Vec<Edge>> {
        let n = self.assets.len();
        if n < 2 || self.edges.is_empty() {
            return Vec::new();
        }

        let mut dist = vec![0.0f64; n];
        let mut parent_edge: Vec<Option<Edge>> = vec![None; n];

        // Relax edges up to N-1 times
        for _ in 0..(n - 1) {
            for edge in &self.edges {
                let u = self.asset_index[&edge.from];
                let v = self.asset_index[&edge.to];
                if dist[u] + edge.weight < dist[v] {
                    dist[v] = dist[u] + edge.weight;
                    parent_edge[v] = Some(edge.clone());
                }
            }
        }

        // Check for negative cycles on N-th iteration
        let mut cycles = Vec::new();
        let mut visited_cycle_nodes = HashSet::new();

        for edge in &self.edges {
            let u = self.asset_index[&edge.from];
            let v = self.asset_index[&edge.to];

            if dist[u] + edge.weight < dist[v] {
                if visited_cycle_nodes.contains(&v) {
                    continue;
                }

                let mut curr = v;
                let mut cycle_edges = Vec::new();
                let mut visited = HashSet::new();

                while !visited.contains(&curr) {
                    visited.insert(curr);
                    if let Some(ref e) = parent_edge[curr] {
                        curr = self.asset_index[&e.from];
                    } else {
                        break;
                    }
                }

                let cycle_start = curr;
                let mut step = cycle_start;
                loop {
                    if let Some(ref e) = parent_edge[step] {
                        cycle_edges.push(e.clone());
                        step = self.asset_index[&e.from];
                        visited_cycle_nodes.insert(step);
                        if step == cycle_start {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                cycle_edges.reverse();
                if cycle_edges.len() >= 2 {
                    cycles.push(cycle_edges);
                }
            }
        }

        cycles
    }
}
