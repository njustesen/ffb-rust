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
* the **move replay also agrees**. `FFB_MOVEP` (Rust `RMOVEP` / Java `JMOVEP`, a mirror that
  already existed) shows both engines answering the first move prompt with the **identical
  5-square path** `[20,5 19,4 18,3 17,2 16,3]`, both arriving at (16,3), and both being prompted
  again there.

**Layer 5 — WRONG, and corrected here: "Rust walks 5 steps where Java walks 4".** That came from
counting `JAVA_GFI` lines (which stop at `currentMove=4`) instead of reading the move answer.
`JMOVEP` shows Java delivering all five squares. Do not count GFI trace lines to infer path length.

**Layer 6 — narrowed to the move, and the pass exonerated.** `getCallCount()` is per-DIE and equals
`DICE_TRACE`'s `pos`, so the two engines' spends during activation i=20 are directly comparable:

| | dice in i=20 | breakdown |
|---|---:|---|
| Java | **4** | 1 GFI, pass=**2** (FUMBLE), 2 bounce d8 |
| Rust | **7** | **2 GFI**, pass=**5** (INACCURATE), 3 scatter d8, 1 catch |

A step-level probe on `StepPass.execute_step` shows it is entered **twice** and rolls the pass die
**exactly once** in both engines (`roll=0 → roll=5`, then a re-entry after the declined offer with
`source=None`, which correctly does NOT re-roll). The re-roll banks are `r2,1` on both sides at
i=19, 20 and 21, so **no team re-roll is consumed by either engine**. The pass step is therefore
NOT at fault: it rolls one die, and that die differs only because the stream is already one ahead.

**The remaining fact: Rust rolls TWO d6 during the 5-square walk where Java rolls ONE**, for an
identical path. With MA=4 and five steps, `StepGoForIt`'s rule (`going_for_it && current_move > ma`)
should rush exactly once, and `RUST_GFI` does report `MA=4` with `currentMove=1..5`. So one of
Rust's two move d6 is something other than the expected single rush.

**Next tool, and it is an instrumentation gap, not a guess.** Java's `DICE_TRACE` carries a
`caller=` stack that names the rolling step; **Rust's does not** — it prints only `pos/sides/result`.
That is why every attempt above had to infer a die's owner from arithmetic, and it is why layers 1
and 5 went wrong. Add a step tag to Rust's dice trace, then read off which step rolls Rust's second
d6 in that walk. Do not attribute another die by counting.

**What is actually left (superseded — see Layer 6).** Both engines choose the same plan, walk the same path, and arrive at the
same square — yet Rust's pass roll is `pos=46`=**5** (INACCURATE, ball to (14,7)) and Java's is
`pos=45`=**2** (FUMBLE, ball to (17,4)). Rust spends **exactly one more die** than Java between the
start of the game and the throw, and the dice values agree up to `pos=43`. Java's `pos=40..44` are
five consecutive `rollGoingForIt` calls, so the extra Rust die is somewhere in `pos=44..45` during
this walk.

Pinning it needs a **per-activation dice marker** on both sides — the current traces cannot say
which activation a given `pos` belongs to, because Java's `rng_calls` counts CALLS while `pos`
counts DICE. That marker is the next iteration's first job; without it every `pos`-based inference
here is guesswork, which is how layers 1 and 5 went wrong.

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
