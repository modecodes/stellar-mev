use crate::graph::MarketGraph;
use crate::optimal_trade::compute_optimal_amm_input;
use stellar_mev_core::{
    ArbitragePath, ArbitrageType, StroopAmount, SwapHop, VenueType,
};
use stellar_mev_state::MarketState;
use tracing::debug;

pub struct OpportunityEngine {
    pub min_profit_stroops: StroopAmount,
    pub default_trade_size: StroopAmount,
}

impl OpportunityEngine {
    pub fn new(min_profit_stroops: StroopAmount, default_trade_size: StroopAmount) -> Self {
        Self {
            min_profit_stroops,
            default_trade_size,
        }
    }

    /// Scan entire market state for profitable MEV / Arbitrage opportunities
    pub fn scan_opportunities(&self, state: &MarketState) -> Vec<ArbitragePath> {
        let mut opportunities = Vec::new();
        let assets = state.all_assets();
        let pools = state.all_pools();

        if assets.len() < 2 {
            return opportunities;
        }

        let current_ledger = state.current_ledger_sequence();

        // 1. Build Market Graph
        let mut graph = MarketGraph::new();

        // Add AMM Pool edges in both directions
        for pool in &pools {
            let rate_a_to_b = pool.get_spot_price(&pool.asset_a);
            let rate_b_to_a = pool.get_spot_price(&pool.asset_b);

            if rate_a_to_b > 0.0 {
                graph.add_edge(
                    pool.asset_a.clone(),
                    pool.asset_b.clone(),
                    VenueType::AmmPool {
                        pool_id: pool.pool_id.clone(),
                    },
                    rate_a_to_b,
                );
            }
            if rate_b_to_a > 0.0 {
                graph.add_edge(
                    pool.asset_b.clone(),
                    pool.asset_a.clone(),
                    VenueType::AmmPool {
                        pool_id: pool.pool_id.clone(),
                    },
                    rate_b_to_a,
                );
            }
        }

        // Add SDEX Orderbook edges in both directions
        for selling in &assets {
            for buying in &assets {
                if selling == buying {
                    continue;
                }
                if let Some(ob) = state.get_orderbook(selling, buying) {
                    if let Some(best_ask) = ob.best_ask() {
                        let rate = best_ask.as_f64();
                        if rate > 0.0 {
                            graph.add_edge(
                                selling.clone(),
                                buying.clone(),
                                VenueType::SdexOrderbook,
                                rate,
                            );
                        }
                    }
                }
            }
        }

        // 2. Scan for Cross-DEX Arbitrage (SDEX vs AMM)
        for pool in &pools {
            // Direction 1: Buy asset_b on AMM (spending asset_a), sell asset_b for asset_a on SDEX
            if let Some(ob_b_a) = state.get_orderbook(&pool.asset_b, &pool.asset_a) {
                if let Some(best_ask) = ob_b_a.best_ask() {
                    let sdex_b_to_a_rate = best_ask.as_f64();
                    let amm_a_to_b_rate = pool.get_spot_price(&pool.asset_a);

                    let combined_rate = amm_a_to_b_rate * sdex_b_to_a_rate;
                    if combined_rate > 1.002 {
                        let opt_input = compute_optimal_amm_input(pool, true, sdex_b_to_a_rate);
                        let input_amount = if opt_input.is_zero() || opt_input > self.default_trade_size {
                            self.default_trade_size
                        } else {
                            opt_input
                        };

                        let output_amm = pool.get_amount_out(input_amount, &pool.asset_a);
                        let (output_sdex, _) = ob_b_a.simulate_sell_base(output_amm);

                        if output_sdex > input_amount {
                            let net_profit = output_sdex - input_amount;
                            if net_profit >= self.min_profit_stroops {
                                let path = ArbitragePath {
                                    id: format!("crossdex_{}_{}", pool.pool_id, current_ledger),
                                    hops: vec![
                                        SwapHop {
                                            source: pool.asset_a.clone(),
                                            destination: pool.asset_b.clone(),
                                            venue: VenueType::AmmPool {
                                                pool_id: pool.pool_id.clone(),
                                            },
                                            rate: amm_a_to_b_rate,
                                            estimated_fill_amount: output_amm,
                                        },
                                        SwapHop {
                                            source: pool.asset_b.clone(),
                                            destination: pool.asset_a.clone(),
                                            venue: VenueType::SdexOrderbook,
                                            rate: sdex_b_to_a_rate,
                                            estimated_fill_amount: output_sdex,
                                        },
                                    ],
                                    source_asset: pool.asset_a.clone(),
                                    input_amount,
                                    expected_output: output_sdex,
                                    expected_profit: net_profit,
                                    profit_margin_pct: (net_profit.as_units() / input_amount.as_units()) * 100.0,
                                    estimated_gas_stroops: StroopAmount::from_stroops(200),
                                    net_profit,
                                    strategy_type: ArbitrageType::OrderbookVsAmm,
                                    discovered_at_ledger: current_ledger,
                                };
                                opportunities.push(path);
                            }
                        }
                    }
                }
            }
        }

        // 3. Scan Graph for Triangular & Multi-Hop Arbitrage Cycles
        let cycles = graph.find_arbitrage_cycles();
        for cycle in cycles {
            if cycle.is_empty() {
                continue;
            }

            let start_asset = cycle[0].from.clone();
            let mut current_amount = self.default_trade_size;
            let mut hops = Vec::new();
            let mut rate_product = 1.0;

            for edge in &cycle {
                let out_units = current_amount.as_units() * edge.rate;
                current_amount = StroopAmount::from_units(out_units);
                rate_product *= edge.rate;

                hops.push(SwapHop {
                    source: edge.from.clone(),
                    destination: edge.to.clone(),
                    venue: edge.venue.clone(),
                    rate: edge.rate,
                    estimated_fill_amount: current_amount,
                });
            }

            if rate_product > 1.002 {
                let gross_profit = current_amount - self.default_trade_size;
                if gross_profit >= self.min_profit_stroops {
                    let path = ArbitragePath {
                        id: format!("cycle_{}_{}", start_asset.code(), current_ledger),
                        hops,
                        source_asset: start_asset,
                        input_amount: self.default_trade_size,
                        expected_output: current_amount,
                        expected_profit: gross_profit,
                        profit_margin_pct: (rate_product - 1.0) * 100.0,
                        estimated_gas_stroops: StroopAmount::from_stroops(300),
                        net_profit: gross_profit,
                        strategy_type: ArbitrageType::CyclicalOrderbook,
                        discovered_at_ledger: current_ledger,
                    };
                    opportunities.push(path);
                }
            }
        }

        debug!("Discovered {} valid opportunities", opportunities.len());
        opportunities
    }
}
