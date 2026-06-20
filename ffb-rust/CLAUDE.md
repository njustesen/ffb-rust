# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Is

FFB-Rust is a Rust port of a Fantasy Football (Blood Bowl) game engine, originally written in Java. It supports three rule editions (BB2016, BB2020, BB2025). Correctness is validated via **parity testing**: 100 random game seeds run through both Java and Rust, requiring bit-for-bit identical game state.

## Common Commands

```bash
cargo build                                        # Build all crates
cargo build --release                              # Optimized build
cargo test                                         # Run all unit tests
cargo test -p ffb-engine                           # Test a single crate
cargo test -p ffb-engine -- block                  # Run tests matching "block"
cargo clippy                                       # Lint
cargo fmt --check                                  # Check formatting
cargo run --release -p ffb-parity                  # Run 100-seed parity test
cargo run --release -p ffb-parity -- --network     # Network integration test
```

## Crate Architecture

The workspace has six crates with a strict dependency order:

```
ffb-model → ffb-mechanics → ffb-engine → ffb-client
                                       → ffb-parity
ffb-model → ffb-protocol → ffb-client
```

| Crate | Purpose |
|-------|---------|
| **ffb-model** | Data types: enums, domain structs (Game, Team, Player, FieldModel), RNG, events, agent prompts |
| **ffb-mechanics** | Pure rule calculations — block dice, pass, injury, scatter, modifiers. No game state mutation. |
| **ffb-engine** | Full game state machine. Single `GameEngine` struct with `apply(action)` loop. 35k LOC. |
| **ffb-protocol** | Serializable client/server command structs for WebSocket communication |
| **ffb-client** | Client-side state machine and WebSocket handling |
| **ffb-parity** | Parity test harness: runs both Java and Rust headless, diffs JSONL logs |

## Key Architectural Patterns

**Game Loop:** `GameEngine::new(home, away, seed)` → `run_game(engine, agent_home, agent_away)` emitting `AgentPrompt` → agent returns `Action` → `engine.apply(action)` → repeat until game over.

**Agent Protocol:** The `Agent` trait has one method: `respond(prompt: &AgentPrompt) -> AgentResponse`. `RandomAgent` selects random legal actions and is used in all automated tests and parity runs.

**Legal Actions:** `legal_actions/` computes valid `Action` variants given current game state — always called before prompting an agent.

**Deterministic RNG:** All randomness flows through `ChaChaRng` seeded per game. Same seed = same game. This is the foundation of parity testing.

**Parity Validation:** Both engines log `(seed, turn, action, state_hash)` to JSONL. `comparator.rs` diffs the logs and reports the first divergence. Current status: 100/100 seeds passing.

**Edition Support:** Rules that differ between BB2016/BB2020/BB2025 are gated on the `Edition` enum, typically in mechanics modifier code.

## BB2025 Ruleset Reference

`rules/` contains the full Blood Bowl 2025 ruleset as local markdown files, downloaded from bloodbowlbase.ru. Use these when you need to look up rules, skills, or star player stats without hitting the network.

- `rules/core_rules/` — 11 files: game essentials, rules & regulations, the game of Blood Bowl, drafting, league play, matched play, exhibition play, skills & traits, inducements, teams, FAQ
- `rules/star_players/` — 63 individual files, one per star player (stats, skills, team eligibility)

To refresh: `python scripts/download_rules.py`

## Data

`data/` contains JSON configs loaded at runtime by `ffb-model/src/data/loader.rs`:
- `rosters/` — Team definitions per edition (positions, stats, starting skills)
- `skills/`, `inducements/`, `injuries/`, `prayers/`, `cards/`, `star_players/`

## Testing Approach

Unit tests are colocated with implementations (`#[cfg(test)]` modules). Each Java mechanic or engine step has a corresponding Rust test (~2,392 tests total: 1,190 in ffb-mechanics, 822 in ffb-engine, 349 in ffb-model). When porting Java behavior:
1. Write the Rust test mirroring the Java test
2. Implement in Rust
3. Run parity to confirm full game correctness

`COMPONENTS.md` tracks Java→Rust translation completeness per subsystem — check this before starting new work to understand what's done vs. pending. `SESSION.md` tracks current work state and known issues.

`parity/` contains JSONL log pairs (`seed_N_java.jsonl` / `seed_N_rust.jsonl`) written during parity runs. When debugging a divergence, diff the pair for the failing seed to find the first mismatching `(turn, action, state_hash)` tuple.
