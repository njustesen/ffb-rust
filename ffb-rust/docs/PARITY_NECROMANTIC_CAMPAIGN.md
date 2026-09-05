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

## The remaining frontier — now localised to TWO SQUARES

Re-measured on `1566f936b` after the lizardman/nippon fixes: bb2025 still **68 / 83 / 78**,
unchanged. The frontier was chased down several layers this session; each layer is recorded because
each one disproved the layer above it.

**Layer 1 (wrong).** The earlier handoff said "Rust spends one extra die before the pass". The dice
trace disproves it: with callers, Java and Rust agree value-for-value on `pos=40..45` — five GFI
rolls and the pass roll (both **2**). They split at `pos=46`, where **Rust rolls a d6** and Java
rolls a d8 (`bounceBall`). Rust re-rolls the pass and gets 5 → INACCURATE; Java keeps the 2 →
FUMBLE, ball to (17,4). So it is not an extra die *before* the pass.

**Layer 2 (also not the cause).** `bb2025/pass/step_pass.rs:562` calls the ACTING-PLAYER re-roll
overload and hard-codes `re_roll_source = "TRR"` — the same shape as the human ITER1 Dodge/Tackle
fix and the nippon trapdoor fix. It is worth fixing on its own merits, but it is downstream: Rust
makes **no re-roll draws at all** on this seed, so the pass re-roll is a consequence, not the cause.

**Layer 3 — the actual first divergence.** `FFB_CANDSUM` reports its first split at k=25, but that
is already downstream. At **k=24** the summary line matches (`n=1375` both) while the candidate
CONTENT differs from index 0 — **`n` is a count, so it hid the difference**. Dumping the lists with
`FFB_CAND=24`:

* home_03 (a **Flesh Golem**, moving from (13,14)) has **exactly the same 85 destinations** in both
  engines — the reachable SET is right.
* The ORDER differs, because **exactly two of the 85 carry a different weight**:

  | dest | square | Rust | Java |
  |---|---|---:|---:|
  | 276 | (16,10) | **0.026214** | 0.014563 |
  | 277 | (17,10) | **0.026214** | 0.014563 |

  The other **83 match bit-for-bit**, so `DetMath` and the shared scoring are sound — this is not a
  float-drift problem.

Rust's higher weight makes (16,10)/(17,10) sort to the front, so home_03 walks to **(10,11)** where
Java walks to **(11,11)** (both 3 steps, diverging on the last one). That one-square difference is
what the state hash first shows at i=23, and it cascades into the different pass a few activations
later.

**Next step**: `arrival_parts` is `pa*v − (1−pa)*c_turnover(gfi) − rush_penalty(gfi)`. Neither
square is in a tackle zone (the nearest opponent, away_06 at (14,10), is two away), so the suspects
are the route's `gfi` (rush count) or `cost` for those two cells, which the goblin campaign already
identified as route-dependent: `cost[]` is NOT minimised, because Dijkstra settles on the
probability key, so which route wins decides the step count. Put a matching probe on BOTH sides —
Rust's `arrival_parts` and Java's `Plans`/`Reach` equivalent (the harness is co-editable) — and diff
`pa`, `v`, `gfi` and `cost` for cells 276/277 at k=24. Do NOT chase the pass again; it is
three layers downstream.

## Not yet done

Random controls, closed-roster regressions for necromantic itself, coverage harvest, and the nine
gates. This race is NOT closed.
