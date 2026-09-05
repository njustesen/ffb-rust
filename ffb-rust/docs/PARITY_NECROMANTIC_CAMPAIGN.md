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

## The remaining frontier — localised, not yet root-caused

Seed 6 is the earliest red. `first_state_divergence.sh` puts the split at **i=20**, an activation
both engines declare identically (`away_10, PassMove`). After the two fixes above the full boards
agree at i=19..22, so what differs is NOT player state:

* state string at i=21: ball `b14,7` (Rust) vs **`b17,4`** (Java) — Java's ball lands one square
  from the thrower, i.e. Java **FUMBLED**; Rust threw INACCURATE and scattered away.
* `rng_calls` agree through i=20 (43 both) and split at i=21 — R50 vs J47, so **Rust spends 3 extra
  dice** resolving the pass.
* `RUST_STEPPASS` shows dist=ShortPass, roll=5 → INACCURATE. Java's pass die (`DICE_TRACE` pos=45,
  `rollSkill` / `StepPass.executeStep:221`) is **2**.
* The `evaluate_pass` implementations are byte-identical to Java's bb2025 `PassMechanic`, so the
  divergence is in the **inputs**, not the interpretation.

**The sharpest single fact for the next iteration**: at the PassMove's first movement step Java
logs `JAVA_GFI ... action=PASS_MOVE currentMove=1 rng=43` while Rust logs
`RUST_GFI ... action=PassMove currentMove=1 rng=44`. **Rust has already spent one extra die before
the pass begins.** Start there — the pass roll itself is then drawn from a shifted stream, which is
enough to turn Java's fumble into Rust's inaccurate pass. Note Java `rng_calls` are per-CALL and
Rust's are per-DIE, so align on cumulative totals rather than assuming a 1:1 die index.

## Not yet done

Random controls, closed-roster regressions for necromantic itself, coverage harvest, and of course
the nine gates. This race is NOT closed.
