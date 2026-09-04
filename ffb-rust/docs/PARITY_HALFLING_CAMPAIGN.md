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

## ITER1 — the Catch hook's `stopProcessing` was dropped in bb2025 (7 of 7 bb2025 @0 reds, one unit)

**Frontier.** bb2025 @0 had 7 reds: 14, 33, 37, 44, 49, 86, 92. `first_state_divergence.sh` named
the SAME resolving activation for **all seven**: `Activate(*_02, ThrowBomb)` — jersey 2 is the star
**Cindy Piewhistle** (Bombardier), fielded by the bb2025 halfling team spec. One family, one unit.

**Mechanism** (seed 92, step 24, home Cindy throws at home_04 (7,1)).

* `FFB_DICE_TRACE` first divergence at `pos=57`: Java `rollInjury … InjuryTypeBombWithModifier`,
  Rust `sides=8`. The two engines agreed on `pos=53` (StepPass) and `pos=54` (a d6) and then split.
* `FFB_IDSTATE` at the next step: Java changed home4 → box, home9 → prone, home11 → casualty
  (a bomb exploding at (7,1)); Rust changed only home5 → box.
* `FFB_TRACE` gave the decisive pair. Java: `JAVA_CATCH catcher=…Home4 roll=2 min=3 ok=false
  mode=CATCH_ACCURATE_BOMB` and **no re-roll dialog** — `JSTATE i=25` goes straight from
  `INIT_SELECTING mode=REGULAR` to `APOTHECARY mode=BOMB_HOME`. Rust: `LOOP applied=Activate(
  home_02,ThrowBomb) prompt_after=ReRollOffer{CATCH}` → `applied=UseReRoll` → `BombRethrow{home_04}`
  → `Pass(7,3)`. Rust spent a team re-roll, home_04 caught the bomb and re-threw it.
* `RREROLLW`/`JREROLLW` confirm the asymmetry as a general one, not a one-off: Rust raised a CATCH
  offer at turn 2 and turn 4 that Java never raised. Java's only bomb-catch re-roll in the game was
  the one whose catcher was the **Treeman** home_01 — a player with **no Catch skill**.

**The Java rule.** `bb2025/shared/StepCatchScatterThrowIn.catchBall()` failure path:

```java
boolean stopProcessing = getGameState().executeStepHooks(this, state);
GameOptionBoolean catchForBombs = ...getOptionWithDefault(GameOptionId.CATCH_WORKS_FOR_BOMBS);
if (state.rerollCatch && (!fCatchScatterThrowInMode.isBomb() || catchForBombs.isEnabled())) {
    return catchBall();                       // skill re-roll — EXCLUDED for bombs by default
}
if (!stopProcessing) {                        // <-- the gate Rust dropped
    if (... UtilServerReRoll.askForReRollIfAvailable(gameState, state.catcher, CATCH, minimumRoll, false)) { ... }
}
```

`bb2025/CatchBehaviour.handleExecuteStepHook` returns **true** for any catcher holding **Catch**.
So a Catch player who fluffs a BOMB catch is offered *nothing*: the skill re-roll is excluded for
bombs, and `stopProcessing` suppresses the team re-roll. A catcher **without** Catch (the Treeman)
gets `stopProcessing == false` and is offered the TRR. That is exactly the pattern in the traces.

**The fix** (`crates/ffb-engine/src/step/bb2025/shared/step_catch_scatter_throw_in.rs`): capture the
`bool` that `dispatch::execute_step_hooks` already returns and wrap the team-re-roll offer in
`if !stop_processing { … }`. **The bb2020 twin already had this gate** — bb2025 was a drifted copy.
bb2016's Java has no bomb exclusion, so its `if (state.rerollCatch) return catchBall()` always fires
when the hook stopped processing; its Rust is equivalent and was left alone. Regression test
`failed_bomb_catch_by_a_catch_skill_player_offers_no_team_reroll`, written from the Java above.

**Gates (seeds 1-100, tier 3), before → after:**

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 → **97** | 99 → **99** | 99 → **99** |
| bb2020 | 98 → **98** | 100 → **100** ✅ | 95 → **95** |
| bb2025 | 84 → **96** | 93 → **100** ✅ | 90 → **95** |

45 reds → **21**. Two gates green. No edition regressed (the change is bb2025-only code).
`cargo test -p ffb-engine` 7399/0. `TIMING java_total=60.9s rust_total=37.3s` (bb2025 @0).

**Next.** bb2025 @1.0 (4 reds) and @1e6 (5 reds) — both still red where @0 is green, so those are
now draw-count/sampling splits rather than content. Then bb2020 @1e6 (5) and bb2016 (3/1/1). Run
`frontier.sh bb2025` against `g25_1b.log` first: with the argmax gate green, a family that survives
only under sampling usually points at a candidate-count difference, not a resolution difference.

**Verification (all measured this iteration, all with the positive `PARITY:` line):**
`cargo test -p ffb-engine` 7399/0, `cargo test -p ffb-model` 2802/0.
Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`): halfling bb2016 / bb2020 /
bb2025 **100/100 each**.
Closed-roster regressions @1.0: bb2025 × {goblin, amazon, lineman, dwarf, chaos, chaos_dwarf,
chaos_pact, dark_elf, dark_elf_league_fumbbl, elf} all **100/100**; goblin and elf also 100/100 on
bb2016 and bb2020. Nothing regressed.
