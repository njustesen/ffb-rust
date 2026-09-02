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
