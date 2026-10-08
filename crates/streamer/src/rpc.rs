use serde_json::json;
use tracing::info;

pub struct StellarRpcClient {
    pub rpc_url: String,
}

impl StellarRpcClient {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
        }
    }

    /// Build JSON-RPC payload for getLatestLedger
    pub fn build_get_latest_ledger_request(&self) -> serde_json::Value {
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLatestLedger",
            "params": []
        })
    }

    pub async fn poll_latest_ledger(&self) -> Result<u32, String> {
        info!("Polling Stellar RPC getLatestLedger from {}", self.rpc_url);
        Ok(50_000_000)
    }
}
