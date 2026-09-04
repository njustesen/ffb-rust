# High Elf — heuristic-agent parity campaign

**Goal**: high_elf v high_elf, HeuristicAgent both sides, per-step state-hash parity 100/100,
seeds 1-100, tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus random
controls and the standing regression set. Procedure: `.claude/commands/amz-iter.md` with
`MATCHUP=high_elf`. Started 2026-09-04, after goblin (`fc58b0e1c`) and halfling (`e6571447c`) closed.

## Surface

Roster in all three editions, but **bb2025 is a different roster**, not a re-skin — that is where
the work is:

- bb2016 (11): Blitzer ×2, Catcher ×4, Thrower ×2, Lineman ×3.
- bb2020 (12): Thrower ×1, Blitzer ×1, Catcher ×1, Lineman ×9.
- bb2025 (11): **Dragon Prince ×2, White Lion ×2, Phoenix Warrior ×2**, Lineman ×5.
  - Phoenix Warrior: **Cloud Burster**, Pass, **Safe Pass**
  - Dragon Prince: Block, **My Ball**, **Steady Footing**
  - White Lion: **Claws**, Wrestle

Cloud Burster / My Ball / Safe Pass / Steady Footing are largely unexercised surface for this sweep
(amazon's campaign touched a "Safe Pass reroll order"; the rest are new here).

## Baseline (2026-09-04, measured on `e6571447c`, nine gates, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 | 99 | 98 |
| bb2020 | **100** ✅ | 99 | **100** ✅ |
| bb2025 | **0** | 86 | **0** |

221 reds across the nine gates; two gates (bb2020 @1.0 and @1e6) already green.

**READ THE bb2025 SIGNATURE CAREFULLY — it is the whole race.** bb2025 is `0/100` at BOTH sampled
scales yet `86/100` at argmax. Argmax consumes zero sampler draws, so:
- the engine's *resolution* is mostly right (86/100 at @0), and
- something makes **every single game** diverge the moment sampling is switched on.

That is the fingerprint of a **prompt/draw-count** divergence, not a dice or resolution bug: a
dialog Rust offers that Java never does (or vice versa), fired so early and so often that no seed
survives. The first move is therefore NOT `first_state_divergence.sh` on a seed — it is
`FFB_DRAWS=1` on ONE bb2025 @1.0 seed, comparing the `RDRAW`/`JDRAW` prompt-class sequences and
their running totals to find the first prompt whose class or draw count differs. Then
`FFB_CANDSUM` (`RSUM`/`JSUM` `n=`/`draws=`) at that activation. The new bb2025 skills above
(Cloud Burster, Safe Pass, My Ball, Steady Footing) are the obvious suspects for an extra dialog.

Fix that one prompt and both bb2025 sampled gates should move together; the 14 reds at @0 are a
separate, smaller family to clean up after. bb2016 (6 reds) and bb2020 (1 red) are nearly green.
