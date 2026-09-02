# Elf — heuristic-agent parity campaign

**Goal**: elf v elf, HeuristicAgent both sides, per-step state-hash parity 100/100, seeds
1-100, tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus random
controls and the standing regression set. Procedure: `.claude/commands/amz-iter.md` with
MATCHUP=elf. Started 2026-09-02 after dark_elf_league_fumbbl closed (5e4c4abb7).

## Surface

Roster exists in ALL THREE editions (`data/rosters/*/roster_elf.json`):

| edition | Lineman | Thrower | Catcher | Blitzer |
|---|---|---|---|---|
| bb2016 | 16, none | 2, Pass | 4, Catch + Nerves of Steel | 2, Block + Side Step |
| bb2020 | 12, none | 2, Pass | 4, Catch + Nerves of Steel | 2, Block + Side Step |
| bb2025 | 16, Fumblerooski | 2, Hail Mary Pass + Pass | 2, Catch + Diving Catch + NoS | 2, Block + Sidestep |

Skills to expect: **Side Step** (pushback square override — defender chooses), **Nerves of
Steel** (ignore TZ on catch/pass/intercept), **Diving Catch** (bb2025: +1 catch on accurate,
adjacent-landing claim prompt), **Fumblerooski** (bb2025 lineman: place the ball deliberately),
**Hail Mary Pass** (bb2025 thrower — the dwarf campaign drove the bb2016 twin; this is the
native edition), Pass/Catch re-rolls. AG4 everywhere: high dodge/pass volume.

## Baseline

(to be measured)
bb2016: @1.0 **0/100**, @0 **100/100**, @1e6 **2/100**. Randoms ×3: 100/100.
Argmax green + sampling red + randoms green ⇒ the two agents disagree in candidate WEIGHTS
(or draw counts) somewhere elf-specific: argmax hides a weight gap that sampling exposes.
Failures start as early as step 1 with the pre-state hash already differing ⇒ the extra/short
draw happens in the very first sampled decisions.
bb2020: @1.0 1/100, @0 100/100, @1e6 1/100. bb2025: @1.0 87/100, @0 100/100, @1e6 91/100.
All three editions share the argmax-green/sampled-red signature; bb2016/bb2020 nearly total
⇒ the mispriced weight is on a high-frequency candidate class (activation/move), strongest
where the older-edition rosters (Catcher ×4) shape candidate sets.

## ITER1 — Side Step must PROMPT, not auto-use (bb2016 + bb2020 behaviours)

Inner loop on bb2016 @1.0 seed 98 (fails step 1): FFB_CANDSUM first mismatch k=2 (n 2145 vs
2133, draws 12 vs 14); candidate diff showed ZERO weight diffs — pure board divergence (the
side-stepped Away2 at (11,7) Rust vs (11,6) Java). RDRAW/JDRAW totals align at 8 entering the
game's first block, then Java runs `SKILL_USE SideStep` (2 draws) + the defender-chosen square
(2 draws) while Rust jumped straight to its own attacker-chosen pushback — 2 draws short and a
different square.

Root cause: `bb2016/side_step_behaviour.rs` auto-ACCEPTED an undecided Side Step (a
random-agent-era shortcut mirroring ParityRunner's free SKILL_USE answer); `bb2020`'s twin did
the same via `or_insert(true)`. The HEURISTIC contract scores the dialog (HeuristicDriver
useSkill, default w=0.50) and spends two sampler draws. bb2025's twin was already fixed in the
amazon campaign (Estelle) — this is the same fault in the other two editions.

Fix: park the offer in `pending_skill_use` (the StandFirm/dwarf-ITER1 bridge; both pushback
steps already raise AgentPrompt::SkillUse from it and file the answer into `side_stepping`).
The random agent answers the new prompt USE=true with 0 rng (contract §7), identical to the
old auto-use — randoms unaffected by construction, re-gated anyway. Tests: bb2016
`side_step_undecided_parks_the_offer` (replaces the auto-use test), bb2020 twin updated.
Seed 98 GREEN after fix.
Post-ITER1 gates: bb2016 100/100 ×3 ✅, bb2020 100/100 ×3 ✅, randoms ×3 100/100 ✅, engine
tests 7390/0. bb2025 UNCHANGED (87/13 @1.0, 91/9 @1e6, @0 green) — its Sidestep already
parked (Estelle), so the bb2025 reds are a different fault. ITER2 target: bb2025 seed 51
(fails step 3).
