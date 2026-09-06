use stellar_mev_core::LedgerDelta;
use tokio::sync::mpsc::Sender;
use tracing::info;

pub struct HorizonStreamer {
    pub horizon_url: String,
}

impl HorizonStreamer {
    pub fn new(horizon_url: impl Into<String>) -> Self {
        Self {
            horizon_url: horizon_url.into(),
        }
    }

    pub async fn start_stream(&self, _tx: Sender<LedgerDelta>) -> Result<(), String> {
        info!("Connecting to Horizon stream endpoint at: {}", self.horizon_url);
        Ok(())
    }
}
