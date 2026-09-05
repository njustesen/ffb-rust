# Nippon — heuristic-agent parity campaign (8/9)

**Status 2026-09-05: one gate open.** Started after lizardman (`80edc5afd`).

## Surface

Ashigaru (no skills), Samurai (Block), Warrior Monk (Claws), Ninja (Dodge, **Leap**).

## Baseline (measured on `3f9f2b37b`, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | 99 (seed 29) | 99 (seed 37) | **100** |
| bb2025 | **100** | 99 (seed 37) | **100** |

The smallest frontier of the sweep: three reds, one seed each, and **seed 37 fails in two editions**
— a strong hint of a single shared fault, which it was.

## ITER1 — seed 37 (both editions) closed by one fix

`first_state_divergence.sh` at `HEUR_SCALE=0` put the split at i=167, an activation both engines
declare identically. The state strings differ in exactly **three tokens**:

```
R  b1,5   h00:-1,-1,Reserve   h01:0,5,Standing
J  b0,5   h00:0,5,Standing    h01:-1,-1,Reserve
```

Same two players, opposite fates. At i=167 both had h00 at (1,5) and h01 at (0,5), hard against the
sideline. Java pushed h00 into (0,5) and chained h01 into the crowd, the ball following the carrier
to (0,5). Rust sent **h00** — the block's original defender — into the crowd, left h01 standing, and
stranded the ball at (1,5).

**Root cause.** Java reassigns `state.defender = fieldModel.getPlayer(defenderCoordinate)`
(bb2025 `StepPushback:154`) *before* the crowd-push branch, so `remove(state.defender)` takes the
occupant of the starting pushback square. Rust already computes that chain-aware `defender_id` at
line 247 — with a comment from an earlier fix for exactly this confusion (chaos seed 51) — but the
crowd-push branch still read `game.defender_id` for `sameTeam`, the injury target and
`remove_player`. The earlier fix corrected the *push* path and left the *crowd* path behind.

Fix: use the chain-aware `defender_id` in all three places. Test
`chain_crowd_push_removes_the_occupant_not_the_original_defender`, verified failing before and
passing after.

Result: bb2025 @0 99->100 **and** bb2020 @0 99->100 — one change, both editions, because bb2020
runs the bb2025 step.

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | 99 (seed 29) | **100** | **100** |
| bb2025 | **100** | **100** | **100** |

`ffb-engine` 7420/0. Regression spot-check bb2025 @1.0 seeds 1-60: lizardman, chaos, amazon
all 60/60.

## ITER2 — seed 29 closed by three more fixes, all in `StepTrapDoor`

The bb2020 @1.0 red turned out to be the **Treacherous Trapdoor** prayer (trapdoors at (6,1) and
(19,13)) and took three separate faults, each uncovered only once the previous one was fixed.

**(a) A follow-up onto a plain square stole the trapdoor victim.** A block publishes
`PLAYER_ENTERING_SQUARE` twice — once for the player it pushes, again for the attacker's follow-up.
Java assigns `playerId` ONLY when that player stands on a trap door (`StepTrapDoor.java:68`), so the
follow-up cannot overwrite the pushed player who just landed on one. Rust assigned unconditionally,
so the follow-up clobbered it and the step no-oped entirely. Rust's `set_parameter` has no `&Game`,
so the ids are collected and the same filter applied in `execute_step`. Also stores
`PLAYER_WAS_PUSHED`, which Java consumes and Rust previously ignored — the existing comment had
already flagged it.

**(b) The re-roll was asked for the wrong player.** Java asks on behalf of the trapdoor VICTIM
(`askForReRollIfAvailable(gameState, player, ...)`, `:122`). Rust called the ACTING-PLAYER overload,
which re-derives the source from whoever is activated — the ATTACKER. A player pushed onto a
trapdoor is normally an OPPONENT of the acting team, so Java finds no usable re-roll and shows no
dialog while Rust found the acting team's and burned two sampler draws. **Same wrong-overload shape
as the Dodge/Tackle fix in `bb2025/move_/step_move_dodge.rs`** — that makes two races running.

**(c) The no-re-roll path never rolled the fall injury.** Java runs the full `trapDoorTriggered(...)`
there, whose first act is `handleInjury(InjuryTypeTrapDoorFall..., ApothecaryMode.TRAP_DOOR)` — an
armour and an injury roll. Rust published the parameters and removed the player but skipped the
injury, coming TWO DICE short for every un-rerolled fall (i=52: R52 vs J54). The re-rolled branch
already called it.

**Two colocated tests had to be corrected FROM THE JAVA**, not kept: they gave the victim a
coordinate but no team, so they only ever passed because the acting-player overload read the home
TRR regardless of whose player it was. They now put the victim on the home team, which is what
Java's player overload actually requires.

## Final gates

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** | **100** |
| bb2025 | **100** | **100** | **100** |

Random controls: bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.
`cargo test -p ffb-engine` **7421/0**.
Closed-roster regressions bb2020 @1.0 seeds 1-60: lizardman, goblin, chaos, khemri, human all 60/60
(the trapdoor step is shared by every race and every edition).

Coverage harvested ×3.

**🏁 nippon CLOSED.** Frontier empty.
