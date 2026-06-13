# NEXT_SESSION.md — Phase D, batch 1: first-activation parity

Read `PROGRESS.md` first, then this. Source-of-truth specs: `10_sequences.md` (step order),
`20_steps/pass_kickoff_end.md` + `20_steps/move_select.md` (per-step entries), `INVARIANTS.md`
(frozen primitives), `TESTING.md` (per-step test protocol). Java ground truth = `ParityRunner`
on branch `t3-phase2-wip`.

## Milestone (the clear target)

**Lineman vs lineman, seed 1: the new engine reaches the first `ActivatePlayer` prompt of the
receiving team's first turn, and matches Java exactly there** —
1. `FFB_DICE_TRACE` streams align **step-for-step** from game start through the opening kickoff
   (fan d3×2 · weather 2d6 · coin d2 · kickoff scatter d8+d6 · kickoff-result 2d6 · any result
   dice · catch/bounce), and
2. the logged `state_hash` matches Java at **GameStart (i:0)** and at the **first ActivatePlayer**.

**Definition of done:** `cargo run -p ffb-parity -- --tier 3 --home lineman --away lineman
--seeds 1` produces a Rust log whose GameStart + first-activation lines hash-match the Java log
(comparator clean to that point), and the dice trace diff is empty through first activation.

**Stretch:** extend to seeds 1–10 (fills in whichever kickoff-result events those seeds roll).

**Fallback checkpoint** (if the full kickoff overruns the session): parity through **Setup + the
KickBall placement**, i.e. dice/hash match up to (not including) the kickoff scatter roll. Still
a real, committed parity gain.

## Why this milestone

It's the smallest end-to-end slice that produces a *playable* game state, and it exercises the
whole shared spine every game and every race depends on: pregame → setup → kickoff → activation.
Getting it 1:1 means the by-construction RNG-order advantage is proven on real dice, and every
later step plugs into a validated foundation.

## Ordered work

### 0. Harness reconciliation (do first — unblocks all verification)
- **GameStart hash point.** Java logs `initial_hash` on a fresh game *before any roll*. The
  current `GameState::new` rolls the pregame in the constructor, so its post-construction hash
  already includes weather/fan. Split it: build via `from_game` (no rolls) → harness snapshots
  the GameStart hash → then push StartGame + `run_until_prompt`. Adjust `runner.rs` accordingly.
- **Side inference.** Confirm `active_side()`/`apply(side,_)` agree with who Java expects to act
  at each prompt (the `side` arg is advisory; the engine infers from `home_playing`).
- Re-confirm the harness loop logs the existing `LogLine` shapes and runs the comparator vs the
  Java seed-1 log. Get an (expected) early divergence report as the working signal.

### 1. Restructure sequences to match Java (`10_sequences.md`)
- `StartGame` = INIT_START_GAME · SPECTATORS · WEATHER · **PETTY_CASH · BUY_INDUCEMENTS**; the
  last continues into **Kickoff**. Move `CoinChoice`/`ReceiveChoice` OUT of `start_game_sequence`
  and INTO the `Kickoff(withCoinChoice)` generator (its steps 1–2), where Java has them.
- Port **PettyCash** (emit PettyCash only on TV diff — equal lineman TVs ⇒ no-op) and
  **BuyInducements** (lineman: none ⇒ no-op that pushes the Kickoff sequence). Keep the
  ReceiveChoice tail Java runs here: **StartHalf**, and **Prayers to Nuffle** (lineman = 0 ⇒
  no-op) — verify order against Java.

### 2. Kickoff sequence → first activation (`20_steps/pass_kickoff_end.md`)
Port in this order (skip skill/inducement-only steps as no-ops for lineman: SWARMING,
MASTER_CHEF, KICKOFF_RETURN, BLITZ_TURN):
- **InitKickoff** → **Setup ×2** (canonical placement of all 11 per side; no dice — mirror the
  parity agent's placement) → **Kickoff** (KickBall: agent picks target square, place ball) →
  **KickoffScatterRoll** (d8 dir + d6 dist; gust-of-wind extra per weather) → **KickoffResultRoll**
  (2d6 → kickoff table) → **ApplyKickoffResult** → **CatchScatterThrowIn** (catch d6 / bounce d8 /
  touchback) → **Touchback** (if off-pitch) → **EndKickoff** → **InitSelecting** → first
  **ActivatePlayer** prompt.
- **ApplyKickoffResult**: implement only the result(s) seed 1 actually rolls; guard the rest with
  an explicit `unimplemented!("kickoff result {x} — port when a seed hits it")` so coverage grows
  visibly as seeds expand. (BB2025 table: Get the Ref, Riot, Perfect Defence, High Kick, Cheering
  Fans, Brilliant Coaching, Quick Snap, Blitz, Throw a Rock, Pitch Invasion, Solid Defence.)
- All dice/targets via `ffb-mechanics` (kept, validated); placement eligibility via
  `legal_actions` (`&Game`).

### 3. Agent (`agent.rs`)
Extend `act` for the new prompts, each per `AGENT_CONTRACT.md`: **KickBall** target (decisionRng),
**Setup** if prompted (canonical, no RNG or fixed), **ActivatePlayer** selection (decisionRng pick
over eligible). Keep the panic-on-unhandled for anything not yet reached.

### 4. Per-step loop (every step, per `TESTING.md`)
read 20_steps entry + Java class → Rust characterization test (fixture+seed → assert dice order,
events, post-hash) → port step → test green → run seed-1 dice-trace diff vs Java → tick the
`20_steps/` checkbox with the validating seed.

## Verify & commit
- `FFB_DICE_TRACE=1` on both engines for seed 1; diff streams (names the diverging step).
- Comparator: GameStart + first-activation hashes match Java seed-1 log.
- Commit per coherent step or small batch; update `PROGRESS.md` Phase D line + tick `20_steps/`.

## Watch-outs
- Setup placement order/coords must match Java's canonical placement exactly (it feeds the hash).
- Kickoff scatter + catch are where `ffb-mechanics` AUDIT_scatter.md findings matter — cross-check.
- Don't silently no-op a step that Java rolls dice in — that desyncs the stream. If unsure whether
  a lineman step consumes dice, check the Java class, not the assumption.
