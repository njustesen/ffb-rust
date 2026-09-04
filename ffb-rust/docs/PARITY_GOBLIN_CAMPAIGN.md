# Goblin — heuristic-agent parity campaign

**Goal**: goblin v goblin, HeuristicAgent both sides, per-step state-hash parity 100/100,
seeds 1-100, tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus
random controls and the standing regression set (now amazon, lineman, dwarf, chaos family,
dark elves, elf). Procedure: `.claude/commands/amz-iter.md` with MATCHUP=goblin. Started
2026-09-02 after elf closed (1f5e7effc).

## Surface

Roster in ALL THREE editions. The heaviest skill surface of the sweep so far:

- bb2016: Goblin ×16 (Dodge/Right Stuff/Stunty/Thick Skull/Animosity/Regen), Troll ×2
  (Always Hungry/Loner/MB/Really Stupid/Regen/TTM), Bombardier, Pogoer (Leap/VLL),
  Looney (Chainsaw), Fanatic (Ball & Chain) — three secret weapons.
- bb2020: + 'Ooligan (DP1/Disturbing Presence), Doom Diver (Swoop), Trained Troll
  (Loner 3/MB1/Projectile Vomit/TTM), Pogo Stick.
- bb2025: as bb2020 with No Ball on Looney/Fanatic, Taunt on 'Ooligan, Pogo, trolls
  without Loner.

Prior art: goblin is GREEN under the RANDOM agent in all editions (TTM/KTM/chainsaw/B&C/
secret-weapon campaigns). Expect heuristic-contract gaps of the elf class: dialogs the
random contract answers for free (SKILL_USE auto-uses) and windows the heuristic scores.

## Baseline

(to be measured)
bb2016: @1.0 0/100, @0 0/100, @1e6 2/100. bb2020: 0/0/0. bb2025: heavy reds (see below).
Randoms ×3: 100/100. RED AT ARGMAX ⇒ not a draw-count split alone: the agents disagree in
candidate content/eligibility — divergent rows repeatedly show TTM / THROW_BOMB / FOUL_MOVE
on one side only. Suspects: TTM eligibility/scoring, Bombardier ThrowBomb scoring, Ball&Chain
moves, secret-weapon effects on activation sets, Really Stupid/Animosity gating.

## ITER1 — ThrowBomb/AllYouCanEat belong in the DEFAULT immediate arm, not the pass arm

