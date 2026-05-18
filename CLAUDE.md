# CLAUDE.md — ffb-rust

Rust port of the FFB Blood Bowl 2025 game engine. Goal: replace the Java headless sim (~500 ms/game) with a fast Rust engine (~20–50 ms/game) for RL training and deep MCTS.

Java source lives at `/Users/noju/niels/ffb/`.

## Build & Test

```bash
cargo build                                  # build all crates
cargo check                                  # fast type-check
cargo test                                   # all tests
cargo test -p ffb-core                       # single crate tests
cargo bench -p ffb-sim --bench full_game     # benchmark
cargo clippy                                 # lint
python scripts/compare_parity.py            # cross-engine parity check
```

## Crate Map

| Crate | Purpose | Key files |
|-------|---------|-----------|
| **ffb-core** | Game state, rules, mechanics, RNG | See below |
| **ffb-sim** | Headless simulation engine | `src/simulation.rs`, `src/roster.rs` |
| **ffb-mcts** | Monte Carlo Tree Search | `src/search.rs`, `src/node.rs` |
| **ffb-server** | Axum HTTP + WebSocket API (port 8080) | `src/api.rs`, `src/main.rs` |
| **ffb-py** | PyO3 Python bindings for RL | `src/lib.rs` |

Dependency order: `ffb-core` ← `ffb-sim` ← `ffb-mcts` ← `ffb-server` / `ffb-py`

## ffb-core File Index

Navigate here first for any game logic change:

```
crates/ffb-core/src/
├── lib.rs                    # re-exports
├── types.rs                  # ALL core enums and type aliases
├── actions.rs                # Action enum + validation logic
├── skills.rs                 # SkillSet, Skill enum, skill definitions
├── modifiers.rs              # Modifier system
├── pathfinding.rs            # Movement pathfinding
├── rng.rs                    # GameRng, TestRng, Xoshiro256** parity RNG
├── model/
│   ├── game_state.rs         # GameState (top-level, fast_clone())
│   ├── player.rs             # Player struct
│   ├── team.rs               # Team struct
│   ├── field_model.rs        # FieldModel (26×17 grid) + TacklezoneMap
│   └── mod.rs
├── mechanics/
│   ├── block.rs              # Block resolution
│   ├── injury.rs             # Injury table
│   ├── movement.rs           # Movement helpers
│   ├── pass.rs               # Pass mechanics
│   ├── special_rules.rs      # Special rule effects
│   ├── inducements.rs        # Inducement handling
│   └── mod.rs
└── steps/
    ├── turn_step.rs          # Turn flow
    ├── move_step.rs          # Move step handler
    ├── block_step.rs         # Block step handler
    ├── pass_step.rs          # Pass step handler
    ├── kickoff_events.rs     # Kickoff event handling
    └── mod.rs
```

## ffb-sim File Index

```
crates/ffb-sim/src/
├── simulation.rs             # Main game loop (largest file, 54K)
├── roster.rs                 # All team/player definitions (77K)
├── setup.rs                  # Game initialization
├── parity_log.rs             # Parity logging for Java comparison
├── evaluation.rs             # Game-state evaluation metrics
├── canonical_strategy.rs     # Default scripted strategy
├── move_policy.rs            # Movement policies
└── bin/parity_runner.rs      # CLI binary for batch parity runs
```

## Key Type Locations

| Type | File |
|------|------|
| `GameState` | `ffb-core/src/model/game_state.rs` |
| `Player` | `ffb-core/src/model/player.rs` |
| `Team` | `ffb-core/src/model/team.rs` |
| `FieldModel`, `TacklezoneMap` | `ffb-core/src/model/field_model.rs` |
| `Action` | `ffb-core/src/actions.rs` |
| `Skill`, `SkillSet` | `ffb-core/src/skills.rs` |
| `GameRng`, `TestRng` | `ffb-core/src/rng.rs` |
| All other enums/type aliases | `ffb-core/src/types.rs` |

## Where to Look for Common Tasks

| Task | Start here |
|------|-----------|
| Add a skill | `ffb-core/src/skills.rs` + `ffb-core/src/types.rs` |
| Add/change a mechanic | `ffb-core/src/mechanics/<relevant>.rs` |
| Add/change a game step | `ffb-core/src/steps/<relevant>_step.rs` |
| Add an action type | `ffb-core/src/actions.rs` |
| Change simulation flow | `ffb-sim/src/simulation.rs` |
| Add team/player roster | `ffb-sim/src/roster.rs` |
| Tune MCTS scoring | `ffb-mcts/src/search.rs`, `ffb-mcts/src/node.rs` |
| Add API endpoint | `ffb-server/src/api.rs` |
| Python binding | `ffb-py/src/lib.rs` |

## Architecture Decisions

- **SkillSet**: `u64` bitmask for first 64 skills + `BTreeSet` overflow for rare skills
- **FieldModel**: `Vec<Option<PlayerId>>` of 26×17=442 squares; `TacklezoneMap` maintained incrementally on every player move
- **MCTS tree**: unified Decision/Chance node tree; `OutcomeController` enum (`Stochastic`/`Stratified`/`Fixed`/`Exhaustive`) controls dice sampling; `RolloutDepth` enum (`None`/`Steps`/`Turns`/`UntilKickoff`/`UntilHalf`/`Full`)
- **Parity target**: 100% outcome match vs Java engine using Xoshiro256** seeded games with canonical policy

## Parity Testing

Cross-engine parity compares Rust vs Java game outcomes step-by-step using identical Xoshiro256** seeds and canonical (deterministic) strategy. Both engines must produce identical JSONL logs.

```bash
# Run parity runner binary
cargo run -p ffb-sim --bin parity_runner -- <args>

# Compare logs
python scripts/compare_parity.py <rust_log> <java_log>
```

Known gaps (V1): step count mismatch, player ID difference in some cases.
