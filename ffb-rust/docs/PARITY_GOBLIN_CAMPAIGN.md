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
