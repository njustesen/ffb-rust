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

## Iter 5 (2026-08-01) — narrowed Open item #2 to the blitzer's post-block MOVE/follow-up path

Correlated first activation (away_03 BLITZ) via FFB_TRACE/FFB_DRIVE_TRACE + rust_events.jsonl + Java
JAVA_*/DICE caller stacks. Block itself matches: both block home_03, 1 die = 4 (pos13), pushback.
Then they DIVERGE in the blitzer's continued move:
- RUST (events): away_03 pushback home_03→(11,9); away_03 moves (14,7); dodgeRoll target 3 roll 6 SUCCESS;
  continues (14,8)→(14,9)→(14,8)… away_03 NEVER falls, no injury.
- JAVA (trace): JAVA_P2 away3 BLITZ_MOVE; pos14,15 StepMoveDodge.dodge (6,4); then InjuryTypeDropDodge
  (armour pos16,17=2,6=8 broken; injury pos18,19=6,5=11) → CASUALTY d16 (pos20). A player falls from a
  dodge during the blitz move and is cas'd.

So the divergence is the BLITZER'S POST-BLOCK MOVE: pushback square, FOLLOW-UP decision, and/or the
continued move-square sequence differ, putting the blitzer into a different dodge (Java fails/falls, Rust
succeeds and wanders). Die VALUES align through pos19 only because both draw the same shared GameRng
stream; the true split is the extra casualty roll Java makes.

### Open item #2 (further refined, next): compare blitzer post-block move step-by-step
Prime suspects (check in this order): (1) FOLLOW-UP after the block push — AGENT_CONTRACT §7 says decline;
verify Rust's blitz actually offers/declines follow-up like Java (a wrong follow-up moves the blitzer to a
different square). (2) The continued BLITZ_MOVE square sequence / count (how many squares the blitzer moves
after the block, and the coord-sorted actionRng move-square picks) — Rust wandered (14,7)→(14,8)→(14,9)→
(14,8) which looks like aimless back-and-forth vs Java's directed move into a dodge. (3) whether Rust even
enters a StepMoveDodge for the blitzer's continuation. Open crates/ffb-engine/src/step/**/*blitz*/*move*
and the Java StepBlitz/StepMove/StepMoveDodge + the follow-up step; find the first behavioral difference;
fix Rust only; add a test; rerun. NOTE the Rust move wander (14,8↔14,9) may itself be an agent move-target
bug — cross-check against AGENT_CONTRACT §3/§6 (coord-sorted move-square pick).

## Iter 6 (2026-08-01) — FIX #2 APPLIED: pushback square selection (min-(x,y), no rng)

Root-caused the FIRST of two blitz divergences: the agent's Pushback handling. Rust
`random_agent.rs` sorted the pushback squares then RANDOM-indexed (`self.pick(len)`) AND consumed a
decision_rng call. Java `ParityRunner.sendPushback` keeps the min-(x,y) non-locked square and consumes
ZERO decisionRng calls (AGENT_CONTRACT §7 = "min-(x,y) on-pitch square"). Rust picked (11,9); Java (11,7).
The spurious decision_rng draw also desynced every later pick. FIX (Rust agent only): extracted
`choose_pushback_square()` = min-by-(x,y), deterministic, no rng; added test
`pushback_picks_min_xy_square_deterministically`. Also fixed a STALE test
(`random_agent_drives_pregame_with_contract_decision_rng`) whose hard-coded pregame dice count (13, from
the old 2d6 spectators) broke after the iter3 d3 fix — rewrote it to assert determinism instead of a magic
number. `cargo test -p ffb-engine random_agent` = 9 passed.

VERIFIED PARTIAL: seed1 step0 post_hash moved b118148452e3779f → 3205e1285d49894b (home_03 now pushed to
the min square like Java), but still ≠ Java 9c55510376cb6887 — because divergence #2 remains.

### Open item #2b (next): blitzer keeps moving after the block (Rust) vs stops (Java)
Java trace: away3 BLITZ → block home3 → pushback (11,7) → JAVA_P2 away5 (away3's activation ENDS after the
block; no continued move). Rust events: away_03 blocks, pushes, then MOVES 7+ squares (14,7→…→14,11 with
GFIs) and dodges before ending. So Rust's agent continues the blitzer's move after the block; Java's does
not. Root-cause: does the Rust blitz agent path issue continued-move commands after the block that Java's
ParityRunner doesn't? Compare the blitz continuation in random_agent.rs / the blitz step vs Java
ParityRunner's BLITZ handling (JAVA_P2 shows away3 does one BLITZ_MOVE=the block, then the next P2 is a
different player). Align Rust to stop the blitzer after its block (or match whatever Java's continued-move
policy is), add a test, rerun. NOTE: also confirm the coord-sorted move-target logic (AGENT_CONTRACT §3/§6)
isn't separately producing the (14,8)↔(14,9) wander.

## Iter 7 (2026-08-01) — FIX #3 APPLIED: blitzer ends after its block (no continued move)

Java ParityRunner (lines 1007-1021): a BLITZ sends the block ONCE (blitzBlockSent guard) then ENDS the
activation (JAVA_BLITZ_END → deselect). The blitzer never spends remaining MA moving. Rust's agent had no
per-activation action memory, so the post-block Move prompt was treated as a normal move → the blitzer
wandered 7+ squares. FIX (Rust agent only): added `current_activation_is_blitz` flag (mirrors Java
blitzBlockSent), set when a Blitz/StandUpBlitz action is picked; the Move-prompt handler now returns
EndPlayerAction (0 rng) on the first Move prompt of a blitz activation. VERIFIED via rust_events.jsonl:
away_03 now blocks home_03, pushes, and STOPS (next player away_02) — 1:1 with Java's block-then-end.

