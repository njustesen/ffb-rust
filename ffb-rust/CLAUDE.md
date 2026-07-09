# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Is

FFB-Rust is a **1:1 Java-to-Rust translation** of a Fantasy Football (Blood Bowl) game engine. The goal is a complete, file-for-file, method-for-method translation of the Java source into idiomatic Rust — every Java class becomes one Rust file, every Java method becomes one Rust function (same name in snake_case), every Java field becomes one Rust struct field.

The project supports three rule editions (BB2016, BB2020, BB2025).

## Translation Ground Rules

- **Before writing any Rust code:** open the corresponding Java source file at `ffb-java/ffb/<module>/src/main/java/com/fumbbl/ffb/<path>.java`
- Translate every field (drop `f` prefix, camelCase → snake_case)
- Translate every method in order (same name in snake_case, same internal logic)
- Do not add any logic that cannot be traced back to a line in the Java source
- Java enum variants use SCREAMING_SNAKE_CASE with `#[allow(non_camel_case_types)]`
- Java `getId()` → `get_id()`, `getName()` → `get_name()`, `forName()` → `for_name()`

See `TRANSLATION_TRACKER.md` for per-file status (`○` not started, `~` partial, `✓` done, `—` skip).

## Common Commands

```bash
cargo build                                        # Build all crates
cargo build --release                              # Optimized build
cargo test --workspace                             # Run all unit tests
cargo test -p ffb-engine                           # Test a single crate
cargo test -p ffb-engine -- block                  # Run tests matching "block"
cargo clippy                                       # Lint
cargo fmt --check                                  # Check formatting
```

## Crate Architecture

The workspace has six crates with a strict dependency order:

```
ffb-model → ffb-mechanics → ffb-engine → ffb-client
                                       → ffb-parity
ffb-model → ffb-protocol → ffb-client
```

| Crate | Purpose | Java Source |
|-------|---------|------------|
| **ffb-model** | Data types: enums, domain structs (Game, Team, Player, FieldModel), RNG, events, agent prompts | `ffb-common` |
| **ffb-mechanics** | Pure rule calculations — block dice, pass, injury, scatter, modifiers. No game state mutation. | `ffb-server/mechanic/` |
| **ffb-engine** | Step-based game state machine. Each Java step class → one Rust file. | `ffb-server/step/` |
| **ffb-protocol** | Serializable client/server command structs for WebSocket communication | `ffb-common/net/` |
| **ffb-client** | Client-side state machine and WebSocket handling | `ffb-client-logic` |
| **ffb-parity** | Parity test harness: runs both Java and Rust headless, diffs JSONL logs | (Rust-only harness) |

## Engine Architecture

**`engine.rs` has been deleted (Phase ZR).** `driver.rs` is the live code path — `Box<dyn Step>` dispatch via `make_step()`, `DriverGameState` game loop, `GameState` type alias for backward compat. All 2,521 step/generator files are translated (100%).

**Java step class → Rust:** `StepBlockRoll.java` → `step/bb2025/step_block_roll.rs` → `struct StepBlockRoll`

**Generator classes → Rust:** Each `XxxSequence.java` or `XxxGenerator.java` → `step/generator/bb2025/xxx.rs`. Generators push ordered step sequences onto the stack.

### Engine output channels

The Rust engine uses two output channels instead of Java's direct networking calls:

| Java pattern | Rust equivalent |
|---|---|
| `server.sendXxx(...)` | Emit `GameEvent::Xxx` via `StepOutcome::with_event()` |
| `UtilServerDialog.showDialog(dialog)` | Return `StepOutcome::cont().with_prompt(AgentPrompt::Xxx)` |

`GameEvent` variants are defined in `ffb-model/src/events/game_event.rs`.  
`AgentPrompt` variants are defined in `ffb-model/src/prompts/agent_prompt.rs`.

The `ffb-server` layer (Phase ZT) will subscribe to these and serialize them as protocol commands to the Java GUI client over WebSocket.

**Loop pattern:** Java `pushCurrentStepOnStack() + setNextAction(NEXT_STEP)` → Rust `StepAction::Repeat`. The driver re-calls `start()` on the same step instance (same struct, same mutable fields). `StepAction::Continue` = waiting for user dialog.

## BB2025 Ruleset Reference

`rules/` contains the full Blood Bowl 2025 ruleset as local markdown files.

- `rules/core_rules/` — 11 files: game essentials, rules & regulations, etc.
- `rules/star_players/` — 63 individual files, one per star player

To refresh: `python scripts/download_rules.py`

## Data

`data/` contains JSON configs loaded at runtime by `ffb-model/src/data/loader.rs`:
- `rosters/` — Team definitions per edition (positions, stats, starting skills)
- `skills/`, `inducements/`, `injuries/`, `prayers/`, `cards/`, `star_players/`

## Testing

Unit tests are colocated with implementations (`#[cfg(test)]` modules). After translating each file, run:

```bash
cargo test --workspace
```

All tests must pass before marking a file `✓` in TRANSLATION_TRACKER.md. Do **not** patch `engine.rs` as a workaround — if tests fail, the translation is incorrect.
