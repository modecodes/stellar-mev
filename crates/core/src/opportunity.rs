use crate::{Asset, StroopAmount};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VenueType {
    SdexOrderbook,
    AmmPool { pool_id: String },
    SorobanDex { contract_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwapHop {
    pub source: Asset,
    pub destination: Asset,
    pub venue: VenueType,
    pub rate: f64,
    pub estimated_fill_amount: StroopAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArbitrageType {
    CyclicalOrderbook,
    OrderbookVsAmm,
    AmmToAmm,
    SorobanTriangular,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArbitragePath {
    pub id: String,
    pub hops: Vec<SwapHop>,
    pub source_asset: Asset,
    pub input_amount: StroopAmount,
    pub expected_output: StroopAmount,
    pub expected_profit: StroopAmount,
    pub profit_margin_pct: f64,
    pub estimated_gas_stroops: StroopAmount,
    pub net_profit: StroopAmount,
    pub strategy_type: ArbitrageType,
    pub discovered_at_ledger: u32,
}

impl ArbitragePath {
    pub fn is_profitable(&self) -> bool {
        self.net_profit.is_positive()
    }
}
