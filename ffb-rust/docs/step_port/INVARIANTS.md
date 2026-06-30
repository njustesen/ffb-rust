# INVARIANTS.md — frozen primitives & reuse boundary

These must hold byte-for-byte for parity. Any change that violates one breaks cross-engine
determinism. Treat them as non-negotiable while porting.

## Frozen primitives (DO NOT alter behaviour)
- **`GameRng`** (`crates/ffb-model/src/util/rng.rs`) — Xoshiro256** seeded via SplitMix64,
  rejection-sampling `die(sides)`; matches Java `Fortuna`/`Xoshiro256StarStar` byte-for-byte.
  The **order** of draws is the parity contract: it must equal Java's *step execution order*.
  Porting steps in Java's order reproduces it by construction — that is the whole point of the
  rewrite. `call_count` + `FFB_DICE_TRACE` expose the stream for diffing.
- **`state_hash` / `state_string`** (`crates/ffb-model/src/util/state_hash.rs`) — FNV-1a 64-bit
  over the canonical string `h{half}t{th}{ta}a{active}s{hs},{as} b{bx},{by},{inplay} p{players
  sorted by nr, h00.. then a00.., as x,y,state}`. Must equal Java `ParityRunner.fnv1a64`. KO/
  CAS/banned/off-pitch players render as `-1,-1`. Computed after every activation.
- **`GameEvent` order** — each step emits the same events in the same order as Java's reports.
  Events are NOT part of the hash, but the parity log + coverage checklist compare them.
- **Agent RNG contract** (`AGENT_CONTRACT.md`) — decisionRng (`seed ^ 0xDEAD_BEEF_CAFE_0001`)
  and actionRng (`seed ^ 0xC0FFEE_ACE0_0001`); pick = `next_u64() % n`; consumption order
  (coin, receive, kickball x2, player pick incl. skip-loop rejections, action pick, target
  pick, …). The RandomAgent and Java ParityRunner both implement it; the new engine must feed
  the agent the same prompts in the same order.

## Reuse boundary (kept; NOT rewritten by the step port)
- `ffb-model` — domain types, `GameRng`, `state_hash`, `GameEvent`/`Action`/`AgentPrompt`
  enums. (Perf tweaks only if behaviour-neutral.)
- `ffb-mechanics` — pure rule calcs (block dice/assists, dodge/GFI/catch/pickup targets,
  injury/casualty tables, pass result, scatter coordinate, modifiers). Mirrors Java
  `ffb-common` `mechanic/`+`modifiers/`+`DiceInterpreter`; ~1,219 tests; steps call it.
- `legal_actions/` — pure eligibility/target queries; used by the new `StepInitSelecting`.
- `ffb-parity` (harness, comparator, log_format, FFB_DICE_TRACE, `t3_checklist`),
  `ffb-protocol`, `ffb-client`, and the parity `RandomAgent`.
- Java `ParityRunner` (branch `t3-phase2-wip`) = ground-truth oracle/driver.

## Replacing (the monolith — in progress)
`ffb-engine/src/step/engine.rs` (3,920-line monolith) is being replaced by individual step
files under `step/bb2025/`, each a 1:1 translation of the corresponding Java step class.
Once all step files are complete, `engine.rs` is deleted and replaced by a thin `driver.rs`
(< 300 lines). Steps are written directly from Java source — **never** extracted from engine.rs.

## Rust representation choices (behaviour-neutral, for perf/idiom)
Step = enum + match dispatch (no `Box<dyn>`); StepStack = `Vec<Step>` (top = last);
StepParameter = typed enum + `consumed` flag; driver loop is flattened (no recursion). See
`00_framework.md` §7 for the full list and the rules that MUST stay identical (goto popping,
sequence push order, publish/consume order, forwardCommand, REPEAT drain, sync-not-resetting-
next_action, driver-ignores-StepCommandStatus, session/acting-player gating, init validation).
