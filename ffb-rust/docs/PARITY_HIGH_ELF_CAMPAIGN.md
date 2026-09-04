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

## ITER1 (2026-09-04) — the Steady Footing dialog and its re-roll offer

**bb2025: 0 / 86 / 0 → 95 / 95 / 96.** Two bugs, both in
`crates/ffb-engine/src/step/bb2025/shared/step_steady_footing.rs`, both about PROMPTS rather than
resolution — exactly the fingerprint the baseline predicted.

### How they were named

`FFB_DRAWS=1` on bb2025 @1.0 seed 1, comparing cumulative totals at the classes both engines emit:

```
RDRAW cls=followup total=14                                   JDRAW cls=FOLLOWUP_CHOICE total=14
RDRAW cls=move     total=16     <- no Rust prompt              JDRAW cls=SKILL_USE total=16 skill=SteadyFooting pid=teamHighElfParity25Home1
```

The totals agree up to the follow-up choice and part company at the very first Steady Footing
fall. Java raises `DialogSkillUseParameter`, `ParityRunner`'s `SKILL_USE` arm routes it through
`heuristic.useSkill("SteadyFooting")` (`HeuristicDriver.java:144`, `_ -> 0.50`), and that spends
**two sampler draws**. Rust's `execute_step` had a placeholder:

```rust
if self.use_skill.is_none() { self.use_skill = Some(true); }   // "auto-accept so the random agent rolls"
```

so Rust never asked, never spent the draws, and the two random streams split on the first Dragon
Prince fall of every game. That is why **every** bb2025 sampled seed was red while argmax (which
consumes no draws) was 86/100.

**Fix 1** — raise the prompt, 1:1 with Java's `showDialog(...); setNextAction(CONTINUE)`; the
answer already came back through `handle_command`'s `Action::UseSkill` arm. The offered skill is
now RESOLVED (`getSkillWithProperty(canAvoidFallingDown)`) rather than the `SteadyFooting`
constant, for the dialog and for `ReportSkillUse`, and the eligibility guard is now Java's
`!skill.isPresent()` instead of the property union (bb2025 `BallAndChain` lists
`canAvoidFallingDown` only as a `registerConflictingProperty`, so SteadyFooting is in fact the
only skill that registers it).

That alone took bb2025 @1.0 from **0 → 30** and @1e6 from **0 → 49**. `FFB_DRAWS` on the new
lowest red (seed 4) then showed the second fault:

```
RDRAW cls=skill  total=26 skill=SteadyFooting pid=away_02      JDRAW cls=SKILL_USE total=26 skill=SteadyFooting pid=teamHighElfParity25Away2
RDRAW cls=reroll total=28 src=TRR action=STEADY_FOOTING        (no JDRAW - Java raises nothing)
```

**Fix 2** — Java's failure branch calls the **PLAYER** overload,
`UtilServerReRoll.askForReRollIfAvailable(gameState, player, STEADY_FOOTING, 6, false)`, and
`RollMechanic.isTeamReRollAvailable` gates the TRR on `actingTeam.hasPlayer(pPlayer)`. `away_02`
was a blocked DEFENDER while home was playing, so Java offers no re-roll dialog at all. Rust
called the ACTING-PLAYER overload (`ask_for_reroll_if_available`), which skips that membership
gate and additionally resolves a re-roll source from the *acting* player's skills by action — two
extra sampler draws on every defender's failed Steady Footing. Switched to
`ask_for_reroll_if_available_for(game, Some(player_id), ...)`, the existing 1:1 player overload.

bb2025 @1.0 **30 → 95**, @0 **86 → 95**, @1e6 **49 → 96**.

### Gates measured this iteration (all nine, seeds 1-100 tier 3, fresh JVM, no `--reuse-java`)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 (=) | 99 (=) | 98 (=) |
| bb2020 | **100** ✅ (=) | 99 (=) | **100** ✅ (=) |
| bb2025 | **95** (was 0) | **95** (was 86) | **96** (was 0) |

221 reds → **21**. bb2016 and bb2020 are unchanged, as expected: the file is bb2025-only.

Tests: `cargo test -p ffb-engine` **7411 / 0**, `ffb-model` **2802 / 0**. Three new colocated
tests written from the Java: the undecided offer must ASK and draw no die, the answer arrives as
`Action::UseSkill` and a decline fails the step, and a non-acting-team faller gets NO re-roll
offer while the same faller on the acting team does.

Closed-roster regressions, bb2025 (the only edition this file serves), seeds 1-100 tier 3 @1.0:
goblin **100**, halfling **100**, amazon **100**, lineman **100**, dwarf **100**, chaos **100**,
chaos_dwarf **100**, chaos_pact **100**, dark_elf **100**, dark_elf_league_fumbbl **100**, elf
**100**. goblin and halfling additionally re-run at @0 and @1e6: **100 / 100** each.

### Frontier for ITER2

- bb2025 @1.0 reds: **16, 37, 56, 62, 80**
- bb2025 @0 reds: **22, 73, 86, 91, 99**
- bb2025 @1e6 reds: **54, 72, 88, 100**
- bb2016 reds: @1.0 **49, 91, 94**; @0 **77**; @1e6 **2, 80** — untouched all iteration
- bb2020 reds: @0 **71** only

Next step: `frontier.sh bb2025` over the 14 remaining bb2025 reds and read the families by
declared action; the bb2025 sampled and argmax red sets are now the same size, so the remaining
faults are most likely resolution, not draw counts. The still-unexercised bb2025 surface is
**Cloud Burster**, **My Ball** and **Safe Pass** (Steady Footing is now driven live).
