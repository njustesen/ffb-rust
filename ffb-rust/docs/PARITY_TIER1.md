# Parity Tier-1 — human lineman vs human lineman, seeds 1-100 (state-hash parity)

Goal: full deterministic random agent (all player-turn actions + pre-game inducements), Java↔Rust
identical per-step state hashes. Engine frozen (fix Rust only); ParityRunner.java harness co-editable.
Run: `./target/release/ffb-parity --home lineman --away lineman --edition bb2025 --tier 3 --seeds 1-100`.
See AGENT_CONTRACT.md (authoritative agent spec) + memory parity-tier1-goal.

## Baseline (2026-08-01, iter 1) — FAIL at seed 1, step 1

- **GameStart hash MATCHES** (both `384ccaed1d572749`) → game construction + setup are already identical.
- **First divergence = which team receives / plays turn 1.** First logged step (tier-3 = per-activation):
  - Java: `active=away`, `Activate(teamLinemanParityAway3, BLITZ)`, state_hash `dc491fa21c88693b`.
  - Rust: `active=home`, `Activate(home_01, Blitz)`, state_hash `f532293736cdbe8b`.
  The pre-game coin-toss/receive-choice (NOT logged as a step at tier 3) resolves to a different
  receiving team. Everything downstream diverges from there.

### Open item #1 (next): coin-toss / receive-choice resolution
Root-cause why Rust picks HOME to receive but Java picks AWAY, given both share `new_parity(seed=1)` and
AGENT_CONTRACT §2.1 CoinChoice (1 decisionRng call, %2==0→heads) + §2.2 ReceiveChoice (1 call,
%2==0→receive). Candidates: (a) coin-winner→chooser mapping differs; (b) receive-choice→acting-team
mapping differs (who kicks vs receives); (c) decisionRng consumption order diverged before turn 1.
Compare Rust StepCoinChoice / kickoff sequence vs Java StepCoinChoice.java. Fix Rust, add a unit test
asserting the receiving team for seed 1 matches Java (away). Then re-run seeds 1-3.

## Inducement note
Mirror lineman teams → equal TV → likely 0 petty-cash budget; the pre-game inducement/prayer PHASE must
still state-hash-match. If exercising real purchases needs a TV asymmetry, flag to the user.

## Iter 2 (2026-08-01) — ROOT CAUSE of the coin/receive cascade: wrong fan-factor die in bb2025 pre-game

