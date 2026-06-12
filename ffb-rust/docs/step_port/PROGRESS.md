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
- [ ] **Phase C — step framework** (`step/`: Step enum+dispatch, StepStack=Vec, StepParameter
  enum, StepResult, driver loop, Action/Prompt↔ClientCommand/Dialog adapter). Gate: StartGame→
  coin→setup runs to first prompt.
- [ ] **Phase D — BB2025 skill-less lineman steps** (~8 generators + ~50 steps, 0 behaviours).
  Gate: `--tier 3 lineman --seeds 1-100` = 100/100 + coverage checklist green; `--tier 2`
  lineman = 100/100. (This is the milestone the monolith reached only at 4/100.)
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
