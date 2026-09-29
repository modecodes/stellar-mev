use stellar_mev_core::ArbitragePath;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedTransaction {
    pub tx_hash: String,
    pub source_account: String,
    pub sequence_number: u64,
    pub fee_stroops: u64,
    pub operations_summary: String,
    pub xdr_envelope_base64: String,
}

pub struct TransactionBuilder {
    pub source_account: String,
    pub base_fee: u64,
}

impl TransactionBuilder {
    pub fn new(source_account: impl Into<String>, base_fee: u64) -> Self {
        Self {
            source_account: source_account.into(),
            base_fee,
        }
    }

    /// Build a Stellar PathPayment / Soroban MEV transaction envelope
    pub fn build_arbitrage_tx(
        &self,
        path: &ArbitragePath,
        sequence: u64,
        priority_fee_stroops: u64,
    ) -> PreparedTransaction {
        let total_fee = self.base_fee + priority_fee_stroops;

        let ops_desc = path
            .hops
            .iter()
            .map(|h| format!("{}->{}", h.source.code(), h.destination.code()))
            .collect::<Vec<_>>()
            .join(" | ");

        let dummy_xdr = format!(
            "AAAAAgAAAAA_{}_{}_{}",
            self.source_account, sequence, path.id
        );

        PreparedTransaction {
            tx_hash: format!("{:064x}", sequence ^ 0xDEADBEEF),
            source_account: self.source_account.clone(),
            sequence_number: sequence,
            fee_stroops: total_fee,
            operations_summary: ops_desc,
            xdr_envelope_base64: dummy_xdr,
        }
    }
}