Traced game dice with FFB_DICE_TRACE=1 (Rust `DICE_TRACE pos=N sides=S result=R` [no caller]; Java
`DICE_TRACE ...caller=<Java stack>`). The streams diverge from ROLL #1:
- Rust pos1-2: d6,d6 (`rng.d6_two()` per team). Java pos1-2: **d3,d3** (`rollFanFactor()=rollDice(3)`).
- Everything after shifts out of phase → coin flip winner, receive, who-plays-turn-1 all diverge. (Rust
  rolls 311 game dice for seed 1 vs Java's 79 — games diverge immediately.)

Coin/receive STEP logic itself is already 1:1 (StepCoinChoice/StepReceiveChoice verified line-by-line);
`rng.bool()` (`range(2)==0`) == Java `throwCoin()` (`rollDice(2)==1`); both default home_playing=true. So
the coin divergence is a SYMPTOM, not the cause — the cause is the fan-factor roll.

**BUG (Rust-only fix): `crates/ffb-engine/src/step/mixed/start/step_spectators.rs`** rolls `rng.d6_two()`
per team (2d6). Java ground truth `ffb-server/.../step/mixed/start/StepSpectators.java`:
```
int fanRollHome = rollFanFactor();  // = rollDice(3) → a SINGLE d3
teamResultHome.setFanFactor(teamHome.getDedicatedFans() + fanRollHome);
int fanRollAway = rollFanFactor();  // single d3
teamResultAway.setFanFactor(teamAway.getDedicatedFans() + fanRollAway);
addReport(new ReportFanFactor(homeId, fanRollHome, homeDedicatedFans));  // + away
```
Rust's own comment is even self-contradictory ("rollFanFactor() → 2D6"). The bb2016 StepSpectators
(2d6 via rollSpectators()) is correct for bb2016 but was wrongly copied into the mixed bb2020/bb2025 step.

**FIX (next iter):** in mixed/start/step_spectators.rs replace `rng.d6_two()` with a single d3 roll
(verify rng has a d3/`range(3)+1` equivalent; check ReportFanFactor takes a single int, not an array).
Add a Rust unit test asserting the mixed spectators step consumes exactly ONE d3 per team and sets
fan_factor = dedicated_fans + d3. Then `cargo build -p ffb-parity --release` + rerun `--seeds 1-3` and
re-diff. Expect the coin/receive/first-activation divergence to clear; proceed to the next first-divergence.

## Iter 3 (2026-08-01) — FIX APPLIED & VERIFIED: bb2025 fan-factor now single d3

Changed `mixed/start/step_spectators.rs`: `rng.d6_two()` → `rng.d3()` per team (matches Java
`rollFanFactor()`=`rollDice(3)`). Added Rust tests: `consumes_exactly_one_d3_per_team_two_draws_total`
(asserts rng.call_count==2, the crux), `fan_factor_is_dedicated_fans_plus_one_d3_each`, updated bounds.
`cargo test -p ffb-engine step_spectators` = 13 passed.

RESULT: the ENTIRE pre-game now aligns — seed 1 step 0 `state_hash` is IDENTICAL on both
(`dc491fa21c88693b`), both pick `away3 BLITZ`. Coin/receive/setup/kickoff cascade resolved.

### Open item #2 (next): first-activation BLITZ execution diverges
seed 1, step 0: same pre-state (dc49…) + same action (away3 BLITZ), but `post_hash` differs:
Java `9c55510376cb6887` vs Rust `b118148452e3779f`. So executing the blitz (block dice / target / push /
armour / follow-up) produces different state. Dice-trace the blitz: FFB_DICE_TRACE=1, compare Rust
(`DICE_TRACE` no caller) vs Java (`caller=`) game dice AT/after the first activation to find the first
differing block/armour/injury die or a step-logic difference. Root-cause in the relevant Rust block/blitz
step vs its Java ground-truth class; fix Rust; add a test; rerun seeds 1-3.

## Iter 4 (2026-08-01) — Open item #2 investigation: blitz-move dodge/fall divergence (NOT yet fixed)

Dice-traced seed 1 (FFB_DICE_TRACE=1). Rust vs Java game dice are pos-for-pos IDENTICAL through pos 19
(fan 2,2 d3; weather 3,6; coin d2=2; kickoff scatter/result/cheering/throw-in; then the first activation's
rolls). First die divergence at **pos 20**: Java `s16=16` (RollMechanic.rollCasualty via
InjuryTypeServer.setInjury) vs Rust `s6=2`.

Java path (caller stacks): first activation = away3 BLITZ; the blitzer fails a DODGE during its move →
`InjuryTypeDropDodge` → rollArmour (pos16,17 = 2,6 = 8, broken) → rollInjury (pos18,19 = 6,5 = 11) →
11 ≥ 10 = CASUALTY → rollCasualty d16 (pos20) + d6 (pos21).

BUT it's not a simple casualty-threshold miss: `interpret_injury_total_bb2020(11,false,false)` DOES return
None→Casualty, and find_injury_modifiers is empty for two skill-less linemen — so Rust *should* roll the
d16. The Rust events log instead shows away_03 dodging SUCCESSFULLY (target 3, roll 6) and a DIFFERENT
player (away_02) failing/rerolling/falling later. So the two engines diverge in the blitz-move DODGE
resolution / which player falls — the pos-value match through 19 is partly coincidental, and the true
divergence is UPSTREAM of the casualty roll (in the blitz move / dodge path or activation sequencing).

### Open item #2 (refined, next): correlate each die to its player/step on BOTH sides
Rerun with FFB_DICE_TRACE=1 AND FFB_TRACE=1 (Rust `LOOP applied=`) AND FFB_DRIVE_TRACE=1 (Rust step
dispatch). For pos13-20 map each roll to (step, acting player, action, dodge target) on Rust vs Java
(Java caller stacks already identify the step/injury type). Determine at which pos the ACTING PLAYER or
dodge CONTEXT first differs (e.g. does away3 attempt the same blitz path / same dodge square?). That first
context divergence — not the casualty roll — is the real bug. Likely candidates: blitz move-square choice,
dodge modifier/target computation, or the order in which the blitzer's move dodges resolve. Fix Rust only,
add a test, rerun. (Fan-factor fix from iter 3 stands: pre-game fully aligned.)
