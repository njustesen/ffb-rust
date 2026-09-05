# Ogre — heuristic-agent parity campaign (8/9)

**Status 2026-09-05: ONE gate open** (bb2020 @0, two seeds). Started after nurgle (`117a532cd`).

Note there is an older *random*-agent ogre campaign that reached 100/100 (memory
`parity_roster_progression`); this is the **heuristic** nine-gate version and is a different bar.

## Surface

Snotling (Dodge, **Right Stuff**, Sidestep, Stunty, **Titchy**), Ogre (**Bone Head**, Mighty Blow,
Thick Skull, **Throw Team-mate**), Ogre Runt Punter (Bone Head, **Kick Team-mate**, Mighty Blow,
Thick Skull).

Both TTM and KTM on one roster, plus a negatrait and Stunty/Titchy landing targets — the heaviest
throw/kick surface in the sweep.

## Baseline (measured on `e025e6f0d`, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | 98 (seeds 73, 91) | **100** |
| bb2025 | **100** | **100** | **100** |

Random controls: bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.

## The open red — bb2020 @0 seed 73, localised to the landing square

Resolving activation **i=231** (`home_01, KickTeamMate`), declared identically by both engines. The
state at i=232 differs in exactly **one token**:

```
R  h09:-1,-1,Ko,5/1/3/6      J  h09:-1,-1,Bh,5/1/3/6
```

h09 is a **Snotling** (Stunty/Titchy) — the kicked player. Per-die comparison of that activation:

| pos | Java | Rust |
|---|---|---|
| 260, 261 | Bone Head ×2 | same values |
| 262 | KTM skill roll | same |
| 263, 264, 265 | scatter d8 = 2, 7, 1 | **same 2, 7, 1** |
| 266, 267 | **InjuryTypeCrowd** 6, 5 → 11 | 6, 5 (a DIFFERENT player's injury) |
| 268, 269 | casualty **d16**=4, d6=3 → Badly Hurt | more **d8** scatters |

So with **identical scatter dice** Java's kicked Snotling ends **out of bounds** (crowd injury →
casualty → Badly Hurt) while Rust's ends **on the pitch** (a landing injury; an `interpret_and_set_injury`
probe showed h09 rolling d=[1,4] → total 5 → Stunned, converted to KO by
`stun_is_treated_as_ko`).

**Ruled out already — do not re-chase:**

* `kick_player` is line-for-line identical to Java's `UtilThrowTeamMateSequence.kickPlayer`
  (same `signum` walk, same `isInBounds` guard, same early return).
* `scatter_player` is line-for-line identical to Java's `scatterPlayer` (same break condition, same
  `lastValid = startCoordinate` on going out of bounds, same `startCoordinate = endCoordinate`).
* The direction lookup matches: Java's `interpretScatterDirectionRoll(game, roll)` is just
  `DirectionFactory.forRoll(roll)` with no game/edition transform, i.e. Rust's `Direction::for_roll`.
* Stunty is NOT the cause: Java's Badly Hurt comes from a **casualty** roll, not from the
  9-and-stunty injury-table row.

**So the divergence is the square the scatter STARTS from**, and it has been narrowed to one
assignment.

* A probe on `kick_player` never fired: the live `StepId::InitScatterPlayer` (`driver.rs:240`) is the
  **bb2025** twin, which imports only `scatter_player`. The bb2020 twin — the one carrying the
  `isKickedPlayer && throwScatter → kick_player` branch at its line 114 — is **DEAD**, the usual
  dead-file trap. Java's own stack shows `scatterPlayer` called directly too, so `kick_player` is
  probably not the missing piece; the dead twin is recorded because a probe there proves nothing.
* Java bb2020 `StepInitScatterPlayer:183-191`:

  ```java
  FieldCoordinate startCoordinate = thrownPlayerCoordinate;
  if (deviate) { ... }
  else if (throwScatter) { ...; startCoordinate = game.getPassCoordinate(); }
  ```

* Rust's live twin has the same branch, but guarded:
  `else if self.throw_scatter { if let Some(pc) = game.pass_coordinate { start_coord = pc; } }`.

**When `game.pass_coordinate` is `None`, Rust silently keeps the thrown player's square while Java
would take the pass coordinate.** That is the prime suspect: for a KICK Team-Mate the target square
may never be written into `pass_coordinate` on the Rust side.

**Next step**: print `game.pass_coordinate`, `throw_scatter`, `is_kicked_player` and the resolved
`start_coord` on both sides at seed 73 i=231. If Rust's is `None`, find where a KTM should set it
(Java's `StepKickTeamMate` / the KTM sequence) rather than patching the guard — the guard is a
symptom. Seed 91 is unclassified; check it is the same family before assuming.

## Not yet done

Coverage harvest, seed 91 classification, and the ninth gate. This race is NOT closed.
