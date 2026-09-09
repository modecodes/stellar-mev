use stellar_mev_core::{Asset, LedgerDelta, LedgerHeader, LiquidityPool, Offer, Price, StroopAmount};
use tokio::sync::mpsc::Sender;
use tracing::{info, warn};
use rand::Rng;

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
        let aqua = Asset::aqua("GBNYLNC5A3G4HKTX4BMBNTXDQCHV4EHSVYDEM26W2W2T5LWWNWEXAQUA");
        let yxlm = Asset::yxlm("GARDNV3SOC72EMRBDM2P2W2ACVFLXZB44DHNRFEJPCCIBEZ522G4WXAL");

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
        let mut rng = rand::thread_rng();

        for seq in start_ledger..(start_ledger + self.num_ledgers) {
            let header = LedgerHeader {
                sequence: seq,
                hash: format!("{:064x}", seq),
                close_time: chrono::Utc::now().timestamp() as u64,
                tx_count: rng.gen_range(15..120),
                operation_count: rng.gen_range(50..400),
            };

            if tx.send(LedgerDelta::LedgerClosed(header)).await.is_err() {
                warn!("Stream receiver dropped!");
                break;
            }

            for offer_idx in 0..8 {
                let id = (seq as u64 * 100) + offer_idx;
                let price_val = match offer_idx % 4 {
                    0 => 8.85 + rng.gen_range(-0.2..0.5),
                    1 => 105.0 + rng.gen_range(-2.0..8.0),
                    2 => 0.095 + rng.gen_range(-0.005..0.02),
                    _ => 1.02 + rng.gen_range(-0.01..0.05),
                };
                let amt = rng.gen_range(2000.0..100000.0);

                let (selling, buying) = match offer_idx % 4 {
                    0 => (usdc.clone(), xlm.clone()),
                    1 => (usdc.clone(), aqua.clone()),
                    2 => (aqua.clone(), xlm.clone()),
                    _ => (yxlm.clone(), xlm.clone()),
                };

                let offer = Offer::new(
                    id,
                    "GAARBITRAGEBOTSELLERADDRESS1234567890".to_string(),
                    selling,
                    buying,
                    StroopAmount::from_units(amt),
                    Price::from_f64(price_val),
                    seq,
                );

                let _ = tx.send(LedgerDelta::OfferCreated(offer)).await;
            }

            if self.delta_delay_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(self.delta_delay_ms)).await;
            }
        }

        info!("Synthetic Ledger Replay finished successfully.");
        Ok(())
    }
}
