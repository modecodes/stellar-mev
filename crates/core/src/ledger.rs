use crate::{Offer, StroopAmount};
use serde::{Deserialize, Serialize};

/// Stellar Ledger Header
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerHeader {
    pub sequence: u32,
    pub hash: String,
    pub close_time: u64,
    pub tx_count: u32,
    pub operation_count: u32,
}

/// Ledger ingestion event / delta stream item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LedgerDelta {
    LedgerClosed(LedgerHeader),
    OfferCreated(Offer),
    OfferUpdated(Offer),
    OfferRemoved { offer_id: u64 },
    PoolReserveUpdated {
        pool_id: String,
        reserve_a: StroopAmount,
        reserve_b: StroopAmount,
    },
    SorobanContractEvent {
        contract_id: String,
        topic: String,
        value: String,
    },
}
