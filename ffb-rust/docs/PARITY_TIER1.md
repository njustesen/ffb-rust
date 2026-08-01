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

## Iter 10 (2026-08-01) — FIX #5 APPLIED (verified): non-carrier moves exactly one square

Re-applied the INIT_MOVING mirror (moved_this_activation flag) — this time VERIFIED via bounded trace:
each non-carrier now makes EXACTLY ONE RUST_PICK per activation (away_10, away_05: 1 move each; were 7-8).
The iter9 "0 moves" was an ID-mapping misread. cargo test 9 passed. Java: carrier moves until MA spent,
all others 1 square then deselect.

### Open item #3B (next): move-target set includes OCCUPIED squares (Rust engine bug)
step1 still fails because away5's FIRST move goes to the wrong square. Direct compare:
- JAVA_SMA away5 coord=(20,7) targets=4 → idx=2 → (20,8). Java builds adjacent, UNOCCUPIED, on-pitch squares:
  of the 8 neighbours of (20,7), four are occupied ((19,6)a06,(19,8)a07,(21,6)a08,(21,8)a09) → 4 free:
  (19,7),(20,6),(20,8),(21,7).
- RUST away5 N=6 → idx=5 → (21,8). Rust offers 6 targets INCLUDING (21,8), which is OCCUPIED by a09.
So Rust's legal-move-square set includes occupied squares that Java excludes → wrong count → wrong idx →
wrong destination. FIX (Rust): exclude occupied (and off-pitch) squares from the Move prompt's target set
to match Java sendMoveAction. Find the source of the Move prompt `squares` (engine legal_move_targets /
the step that emits AgentPrompt::Move) — if the ENGINE includes occupied squares that's a Rust engine bug;
fix it there. If instead the agent should filter (mirroring ParityRunner computing its own list), filter in
the agent. Verify targets=4 for away5 and step1 post_hash == 69d972900b86dfd0. Add a Rust test. NOTE ordering
also matters: after excluding occupied, both sort by (x,y) then actionRng-pick — confirm idx aligns.

## Iter 10 CORRECTION — step2 is a PLAYER-PICK divergence, not move-execution

