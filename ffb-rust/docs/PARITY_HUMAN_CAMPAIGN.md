# Human — heuristic-agent parity campaign

**Goal**: human v human, HeuristicAgent both sides, per-step state-hash parity 100/100, seeds 1-100,
tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus random controls and the
standing regression set. Procedure: `.claude/commands/amz-iter.md` with `MATCHUP=human`.
Started 2026-09-04, after goblin (`fc58b0e1c`), halfling (`e6571447c`) and high_elf (`77d99aacb`).

## Surface

Roster in all three editions, and it is the **Ogre + Halfling Hopeful** mix that matters:

- bb2016 (12): Ogre ×1, Blitzer ×4, Catcher ×4, Thrower ×2, Lineman ×1.
- bb2020 (12): Ogre ×1, Blitzer ×4, Thrower ×2, Catcher(old) ×1, **Halfling Hopeful ×3**, Lineman ×1.
- bb2025 (12): Ogre ×1, Blitzer ×2, Catcher ×2, Thrower ×2, Lineman ×2, **Halfling Hopeful ×3**.

The Ogre brings Bone Head (a Select-sequence negatrait) and Throw Team-Mate; the Halfling Hopefuls
bring Right Stuff/Stunty — i.e. the same TTM/Right Stuff and negatrait machinery that goblin
(ITER19/ITER21) and halfling (ITER5) just hardened, which is very likely why this race starts
almost green.

Prior art: there is an older **tier-1 / full-human** campaign in this repo that took human-vs-human
to 100/100 under the *random* agent and the earlier tiers (`docs/PARITY_TIER1.md`,
`docs/PARITY_TTM.md`, memory `parity_tier_human.md`). This campaign is the *heuristic*-agent
nine-gate version of that, and it inherits the benefit.

## Baseline (2026-09-04, measured on `77d99aacb`, nine gates, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** ✅ | **100** ✅ | **100** ✅ |
| bb2020 | **100** ✅ | 99 | **100** ✅ |
| bb2025 | **100** ✅ | **100** ✅ | 96 |

**Only 5 reds across the nine gates — seven gates already green.** The smallest frontier of the
sweep so far (goblin opened at 221-equivalent, high_elf at 221, halfling at 45).

Exact reds:
- **bb2020 @0: seed 45** — a single argmax red. Argmax consumes no sampler draws, so this is a
  resolution/content divergence, not a draw-count split: `first_state_divergence.sh` is the right
  entry point.
- **bb2025 @1e6: seeds 3, 51, 60, 66** — red ONLY at @1e6 (both @1.0 and @0 are 100/100). A fault
  that appears only at the uniform-sampling scale is a draw-count/candidate-count split, so read
  `FFB_CANDSUM` (`RSUM`/`JSUM` `n=` and `draws=`) at the first differing activation before tracing
  dice. Note @1e6 makes the softmax uniform over the SAME option set, so a divergence here with
  @1.0 green points at candidate COUNT/order rather than weights.
