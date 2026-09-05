# Necromantic — heuristic-agent parity campaign (IN PROGRESS)

**Status 2026-09-05: OPEN.** bb2016 and bb2020 are fully green; bb2025 has a 32-seed frontier.
Two real engine bugs found and fixed, neither gate-moving. Recorded honestly so the next iteration
starts from fact, not from a number.

## Baseline (measured on `ff1a6ee00`, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** | **100** |
| bb2025 | 68 | 83 | 78 |

**@0 is red**, so unlike lizardman this is a resolution/content divergence, not purely a draw-count
split. All reds are bb2025-only.

## Surface

bb2025 necromantic is the richest roster in the sweep so far: Zombie (Eye Gouge, Regeneration,
**Unsteady**), Ghoul (Dodge, Regeneration), Wraith (Block, Foul Appearance, **No Ball**, Sidestep),
Flesh Golem (Stand Firm, Thick Skull, **Unsteady**), Werewolf (Claws, Frenzy, Regeneration).
Plus the **Masters of Undeath** special rule — this is the only race in the sweep that raises the
dead, and Java's harness already carries a necromantic-specific SecureTheBall/Unsteady fix.

## Two fixes landed (`3f9f2b37b`) — correct, but NOT gate-moving

Both found with `FFB_IDSTATE`, and both are invisible to the state hash, which clamps every
off-pitch coordinate to (-1,-1) and hashes only the first 11 players by nr.

1. **The regenerated player was never boxed.** `handle_regeneration_reporting` set the state to
   RESERVE but Java's `handleRegeneration` success branch also clears the player result's serious
   injury and calls `UtilBox.putPlayerIntoBox` + `refreshBoxes`. Rust left the regenerated Werewolf
   in the RIP box column (x=-5) while its state read RESERVE; Java had it at x=-1.

2. **A zombie was raised for a player who had just got back up.** Java's `handleRegeneration`
   javadoc says *"Callers need to apply that to the injury context themselves"*, and every Java call
   site sets `injuryContext().setInjury(...)`, `setApothecaryStatus(RESULT_CHOICE)`,
   `setSeriousInjury(null)`. Rust did not, so the context still read RIP after a SUCCESSFUL
   regeneration and `handleRaiseDead` (which gates on `injuryContext.playerState == RIP`) raised a
   zombie anyway. **Rust fielded 25 players where Java had 24.**

Verified by `FFB_IDSTATE`: home_01's box now matches and the phantom `home_01R1` is gone; the boards
agree completely at i=19..22.

**They did not move the gate.** bb2025 @1.0 stayed 68/100 and the failing seed set is byte-identical
before and after — 32 seeds, zero fixed, zero newly broken. Kept because they are 1:1 with the Java
and fix a real 25-vs-24 roster difference, not because they improved a number. Regression: khemri
(Regeneration on all 12) 60/60 unchanged; `ffb-engine` 7419/0.

## ITER2 — CLOSED. One fix took bb2025 from 68/83/78 to 100/100/100

**Root cause.** Java only ever SETS `game.defenderId` for an action that can block —
`StepInitBlocking` (block), `StepInitFouling` (foul), `StepEndMoving` (the blitz's block defender).
A `PASS_MOVE` / `HAND_OVER_MOVE` leaves it null, so Java's `getDefender()` is null inside
`StepFoulAppearance` and it rolls nothing.

Rust reuses `Action::ActivatePlayer.block_defender_id` as a **generic target channel**
(`step_init_selecting.rs:382` — TTM and the give chain need it), so a pass **receiver** lands in
`game.defender_id`. `StepFoulAppearance` then rolled a Foul Appearance check **against our own
receiver** whenever that receiver happened to have the skill — one d6 Java never spends, putting
Rust's entire dice stream one ahead for the rest of the game.

Seed 6, i=20: away_10's PassMove resolved `defender=away_05`, a **Wraith** (Foul Appearance).
Rust's die 44 is `StepFoulAppearance`; its pass roll therefore lands at 46 = **5** (INACCURATE, ball
to (14,7)) where Java's is 45 = **2** (FUMBLE, ball to (17,4)).

**Fix**: gate the step on the acting ACTION rather than on the channel, so the legitimate
`BLITZ_MOVE` case (whose defender `StepEndMoving` publishes) is untouched. Also mirrors Java's
strict either/or defender resolution — when a `TargetSelectionState` exists it is the ONLY source,
even if its selected id is null; Rust had filtered on `is_selected`/`is_committed` and then fallen
back to the stale `game.defender_id`.

**How it was found — and five wrong turns worth remembering.** Layers 1-5 all chased consequences,
and each was disproved by the next:

1. "Rust spends an extra die *before* the pass" — the dice agree value-for-value to `pos=43`.
2. "The pass re-roll ask at `step_pass.rs:562` is the cause" — Rust offers, the agent **declines**,
   and a step probe shows the pass die is rolled exactly **once**; the re-roll banks are `r2,1` on
   both sides throughout.
3. "Exposure differs" and 4. "the support raster differs" — both artefacts of comparing probe
   streams **positionally** when the two engines emit different entry counts. Gating both probes to
   one activation made every raster agree.
5. "Rust walks 5 squares where Java walks 4" — inferred from counting `JAVA_GFI` lines, which stop
   at `currentMove=4`. `JMOVEP` shows Java delivering the identical 5-square path.

What actually cracked it was **`FFB_DIE_AT`**, which prints a Rust backtrace at a chosen die index:
die 44 = `StepFoulAppearance`, die 45 = `StepGoForIt`, die 46 = `StepPass`. Every earlier layer had
tried to attribute a die by arithmetic. **Attribute dice with the backtrace, not by counting.**

## Gates

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** | **100** |
| bb2025 | **100** | **100** | **100** |

Random controls: bb2016/bb2020/bb2025 **100/100** each. `cargo test -p ffb-engine` **7422/0**.

Closed-roster regressions, bb2025 @1.0, seeds 1-100 — the step is shared by every race and edition:
chaos, chaos_dwarf, nurgle, lizardman, nippon, khemri, human, goblin, amazon, dark_elf, elf, dwarf
— **all 100/100**.

Three colocated tests had set the roll up with no blocking action; corrected FROM THE JAVA, since
the roll is only reachable under a block/blitz/foul. New test
`pass_move_does_not_roll_foul_appearance_against_its_receiver` asserts **zero** dice are spent.

Coverage harvested ×3.

**🏁 necromantic CLOSED.** Frontier empty.