Read the authoritative JSONL (not just traces): step1 (away3 BLITZ) fully matches on both (post 9c55).
The comparator's "step 2" failure is the PLAYER SELECTION at step2:
- JAVA_ACT_PICK step2 = away5, N=1, idx=0.
- RUST_ACT_PICK step2 = away_10, N=1, idx=0.
Both pick idx=0 of a size-1 "remaining" list, but the list's element differs (Java away5 vs Rust away_10).
Step1 picks matched (both N=3 idx=2 → away3). So entering step2 the boards match but the agent's
eligible/remaining list resolves to a DIFFERENT first player. (My earlier "away5 → 21,8 vs 20,8 move" note
was a RED HERRING — comparing away5's move across different activations/turns, not the step2 cause.)

### Open item #3C (next): eligible/remaining player-list ordering or content at step2
Root-cause why Rust's remaining list at step2 has away_10 where Java's has away5. Per AGENT_CONTRACT §4:
eligible_this_turn = the engine's eligible list snapshot in ROSTER ORDER (not sorted); remaining =
eligible minus used_this_turn (Phase-2 also drops players with empty action lists); pick idx via
decisionRng. The size matches (both N=1) but the content differs, so either (a) the engine's
AgentPrompt::ActivatePlayer eligible_players list is in a different ORDER in Rust vs Java, or (b) the
used_this_turn / already-acted bridging removes different players, or (c) the "empty action list" Phase-2
filter differs. Also investigate why N=1 (only 1 remaining) so early — the eligible snapshot semantics.
Diagnose: FFB_TRACE=1, dump the full eligible_players list (add a trace of the list contents, not just N)
at step2 on both sides; compare to Java ParityRunner's eligible construction + StepInitSelecting. Fix Rust
(engine eligible-list order if it's the engine; agent remaining-logic if it's the agent) to match Java.
Add a test. Verify step2 picks away5 and step2 post matches, then advance. NOTE: correct the iter10 commit's
"move-target occupied squares" lead — that was mis-diagnosed; the occupied-square N=6 trace was away5 in a
LATER turn, not step2.

## Iter 11 (2026-08-01) — FIX #6 (verified, big advance): block-die choice = index 0, 0 rng

step2's player-pick divergence root-caused to a decisionRng DESYNC in step1's blitz: Rust prompts
BlockChoice even for a 1-die block and the agent did `self.pick(dice.len())` — consuming a spurious
decision_rng draw. Java BLOCK_ROLL / BLOCK_ROLL_PROPERTIES = `sendBlockChoice(0)`: index 0, deterministic,
0 rng (AGENT_CONTRACT §7/§8). Same pattern as pushback (#2) & follow-up (#4). FIX (Rust agent): BlockChoice
AND BlockChoiceProperties → die_index 0, no rng draw. cargo test 9 passed.

RESULT (big advance): the extra draw had offset every subsequent decisionRng player-pick; removing it
re-synced the whole away turn. Comparator moved from STEP 2 → STEP 9 (steps 2-8, the away team's entire
first turn, now match). 6 of 6 agent-dialog bugs so far share the pattern "Java deterministic 0-rng vs
Rust random + spurious draw".

### Open item #4 (next): step9 — home team's turn 1, player pick diverges
seed1 step9 (i=10, active=home): Java picks home1 MOVE, Rust picks home_05; step9 pre-state (=step8 post)
differs (Java 09c99d383af1f9f1 vs Rust fdad079a524e32ad). So step8 (last away activation) or the away→home
turn transition diverged. Diagnose: JSONL steps 8-9 chosen/hashes; diff post-step8 full state; check the
turn-handover (end-turn/start-turn) and whether another decisionRng/actionRng desync crept in during the
late away activations, or a home-turn-start difference. Likely another instance of the recurring pattern OR
a turn-transition state delta. Root-cause, fix Rust only, add a test, verify step9 before commit.

## Iter 12 (2026-08-01) — FIX #7 (verified, big advance): re-roll offer = always decline, 0 rng

step9 root cause: away1's dodge failed (target 5, roll 4); Rust USED a team reroll (rerolled=true, roll 2,
fail → stunned at 13,6) but Java DECLINES team rerolls. Rust's ReRollOffer handler did `pick_bool()`
(random use/decline + a decisionRng draw). Java RE_ROLL/RE_ROLL_PROPERTIES = sendUseReRoll(action,null) —
always decline, deterministic, 0 rng, no extra game die (AGENT_CONTRACT §7). Same pattern (#7). FIX (Rust
agent): ReRollOffer → use_reroll:false. cargo test 9 passed.

RESULT: comparator advanced STEP 9 → STEP 23 (now turn 2)! The spurious reroll had both changed injuries
(extra game die) and desynced decisions; removing it re-synced home turn 1 + into turn 2.

### DIALOG-HANDLER AUDIT (found, NOT yet fixed — verify each vs ParityRunner before changing):
Still random-sampling via pick_bool/pick where AGENT_CONTRACT §7 says deterministic:
- SkillUse (§7 "always use") — Rust pick_bool → should be true.
- PilingOn (§7 "always use") — Rust pick_bool → should be true.
- ApothecaryChoice (§7 "decline") — Rust pick_bool → should be false.
- Touchback (§7 "nearest to (13,8) by squared distance, first on ties") — Rust sorts by PlayerId + random
  pick → should be deterministic nearest-center (WRONG choice AND rng).
- ArgueTheCall (§7 "ALWAYS argue") — Rust pick_bool → should be true.
Interception already correct (decline, 0 rng). VERIFY each against the matching ParityRunner case (does Java
draw decisionRng? deterministic value?) before flipping — don't blind-change. These are the likely causes of
step23 and beyond.

### Open item #5 (next): step23 (i=24, turn2, home) player pick diverges (Java home9 vs Rust home_08)
Almost certainly one of the audited dialog handlers fired between step9's fixed point and step23 and desynced
again. Diagnose which dialog occurred (FFB_TRACE LOOP applied= between the last matching step and step23),
map it to the audit list, verify vs ParityRunner, fix, verify step23 matches.

## Iter 13 (2026-08-01) — dialog audit: SkillUse/PilingOn/Apothecary/Argue fixed (verified vs Java); step23 is a DIFFERENT bug

Verified against ParityRunner + fixed (all deterministic 0 rng, were pick_bool): SkillUse → always use
(sendUseSkill(...,true,...)); PilingOn → always use (no Java case; §7 always use); ApothecaryChoice →
decline (sendApothecaryChoice keeps playerStateOld); ArgueTheCall → always argue (ClientCommandArgueTheCall
firstPlayer). cargo test 9 passed. These are correct alignments but did NOT change step23 (none of those
dialogs fire before step23 in seed1) — committed as correctness/no-regression (steps 0-22 still match).

### Open item #6 (next): step23 — prone player NOT skipped in Java but (likely) skipped in Rust
seed1 steps 20-22 match; step23 (turn2, home) diverges on PLAYER PICK: Java picks home1, Rust picks home9.
The step23 eligible list is `[(home_01,[StandUp]), (home_02,[Move]), ...]` — home_01 is PRONE (only StandUp
available). Java picks home_01 (idx 0, stands it up). Rust picks home_09. Hypothesis: Rust's inactive-skip
loop (random_agent.rs ~L167-176: `ps.is_prone() && !ps.is_active()` → skip + the pick already consumed a
decisionRng) WRONGLY skips prone-but-active home_01, then re-picks (extra decisionRng) landing on home9 —
OR home_01's `is_active()` is false in Rust where Java treats a prone (fell, not stun-recovered) player as
activatable (StandUp). A player is inactive ONLY if just-recovered-from-STUNNED this turn; a normally-prone
player CAN stand up. DIAGNOSE: FFB_TRACE ELIGIBLE_CHECK for home_01 at step23 (active flag?), and whether
Rust enters the is_inactive skip for it; compare to Java's eligibility (StepInitSelecting / ParityRunner
usedThisTurn). Java ground truth: prone home1 is pickable (StandUp). Fix Rust's active-flag or inactive-skip
so prone-but-active players are NOT skipped; verify step23 picks home1 and matches. This is a NEW bug class
(eligibility/active-flag), distinct from the dialog 0-rng pattern.