bb2016 @0 seed 1: candsum k=1 (the game's FIRST candidate list) R n=1959 vs J n=1890 with 0
draws. Candidate diff (after normalising ThrowBomb/THROW_BOMB spellings): the only real gap is
home5's bomb — Rust built ONE ROW PER receiver×run-up spot, all weight 0 via `pass_weight`
(no ball), while Java has ONE `THROW_BOMB` IMMEDIATE row at wPlayer*max(0.40,floor)+novelty
(≈0.368). Java's ActivationChoice switch routes only "Pass"/"HailMaryPass" to passCandidates;
"ThrowBomb"/"AllYouCanEat" fall to `default:`. Consequence: Rust's Bombardier NEVER threw a
bomb (w=0 rows can't win argmax) while Java's does — the i=12 THROW_BOMB divergence, plus a
9-row list-size skew shifting every later index. Same fault class as the chaos_pact TTM
phantom-arm lesson recorded at the Rust default arm.

Fix: drop ThrowBomb|AllYouCanEat from the Rust pass-generation arm — they fall through to the
default immediate push(0.40) like TTM/KTM; the phase-2 unconditional `fold_pass_receiver`
override still supplies the declaration target.
Post-ITER1 matrix: bb2016 61/95/79, bb2020 17/84/15, bb2025 0/69/31→(1e6) 3. Randoms ×3
green, suite 7392/0. Commit 174efc9b7. Remaining reds: @0 frontiers small (bb2016 5 seeds:
29/51/71/81/85; bb2020 16; bb2025 31), sampled scales large ⇒ at least one draw-count split
plus possibly more eligibility gaps. ITER2 target: bb2016 @0 seed 29 (step 60).

## ITER3 — a falling Ball & Chain player must roll InjuryTypeBallAndChain (bb2016 StepFallDown)

bb2016 @0 seed 29: hashes diverge at idx 60, but the candidate diff at the first count
mismatch (k=63) shows a fully-diverged board — downstream noise (the dicediff lesson).
The real split: rng_calls first differ at i=59 (R 87 vs J 89), states still EQUAL.
FFB_DICE_TRACE stacks name the two extra Java dice: during i=58 the crowd-pushed FANATIC's
StepFallDown → UtilServerInjury.dropPlayer → `placedProneCausesInjuryRoll` (Ball & Chain) →
InjuryTypeBallAndChain 2d6 (Java dice 88,89). Rust's bb2016 step_fall_down called the
RNG-LESS `drop_player_no_sph`, whose B&C branch silently skips the roll (`if let Some(rng)`).
Every die after shifted (the next Really Stupid roll read 1 vs 3) — states coincidentally
matched for two more activations, hiding the split from the state hash.

Fix: `drop_player_rng(game, rng, &player_id, false, ApothecaryMode::Attacker)` — the same
rng-aware variant the Pitch-Invasion stun fix introduced (random campaign ITER78-81).
LATENT: other rng-less `drop_player_no_sph` callers (step_stab, drop_diving_tackler ×2,
right_stuff/bb2025 right_stuff_command) have the same skip; unreachable for a B&C player in
mirror matches (a Fanatic can't be TTM'd, dive-tackle, and mirror goblins have no Stab) —
noted for the mixed-matchup era.

## ITER4 — a bomb turnover belongs to the INTERRUPTED team, not the momentary acting team

bb2016 @0 seed 85 (step 80): identical dice through the whole bomb hot-potato (home_05 throws,
away_02 intercepts and re-throws, explosion drops an AWAY ball carrier — die-for-die equal to
190), then Rust ends HOME's turn (t8 away) while Java's home continues. Probing the LIVE
EndBomb (the bb2016 step_end_bomb.rs is a DEAD FILE — driver routes all editions to
mixed/special; the probe-dead-file lesson re-confirmed) showed Rust arriving with
end_turn=TRUE. Publisher: `drop_player`'s ball-drop turnover used `active_team()` — during a
re-thrown bomb home_playing is flipped to the re-thrower, so an AWAY carrier down looked like
an acting-team turnover. Java's dropPlayer switches on TURN MODE: BOMB_HOME → a HOME carrier
is the turnover, BOMB_AWAY → AWAY, PASS_BLOCK → never, default → acting team; and the whole
ball-scatter block is gated `turnMode != BLITZ`. Ported 1:1 (util_server_injury.rs).

## ITER5 — bb2016 chainsaw block AND foul: dead twins (driver routing)

bb2016 @0 seed 51 (i=136): the Looney (home_14) declares BLOCK — Java rolls
rollChainsaw + armour/injury 2d6s; Rust rolled TWO BLOCK DICE. The FFB_BOMB probe in the
bb2016 StepBlockChainsaw printed ZERO times: the generator schedules the step, but the
driver's bb2016 override table had no BlockChainsaw arm, so the SHARED bb2020 twin ran —
and it gates on the UsingChainsaw step parameter, which only bb2020/25 InitBlocking
publishes. In bb2016 the flag stays false → silent NEXT_STEP → normal block. Java bb2016
has no usingChainsaw: the chainsaw is MANDATORY when the attacker has blocksLikeChainsaw.
Routed bb2016 to its own translated-but-never-dispatched twin (dead-twin fault pattern,
third instance this sweep). Java bb2016 StepFoulChainsaw has the same property-only gate
vs the mixed twin's usingChainsaw gate — routed the bb2016 FoulChainsaw twin in the same
change (unit-port rule: one mechanism, one change set).

## ITER6 — an out-of-bounds thrown player stays FALLING until the apothecary applies (bb2016)

bb2016 @0 seeds 71+81: a TTM throw scatters out of bounds; both engines roll the crowd
injury 2d6, then Rust rolls a phantom LANDING d6 (9 dice vs Java's 8). The bb2016
InitScatterPlayer OOB branch set FALLING then immediately `apply_to`'d the injury — a KO'd
player went to the box, so StepRightStuff's FALLING check missed and the landing roll fired
for a player in the crowd. Java only rolls (the THROWN_PLAYER apothecary applies later); the
player stays FALLING and RightStuff skips. Removed the early apply — same class as the
ITER78-81 random-campaign "TTM landing drop-before-apply" fix. NOTE: the hit-player branch's
`apply_to` is the same suspect class (hash-blind, unexposed) — left for evidence.

## ITER7 — the bb2016 chainsaw kickback roll is a D6, not a D8

bb2016 @0 seed 24 (i=151): with the chainsaw twin now LIVE (ITER5), the Looney's block rolls
the kickback — Java `DiceRoller.rollChainsaw()` = rollDice(6) → d6=1 → KICKBACK, attacker
down, turnover; Rust rolled a d8 (=5 on the same stream position) → "hit". The Rust twin's
own comment claimed "(rolls d8)". One-character fix; the bb2016 FOUL chainsaw twin already
rolled d6.

## ITER8 — the bb2016 chainsaw kickback happens only on a 1 (minimum roll 2, not 4)

bb2016 @0 seed 51 (i=136), post-ITER7: identical chainsaw d6=3 — Java (DiceInterpreter
minimumRollChainsaw() = 2) HITS away_04 (armour+injury on the DEFENDER, no turnover, home
continues); Rust's hardcoded minimum 4 turned the 3 into a phantom KICKBACK (armour+injury on
the Looney, drop, END_TURN). The four dice were identical either way — the JSTATE final row
(A4:4 stunned, H14 standing) named the real target. The bb2016 FOUL chainsaw twin already
used 2.

## ITER9 — ThrowBomb is gated on bombUsed (never the pass slot)

bb2016 @1.0 seed 2: candsum first mismatch k=58, n off by ONE with equal draws — the classic
availability split. RELIG/JELIG diff: after the team's PASS (pass=true), Rust's
`legal_activate_player_actions` dropped the Bombardier's ThrowBomb (`!turn_data.pass_used`
gate); Java's ParityRunner gates THROW_BOMB on `!td.isBombUsed()` + the enableThrowBombAction
PROPERTY and kept offering it. bombUsed is only ever set by bb2020/bb2025 StepInitBomb — in
bb2016 the bomb stays available all game. Fixed the gate + property check; test updated to
pin "a used pass slot must NOT withdraw ThrowBomb".

## ITER10 — the bomber's markSkillUsed must hit the ACTING PLAYER's set (Estelle family)

bb2016 @1.0 seed 21: candsum k=55 equal n, Rust +2 draws — but the true origin is k=2: after
away5's ThrowBomb (identical dice incl. a catch-and-re-throw by away_07), JELIG drops Away5
while RELIG keeps all 11. Java's BombardierBehaviour calls actingPlayer.markSkillUsed
(Bombardier) — a term of derived hasActed() — so when EndBomb hands the acting slot to the
bomb CATCHER, changeActingPlayer retires the acted bomber. Rust's step_bombardier wrote only
the team Player's used_skills; hasActed() stayed false and the bomber remained eligible all
turn, skewing every later activation list. Fix: `mark_skill_used` (writes both sets).
ITER11 (with ITER10): the retire carve-out must read the ACTING PLAYER's used set — Java is
`UtilCards.hasUnusedSkillWithProperty(actingPlayer, enableThrowBombAction)`; Rust read the
team Player's set, which mark_skill_used no longer writes for a per-activation skill. Both
retire sites (change + to_none) fixed. @0 1-20 green; @1.0 reds moved later (21: 1→51,
81: 9→53) — further faults behind.

## ITER12 — a team re-roll during a bomb belongs to the bomb-OWNING side

bb2016 @1.0 seeds 39/21: equal dice, Rust 2 sampler draws ahead — Rust offered (and the agent
declined) a PASS team re-roll for the INACCURATE re-throw of an intercepted bomb. Java's
RollMechanic.isTeamReRollAvailable carries four bomb-mode terms — during BOMB_HOME/AWAY(_BLITZ)
the roller must be on the bomb-owning team — so the away re-thrower of home's bomb gets no
offer. Ported the four terms into ask_for_reroll_if_available_for.
Post-ITER12: **bb2016 100/100 × 3 GREEN** 🎉 (from 0/0/2 at baseline, 12 iterations).
bb2020 19/85/15, bb2025 0/71/3 — edition-specific faults next. Suite 7392/0 after
updating marks_bombardier_skill_used to the acting-set contract.

## ITER13 — the parity game enables ALLOW_BALL_AND_CHAIN_RE_ROLL (bb2020/25 Fanatic)

bb2020 @0 seed 6 (i=103): identical dice through the Fanatic's move, then Java rolls THREE
directions + BlockProne armour (5 dice, r3→r2) vs Rust's two + a block. A temporary stock
JBCMOVE probe showed Java's second roll REPEATING the same from/orig — a TEAM RE-ROLL of the
scatter direction. Java's mixed StepMoveBallAndChain offers that re-roll only when
ALLOW_BALL_AND_CHAIN_RE_ROLL is enabled, and UtilServerStartGame.addDefaultGameOptions
(STANDALONE — the harness mode) sets it TRUE for every parity game; Rust's
BASELINE_SETUP_OPTIONS (the Rust mirror of that table) lacked it, so the offer never fired.
Added the option (same class as the mbStacksAgainstChainsaw baseline fix, random campaign).

## ITER14 — the B&C COORDINATE_TO publish must run OUTSIDE `if (doRoll)` (dormant, exposed by ITER13)

bb2020 @0 seed 6 (i=103): the Fanatic's multi-square compulsory walk forked — StepMove applied
InitMoving's ORIGINAL planned square (10,6) instead of the B&C-scattered (10,7). Probes
(RSTEPMOVE/RMOVE_SETTO) proved StepMove NEVER received the B&C's CoordinateTo republish. Root
cause: in Java's mixed StepMoveBallAndChain the in-bounds check, `publishParameter(COORDINATE_TO)`
and the blocker check are OUTSIDE the `if (doRoll)` block; Rust had them INSIDE. With ITER13
enabling ALLOW_BALL_AND_CHAIN_RE_ROLL, every scatter now OFFERS a re-roll — on decline the step
re-enters with playerScatter set and doRoll=false, and Rust returned NEXT_STEP with no
COORDINATE_TO, so the following StepMove kept the stale planned square. Restructured: the
publish/blocker block now runs unconditionally from self.coordinate_to, matching Java. Latent
before ITER13 (no re-roll offer → no doRoll=false re-entry for a scatter).

## Matrix checkpoint after ITER14 (2026-09-02, pushed 3bc264ac8)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** ✅ |
| bb2020 | 67 | 99 | 86 |
| bb2025 | 94 | 93 | 99 |

From baseline (0/0/2, 17/85/15, 0/69/3). bb2016 nine-of-three GREEN. Remaining:
- bb2020 @0 seed 92: a B&C Fanatic knocked down mid-walk — Java scatters ONE more square
  (rng 221 throwIn) → OOB crowd-push (222/223) → B&C injury (224/225); Rust stops after its
  casualty. The B&C compulsory walk must CONTINUE after the mover is knocked down/injured.
- bb2020 @1.0 33 reds / @1e6 14; bb2025 @1.0 6 / @0 7 / @1e6 1 — mix of the same B&C-walk
  nuance and draw-count splits to root-cause per seed.

## Remaining frontier — the B&C compulsory-walk-off-the-edge case (NOT yet fixed)

Extensive tracing (bb2020 @0 seed 92, bb2025 @0 seed 7) localized the dominant remaining fault
to a Ball & Chain Fanatic's compulsory random walk in a near-empty blizzard pitch. Precise
findings (probes reverted):

- The Fanatic scatters off the pitch edge repeatedly; each OOB is a crowd push. The mover is
  put into the RESERVE box (RSV_HOME_X=-1, first free row) and the walk CONTINUES scattering
  from the box coordinate.
- bb2025 seed 7 i=56 (home_04): the board is BIT-IDENTICAL at i=55 (verified: same reserve
  occupancy, both empty). During home_04's move, Rust boxes ONLY home_04 (row 0 → (-1,0));
  Java boxes home_04 to row 3 → (-1,3), implying 3 other home players were crowd-pushed to
  reserve first. Square 3's scatter runs FROM the box coordinate: Java from (-1,3) lands on a
  teammate at (0,2) and BLOCKS (rng 138-140 = 3 block dice) → turnover, home's turn ends;
  Rust from (-1,0) scatters to (0,-1) OOB → casualty, turn continues. This is the i=57
  `J=THROW_BOMB / R=Move` divergence (Java flipped to away, Rust continued home).
- Root not fully cracked: why Java reserves 3 more players than Rust during the SAME move with
  the SAME dice, when the board matched at i=55 and Rust boxes only the mover. Likely a
  B&C-blocks-teammate → crowd-push interaction (the mover blocking/pushing own players off the
  edge) that Rust resolves differently — plus goblin's Troll/secret-weapons are nr>11 and
  HASH-BLIND, so an earlier box-occupancy divergence in those could go uncaught.
- Constants and per-step logic verified 1:1 (RSV/KO columns, putPlayerIntoBox row scan,
  InjuryTypeCrowd's `!casualty && !KO → RESERVE` guard, StepFallDown defer-not-apply). The gap
  is in the multi-square walk's block/crowd-push resolution, needing dedicated porting.

Status: bb2016 100/100 x3 GREEN + pushed. bb2020 67/99/86, bb2025 94/93/99 — the remaining
reds are this B&C case + sampled draw splits. Not yet fully green.

## ITER15 — root cause of the B&C-Fanatic divergence found; fix REVERTED (exposes an agent-parity bug) (2026-09-03)

**Outcome: DIAGNOSIS committed (this entry) + a new `FFB_IDSTATE` full-board trace tool; the
engine fix itself is REVERTED** because although it is Java-faithful it regresses a closed roster.
The root cause below is correct and re-appliable once the agent-parity bug it exposes is fixed.

**Root cause of the dominant B&C-Fanatic divergence, finally cracked.** The ITER14 "Remaining
frontier" section guessed "3 other home players were crowd-pushed to reserve first" — wrong. The
3 were the team's **un-fielded reserves** (goblin drafts 14 players; 11 are set up, nr 12-14 sit
out). Java's headless parity harness runs `HeadlessGameSetup.addTeamToGame` →
`GameCache.addTeamToGame`, which sets EVERY player to RESERVE (or MISSING) and calls
`putPlayerIntoBox` at team-join, before the game starts. The Rust **synchronous engine
constructor** (`DriverGameState::new_with_options` / `new_full_pregame`) built the game with
`Game::new` + the start sequence and **never boxed the reserves** — they kept a `None` coordinate
all game.

The per-step **state hash cannot see this**: `collect_player_parts` takes only the first 11 by nr
(reserves are nr>11) AND clamps any off-pitch coord to `-1,-1` (so boxed ≡ @NONE for the hashed
eleven). But the **heuristic agent sees the real board**, and scores the Fanatic's compulsory-move
target off it — Rust (reserves @NONE) picked a different `orig_to` than Java (reserves boxed at
`-1,0/-1,1/-1,2`), so the very first B&C scatter took a different base direction. Proven with a new
`FFB_IDSTATE` full-board dump (mirrors Java's `JIDSTATE`, keyed by parity step index): at bb2025
seed 51 i=42 the ONLY board difference was `H12/13/14 = ?/?` (Rust) vs `-1,0/-1,1/-1,2` (Java).

Two fixes, both 1:1 from Java, colocated tests:
1. `UtilBox::box_all_players_at_game_start` — the boxing loop of `GameCache.addTeamToGame`
   (RESERVE/MISSING + `put_player_into_box`, spps, send-to-box reason), called from both engine
   constructors right after `Game::new`.
2. `UtilBox::refresh_boxes` / `refresh_box` — **was a no-op stub**. Java repacks each dugout column
   (collect coords, sort by y, reassign 0,1,2…) at end of setup/turn/eject/injury. Without it, the
   reserves boxed alongside the eleven fielded players kept rows 11-13 (put_player_into_box's
   first-free scan) instead of collapsing to 0-2 after the fielded players left — the agent saw the
   wrong rows. `StepSetup` already CALLED `refresh_boxes`; implementing it made the call effective.
   (Boxing WITHOUT refreshBoxes regressed amazon bb2025 @1.0 seed 22 → 99; refreshBoxes fixed it.)

Gate movement (heuristic, seeds 1-100, tier 3):

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** ✅ |
| bb2020 | 67 | 99 | 86 |
| bb2025 | 96 (was 94) | 97 (was 93) | 99 |

Regression suite MOSTLY clean: amazon ×3 @1.0, lineman ×3 @1.0, dwarf ×3, chaos ×3, chaos_dwarf ×3,
dark_elf ×3, dark_elf_league_fumbbl ×3, elf ×3 all 100/100 — EXCEPT **chaos_pact bb2020 @1.0 → 99
(seed 50)**, a closed-roster REGRESSION this fix causes. Revert-test confirmed: seed 50 passes
WITHOUT boxing, fails WITH. ffb-engine 7392/0, ffb-model 2801/0, ffb-parity 50/0.

**STATUS: FIX REVERTED — it is correct (Java boxes reserves) but a global change that EXPOSES a
latent AGENT-PARITY bug, regressing closed roster chaos_pact bb2020 @1.0 (100→99).** The heuristic
agent's board reading depends on reserve/box positions; the Rust agent and the Java agent disagree
for certain configurations, and boxing merely SHIFTS which seeds hit the disagreement (fixes goblin
bb2025 @0 seeds 7/37/51/89/96, newly-reds seed 80 + chaos_pact 50). So boxing is necessary but not
sufficient — the Rust↔Java heuristic agent must first be made bit-identical in how it scores off a
boxed/reserve board. Evidence that it is an agent (not engine) divergence:
chaos_pact bb2020 seed 50: through i=54 the rng_call count AND
the ENTIRE hashed state string AND the full-board `FFB_IDSTATE` dump (all players, coord+state) are
BIT-IDENTICAL between Rust and Java, yet step 54 (an away_04 HAND_OVER_MOVE that resolves a
block/armour/injury on home defenders) resolves differently — Java makes 9 rng-calls, Rust 4; home
`h00` ends Stunned (Java) vs Prone (Rust). Deterministic (stable over 3 runs, so NOT HashMap-order
flakiness). The divergent factor is therefore OUTSIDE rng_calls and the hashed state — a
`used_skills`/temporary-property flag or a mechanic that reads the boxed reserve's box position,
DORMANT while the reserve had no coordinate (@NONE) and only triggered once boxing gives it one.
goblin bb2025 @0 seeds 20/80 are the same class (identical board entering home_04's move, divergent
B&C-walk resolution). Cracking this needs per-roll windowing of one activation in both engines.

**Remaining B&C frontier (deeper, NOT the walk-continuation):** bb2025 @0 seeds 20/80, bb2020 @0
seed 92. The Fanatic's compulsory walk DOES correctly continue scattering from the box after a
crowd-push (a guard that stopped it broke the now-passing seeds 7/51/89/96 and was reverted — Java
continues too). The residual divergence is the **box COLUMN/injury outcome**: for seed 20 Rust's
home_04 scatters its box-continuation from `(-2,0)` (KO column) while ending at `(0,2)` vs Java's
`(0,4)` — the crowd-push injury (`InjuryTypeCrowdPush` → RESERVE/KO/casualty) or an earlier walk
die resolves differently, shifting the box the walk resumes from. Needs a fresh sub-step trace to
localise. bb2020 @1.0 (67) / @1e6 (86) unchanged by this fix — a separate family (sampled draw
splits) still to root-cause per seed.

## ITER16 — the composite fix: on_pitch guards (agent) + reserve boxing (engine) (2026-09-03)

ITER15 found the right engine fix (box un-fielded reserves so the B&C walk-continuation's crowd-push
box row matches Java) but it regressed closed roster chaos_pact by exposing a separate AGENT bug.
Two Explore passes over the Rust heuristic agent and the Java `ffb-ai/.../parity/heuristic/*` agent
pinned it exactly: **Java filters `onPitch` at every board-read; the Rust legal-action adjacency
scans in `legal_actions/mod.rs` did not.** A boxed HOME reserve at `RSV_HOME_X = -1` (the only dugout
column Chebyshev-adjacent to the pitch, at x=0) was injected as a phantom **ThrowTeamMate/KickTeamMate**
target for a home actor at x=0 → the agent's action list grew by one → its modulo pick shifted. Block/
Blitz/Foul scans were state-gated on `has_tacklezones` (false for `PS_RESERVE`) so only TTM/KTM leaked.

**Phase 1 (agent, 1:1 with Java):** added the `is_on_pitch()` guard to every player-adjacency scan in
`legal_actions/mod.rs` (TTM, KTM, Block/Blitz, prone-Blitz, Foul, `legal_foul_targets`, TTM/KTM target
lists, the open-square check) — mirroring the HandOff site that already had it. **Inert at baseline**
(reserves are `@NONE`): nine goblin gates + amazon/lineman ×3 @1.0 all UNCHANGED. It also correctly
excludes any mid-game crowd-pushed reserve at `(-1,y)`, so it is a real correctness fix.

**Phase 2 (engine):** re-applied `UtilBox::box_all_players_at_game_start` (1:1 `GameCache.addTeamToGame`
boxing loop; reserves get `PS_RESERVE`) from both `DriverGameState` constructors, and implemented
`UtilBox::refresh_boxes`/`refresh_box` (was a stub; `StepSetup` already calls it). Colocated tests.

Result — goblin nine gates (@1.0 / @0 / @1e6), baseline → now:

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 100 | 100 | 100 |
| bb2020 | 67 | 99 | 86 (unchanged — its reds are a separate family) |
| bb2025 | 94→**96** | 93→**97** | 99 |

No gate dropped. **Closed-roster regression suite ×3 @1.0 all 100/100, INCLUDING chaos_pact (the
ITER15 regression, now fixed by the Phase-1 guard):** amazon, lineman, dwarf, chaos, chaos_dwarf,
chaos_pact confirmed 100/100; dark_elf/dark_elf_league_fumbbl/elf running (expected clean — Phase 1
was inert on them). ffb-engine 7392/0, ffb-model 2801/0.

**Remaining reds → Phase 3 (the standard three-loop):**
- bb2025 @0 seeds **20 / 80** and bb2020 @0 seed **92** — the *deeper* B&C walk-continuation engine
  divergence (identical rng_calls + full `FFB_IDSTATE` board entering `home_04`'s Move, yet the walk
  resolves to a different crowd-push box coordinate — Rust ends `home_04` at (0,2) vs Java (0,4)).
  NOT the agent bug (a guard that stopped the walk on crowd-surf was tried and reverted — Java
  continues the walk too). Needs per-scatter windowing of the one activation.
- bb2025 @0 seed 98 (ThrowTeamMate) + bb2025 @1.0 (~4) + bb2020 @1.0 (~33) / @1e6 (~14) — sampled
  draw-splits to root-cause per seed via `first_state_divergence.sh` + `FFB_CANDSUM`/`FFB_DICE_TRACE`.

## ITER17 — dropPlayer has NO in-bounds guard: the B&C chain injury was skipped once boxed (2026-09-03)

**Six of nine gates GREEN.** The B&C-Fanatic frontier that ITER15/16 chased is closed, and the fix
was one non-Java line.

`drop_player_with_base_rng` (the shared port of Java's private
`UtilServerInjury.dropPlayer(step, player, playerBase, mode, eligibleForSafePairOfHands)`) opened with
a Rust-only early return:

```rust
if !FieldCoordinateBounds::FIELD.is_in_bounds(coord) { return params; }
```

Java guards ONLY on `(playerCoordinate != null) && (playerState != null)` — there is no in-bounds
test (`UtilServerInjury.java:338`). The consequence for a Ball & Chain Fanatic whose compulsory walk
scatters off the pitch: the FIRST crowd-push boxes it, so on EVERY LATER push its coordinate is a box
coordinate → the early return fired → the `placedProneCausesInjuryRoll` branch never ran → **2 fewer
d6 than Java**. Rust then read Java's chain-injury dice as its next scatter direction, and the walk
forked.

Proven bit-exactly with the aligned `pos` counter (Java's `DICE_TRACE pos` is `Xoshiro256StarStar.
callCount`, the SAME global counter behind `rng_calls`, so it aligns with Rust's `call_count`
directly — the earlier belief that it was per-drive was wrong). goblin bb2025 seed 20, home_04:

| pos | die | Java's use (from `caller=`) | Rust's use |
|---|---|---|---|
| 136 | d6=3 | B&C scatter direction | same |
| 137 | d6=1 | B&C scatter direction | same |
| 138,139 | 2,6 | `InjuryTypeCrowd` (crowd push) | same |
| 140,141 | 6,6 | **`InjuryTypeBallAndChain`** (chain injury) | same |
| 142,143 | d16=2,d6=3 | casualty roll | same |
| 144 | d6=2 | B&C scatter direction | same |
| 145,146 | 3,3 | `InjuryTypeCrowd` (crowd push) | same |
| 147,148 | 1,4 | **`InjuryTypeBallAndChain`** | **MISSING — Rust took pos 147 as its scatter** |
| 149 | d6=5 | B&C scatter direction | (Rust never got here) |

Fix: delete the in-bounds guard (1:1 with Java). The colocated test `off_field_player_is_noop`
asserted the REMOVED behaviour, so it encoded the bug; corrected to Java's semantics and renamed
`off_field_player_is_still_dropped` (an off-field STANDING player is still placed PRONE).

Gate movement (heuristic, seeds 1-100, tier 3), ITER16 → ITER17:

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** ✅ |
| bb2020 | 67 | 99 → **100** ✅ | 86 |
| bb2025 | 96 → **100** ✅ | 97 → **99** | 99 → **100** ✅ |

Closed seeds: bb2025 @0 20/80, bb2020 @0 92 (the walk), plus the whole bb2025 @1.0/@1e6 tails.
**Closed-roster regression suite 27/27 gates 100/100 — ZERO regressions** (amazon, lineman, dwarf,
chaos, chaos_dwarf, chaos_pact, dark_elf, dark_elf_league_fumbbl, elf ×3 @1.0). ffb-engine 7392/0.

**Remaining 3 gates:**
- **bb2025 @0 seed 98** — an away_01 Troll THROW_TEAM_MATE at i=129. Dice and board agree; the ONLY
  state difference at i=130 is the thrower's ACTIVE bit (`a00:9,4,Standing,4/5/5/10,` Rust `1` vs
  Java `0`). Java's `UtilActingPlayer.changeActingPlayer` retires a MOVING old acting player as
  `changeBase(STANDING).changeActive(false)` when `hasActed()`; Rust's `retire_old_acting_player`
  reaches the same block (the Troll IS `MOVING`/base 2) but takes the `else` branch, so `acted()`
  reads FALSE there while Java's `hasActed()` is TRUE. Next step: find which flag Java's TTM sets
  (`hasMoved`/`usedSkills`/`hasTriggeredEffect`) that Rust's TTM path does not.
  NOTE: Rust's `acted()` also carries an extra `|| self.has_acted` term that Java's derived
  `hasActed()` does not have — worth auditing separately.
  ALSO latent (hash-blind, unfixed): BANNED secret-weapon players are left with NO coordinate in
  Rust, where Java boxes them in the BAN column (`FFB_IDSTATE` seed 98 i=129: Rust `A4=?/d` vs Java
  `A4=35,0/d`, likewise `A7` and `H4=-6,0/d`).
- **bb2020 @1.0 (67) / @1e6 (86)** — a distinct DRAW-COUNT family: 26 of ~28 reds resolve at a Blitz
  by the bb2020 Fanatic (nr 3). Candidate lists agree through k=3 and then Rust has consumed 6 MORE
  sampler draws (25 vs 19), spent on three B&C DIRECTION team-re-roll offers Java never makes. Not
  an engine-dice divergence (argmax @0 is 100/100) — a prompt/draw-count split to root-cause with
  `FFB_DRAWS` + `FFB_CANDSUM`.

## ITER18 — Always Hungry marked the PLAYER's used set, not the ACTING player's (2026-09-03)

**SEVEN of nine gates GREEN; bb2016 and bb2025 are now 3/3 each.**

Java's `StepAlwaysHungry` escape branch does
`actingPlayer.markSkillUsed(getUnusedSkillWithProperty(actingPlayer, mightEatPlayerToThrow))`.
`ActingPlayer.markSkillUsed` (`ffb-common ActingPlayer.java`) ALWAYS adds to the acting player's
`fUsedSkills` — the set `hasActed()` reads — and writes the team Player's set only when
`skill.getSkillUsageType().isTrackOutsideActivation()`. The availability read one line above is the
matching ACTING-PLAYER overload (`hasUnusedSkillWithProperty(actingPlayer, ..)`, `:122`), while
`doEscape` deliberately reads the PLAYER's *skills* (`hasSkillWithProperty(actingPlayer.getPlayer(),
..)`, `:123`).

All three Rust twins wrote (and read) only the **team Player's** set. Consequence for a Troll whose
Always Hungry roll FAILS: `ActingPlayer::acted()` stayed false, so
`UtilActingPlayer.changeActingPlayer`'s retire took its `else { changeBase(STANDING) }` branch
instead of Java's `hasActed() → changeBase(STANDING).changeActive(false)`, leaving the thrower ACTIVE
where Java retires it — goblin bb2025 seed 98 i=130, the ONLY state difference being
`a00:9,4,Standing,4/5/5/10,` Rust `1` vs Java `0`. Same "acting set, not the Player set" fault as the
Bombardier fix in `util_server_steps::retire_old_acting_player`.

Fix (bb2016/bb2020/bb2025, one change set): write `game.acting_player.used_skills`, gate the
Player-level write on `usage_type().track_outside_activation()`, and move the `doAlwaysHungry`
"unused" READ onto the acting set too. **Fixing only the write regressed bb2016 @0 to 97** — the read
and the write are two halves of one contract ("two callers of one helper aren't one contract"); the
9 colocated tests that simulated "already used" via the Player's set were corrected to Java's
semantics (they now mark the acting set).

Gate movement, ITER17 → ITER18:

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** ✅ |
| bb2020 | 67 → 68 | **100** ✅ | 86 |
| bb2025 | **100** ✅ | 99 → **100** ✅ | **100** ✅ |

**Closed-roster regression suite 27/27 gates 100/100 — ZERO regressions.** ffb-engine 7392/0.

## Remaining frontier — bb2020 @1.0 (68) and @1e6 (86) ONLY

Localized to a single CANDIDATE-COUNT (draw-accounting) divergence; the engine is right (@0 is
100/100 in every edition, and argmax consumes no draws). bb2020 @1.0 seed 2, via `FFB_CANDSUM` +
`FFB_CAND=16`:

- Activations k=1..15 match EXACTLY (n and cumulative draws). First divergence **k=16**:
  Rust `n=297 draws=51` vs Java `n=286 draws=49`.
- The whole 11-candidate gap is one player: **away1, the Troll** — Rust offers **37** Move
  destinations, Java **27** (away11 differs by 1; away10 matches).
- **The budget is NOT the cause** (measured, do not re-chase it): a probe in `budget_of` at that
  exact decision prints `pid=away_01 ma_base=4 prone=true ma=1 spent=0 cap=3 gate=1`, and Java's
  `Reach.budgetOf` computes the identical `cap = max(ma + 2 - spent, 0) = 3` from the identical
  inputs. `STAND_UP_COST` is 3 on both sides; the two `cap` checks in the Dijkstra are also
  identical (`cost >= cap` / `ncost > cap`).
- The **board is identical too**: `FFB_IDSTATE` full-board diffs at i=12/13/14 are EMPTY (every
  player, both teams, including the hash-blind nr>11 pieces), and the hashed state string agrees on
  the Troll itself (`a00:13,7,Prone,4/5/5/10,1` in both).
- So with the SAME board and the SAME `cap=3`, Rust's Dijkstra admits **37** destinations where
  Java's admits **27**. The divergence is therefore in the reach **step-admissibility / pruning**,
  not the allowance: compare `reach_with` (`heuristic_agent.rs`, the `-log(p_step)` Dijkstra, its
  `gate`, any arrival-probability cutoff and how `tops`/`dests` are truncated) against
  `Reach.search` (`ffb-ai/.../parity/heuristic/Reach.java:183-280`) step by step. A per-square
  admissibility difference (occupancy of the START square, a probability floor, or the
  "stay put"/own-square destination) is the shape to look for.
- The draw-count split only bites at sampled scales: the 2 extra draws at k=16 shift every later
  sampler draw (32 reds @1.0, 14 @1e6). At @0 (argmax) no draws are taken, which is why bb2020 @0
  is 100/100 and the ENGINE is not implicated.

## Frontier update — the bb2020 fault is a `currentMove` offset on the BLITZ block, not the Reach

The ITER18 frontier note above scoped this to "the Troll's Move destination set at k=16, 37 vs 27".
That scoping was **wrong and is superseded**: k=16 is DOWNSTREAM of the real divergence (seed 2's
first hash diff is idx **15**, and the candidate lists match exactly for k=1..15), so k=16 was
comparing an already-diverged board. The 37-vs-27 count is a symptom, not the cause.

**Ruled out by measurement (do not re-chase any of these):**
- The Reach BUDGET: probe printed `pid=away_01 start=13,6 ma=1 spent=0 cap=3 gate=1 ag=5
  dodge=false sure_feet=false gt=2 team_rr=true`, and Java's `budgetOf` computes the identical
  `cap=3`. `STAND_UP_COST=3` both sides; the two Dijkstra cap checks are identical.
  (My earlier "Rust reaches distance 4" claim used the WRONG ORIGIN — the true start is (13,6),
  and all 37 Rust cells are within Chebyshev 3, consistent with `cap=3`.)
- The BOARD: `FFB_IDSTATE` diffs at i=12/13/14 **and i=15** are EMPTY (all players, both teams,
  incl. hash-blind nr>11).
- The reach ALGORITHM and the destination enumeration: two independent reads found
  `reach_with`/`Reach.search` and `top_moves`/`Plans.topMoves` line-for-line equivalent — same
  8-neighbourhood, same `f.occupied`, same `cost >= cap` / `ncost > cap`, same key-only relaxation,
  start excluded both sides, and **both** emit one candidate per settled cell with
  `usize::MAX` / `Integer.MAX_VALUE` (no top-K, no probability floor, no dedup).
- The TZ raster: Rust `st.has_tacklezones()` vs Java `Snap.standing`, and `Features.java:459`
  builds the Snap with `ps.hasTacklezones()` — identical.
- `dodge_target`/`gfi_target` vs `dodgeTarget`/`gfiTarget` — identical (Rust's `_ =>` arm covers
  bb2020 and bb2025 exactly as Java's `!bb2016` does), so this is NOT an edition-gated formula.
- `team_rr` (`td.rerolls > 0 && !td.reroll_used`) and `spentBy` — identical formulas.
- `StepMove`'s early `return` when `coordinate_to` is `None`: probed, `coordinate_to` is always
  `Some(..)` on this path, so the increment is not being skipped there. (NOTE: the bb2020
  `step_move.rs` is a **DEAD FILE** — `driver.rs:127` dispatches `StepId::Move` to
  `bb2025::move_::step_move` for both editions. A probe there prints nothing; probe the bb2025 one.)

**The actual divergence — bb2020 seed 2, i=15, the Fanatic's (away_03, nr 3) BLITZ.**
Both engines declare `Activate(away_03, Blitz)` and the board is identical entering it; the blitz
then resolves to different squares (i=16: A3 Rust (13,7) vs Java (14,8); A1 Rust (13,6) vs Java
(13,7)). Rust burns 6 dice, Java 4.

`FFB_MOVEP` shows the mechanism exactly. At the SAME prompt — Fanatic at (12,7), the SAME four
offered squares `[11,7 11,8 13,6 13,8]` — the two agents pick the same FIRST step but different
path lengths:

```
RMOVEP k=15 away_03 at=(13,8) n=5 offered=[12,9 13,9 14,7 14,8 14,9] ans=Block{home_01}   <-- Rust only
RMOVEP k=15 away_03 at=(12,7) n=4 offered=[11,7 11,8 13,6 13,8] ans=Move{[13,6, 13,5, 14,4]}  (3 squares)
JMOVEP k=15 Away3   at=12,7   n=4 offered=[11,7 11,8 13,6 13,8] ans=[13,6 14,6]               (2 squares)
```

Rust's path is exactly ONE square longer, and the matching `cm` (currentMove) traces show Java
running exactly **1 ahead** of Rust from the post-block `INIT_MOVING` onward:

| step | Java `cm` | Rust `cm` |
|---|---|---|
| BLOCK_ROLL | 0 | 0 |
| PUSHBACK | 0 | 0 |
| **INIT_MOVING** (post-block) | **1** | **0** |
| MOVE_BALL_AND_CHAIN | 1 | 0 |
| MOVE_BALL_AND_CHAIN | 2 | 1 |

`spent = currentMove` feeds `cap = ma + 2 - spent`, so Rust's cap is one higher at the (12,7) move
prompt → it reaches one square further → picks a different destination → the B&C base direction
(`coord_from` vs `original_coord_to`) changes → the scatters differ → the whole activation forks.

**So: Java charges the BLITZ's block one movement point; Rust does not.** Note the prompt shapes
also differ — Rust emits an extra Move prompt at (13,8) that the agent answers with
`Block{home_01}` (the declared-blitz-block path), where Java goes straight through
SELECT_BLITZ_TARGET/BLOCK_ROLL. Rust's declared-blitz-block path appears to skip the +1 that Java
applies (Rust's B&C block branch in `step_move_ball_and_chain` DOES `current_move += 1`, matching
Java's, so it is specifically the DECLARED blitz-block dispatch that is missing it).

**Next step:** find where Java charges that movement point for a declared blitz block (Java's only
`setCurrentMove` +1 sites are `StepMove`, `StepGoForIt`, `StepMoveBallAndChain` and the stand-up
minimum in `StepInitSelecting`), then port it 1:1 into Rust's blitz-block dispatch out of
`StepInitMoving` (`Action::Block` → `dispatchPlayerAction(BLITZ)`). Verify with the `cm` table
above, then `RSUM`/`JSUM` `n=`/`draws=` equality at every k for bb2020 seed 2.

### Refinement — the +1 is the `MOVE` step in the post-block sequence resume

Traced further (all read-only): the +1 is NOT in the blitz dispatch and NOT in the follow-up.
- `StepInitMoving.dispatchPlayerAction` only publishes `DISPATCH_PLAYER_ACTION` and
  `GOTO_LABEL_AND_REPEAT` — no `setCurrentMove`.
- `bb2020/block/StepFollowup` moves the blitzer with `updatePlayerAndBallPosition` and only READS
  `getCurrentMove() - 1` for its (client-only) TrackNumber — no `setCurrentMove`.

Java's `BlitzMove` sequence is `INIT_MOVING → MOVE_BALL_AND_CHAIN → MOVE → GO_FOR_IT ×2 → …`
(`generator/bb2020/BlitzMove.java:22-29`); Rust's mirror has the same order
(`generator/bb2020/blitz_move.rs:33-47`, `StepId::Move` at line 43). So after the block
sub-sequence finishes, **Java resumes the BlitzMove sequence at the step AFTER `INIT_MOVING`,
running `MOVE_BALL_AND_CHAIN` and then `MOVE` (+1)**, and only then loops to a fresh `INIT_MOVING`
— which is why Java's first post-block `INIT_MOVING` already reads `cm=1`. Rust instead arrives at
a fresh `InitMoving` with `cm=0`, i.e. its `MOVE` never ran for that leg.

So the fix is in Rust's **sequence control after a declared blitz block** (how the Block sequence
returns into `BlitzMove`), not in any single step's arithmetic. Compare Java's
`StepEndMoving.pushSequenceForPlayerAction(BLITZ)` + the `GOTO_LABEL_AND_REPEAT` resume against
Rust's `step_end_moving` Branch 3 / `push_sequence_for_player_action` and the label the Block
sequence returns to. Confirm the fix with the `cm` table above (Java 1/1/2 vs Rust 0/0/1), then
`RSUM`/`JSUM` `n=`/`draws=` equality at every k for bb2020 seed 2, then the nine gates.

## ITER19 — the missing BB2020 ball-and-chain `GO_FOR_IT` in the blitz-block sequence

**The `currentMove` +1 predicted by the ITER18 refinement is real, and it is a MISSING STEP.**

The refinement note guessed the +1 came from a `MOVE` step in a post-block sequence *resume*. It
does not. A per-step `RCM step=<id> cm=<n>` probe in `driver.rs` on bb2020 seed 2 printed the whole
Fanatic activation, and the blitz-block sequence Rust actually ran was:

```
InitBlocking  GoForIt  SteadyFooting  DumpOff  BlockStatistics  Dauntless  Horns  Trickster
CatchScatterThrowIn  Stab  BlockChainsaw  SteadyFooting  ProjectileVomit  BreatheFire
SteadyFooting  SteadyFooting  HandleDropPlayerContext  Chomp  BlockRoll  BlockChoice
Pushback  RemoveTargetSelectionState  Apothecary  Followup  PickUp  GotoLabel
DropFallingPlayers  SteadyFooting  HandleDropPlayerContext  RemoveTargetSelectionState
ResetFumblerooskie  PlaceBall  Apothecary  SteadyFooting  PlaceBall  Apothecary  TrapDoor
Apothecary  CatchScatterThrowIn  RemoveTargetSelectionState  EndBlocking
```

That is the **BB2025** shape, and it contains exactly **one** `GO_FOR_IT`.
`bb2020/BlitzBlock.java` contains **two**:

```java
sequence.add(StepId.GO_FOR_IT, from(StepParameterKey.GOTO_LABEL_ON_FAILURE, IStepLabel.FALL_DOWN));
...
sequence.add(StepId.PLACE_BALL, IStepLabel.DEFENDER_DROPPED);
sequence.add(StepId.APOTHECARY, from(StepParameterKey.APOTHECARY_MODE, ApothecaryMode.DEFENDER));
// GFI for ball & chain should go here.
sequence.add(StepId.GO_FOR_IT, from(StepParameterKey.GOTO_LABEL_ON_FAILURE, IStepLabel.DROP_FALLING_PLAYERS),
    from(StepParameterKey.BALL_AND_CHAIN_GFI, true));
```

`StepGoForIt.executeStep` gates on
`runGfi = player.hasSkillProperty(goForItAfterBlock) == ballAndChainGfi`, and the +1 lives INSIDE
that gate:

```java
if (runGfi) {
  if ((PlayerAction.BLITZ == actingPlayer.getPlayerAction()) && (getReRolledAction() == null)) {
    game.getTurnData().setBlitzUsed(true);
    actingPlayer.setCurrentMove(actingPlayer.getCurrentMove() + 1);
```

So for a **Ball-and-Chain** blitzer (`goForItAfterBlock = true`) the leading `GO_FOR_IT` is a no-op
(`true == false`) and **only the second copy charges the blitz its movement point**. Rust never had
that copy, so the Fanatic came out of its block with `currentMove` one too low; the agent's reach
budget `cap = ma + 2 - currentMove` was one square too generous, it picked a further destination,
the Ball-and-Chain base direction changed, and the activation forked. Exactly the `cm` table
ITER18 measured (Java 1/1/2 vs Rust 0/0/1), and exactly why only the sampled scales were red.

Why the step was missing: `driver.rs` runs the **BB2025** step-set for BB2020, so the live generator
is `generator/bb2025/blitz_block.rs`, which already carries two `params.rules != Rules::Bb2020`
gates (PICK_UP, FOUL_APPEARANCE). `generator/bb2020/blitz_block.rs` is a faithful BB2020 port that
**nothing live calls** — the same dead-twin trap the ledger already records for `step_move.rs`.

**Fix** (`generator/bb2025/blitz_block.rs`): a third edition gate — `if params.rules ==
Rules::Bb2020`, add the ball-and-chain `GO_FOR_IT` immediately after `APOTHECARY(DEFENDER)`, with
`GOTO_LABEL_ON_FAILURE = DROP_FALLING_PLAYERS`, exactly where `bb2020/BlitzBlock.java` puts it.
Colocated test `bb2020_has_ball_and_chain_gfi_after_defender_apothecary_and_bb2025_does_not`
written from the two Java generators (two GFIs in bb2020, one in bb2025, placement and failure
label pinned); the existing `pick_up_before_catch_scatter_is_bb2025_only` length assertion was
updated from `-2` to `-2 + 1`.

Gate movement, ITER18 → ITER19 (all nine re-measured on the built binary):

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** ✅ | **100** ✅ | **100** ✅ |
| bb2020 | 68 → **95** | **100** ✅ | 86 → **97** |
| bb2025 | **100** ✅ | **100** ✅ | **100** ✅ |

`cargo test -p ffb-engine` 7393 passed / 0 failed.

### Remaining frontier — bb2020 @1.0 (95) and @1e6 (97): a Troll THROW_TEAM_MATE

All FIVE bb2020 @1.0 reds are ONE family. `first_state_divergence.sh` on each:

| seed | first hash diff | the activation that resolved differently |
|---|---|---|
| 35 | i=95 | `Activate(away_01, ThrowTeamMate)` |
| 47 | i=101 | `Activate(home_01, ThrowTeamMate)` |
| 53 | i=79 | `Activate(away_01, ThrowTeamMate)` |
| 88 | i=23 | `Activate(away_01, ThrowTeamMate)` |
| 89 | i=98 | `Activate(home_01, ThrowTeamMate)` |

`away_01` / `home_01` is the **Troll**. Measured on seed 88 (the earliest index, therefore the
cheapest repro):

- **The agent is not implicated.** `FFB_CANDSUM` `RSUM`/`JSUM` agree EXACTLY on `n=` and `draws=`
  at every k through and past the throw (k=23 `n=1384 draws=63`, k=24 `n=1252 draws=65`,
  k=25 `n=1097 draws=66`, k=26 `n=970 draws=68` — identical per-player breakdowns too).
- **The board is identical.** `FFB_IDSTATE` at i=24 diffs EMPTY: all 26 pieces, both teams,
  including the hash-blind nr>11 players, same coordinates and same state base.
- So the differing hash at i=24 is a **non-positional hashed field** — the ball, a turn-data
  counter, a re-roll bank, or an acting-player/active bit. That is the same shape as the ITER18
  bb2025 seed-98 Troll TTM red (`used_skills` written to the Player set instead of the acting set,
  leaving the thrower ACTIVE), so start by re-reading the BB2020 side of that same
  `Always Hungry` / `changeActingPlayer` contract before looking anywhere else.

**Do NOT re-derive state from `parity/goblin_vs_goblin/*.jsonl`** — that directory is shared across
editions, so after a bb2025 run the bb2020 lines are stale (this cost a wrong reading here: seed 88
line 24 still carried `teamGoblinParity25Home1`). Re-run the seed with `FFB_IDSTATE`/`FFB_STEPTRACE`
and read the live trace.

**Also known but NOT measured** (recorded so the next iteration does not rediscover them): beyond
the ball-and-chain GFI, the live BB2025 blitz-block sequence still differs from
`bb2020/BlitzBlock.java` by TENTACLES and SHADOWING after FOLLOWUP, by a second
`DROP_FALLING_PLAYERS`/`HANDLE_DROP_PLAYER_CONTEXT`/`FALL_DOWN` block, and by the BB2025-only
`STEADY_FOOTING` entries. None of them is on any currently-failing path — bb2020 @0 is 100/100 —
so they were deliberately left alone rather than shipped untested.

One more pointer for that dig: `driver.rs:400-420` records (from the ogre bb2020 campaign) that
BB2020's TTM chain is *partly* routed to bb2020 twins and that routing the WHOLE set — including
`AlwaysHungry` — was measured and is WORSE, so BB2020's TTM differences must be edition-gated
INSIDE the shared chain. Check which of the BB2020 TTM steps are live before assuming a twin runs.
The per-step state string (`ffb-model/src/util/state_hash.rs`) hashes half/turns/active/score,
ball, the four once-per-turn flags, both re-roll banks, the acting player + its `currentMove`, and
each player's coordinate/state/ACTIVE bit — `FFB_IDSTATE` shows only the coordinate and the state
base, so an ACTIVE-bit or a turn-flag difference is invisible to it. That is precisely the shape of
the ITER18 bb2025 seed-98 red.


## ITER20 — the BB2020 Troll TTM family: three faults, all in the landing chain

The four bb2020 @1.0 reds (47/53/88/89) and one of the two @1e6 reds (27) were ONE activation shape
— a Troll `THROW_TEAM_MATE` — hiding three separate BB2020-vs-BB2025 divergences, each of which
only became visible once the one in front of it was fixed. bb2020 @1.0 **95 → 100**, @1e6
**97 → 99**.

### Fault 1 — `StepSwoop` never retired the thrower (ACTIVE bit)

Repro bb2020 seed 88, i=24. `FFB_TRACE` prints both engines' full state STRINGS (`RUST_STEP … state=`
and, because it also sets `-Dffb.parityDebug=true`, `JSTEP i=… state=`). They differed in exactly one
character:

```
J  a00:14,8,Standing,4/5/5/10,0
R  a00:14,8,Standing,4/5/5/10,1
```

`a00` = `away_01`, the Troll that threw at i=23. `FFB_IDSTATE` cannot see this — it prints coordinate
and state BASE only, and the base agrees.

A temporary `RRETIRE`/`RRESET`/`RNONE` probe set in `util_server_steps.rs` showed
`retire_old_acting_player` was never called for the thrower; he left MOVING through
`reset_blocked_and_moving_players` instead (`RRESET id=away_01 base=2 act=true`), which changes the
base and leaves ACTIVE alone. Cause, in `bb2025/ttm/step_swoop.rs`: Java's

```java
game.getFieldModel().setPlayerCoordinate(thrownPlayer, passCoordinate);
UtilActingPlayer.changeActingPlayer(game, state.thrownPlayerId, PlayerAction.SWOOP, false);
```

had been ported as four hand-written field writes that install the THROWN player and never touch the
OUTGOING one. The missing half is `changeActingPlayer`'s `oldPlayer != newPlayer` branch:
`if (currentState.getBase() == MOVING) { if (actingPlayer.hasActed() …)
setPlayerState(oldPlayer, currentState.changeBase(STANDING).changeActive(false)); }`. The thrower is
MOVING and has acted, so Java retires him STANDING **and INACTIVE**. A file comment had even recorded
the gap as unportable ("this crate has no port of `UtilActingPlayer` at all") — it does have one.

**Fix**, mirroring Java's own two-layer split:

1. `util_server_steps.rs` — `change_player_action` (Java `UtilServerSteps.changePlayerAction`) now
   wraps a new `pub fn change_acting_player` (Java `UtilActingPlayer.changeActingPlayer`); the
   wrapper adds only `updateMoveSquares` + `updateDiceDecorations`, exactly as Java does. No
   behaviour change for existing callers.
2. `bb2025/ttm/step_swoop.rs` — call `change_acting_player`. Java's `StepSwoop` calls the BARE
   helper, so the move-square/dice refresh is correctly absent; `setCurrentMove(MA - 3)` still runs
   after.

`bb2025/ttm/step_swoop.rs` is the LIVE step for all three editions (`driver.rs:243`), and Java's
`mixed/ttm/StepSwoop` and `bb2025/ttm/StepSwoop` carry the identical call, so covering all three is
correct rather than convenient. Test
`a_landing_swoop_retires_the_thrower_standing_and_inactive`.

Measured alone: bb2020 @1.0 95 → 96, @1e6 97 → 98, everything else unchanged, closed-roster 27/27.

### Faults 2 and 3 — BB2020's landing re-roll: Swoop is not a BB2020 re-roll source

With fault 1 fixed, seed 88's divergence moved from i=23 to i=93 — still the Troll TTM, now a
DRAW-COUNT split. `FFB_STEPTRACE`, the same activation on both sides:

| | Java (`JSTATE`) | Rust (`RSTATE`) |
|---|---|---|
| 1 | `INIT_SELECTING` (activation pick) | `InitSelecting` |
| 2 | `INIT_THROW_TEAM_MATE` | `InitThrowTeamMate prompt=ThrowTeamMateTarget` |
| 3 | `SWOOP` | `Swoop prompt=SwoopTarget` |
| 4 | **`RIGHT_STUFF dialog=RE_ROLL`** | *(no prompt at all)* |
| 5 | `INIT_SELECTING ap=null` | `InitSelecting ap=null` |

`FFB_CANDSUM` quantified it: `draws=257` on both sides before the throw, `260` (Java) vs `258`
(Rust) after, with IDENTICAL candidate sets either side (`k=99 n=1136`, `k=100 n=854`, same players,
same per-player counts). Nothing about the board had drifted; the two engines were simply two draws
apart in the decision stream, and the next pick split.

Two distinct BB2025-isms were feeding a BB2020 landing:

**Fault 2 — the step's `usingSwoop` shortcut.** `bb2025/ttm/StepRightStuff.java`'s failure branch is
`if (usingSwoop) { setReRollSource(SWOOP); REPEAT; } else { askForReRollIfAvailable(...); }`, and its
`handleCommand` has a `CLIENT_USE_SKILL` arm setting the same source.
`bb2020/ttm/StepRightStuff.java` has NEITHER — no `usingSwoop` field, no `CLIENT_USE_SKILL` override,
just `setReRolledAction(RIGHT_STUFF); doRoll = askForReRollIfAvailable(...)`. The shared Rust step
serves BB2020 too, so both sites are now gated on `game.rules == Rules::Bb2025`. Test
`the_swoop_reroll_source_is_bb2025_only`.

**Fault 3 — the SKILL re-roll table.** Gating the step was not enough: an `RRS-SKILLSRC` probe showed
`find_skill_reroll_source(game, "RIGHT_STUFF")` returning `Some("Swoop")` anyway, so BB2020 still
re-rolled silently instead of asking. `SkillId::reroll_sources()` is documented as a cross-edition
UNION, safe only while each edition's steps ask about action strings no other edition uses —
and `RIGHT_STUFF` breaks exactly that, because BOTH editions' `StepRightStuff` ask for it while Java
registers the source in one edition only:

```java
// ffb-common/.../skill/bb2025/Swoop.java
registerRerollSource(ReRolledActions.RIGHT_STUFF, ReRollSources.SWOOP);
// ffb-common/.../skill/bb2020/Swoop.java  — three registerProperty calls, no re-roll source
// ffb-common/.../skill/bb2016/Swoop.java  — likewise
```

**Fix**: a new `SkillId::reroll_sources_for(rules)`, the sibling of the existing
`properties_for(rules)`, returning `&[]` for `(Swoop, Bb2016 | Bb2020)` and delegating otherwise;
`find_skill_reroll_source` now goes through it. Test
`the_swoop_right_stuff_reroll_source_is_bb2025_only` (skill_id.rs — the split entry plus two
unchanged controls).

Faults 2+3 together took bb2020 @1.0 96 → **100** and @1e6 98 → **99**.

### Gate movement, ITER19 → ITER20 (all nine re-measured on the final binary)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** ✅ | **100** ✅ | **100** ✅ |
| bb2020 | 95 → **100** ✅ | **100** ✅ | 97 → **99** |
| bb2025 | **100** ✅ | **100** ✅ | **100** ✅ |

`cargo test -p ffb-engine` 7395 / 0, `ffb-model` 2802 / 0. Random controls ×3 editions 100/100 under
`FFB_PARITY_ROOT=parity_random`. Closed-roster regression suite 27/27 gates 100/100.

### The ONE remaining gate — bb2020 @1e6 seed 83: a TTM landing injury, KO vs Badly Hurt

Not a regression: seed 83 was already red before this iteration's fixes (the ITER19 @1e6 reds were
27 and 83; 27 is now closed). The divergence is at i=14, resolving the away Troll's
`THROW_TEAM_MATE` at i=13, and the two state strings differ in ONE field:

```
J  a02:-1,-1,Bh,3/7/3/8,0
R  a02:-1,-1,Ko,3/7/3/8,0
```

`a02` is `away_03`, NOT the thrown player (`away_10`, who lands prone at 12,8 in both). Everything
else — all 22 parts, ball, flags, banks, weather — is identical, and `rng_calls` is **51 on both
sides at i=14** (36 at i=13), so the two engines spent the SAME number of dice on the activation:
this is not a missing or extra roll, it is the same rolls read differently. Java ends with a
casualty (Badly Hurt, off pitch); Rust ends with a Knock-Out.

Start next iteration at the BB2020 TTM landing injury path — `InjuryTypeTTMLanding` and the
`ApothecaryMode.THROWN_PLAYER` step that follows it in `bb2020/ttm/StepRightStuff.java` — since an
apothecary is precisely the thing that turns a casualty into a KO without spending a die. Confirm
with `FFB_DICE_TRACE` (Java's `pos` is the same global counter as `rng_calls`, so the two align
directly) that both engines read the same injury total and the same d16, then find which side
applies the apothecary.


## ITER21 — the ninth gate: BB2020's TTM landing runs `dropPlayer` INLINE, not deferred

bb2020 @1e6 seed 83 closes, and **all nine goblin gates are 100/100**.

### It was not the landing injury, and not an apothecary

The ITER20 hand-off pointed at `InjuryTypeTTMLanding` / `ApothecaryMode.THROWN_PLAYER`. That was
wrong, and `FFB_DICE_TRACE` said so before a line of code changed. Java's `caller=` stack names the
step behind every die, and positions 43–48 of seed 83 are:

```
pos=43 sides=6 result=3  InjuryTypeTTMHitPlayer.handleInjury:44  StepInitScatterPlayer.executeStep:220
pos=44 sides=6 result=5  InjuryTypeTTMHitPlayer.handleInjury:44  StepInitScatterPlayer.executeStep:220
pos=45 sides=6 result=6  InjuryTypeBallAndChain.handleInjury:28  UtilServerInjury.dropPlayer:341  StepInitScatterPlayer.executeStep:259
pos=46 sides=6 result=5  InjuryTypeBallAndChain.handleInjury:28  UtilServerInjury.dropPlayer:341  StepInitScatterPlayer.executeStep:259
pos=47 sides=16 result=4 RollMechanic.rollCasualty:58 …InjuryTypeBallAndChain
pos=48 sides=6  result=1 RollMechanic.rollCasualty:58 …InjuryTypeBallAndChain
```

Rust's per-die trace is byte-identical at every one of those positions (and at 49–51, the landing
armour roll). So the differing player is not the thrown Goblin at all: `a02` is the away **Fanatic**
(3/7/3/8), who was standing on the landing square. He takes TWO injuries in one step — the
`TTMHitPlayer` roll for being landed on (3+5 = 8 → Knocked Out), and then the `BallAndChain` chain
injury for being dropped (6+5 = 11 → casualty, d16=4/d6=1 → Badly Hurt). Both engines rolled both.
Only the *surviving* result differed.

An `FFB_INJPROBE` probe pair (one line in `util_server_injury::handle_injury`, one in
`InjuryResult::apply_to`) made it exact:

```
RINJ   def=away_03 apo=HitPlayer injury=PlayerState(5) ko=true            <- TTMHitPlayer  → KO
RINJ   def=away_03 apo=HitPlayer injury=PlayerState(6) cas=Some([4, 1])   <- BallAndChain  → Badly Hurt
RAPPLY def=away_03 new=PlayerState(5) set=true                            <- the KO is what gets applied
```

### The mechanism: a `Map` slot, and which write lands in it last

`bb2020/ttm/StepInitScatterPlayer` resolves a landing-on-a-player INLINE:

```java
InjuryResult injuryResultHitPlayer = UtilServerInjury.handleInjury(this, new InjuryTypeTTMHitPlayer(), …);
publishParameter(new StepParameter(StepParameterKey.INJURY_RESULT, injuryResultHitPlayer));
…
publishParameter(THROWN_PLAYER_ID / THROWN_PLAYER_STATE / THROWN_PLAYER_HAS_BALL / IS_KICKED_PLAYER);
if (playerLandedUpon != null) {
  publishParameters(UtilServerInjury.dropPlayer(this, playerLandedUpon, ApothecaryMode.HIT_PLAYER, true));
}
```

`StepParameterSet` is a `Map<StepParameterKey, StepParameter>` (`fParameterById.put(...)`), so the
SECOND `INJURY_RESULT` — the one `dropPlayer` publishes from its `placedProneCausesInjuryRoll`
branch for a Ball & Chain player — **replaces** the first, and the apothecary step applies the chain
injury. Order is the whole semantics.

`bb2025/ttm/StepInitScatterPlayer` has a different shape: it publishes no `INJURY_RESULT`, and
instead wraps the hit injury plus a deferred `DropPlayerCommand` in a `STEADY_FOOTING_CONTEXT`.
`StepSteadyFooting.fail()` then runs the deferred commands FIRST and republishes the context's own
injury result AFTERWARDS — the exact inverse order. Correct for BB2025, whose `ScatterPlayer`
generator contains the Steady Footing step.

Rust ran the BB2025 shape under BB2020, because `driver.rs` routes `InitScatterPlayer` to the
bb2025 impl for both editions and `bb2025/ttm/step_dispatch_scatter_player.rs` builds the bb2025
`ScatterPlayer` sequence. Note the trap this hides: `generator/bb2020/scatter_player.rs` exists,
correctly has NO Steady Footing step, and even carries a test asserting so — and is **dead** on this
path. Reading it is what makes "BB2020 has no Steady Footing, so the deferred command can never run"
look true; in fact the bb2025 generator was live, the step did run, and it ran in the wrong order.

### The fix

`crates/ffb-engine/src/step/bb2025/ttm/step_init_scatter_player.rs`, landing-on-a-player branch,
gated `game.rules == Rules::Bb2020` (bb2016 has its own routed step, so only bb2020 reaches here):
publish `INJURY_RESULT` for the hit player, `END_TURN` when `alwaysTurnOver || own team`,
`THROWN_PLAYER_COORDINATE` / `CRASH_LANDING(false)` / `PLAYER_ENTERING_SQUARE`; then, AFTER the
four always-published parameters, run `util_server_injury::drop_player_rng(…, HIT_PLAYER, true)`
inline and publish everything it returns. No `SteadyFootingContext`, no deferred commands, and no
`ReportPlayerEvent("was hit")` (the bb2020 step has none). BB2025 is untouched.

Rust's `apply_effects` drains `outcome.published` in order, calling `set_parameter` per step, so a
later `InjuryResult` overwrites an earlier one exactly as Java's map does — the ordering fix is
sufficient and needs no new machinery.

Three colocated tests written from the Java: BB2020 landing on a Ball & Chain player publishes
`INJURY_RESULT` twice with the chain one LAST and no `SteadyFootingContext`; BB2020 landing on a
plain player publishes it once (`dropPlayer`'s else branch places PRONE and publishes nothing);
BB2025 still publishes a `SteadyFootingContext` and no `INJURY_RESULT`.

After the fix the probe reads `RAPPLY def=away_03 new=PlayerState(6) set=true` and seed 83 is
`PARITY: 1/1 games match`.

### 🏁 Gate movement, ITER20 → ITER21 — ALL NINE GREEN

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** ✅ | **100** ✅ | **100** ✅ |
| bb2020 | **100** ✅ | **100** ✅ | 99 → **100** ✅ |
| bb2025 | **100** ✅ | **100** ✅ | **100** ✅ |

All nine re-measured on the final release binary (goblin v goblin, seeds 1-100, tier 3, heuristic,
`--heur-classes all`), each printing `PARITY: 100/100 games match`. `cargo test -p ffb-engine`
7398 / 0, `ffb-model` 2802 / 0. Random controls ×3 editions 100/100 under
`FFB_PARITY_ROOT=parity_random`.
