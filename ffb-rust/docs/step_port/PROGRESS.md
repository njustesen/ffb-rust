# PROGRESS.md — step-architecture rewrite tracker (read this first each session)

## How to resume (start every session here)
1. Read this file, then `00_framework.md` (lifecycle), `INVARIANTS.md` (frozen primitives),
   and the `20_steps/` entry for whatever you're porting. `10_sequences.md` gives step ordering;
   `TESTING.md` the per-step test protocol; `30_skills.md` is Phase E.
2. The monolith is gone; the engine is `ffb-engine/src/step/`. Ground truth = Java
   `ParityRunner` on branch `t3-phase2-wip`. Parity edition = **BB2025**.
3. Port loop per step (TESTING.md): read entry + Java class → capture golden dice/events/hash
   (runner trace, or JUnit pin if subtle) → port to Rust → Rust characterization test green →
   seed parity green → tick the `[ ]` in its `20_steps/` entry with validating seeds.
4. Rollback safety net: git tag `pre-step-rewrite` holds the working monolith.

## Phase status
- [x] **Phase A — porting spec** (this `docs/step_port/` tree): 00_framework, 10_sequences,
  20_steps/ (57 step entries: move_select 12, block_foul_injury 20, pass_kickoff_end 24),
  TESTING, INVARIANTS, 30_skills (stub), PROGRESS. DONE.
- [ ] **Phase B — commit + tag `pre-step-rewrite` + remove monolith** (gut `engine/mod.rs`
  apply/pending_*/inline helpers; keep crate compiling against new `step/`). NOT STARTED.
- [x] **Phase C — step framework** (`step/`). DONE — the framework MECHANISM is complete and
  exercised end-to-end through the pregame; rule-step CONTENT (kickoff→activation) is Phase D.
  - [x] slice 1 (commit 3505560): framework primitives; `Step` enum + match dispatch;
    `StepStack` (Vec, push_sequence reverse, goto_label); `GameState` driver loop (start-mode
    NextStep chain + goto); StartGame pregame steps InitStartGame/Spectators/Weather ported 1:1
    with a characterization test pinning d3,d3,d6,d6 order.
  - [x] slice 2 (commit d9cf5f8): command-mode driver (`apply`→waiting step's `handle_command`,
    forwardCommand/goto/repeat), AgentPrompt emission + Action boundary (`current_prompt`/`apply`/
    `take_events`), StepParameter publish/consume-down-stack plumbing (`StepStack::publish`,
    `Step::set_parameter`). Ported CoinChoice (flip = 1× d2) + ReceiveChoice 1:1. 890 tests green.
  - GATE REFINED (transparent): the former "slice 3 → first ActivatePlayer prompt" requires
    Setup placement + kickoff scatter(d8+d6) + result(2d6, 11 handlers) + catch/touchback — all
    dice-bearing RULE steps that must be parity-validated vs Java seeds. That is Phase D scope,
    not framework. Building them here (before the harness is wired to the new engine) would be
    unvalidated rule code. So Phase C ends at the framework boundary; the kickoff→activation
    chain is Phase D's opening batch.
- [ ] **Phase D — BB2025 skill-less lineman steps** (~8 generators + ~50 steps, 0 behaviours).
  Gate: `--tier 3 lineman --seeds 1-100` = 100/100 + coverage checklist green; `--tier 2`
  lineman = 100/100. (This is the milestone the monolith reached only at 4/100.)
  - START HERE: (a) point the KEPT parity harness run loop (`ffb-parity/src/runner.rs`:
    comparator/log/coverage are reused) at the new `GameState` in place of the deleted monolith
    `GameEngine`. The harness loop is `engine.current_prompt()` → agent → `engine.apply(...)`;
    `GameState` already exposes `current_prompt`/`apply`/`take_events` + per-activation
    `state_hash`. NOTE the signature gap: monolith `apply(side, action)` takes a side; the new
    `GameState::apply(action)` infers the acting side from game state — reconcile this when
    wiring. Use full lineman team fixtures (11 players). Validate the agent RNG contract vs
    AGENT_CONTRACT.md on the pregame coin/receive FIRST (decisionRng channel) before rule steps.
    (`run_game` in ffb-engine/agent/mod.rs is monolith-only test glue and dies with the monolith;
    it is NOT what the harness uses.)
  - then the kickoff→activation batch (the former Phase C "slice 3"): StartHalf/PettyCash/Prayers
    tail of ReceiveChoice, StepSetup (canonical placement), Kickoff generator (KickBall prompt →
    KickoffScatterRoll d8+d6 → KickoffResultRoll 2d6 → ApplyKickoffResult → catch/touchback →
    EndKickoff), InitSelecting → first ActivatePlayer. Each per `20_steps/` with its
    characterization test + seed parity, ticking its checkbox.
- [ ] **Phase E — full BB2025** (remaining steps + ~40 skill behaviours per `30_skills.md`).
  Gate: 26 races ×100 tier-3 = 100/100 + restored T2 26×100.
- [ ] **Phase F — editions** (BB2020/BB2016 overrides). Optional.

### Networking (full framework, no GUI) — spec `40_network.md`. Runs after the engine is playable.
- [ ] **Phase G — wire protocol fidelity**: IJsonOption-equiv key registry (643), 138
  NetCommandIds, 32 ServerCommand + 91 ClientCommand classes byte-exact. Gate: protocol
  round-trip parity vs Java-emitted JSON.
- [ ] **Phase H — model serialization (dominant effort)**: full Game JSON + 169 ModelChange
  encoder/applier with exact keys. Gate: Rust emits byte-identical GameState + ModelSync stream
  vs the Java server for a scripted game (commandNr monotonic, away transform()).
- [ ] **Phase I — Rust WebSocket server**: `/command`, session manager, version→challenge→join
  (STANDALONE auth), single-thread receive→GameState.handleCommand→syncGameModel→sendModelSync,
  entropy→GameRng.
- [ ] **Phase J — interop**: unmodified Java Swing GUI client connects to the Rust server
  (compression off) and plays a full game without desync. Plus a Rust headless client (portable
  seam) for automated interop regression.

## Current parity counts (pre-rewrite baseline, monolith)
- T2 (no-action): 25/26 races ×100 — goblin 94 fails (a session-44 regression; moot post-rewrite).
- T3 lineman (real actions): 4/100 (the monolith burn-down; superseded).
- Carried-forward Java ground truth: `AUDIT_scatter.md` (6 scatter discrepancies),
  `AGENT_CONTRACT.md`, and the dice-sequence notes embedded in the 20_steps entries.

## Next concrete action
Phase B: commit the spec, `git tag pre-step-rewrite`, then begin removing the monolith and
scaffolding `step/` per `00_framework.md`. Then Phase C framework.

## Open (non-rewrite) items, deferred
- visualize seed-0 push loop (task #14); these become moot or guide the new engine.
