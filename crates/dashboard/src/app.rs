use stellar_mev_core::ArbitragePath;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct MetricLog {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub struct DashboardApp {
    pub current_ledger: u32,
    pub delta_count: u64,
    pub ops_per_sec: f64,
    pub total_opportunities_found: u64,
    pub total_transactions_executed: u64,
    pub cumulative_profit_xlm: f64,
    pub active_assets_count: usize,
    pub active_pools_count: usize,
    pub opportunities: VecDeque<ArbitragePath>,
    pub logs: VecDeque<MetricLog>,
}

impl Default for DashboardApp {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardApp {
    pub fn new() -> Self {
        Self {
            current_ledger: 0,
            delta_count: 0,
            ops_per_sec: 0.0,
            total_opportunities_found: 0,
            total_transactions_executed: 0,
            cumulative_profit_xlm: 0.0,
            active_assets_count: 0,
            active_pools_count: 0,
            opportunities: VecDeque::with_capacity(50),
            logs: VecDeque::with_capacity(100),
        }
    }

    pub fn add_opportunity(&mut self, path: ArbitragePath) {
        self.total_opportunities_found += 1;
        if self.opportunities.len() >= 30 {
            self.opportunities.pop_back();
        }
        self.opportunities.push_front(path);
    }

    pub fn add_log(&mut self, level: &str, msg: &str) {
        let time_str = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        if self.logs.len() >= 100 {
            self.logs.pop_back();
        }
        self.logs.push_front(MetricLog {
            timestamp: time_str,
            level: level.to_string(),
            message: msg.to_string(),
        });
    }
}