## Iter 14 (2026-08-01) — step23 is a decisionRng DESYNC (NOT a prone-skip); source between turn-2 picks #2 and #3

Corrected iter13's hypothesis: home_01 is PRONE but active=TRUE in Rust (ELIGIBLE_CHECK h=1 t=2:
prone=true active=true), so it is NOT skipped by the is_inactive loop. Authoritative JSONL turn-2 home pick
order:
- JAVA: home7, home3, home1, home9, home8, home4, home5, home11, home2, home10
- RUST: home7, home3, home9, home8, home4, home5, home11, home2, home10, home6
Picks #1-2 (home7, home3) MATCH; pick #3 diverges — Java home1 (idx 0 of the 9-remaining), Rust home9
(idx 6). Same remaining list/size, different decisionRng value → a decisionRng desync that occurred
BETWEEN turn-2 pick #2 (home3, step22) and pick #3 (step23). Rust's order is Java's with home1 displaced to
last — a permutation shift consistent with one extra/missing decisionRng draw around step22.

Puzzle: step22 (home3 MOVE) shows NO decisionRng-consuming dialog in the trace (player pick=matched pick#2;
move-square=actionRng; 2nd prompt=EndPlayerAction 0 rng). So a hidden decisionRng draw (or a missing one)
happens during home3's activation or the step22→23 transition. Note home3 was activated in turn 2 with
action MOVE; but the step23 eligible list shows home_01 with action [StandUp] (prone) — check whether a
prone player's StandUp/Move activation path, or home3's move (dodge? carrier?), draws decisionRng
asymmetrically.

