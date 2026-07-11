# TESTING.md — per-step testing protocol (fundamental, not optional)

Testing is part of the **definition of done** for every translated step/sequence/skill. A step is
"done" only when it is a complete 1:1 translation of the Java source AND its tests pass.
Verification: `cargo test --workspace`. Three layers, from smallest to largest:

## Layer 1 — Rust per-step characterization test (MANDATORY, one per step)
For each step, a `#[test]` that:
1. Builds a fixed `Game` fixture + a seeded `GameRng` (so dice are deterministic).
2. Runs the step (push it, drive the loop, or call its lifecycle hooks directly).
3. Asserts, exactly: **the dice drawn and their order**, **the `GameEvent`s emitted and their
   order**, and **the resulting `state_hash`** (and key state fields).
This is the executable form of the step's `20_steps/` spec entry. It pins behaviour so later
changes can't silently drift, and a failure names the exact step. Co-locate in the step's
module (`#[cfg(test)]`).

## Layer 2 — Java oracle: per-step golden trace (near-free, from the runner)
Java's step layer has ~no unit tests, and hand-writing a JUnit test per step is expensive in a
build we touch rarely. Instead the **oracle comes from the ParityRunner**, which already runs
every step of every seed. Extend its instrumentation (the existing `FFB_DICE_TRACE` /
`-Dffb.diceTrace`) to emit, per step: the step id, dice (type+result in order), events, and the
post-step `state_hash`. That gives a Java "golden trace" for the lineman seeds at step
granularity — the authoritative expected values the Rust Layer-1 tests are written against.
- Write a **targeted JUnit step test only where a step's logic is subtle** and isolated pinning
  pays off (e.g. pushback square selection, block-result branching, foul referee) — not
  dogmatically for all ~50 steps. Add these under `ffb-server/src/test/java/.../server/step/`.

## Layer 3 — Whole-game parity (the integration gate, already built)
`ffb-parity` runs Rust vs the Java ParityRunner over seeds 1-100 and compares per-activation
`state_hash`. This is the ultimate cross-engine check and the phase gate. The coverage
checklist (`t3_checklist.rs`) additionally proves every action/event actually occurred.

**Current parity status:** `engine.rs` (the former 3,920-line monolith) was deleted in Phase ZR;
`driver.rs` is now the sole live code path (`Box<dyn Step>` dispatch via `make_step()`,
`DriverGameState` game loop). Parity/integration testing against the Java engine is explicitly
deferred again during the current client-logic translation phase (ZW) per project instruction —
see `TRANSLATION_TRACKER.md`'s Progress Summary for the current phase's scope.

## How the layers fit the port loop (TDD against the Java oracle)
For each step, in order:
1. Read its `20_steps/` entry + the Java class.
2. (If subtle) add the Java JUnit pin; capture its golden values. Otherwise capture from the
   runner's step trace.
3. Port the step to Rust.
4. Write the Layer-1 Rust test asserting the golden dice/events/hash. Make it green.
5. Run the seed parity covering that step (Layer 3). Green → tick the spec checkbox with the
   validating seeds.

## Generator testing

Generators have no layer-1 test in the traditional sense (they don't emit events or draw dice
directly). Instead verify a generator by:
1. Read `10_sequences.md` for the sequence you implemented.
2. Add a `#[test]` that creates a minimal `Game` fixture, calls `YourGenerator::push_sequence()`,
   and asserts the resulting `StepStack` contains the exact steps in the documented order with
   the correct `StepParameter` values. No dice, no events — just stack contents.
3. Layer 3 parity (`driver.rs`, the live code path) is the integration gate for generators
   as well — step order differences surface immediately in the state_hash comparison.

## What NOT to test (low value)
- Trivial model getters/setters and serde round-trips (ffb-model). Keep these minimal.
- `ffb-mechanics` already has ~1,219 passing tests mirroring Java's mechanic/modifier calcs —
  keep and extend at the edges, don't rewrite.

## Invariants the tests must lock (see INVARIANTS.md)
GameRng draw order == Java step order; identical `GameEvent` order; identical `state_hash` per
activation; the agent RNG contract (`AGENT_CONTRACT.md`). Any test that would pass while one of
these drifts is not strong enough.