VERIFIED PARTIAL: seed1 step0 post_hash 3205e1285d49894b → 88336fa29764452e (still ≠ Java 9c55510376cb6887).
home_03 (the pushed defender) now lands at (11,7) on BOTH engines (pushback fully correct). RESIDUAL: one
away player's final position still differs (state-string a03 = 20,5 Rust vs 18,4 Java).

ID MAPPING RESOLVED (important for reading state strings): the rust_events/JAVA player ids are 1-based
(away_03) but the RUST_STEP/JSTEP state string is 0-based (a00..a10, h00..h10). So away_03 = a02,
home_03 = h02. At step0 start a02(away_03)=13,8 and h02(home_03)=12,8 — ADJACENT, so the blitzer blocks
with NO pre-block move (consistent with §8). The residual difference is at a03 = away_04 (20,5 vs 18,4),
NOT the blitzer — a04 should NOT move during away_03's activation, so investigate why a04's position
diverges in step0 (candidates: a chain-push of a04 during home_03's pushback? a04 adjacent to the push
path? or the state snapshot timing). NEXT: diff the FULL step0 pre- vs post- state (all 22 players) Rust
vs Java to find exactly which player(s) and coordinate differ, then root-cause that single delta.

NOTE: fix #3 is verified at integration level (parity events match Java's blitz-end). A focused agent
unit test needs a driver fixture that presents a Move prompt with the blitz flag set — add next iteration.

## Iter 8 (2026-08-01) — FIX #4 APPLIED: follow-up after push = always decline (no rng). STEP 0 NOW MATCHES.

Full 22-player step0 state diff isolated the residual to ONE coordinate: the blitzer a02 (away_03) —
Java (13,8, its start) vs Rust (12,8, home_03's VACATED square). That's a FOLLOW-UP into the pushed
player's square. AGENT_CONTRACT §7 = decline; Java ParityRunner FOLLOWUP_CHOICE = `sendFollowupChoice(false)`
(deterministic, 0 rng). Rust's FollowUp handler used `self.pick_bool()` (random + consumed a decision_rng
draw) — same bug pattern as the pushback bug: wrong choice AND a stream desync. FIX (Rust agent only):
FollowUp → `follow_up: false`, 0 rng.

RESULT (big step): seed1 step0 post_hash now == Java 9c55510376cb6887 (blitzer stays at 13,8). Removing the
spurious decision_rng draw ALSO re-synced the decision stream so step1's player pick now matches (both pick
away5 MOVE). Comparator now fails at STEP 2 (was step 1) — steps 0 and 1 (pick+pre-state) align.

### Open item #3 (next): step1 move execution (away5 MOVE) diverges
seed1 step1 = away5 MOVE; pre-state matches but post_hash differs (Java 69d972900b86dfd0 vs Rust
bd7994458e997710), and the subsequent step2 pick differs (Java away7 vs Rust away_06) as a consequence.
So a plain MOVE activation's execution/path diverges. Diff step1 pre- vs post- state (full 22 players) to
find which coordinate(s) differ after away5's move; compare the move-square sequence in rust_events.jsonl
(playerMoved for away5) vs Java's JAVA_P2 away5 + JAVA_GFI/move trace. Candidates: move-square pick
(coord-sorted actionRng §3/§6), number of squares moved / MA-spend policy (INIT_MOVING keep-moving), GFI
handling, or a dodge along the path. Root-cause, fix Rust only, add a test, rerun. Carry-over: still add a
focused unit test for the blitz-end + follow-up agent fixes if a driver fixture is feasible.

## Iter 9 (2026-08-01) — Open item #3 diagnosis: non-carrier move continuation (fix attempt REVERTED, unverified)

Confirmed the Java policy from ParityRunner.java INIT_MOVING (lines 419-444): after the first move, the
agent continues moving ONLY if `carrying && movesLeft`; every OTHER player DESELECTS after that single
square. So: ball carrier moves until MA spent; all others move exactly ONE square then end. Rust's Move
handler keeps moving every player until squares are empty → non-carriers wander their whole MA.

Attempted a fix (moved_this_activation flag: after 1 square, end unless carrier). cargo test green, but the
seed-1 rerun did NOT cleanly verify: post-step1 diff showed away5 (state-string a04) at Rust (20,7)=its
START vs Java (20,8) — i.e. Rust appeared to move 0 squares while Java moved 1, suggesting the flag guard
may fire BEFORE the first move (or a reset path is missed), plus a secondary a09 diff. Rather than commit
an unverified/possibly-buggy change, REVERTED it (fixes 1-4 remain). ID-mapping + whole-game event
interleaving made live verification error-prone this iteration.

### Open item #3 (re-scoped, next): implement non-carrier-1-square carefully + VERIFY
Re-implement the INIT_MOVING mirror with care: (1) the FIRST Move prompt of an activation must ALWAYS move
one square (carrier or not) — confirm `moved_this_activation` is false at that point (reset on EVERY
activation start, including StandUp/Move/Foul/Pass, not just via one return path); (2) the SECOND+ Move
prompt ends unless carrying. Add a DRIVER-LEVEL test (drive a bare non-carrier Move activation and assert
exactly one playerMoved) — the unit-level flag test isn't enough. VERIFY on seed 1: away5 (a04) must end
at Java's (20,8), step1 post_hash == 69d972900b86dfd0. Watch the ID mapping: events 1-based (away_05),
state string 0-based (a04). Also re-examine whether Java's away5 first-square pick (→20,8, moving AWAY from
the ball) vs Rust's (→21,8, toward ball) is a separate move-square-PICK divergence (coord-sort/actionRng,
§3/§6) that must ALSO be aligned — the destination difference may be the real step1 bug, independent of the
continuation count.
