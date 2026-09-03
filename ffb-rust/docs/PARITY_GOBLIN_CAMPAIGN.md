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