### Open item #7 (next): add a decisionRng counter and localize the desync
Add `decision_rng_count` to RandomAgent (mirror the existing `action_rng_count`): increment in pick()/
pick_bool(), print in RUST_ACT_PICK (like arc). Have ParityRunner print its `decisionRngAdvances` at each
JAVA_ACT_PICK (it already tracks the field — add to the DEBUG line; ParityRunner is co-editable harness).
Rerun seed1; compare decision-count at each turn-2 pick. The pick where the counts first differ localizes
the extra/missing draw → root-cause that specific site (likely a dialog or the prone-StandUp path drawing
decisionRng where Java doesn't, per the recurring pattern). Fix Rust (0-rng align), verify step23 picks
home1, commit. (8 fixes so far; seed1 matches through step22.)

## Iter 15 (2026-08-01) — INSTRUMENTED decisionRng counter; step23 localized to engine active-bit bug

Added `decision_rng_count` to RandomAgent (counts pick/pick_bool/KickBall draws), printed as `drc=` in
RUST_ACT_PICK, plus a DRC_DRAW per-draw trace; mirrored `decisionRngAdvances` into ParityRunner's
JAVA_ACT_PICK (drc=) and rebuilt the jar. (Java change stays uncommitted in the ffb repo per precedent;
Rust instrumentation committed — a keeper for all future desync hunts.)

LOCALIZED: turn-2 picks — Java drc 26(home3)→27(home1); Rust drc 26(home3)→28(home9), i.e. Rust draws +1.
DRC_DRAW shows the pick loop after home3: pick(len=9) [n=27] then pick(len=8) [n=28] — the FIRST pick
(idx0 = home_01) is REJECTED as inactive, then home9 is picked. Java picks home_01 (draw 27) with NO
rejection. Java's reject condition (ParityRunner L371) is `!pickedState.isActive()`; Rust's is
`ps.is_prone() && !ps.is_active()`. So the real divergence: **home_01.is_active() == false in Rust but
true in Java** at step23. home_01 is PRONE at turn 2 (fell in turn 1). A player that fell KD is
prone+ACTIVE (can stand up); only a just-recovered-from-STUNNED player is prone+INACTIVE. Rust wrongly
marks home_01 inactive.

### Open item #8 (next): why is home_01.is_active()==false at step23 (Rust engine turn-processing)
ELIGIBLE_CHECK at turn-2 BUILD showed home_01 active=true, but by the step23 PICK it's inactive → the
active bit changed DURING turn 2 (after home7/home3 activated), OR the turn-start stun-recovery wrongly
marked it inactive. Investigate the Rust engine: (a) start-of-turn player refresh / stun-recovery
(STUNNED→PRONE sets inactive only for THIS turn) — did home_01 get stunned (not just KD) in turn 1, and is
Rust mis-classifying KD vs stunned? (b) does activating another player (home3) wrongly clear home_01's
active bit? Compare Rust's turn-start/active-bit logic (refreshPlayersForTurnStart / StepEndTurn start_half
/ engine turn handover) to Java (ffb-server engine — READ-ONLY ground truth; fix RUST only). Determine
home_01's turn-1 fate (KD vs stunned) from the events log. Fix Rust so home_01 is active at step23 (matching
Java); verify step23 picks home1 and matches. This is an ENGINE bug (active-bit), distinct from the agent
0-rng pattern.

## Iter 16 (2026-08-01) — step23 root cause narrowed: turn-start fails to reactivate a KD-prone player

Facts (from events + trace): home_01 fell KD in turn 1 (playerFellDown at (11,8); armor_roll [4,3]=7,
armor NOT broken, injury_roll null, was_ko/was_cas false) → a knocked-down, uninjured player. When a player
falls, Rust sets change_active(false) (see step_wrestle / action/common/mod.rs / injury paths). At its
team's NEXT turn (turn 2), a KD-prone player must be REACTIVATED (active=true) so it can stand up — Java
does this and activates home_01 at step23. Rust's pick loop reads field_model.is_active(home_01)==FALSE and
REJECTS it (is_prone && !is_active), drawing an extra decisionRng → picks home9 → desync. The active bit is
NOT part of state_hash (steps 1-22 matched despite this), so the divergence only surfaces via the pick.

HYPOTHESIS (precise): Rust's start-of-turn processing does NOT reset fallen (KD-prone) players' active bit
to true in field_model. (The eligible-list build's ELIGIBLE_CHECK printed active=true — that may be a
computed/snapshot value that does NOT reflect the persisted field_model bit the pick loop reads; OR the
reset runs for standing players only.) Java's turn-start (refreshPlayersForTurnStart / StepEndTurn
start-turn) sets all non-stunned players active, including prone KD ones; a JUST-recovered-from-STUNNED
player is the only prone player left inactive that turn.

### Open item #8 (next, engine): reactivate KD-prone players at turn start (Rust engine)
Find the Rust start-of-turn/turn-handover step (StepEndTurn → start next turn; where turns_played
increments, where STUNNED→PRONE recovery happens) and confirm whether it sets active=true for prone
non-stunned players in field_model. Compare to Java ffb-server (StepEndTurn / UtilServerGame /
refreshPlayersForTurnStart — READ-ONLY). Fix RUST engine so a KD-prone player is active at its team's turn
start (and a just-unstunned player is NOT). Add a Rust test (fall a player, advance to its next turn,
assert is_active()==true; stun a player, assert its recovery turn is_active()==false). Verify home_01's
field_model is_active()==true at step23 (add a temp trace of home_01.is_active at the pick), step23 picks
home1, and matches. NO hacks. (8 agent fixes + drc instrumentation committed; seed1 matches through step22.)

## Iter 17 (2026-08-01) — FIX #9 (verified): live StepEndTurn now calls refresh_players_for_turn_start

