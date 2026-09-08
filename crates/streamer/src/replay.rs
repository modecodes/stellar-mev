use stellar_mev_core::{Asset, LedgerDelta, LedgerHeader, LiquidityPool, StroopAmount};
use tokio::sync::mpsc::Sender;
use tracing::info;

pub struct SyntheticReplayStreamer {
    pub num_ledgers: u32,
    pub delta_delay_ms: u64,
}

impl SyntheticReplayStreamer {
    pub fn new(num_ledgers: u32, delta_delay_ms: u64) -> Self {
        Self {
            num_ledgers,
            delta_delay_ms,
        }
    }

    pub async fn start_stream(&self, tx: Sender<LedgerDelta>) -> Result<(), String> {
        info!("Starting Synthetic Ledger Replay Streamer...");

        let xlm = Asset::native();
        let usdc = Asset::usdc("GA5ZSEJYB37JRC5AVCIA5XYGEVYCH3ZSH4BVW6M5S3LX46B45W55E4KP");

        let pool_xlm_usdc = LiquidityPool::new(
            "pool_xlm_usdc_001".to_string(),
            xlm.clone(),
            usdc.clone(),
            StroopAmount::from_units(1_000_000.0),
            StroopAmount::from_units(120_000.0),
            30,
        );

        let _ = tx.send(LedgerDelta::PoolReserveUpdated {
            pool_id: pool_xlm_usdc.pool_id.clone(),
            reserve_a: pool_xlm_usdc.reserve_a,
            reserve_b: pool_xlm_usdc.reserve_b,
        }).await;

        let start_ledger = 50_000_000u32;
        for seq in start_ledger..(start_ledger + self.num_ledgers) {
            let header = LedgerHeader {
                sequence: seq,
                hash: format!("{:064x}", seq),
                close_time: chrono::Utc::now().timestamp() as u64,
                tx_count: 50,
                operation_count: 150,
            };

            let _ = tx.send(LedgerDelta::LedgerClosed(header)).await;
            if self.delta_delay_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(self.delta_delay_ms)).await;
            }
        }

        info!("Synthetic Ledger Replay finished successfully.");
        Ok(())
    }
}
