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

## Open: bb2020 @1.0 seed 29 — a SEPARATE fault

Localised, not root-caused. First state-string divergence is at **i=52**, and it is a **single
token**:

```
R  a02:19,13,Prone,7/3/3/9,1
J  a02:-1,-1,Reserve,7/3/3/9,0
```

a02 stood at (18,13) through i=51 in both engines. After the i=51 block (`home_01, Block`):

* **Java** removed it from the pitch — a crowd push.
* **Rust** pushed it to (19,13) and knocked it Prone.

(19,13) is comfortably in bounds (the pitch is x 0..25, y 0..14), so this is **not** the sideline
chain-push fault fixed above — Java found no legal pushback square where Rust found one. Next step:
dump the pushback-square set for that block on both sides and compare the candidate squares and
their occupancy, rather than assuming an edge case.

## Not yet done

Random controls, the closed-roster regression set, and coverage harvest. This race is NOT closed.