ROOT CAUSE nailed: the LIVE driver path (bb2025 StepEndTurn) flips home_playing/turn_nr at a turn
transition but NEVER called UtilPlayer::refresh_players_for_turn_start — only the DELETED monolithic
engine.rs:942 did. Java calls it at StepEndTurn:358. So a knocked-down (PRONE, non-stunned) player was
never reactivated (active bit left false from the fall's change_active(false)); the active bit is NOT in
the state_hash, so steps 1-22 matched anyway, and it only surfaced at step23 when the agent's pick loop
rejected the (wrongly-inactive) prone home_01. FIX (Rust engine, bb2025/step_end_turn.rs): after the
Kickoff/Regular flip (turn_mode==Regular && !new_half), call refresh_players_for_turn_start with the
edition mechanic's enhancement sets (mirrors engine.rs:942 + Java StepEndTurn:358). The refresh FUNCTION
itself was already correct + unit-tested (util_player); the bug was the missing call.

VERIFIED (parity): home_01 is now active at turn 2 and ACTIVATED at step23 (matching Java's player choice);
the pick order re-synced — Rust turn-2 picks now home7,home3,home1,home9,home8,... matching Java (were
home7,home3,home9,... skipping home1). Comparator's failing step index is still 23 but the DIVERGENCE
CHANGED (pick desync → resolved; new issue is home_01's StandUp execution). cargo step_end_turn tests pass.
NOTE: bb2016 + bb2020 StepEndTurn have the SAME missing-call bug — apply the same fix there when those
editions are parity-tested (bb2025 is the current target).

### Open item #9 (next): prone-player activation — action label + no state change
seed1 step23 (home_01 activation): Java chosen=Activate(Home1,MOVE) post=90e19713af7cd274; Rust
chosen=Activate(home_01,StandUp) post=661cf20f9f6af1c9 (== pre-state, UNCHANGED). Two problems: (1) ACTION
LABEL — Java activates a prone player with action MOVE (prone MOVE = stand up then move); Rust's eligible
action for a prone player is StandUp (AGENT_CONTRACT §5 says "StandUp for prone" but Java's ParityRunner/
engine uses MOVE — reconcile: what action does Java's eligible list give a prone player, and what does the
Rust agent send?). (2) Rust's StandUp produced NO state change (home_01 stayed prone? post==pre), whereas
Java's stands the player up (prone→standing, 90e1). Investigate the Rust StandUp activation path (does it
actually stand the player up and spend/refresh state?) vs Java (StepInitMoving/standup + move). Also confirm
the agent should pick the SAME action Java does (MOVE vs StandUp) so the `chosen` + post align. Fix Rust,
add a test, verify step23 fully matches (chosen + post 90e1).

## Iter 18 (2026-08-01) — FIX #10 (partial-advance): prone player's eligible action = Move (was StandUp)

legal_activate_player_actions offered a PRONE player [StandUp], but its OWN comment said "Java offers MOVE
for prone", and JAVA_ACT_PICK confirms action=MOVE. Changed the prone eligible action StandUp→Move
(mod.rs). RESULT: step23 `chosen` now matches (both Activate(home_01,Move)) AND home_01's coordinate now
matches Java (both moved to (11,7)). REMAINING (one bit): Java h00=(11,7)**Standing**, Rust=(11,7)**Prone**
— Rust's Move for a prone player moved the coordinate WITHOUT standing the player up (base stayed PRONE). In
BB a prone player activating Move must STAND UP first (base prone→standing, cost 3 MA / a 4+ roll if MA<3)
before moving. Rust's move/stand-up execution skips the stand-up.

### Open item #10 (next): Move-for-prone must stand the player up (Rust engine)
Find the Rust move step (StepMove / StepInitMoving / the move sequence) and make a prone player's first
Move action STAND IT UP (change_base prone→STANDING, current_move += 3 or the MA<3 stand-up roll, coordinate
UNCHANGED on the stand-up), matching Java StepMove/standUp. Currently Rust just changes the coordinate and
leaves base=PRONE. Compare to Java ffb-server StepMove stand-up handling (READ-ONLY). Fix RUST engine, add a
Rust test (activate a prone player with Move → base becomes STANDING, MA spent), verify step23 post ==
90e19713af7cd274. Note the stand-up may also consume game dice (MA<3 stand-up roll) — for MA6 linemen it's
free (3 MA), no dice. Verify the whole game dice stream stays aligned after this.

## Iter 18b (2026-08-01) — step23 down to ONE bit: StepStandUp doesn't set base→STANDING (live driver)

After the prone-eligible-action=Move fix (c5da8bee), step23 chosen + home_01 coordinate (11,7) match Java;
the ONLY remaining diff is home_01's base: Java STANDING, Rust PRONE. So Rust's stand-up execution is broken.
Traced: the live driver runs the select sequence which includes StepStandUp (bb2025 select.rs:51). StepStandUp
(crates/ffb-engine/src/step/bb2025/move_/step_stand_up.rs) execute_step:
- Guard (~L82): early-returns unless `game.acting_player.standing_up` is true. (bb2016 step_init_selecting
  sets standing_up=true for a prone activation at L468/L495 — VERIFY the bb2025/live path also sets it for a
  Move activation of a prone player; if not, StepStandUp is skipped entirely.)
- Free-stand-up branch (L105-110, MA>=3 / canStandUpForFree): sets `has_moved=true; standing_up=false;
  return` — but does NOT set the player's base to PS_STANDING. The DELETED engine.rs did
  (`set_player_state(PS_STANDING); current_move = STAND_UP_COST`), and Java sets STANDING for a successful/
  free stand-up (pin the exact site — Java StepStandUp L47-48 sets PRONE for a sub-case, so STANDING is set
  elsewhere: possibly on setStandingUp(false)+move, or a base-set the Rust free branch omitted).

### Open item #10 (next): make StepStandUp actually stand the player up (base→STANDING) + set current_move
Fix (Rust engine, bb2025 StepStandUp, and bb2016/bb2020 if same): in the free-stand-up branch (and after a
successful roll) set the player's base PRONE→STANDING and current_move += STAND_UP_COST (3), mirroring Java +
the deleted engine.rs. FIRST verify `standing_up` is set true for the bb2025 Move activation of a prone
player (trace: is StepStandUp even reaching the free branch, or early-returning on the guard?). Read Java
bb2025 StepStandUp fully to pin where STANDING is set for a free/successful stand-up. Add a Rust test
(prone MA6 player activates Move → base STANDING, current_move==3, no dice). Rebuild, rerun seeds 1-3, VERIFY
step23 post == 90e19713af7cd274 (fully matches). This is the LAST bit for step23. 10 fixes committed;
seed1 matches through step22; step23 is one base-bit from green.

## Iter 19 (2026-08-01) — step23 stand-up: standing_up flag NOT set on the live prone Move activation

Confirmed (reverted an unreached free-branch STANDING edit): StepStandUp's guard (~L82) early-returns
unless `game.acting_player.standing_up`==true, and for home_01's Move activation standing_up is FALSE — so
StepStandUp is a no-op and the player never leaves PRONE. The `standing_up=true` setters found in
bb2016/move_/step_init_selecting.rs L468/495 are TEST code, not production. In the live driver path a prone
player activated with Move does not get standing_up set, so no stand-up happens. (Build gotcha this iter:
cargo returned 0.25-0.27s "Finished" without recompiling even after `rm` the exe and `cargo clean -p
ffb-engine`; a real recompile only happened when a build actually showed "Compiling ffb-engine". Always
confirm a Compiling line, not just Finished, before trusting a rerun.)

### Open item #10 (next, TWO-PART engine fix): make a prone Move activation stand the player up
Java: activating a prone player with MOVE sets it standing-up (StepInitSelecting sets currentMove =
min(MINIMUM_MOVE_TO_STAND_UP, MA) and the standing-up flow), then StepStandUp resolves the stand-up (free if
MA>=3), ending STANDING. Rust must mirror:
(1) In the LIVE activation path (the production StepInitSelecting the driver uses via
    `step_init_selecting::StepInitSelecting`, driver.rs:104 — FIND the actual production file, NOT the test
    module), set `game.acting_player.standing_up = true` (and current_move = min(STAND_UP_COST, MA)) when a
    PRONE player is activated (Move/Blitz), matching Java StepInitSelecting. Verify via FFB_TRACE that
    standing_up is true entering StepStandUp for home_01.
(2) In StepStandUp free-stand-up branch (and successful-roll branch), set the player base PRONE→STANDING
    (`ps.change_base(PS_STANDING)`), mirroring the deleted engine.rs:1031-1036 (set_player_state(PS_STANDING)
    + current_move = STAND_UP_COST). The Java StepStandUp free branch only sets hasMoved(true) — Java's
    STANDING comes from the InitSelecting/standing-up flow, so setting it in StepStandUp is the pragmatic
    equivalent; VERIFY the post_hash == 90e19713af7cd274 and REVERT if it over/under-sets.
Add a driver-level test: prone MA6 player activated with Move → ends STANDING, current_move==3, 0 dice.
VERIFY step23 fully matches + comparator advances PAST step23. 10 fixes committed; seed1 through step22.

## Iters 20-23 (2026-08-01) — step23 stand-up SOLVED + half-2 kickoff + touchback + prone-blitz

Seeds 1 and 2 now FULLY match Java per-step. Four root-caused engine/agent fixes:

- **Iter 20 (7cf718e4) — set_player full field reset + same-id guard.** The step23 stand-up bug's real
  cause: `ActingPlayer::set_player` reset only a subset of Java `setPlayerId`'s fields, OMITTING
  `has_moved` (and dodging/has_blocked/fouled/passed/fed). A prone player re-activated after moving earlier
  carried a stale `has_moved=true` into StepStandUp, whose Java guard `isStandingUp() && !hasMoved()` then
  skipped the free stand-up → player stayed Prone. Fix: set_player now mirrors setPlayerId's full reset and
  adds Java's same-id early-return (so Move→Block on one blitzer keeps has_moved). StepStandUp free branch
  also sets base PRONE→STANDING. `standing_up` itself is set in `change_player_action` = (old base==PRONE),
  mirroring Java UtilActingPlayer. Seed 1: step 23 → 156.

- **Iter 21 (1e8a8f49) — away kickoff coord pre-transform.** Java ParityRunner sends the kick target as
  `home ? kickCoord : kickCoord.transform()`; StepKickoff transforms again for the away side, netting the
  server-frame target. The Rust agent omitted the away pre-transform, so StepKickoff mirrored (6,8)→(19,8),
  landing the half-2 kick in the kicking half → spurious touchback + diverged kickoff. Seed 1 fully matches.

- **Iter 22 (ab26e19e) — touchback nearest-to-(13,8), 0 rng.** Java ParityRunner TOUCHBACK picks the
  receiving player nearest to fixed kick-from (13,8) by squared distance (team order breaks ties), NO
  decisionRng draw. Rust random-picked a PlayerId-sorted candidate, consuming a spurious decision_rng draw
  that desynced every later pick in the half. Seed 2: step 157 → 233.

- **Iter 23 (d900548d) — prone blitz offers BLITZ not STAND_UP_BLITZ.** Java computeEligiblePlayers offers a
  prone player [MOVE, BLITZ] (BLITZ dispatched as BLITZ_MOVE; stand-up via the standing_up flow). Rust
  offered StandUpBlitz → different step sequence, diverged post-state (seed 2 step 233). Updated 2 stale
  legal_actions tests. Seeds 1-2: 2/2 match.

Next: seed 3 diverges early at step 6 (turn 1 half 1) — a distinct seed-3 divergence under investigation.

## Iter 24 (2026-08-02) — prone player always runs Select StandUp (fbb90cd7)

Seed 3 advanced step 6 → 131. Root cause: InitSelecting force-gotos to END_SELECTING for
Block/Blitz (force_goto_on_dispatch), skipping the Select sequence's StandUp. Java skips it there
too but runs StandUp inside SelectBlitzTarget (dispatched by BLITZ_SELECT); Rust bypasses
SelectBlitzTarget entirely (dispatches Blitz straight to BlitzBlock, auto-picking the adjacent
target in InitBlocking), so a prone blitzer with an ADJACENT target (no move → the move-path
stand-up never fires) blocked while prone → attacker ended Prone vs Java Standing (seed 3 step 6).
Fix: `if standing_up { next() }` (was `standing_up && !force_goto_on_dispatch`) — a prone player
always runs StandUp; force_goto only skips it for an already-standing player. +2 unit tests. Seeds
1-2 still 2/2.

### Next (seed 3, first divergence i=131): blitz TARGET SELECTION diverges
away_02 (PRONE at (13,6)) blitzes. Both pre-states match (2b15571f). Rust auto-picks the adjacent
home_01@(12,7) and blocks directly: 1 die [2]=BOTH DOWN → away_02 (lineman, no Block) falls →
TURNOVER (Rust jumps to home turn 2). Java instead stands away_02 up and DODGES/moves
(dice pos75 = StepMoveDodge, pos76 = dodge-drop injury) — Java rolls 1 MORE die (74 vs Rust 75 at
i=132; note stream already diverged) and stays in away turn 1 (away_07 acts next). So Java's blitz
target/path differs from Rust's adjacent auto-pick. ROOT to check: Rust bypasses SelectBlitzTarget,
auto-selecting the adjacent opponent in StepInitBlocking, whereas Java's ParityRunner SELECTS the
blitz target (possibly a non-adjacent one, requiring a move+dodge) via the SelectBlitzTarget dialog.
Compare the Rust parity agent's blitz-target pick (RUST_BLOCK_PICK, arc=) against Java ParityRunner
SELECT_BLITZ_TARGET case + computeBlitzTargets; align the target-selection so both pick the same
defender and the same move path. This likely needs Rust to route blitzes through SelectBlitzTarget
(agent target choice) rather than auto-picking in InitBlocking — verify it doesn't regress seeds 1-2.

## Iter 25 (2026-08-02) — bb2025 additional block assist (Cheering Fans) — seeds 1-4 green (fbbc7714)

Seed 3 i=131 turnover traced to a +1 game-die rng offset originating at seed 3 i=129 (home_01 BLOCK:
Java 2 dice, Rust 1). Deep dive (findBlockStrength gave 3,3 in both, but Java's findNrOfBlockDice=2):
the bb2025 RollMechanic.getTotalAttackerStrength adds `gameState.getAdditionalAssist(actingTeam)` — the
Cheering Fans kickoff event grants a team +1 offensive block assist. Rust's handle_cheering_fans set
game.home/away_additional_assists but the block-dice calc never added it → att 3 not 4 → 1 die not 2.
FIX 1 (StepBlockRoll bb2025): add acting-team additional_assists to attacker_str before block_dice_count.
That alone REGRESSED seed 1 (its assist leaked into a later turn) → FIX 2 (StepEndTurn bb2025): mirror
Java removeAdditionalAssist(actingTeam) at end of a REGULAR/BLITZ turn. Seeds 1-4: 4/4 match. +1 test.
(Method: temporarily instrumented the co-editable ParityRunner harness with ServerUtilBlock/ServerUtilPlayer
calls to read Java's engine strengths — reverted + jar rebuilt clean afterward.)

Next: seed 5 first divergence i=159 (turn 1 half 2, home BLITZ) — investigate.

## Iter 26 (2026-08-02) — bb2025 knocked-down ball carrier (attacker) drops the ball (9ff8c5d8)

Seed 5 i=159: away_03 (the BALL CARRIER at (13,8)) blitzes home_03, Both Down → away_03 falls. Java
bounces the dropped ball to (14,9) via StepCatchScatterThrowIn (extra d8 at pos54); Rust left the ball at
(13,8) — the only i=160 state diff, and the +1 game-die rng offset that cascaded to the i=159/i=160 hash
mismatch. ROOT: Java bb2025 StepDropFallingPlayers adds a deferred DropPlayerCommand(attacker, ATTACKER,
true) in the normal Both-Down attacker-fall branch, which scatters the ball. Rust left the DeferredCommand
mechanism unported AND gated the direct drop_player(attacker) on `piling_on_supported` (bb2016/bb2020
only) → in bb2025 a ball-carrying attacker knocked down on Both Down never dropped the ball. FIX: in the
non-saboteur attacker-fall branch, for `!piling_on_supported` (bb2025), apply drop_player(attacker, true)
— converts FALLING→PRONE and sets the ball scattering (bounce d8 rolls later in StepCatchScatterThrowIn).
Seeds 1-6: 6/6 match. +1 test. (Diagnosis: first rng_calls divergence → prior step's dice COUNT; Java
DICE_TRACE caller=StepCatchScatterThrowIn.bounceBall pinpointed the missing bounce.)

Next: seed 7 first divergence i=39/40 (turn 3 half 1, turn-structure desync — Java home vs Rust away active).

## Iter 27 (2026-08-02) — agent caches turn-start eligibility; no-target blitz ends turn prone (62528277)

Seed 7 i=39: Java Activate(away_01,BLITZ) vs Rust Activate(away_01,Move) — same board. away_01 (prone at
(13,6)) was blitz-eligible at TURN START (home_02 adjacent-standing at (12,6)); away_02's Block at i=37
knocked home_02 prone. Java ParityRunner computes eligibleThisTurn ONCE per turn (keeps the cached BLITZ);
the Rust agent used the engine's LIVE per-activation list (recomputed to [Move] once home_02 fell). Three
linked fixes: (1) RandomAgent snapshots the eligible player->actions list at turn start (eligible_this_turn),
picking from it (targets stay live); (2) StepInitBlocking: a blitz with no defender ends the turn (Java
BLITZ_TARGET_NONE -> EndTurn) instead of waiting forever; (3) StepInitSelecting: a prone Blitz with NO
target suppresses the stand-up (Java resolves the target BEFORE standing up, so a null-target blitz ends
the turn PRONE). Seeds 1-7: 7/7. Updated 1 test + added the no-target suppression test.

Next: seed 8 first divergence i=9 (away_03 BLOCK) — Rust rolls 1 MORE game-die than Java (Rrng 15->20 vs
Jrng 15->19). A block-resolution divergence (extra Rust roll); use FFB_DICE_TRACE Java caller= to pinpoint.
