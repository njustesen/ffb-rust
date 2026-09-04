# Halfling — heuristic-agent parity campaign

**Goal**: halfling v halfling, HeuristicAgent both sides, per-step state-hash parity 100/100,
seeds 1-100, tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus random
controls and the standing regression set. Procedure: `.claude/commands/amz-iter.md` with
`MATCHUP=halfling`. Started 2026-09-04, immediately after goblin closed (`fc58b0e1c`).

## Surface

Roster in ALL THREE editions. Two Treemen every edition — this race is dominated by the
**Throw Team-Mate / Right Stuff** machinery plus the Treeman negatrait:

- bb2016: Treeman ×2, Halfling ×12.
- bb2020: Treeman ×2, Halfling Catcher ×2, Halfling Hefty ×2, Halfling ×7.
- bb2025: Treeman ×2, Halfling Catcher ×2, Halfling Hefty ×2, Halfling ×8.
- bb2025 Treeman: Mighty Blow, Stand Firm, Strong Arm, **Take Root**, Thick Skull,
  **Throw Team-mate**, **Timmm-ber!**. The halflings are Stunty/Dodge/Right Stuff.

Prior art that matters: goblin ITER19/ITER21 (just closed) both landed in exactly this machinery —
the bb2020 blitz-block's second `GO_FOR_IT` carrying `BALL_AND_CHAIN_GFI`, and BB2020's TTM landing
resolving `dropPlayer` INLINE (so the chain injury REPLACES the hit injury in Java's parameter map,
where bb2025 defers it through a `SteadyFootingContext`). Halfling has no Ball & Chain, but it does
have TTM landings on both editions' paths, so the BB2020-vs-BB2025 landing shape is the first place
to look if bb2020 is weak. `Take Root` is a Select-sequence negatrait (like Really Stupid / Bone
Head) and `Timmm-ber!` is a stand-up assist — both are new surface for this sweep.

## Baseline (2026-09-04, measured on `fc58b0e1c`, nine gates, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 | 99 | 99 |
| bb2020 | 98 | **100** ✅ | 95 |
| bb2025 | 84 | 93 | 90 |

45 reds across the nine gates; one gate (bb2020 @0) already green. **bb2025 is the weak edition**
(84/93/90 = 33 of the 45). Note bb2025 is red even at @0 (argmax), so at least part of the bb2025
fault is candidate CONTENT/eligibility or a genuine engine divergence — not only a sampled
draw-count split. bb2016 and bb2020 are within a few seeds of green at every scale.

Starting far healthier than goblin did (goblin's baseline was 0/0/2, 17/85/15, 0/69/3).
