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

## The remaining frontier — both engines pick the SAME plan, then execute it differently

bb2025 remains **68 / 83 / 78**. This session drove the frontier down four layers with matched
probes on BOTH engines (the Java harness is co-editable). Every layer is recorded because each one
**disproved the layer above it** — three of my own conclusions were wrong and are corrected here.

**Layer 1 — WRONG: "Rust spends one extra die before the pass".** The dice trace with `caller=`
stacks shows Java and Rust agreeing value-for-value on `pos=40..45`.

**Layer 2 — WRONG: "the pass re-roll ask at `step_pass.rs:562` is the cause".** It does use the
acting-player overload with a hard-coded `"TRR"` (the same shape as the human Dodge/Tackle and
nippon trapdoor fixes) and is worth fixing on its own, but Rust *does* offer and the agent
*declines* — `LOOP applied=NoReRoll` — so it is downstream.

**Layer 3 — WRONG: "exposure differs" / "the support raster differs".** Both came from comparing
probe streams **positionally** when the two engines emit different numbers of entries. Gating both
probes to the single activation (`FFB_CAND`) made them agree: exposure, `threat_reach`,
`threat_str`, `threat_mark` and the mover are all identical.

**Layer 4 — the verified fact.** At the deciding activation (k=22, seed 6):

* the candidate lists are **IDENTICAL** — 1740 rows, same pid/pac/target/dest/weight throughout;
* both engines **pick the same row**: `idx=1633, away_10, Pass, tgt=away_05, dest=94` — i.e. run to
  **(16,3)** and throw to away_05. Rust's stored path is **5 squares**;
* yet **Rust executes 5 move steps (two rushes) and Java executes 4 (one rush)**. Rust's extra rush
  consumes `pos=45`, so Rust's pass roll is `pos=46`=**5** (INACCURATE, ball to (14,7)) where Java's
  is `pos=45`=**2** (FUMBLE, ball to (17,4)).

So this is **not** planning, scoring, candidate order, or the value model. Same plan in, different
number of executed steps out. The next iteration should instrument the move REPLAY — how many
squares of the delivered path each engine actually walks, and each side's movement-left/rush budget
at the moment of the 5th step (both agree the player has 3 free movement, since Java's 4th step is
already a rush).

### Landed this session

`heuristic_agent.rs` Pass candidates now record the run-up square as `dest` (Java's
`PlanBuilder.passCandidates` does; Rust passed `None`). **Behaviour-neutral** — `dest` is only a
path fallback and the run-up path was already stored — and the gates confirm it: 68/83/78 before and
after. It is kept for *reporting fidelity*: every Pass row read `dest=null`, so no Pass candidate
could be compared against Java's. That blindness is what sent layers 1-3 chasing consequences, and
it is the same failure mode as `FFB_CANDSUM`'s `n` (a count that matched while the contents did not).

## Not yet done

Random controls, closed-roster regressions for necromantic itself, coverage harvest, and the nine
gates. This race is NOT closed.
