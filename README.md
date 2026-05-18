# ffb-rust

Rust port of the [FFB](https://github.com/christerk/ffb) Blood Bowl game engine by Christer Kaivo-oja, extended with an MCTS AI, RL training infrastructure, and a REST/WebSocket server.

## Goal

Replace the Java headless simulation (~500 ms/game) with a fast Rust engine (~20–50 ms/game) to enable deep MCTS search and reinforcement learning at scale.

## Crates

| Crate | Purpose |
|-------|---------|
| `ffb-core` | Game state, rules, mechanics, pathfinding, RNG |
| `ffb-sim` | Headless game runner, roster definitions, parity logging |
| `ffb-mcts` | Monte Carlo Tree Search with arena allocator |
| `ffb-server` | HTTP + WebSocket server for human vs AI play (Axum) |
| `ffb-py` | PyO3 Python bindings for RL training |

## Build

```bash
cargo build
cargo test
cargo bench -p ffb-sim --bench full_game
```

## License

MIT — see [LICENSE](LICENSE).

Original Java engine copyright © 2024 Christer Kaivo-oja.
Rust port and extensions copyright © 2026 Niels Justesen.
