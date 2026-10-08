# Stellar MEV & DEX Execution Engine (Rust)

A high-throughput, low-latency MEV (Maximal Extractable Value) and DEX execution engine built in Rust for the **Stellar Network** and **Soroban** environment.

Designed for real-time ledger stream ingestion, in-memory market state tracking, graph-based cyclical arbitrage detection, cross-DEX (SDEX vs. AMM) optimal trade sizing, pre-flight simulation, and atomic transaction execution.

---

## Architecture Overview

```
              Stellar Ledger
                    │
                    ▼
           Stellar Streamer
      (Horizon SSE / RPC / Replay)
                    │
                    ▼
             Market State
         (Lock-Free & RwLock)
                    │
          ┌─────────┼─────────┐
          ▼         ▼         ▼
       Orderbook  Liquidity  Prices
        (SDEX)     (AMMs)   (Oracles)
          │         │         │
          └─────────┼─────────┘
                    ▼
            Opportunity Engine
          ┌─────────┴─────────┐
          ▼                   ▼
      Arbitrage            Routing
    (Bellman-Ford)       (Dijkstra/A*)
          │                   │
          └─────────┬─────────┘
                    ▼
           Transaction Engine
                   &
          Pre-Flight Simulator
                    │
                    ▼
             Stellar Ledger
```

---

## Key Modules & Capabilities

1. **`stellar-mev-core`**: Core primitives (`Asset`, `Price`, `StroopAmount`, `Offer`, `LiquidityPool`, `LedgerDelta`, `ArbitragePath`).
2. **`stellar-mev-streamer`**: High-throughput ingestion stream with real-time SSE, RPC polling, and a high-speed synthetic replay engine for micro-benchmarking.
3. **`stellar-mev-state`**: Thread-safe in-memory state engine tracking SDEX limit orderbooks (BTreeMap price-time priority) and AMM constant product liquidity pools.
4. **`stellar-mev-opportunity`**:
   - **Graph Log Cycle Detection**: Directed multi-graph using Bellman-Ford to detect negative logarithmic weight cycles (\(w = -\ln(\text{rate})\)) for multi-hop arbitrage.
   - **Cross-DEX Arbitrage**: SDEX Orderbook vs AMM Pool price discrepancy finder.
   - **Optimal Input Size Calculator**: Closed-form analytical solver for AMM pools (\(\text{dx}^*\)) and orderbook depth walking algorithms.
5. **`stellar-mev-simulation`**: Pre-flight transaction simulator enforcing strict slippage tolerances and net-profit verification before submission.
6. **`stellar-mev-transaction`**: Transaction builder and coordinator managing sequence numbers, priority fees, envelope generation, and non-blocking submission.
7. **`stellar-mev-dashboard`**: Interactive terminal UI (`ratatui` + `crossterm`) displaying live ledger sequence, delta throughput, active DEX depth, opportunity feed, and realized PnL.

---

## Getting Started

### Prerequisites
- Rust 1.75+ / 1.80+

### Building the Workspace
```bash
cargo build --release
```

### Running Unit & Integration Tests
```bash
cargo test --workspace
```

---

## CLI Commands

### 1. Run Ingestion & Execution Engine (Headless Mode)
```bash
cargo run --release -- --mode replay --ledgers 100 --delay 10 --min-profit 0.1 --trade-size 1000
```

### 2. Launch Interactive TUI Dashboard
```bash
cargo run --release -- --mode replay --ledgers 500 --delay 100 --dashboard
```

### Flags
- `-m, --mode <MODE>`: Streamer mode (`replay`, `live`, `testnet`).
- `-l, --ledgers <N>`: Number of ledgers to process.
- `-d, --delay <MS>`: Delay per ledger in milliseconds.
- `-p, --min-profit <XLM>`: Minimum profit threshold in XLM.
- `-s, --trade-size <XLM>`: Default capital trade size in XLM.
- `--dashboard`: Launch interactive Ratatui TUI dashboard.

---

## License
MIT / Apache-2.0
