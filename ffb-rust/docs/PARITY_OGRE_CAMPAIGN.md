# Ogre — heuristic-agent parity campaign

**🏁 CLOSED 2026-09-06 (ITER2).** Nine gates 100/100, three random controls 100/100. Two Rust
engine bugs, both edition-blindness in a step/util shared across editions. Started after nurgle
(`117a532cd`).

Note there is an older *random*-agent ogre campaign that reached 100/100 (memory
`parity_roster_progression`); this is the **heuristic** nine-gate version and is a different bar.

## Surface

Snotling (Dodge, **Right Stuff**, Sidestep, Stunty, **Titchy**), Ogre (**Bone Head**, Mighty Blow,
Thick Skull, **Throw Team-mate**), Ogre Runt Punter (Bone Head, **Kick Team-mate**, Mighty Blow,
Thick Skull).

Both TTM and KTM on one roster, plus a negatrait and Stunty/Titchy landing targets — the heaviest
throw/kick surface in the sweep.

## Baseline entering this iteration (`e025e6f0d`)

bb2016 and bb2025 all three scales 100; bb2020 @1.0 and @1e6 100; **bb2020 @0 98** (seeds 73, 91).

---

## ITER2 — the two bb2020 @0 reds, and what the previous localisation got wrong

### Red 1, seed 73 i=231 — `IS_KICKED_PLAYER` must not reach BB2020's `StepInitScatterPlayer`

A Runt Punter KICKS a Snotling. Both engines rolled the **same** three scatter d8 (2, 7, 1) from the
same square (7,1) and both put the Snotling **out of bounds** at (7,0). Then:

* Java rolled a crowd **injury** (d6 6 + d6 5 = 11) and a casualty (d16 4, d6 3) → Badly Hurt;
* Rust rolled **nothing** and knocked the player out.

Rust was using `InjuryTypeKTMCrowd` (armour auto-broken, injury hard-set to KNOCKED_OUT, zero dice);
Java was using `InjuryTypeCrowdPush`. `bb2020/StepInitScatterPlayer:251-252` is
`isKickedPlayer ? new InjuryTypeKTMCrowd() : new InjuryTypeCrowdPush()` — so **Java's
`isKickedPlayer` was false**, in an activation that unambiguously was a Kick Team-Mate.

Why it is always false in stock Java bb2020:

* neither `generator/bb2020/ScatterPlayer` nor `generator/bb2025/ScatterPlayer` lists
  `IS_KICKED_PLAYER` among `INIT_SCATTER_PLAYER`'s parameters — the step can only ever learn it from
  a **published** parameter;
* `bb2020/StepDispatchScatterPlayer` **consumes** `IS_KICKED_PLAYER` in `setParameter` (returns
  `true`, which stops the delivery) and **publishes nothing**;
* `bb2025/StepDispatchScatterPlayer:129` **does** republish it.

So under BB2020 the step's `isKickedPlayer` is permanently false, and both its
`isKickedPlayer && throwScatter → kickPlayer` branch and its `InjuryTypeKTMCrowd` arm are
**unreachable in stock Java**. (The Java stack in the dice trace confirms it: every scatter came
from `StepInitScatterPlayer.executeStep:200`, the plain `scatterPlayer` branch, never from line 194.)

Rust had it twice over: the bb2025 `ScatterPlayer` generator handed `IsKickedPlayer` straight to
`InitScatterPlayer` (**not in Java at all**), and the shared dispatch published it unconditionally.

**Fix** (`step/generator/bb2025/scatter_player.rs`, `step/bb2025/ttm/step_dispatch_scatter_player.rs`):
drop the generator parameter, and gate only the `IsKickedPlayer` publish on `rules != Bb2020`.
Regression test `is_kicked_player_reaches_init_scatter_only_in_bb2025` — verified to fail with either
half reverted.

**Measured, and recorded because it constrains the fix:** suppressing *all three* of the bb2020
dispatch's publishes (`USING_BULLSEYE`, `IS_KICKED_PLAYER`, `OLD_DEFENDER_STATE`), which is literally
what the Java bb2020 twin does, regressed bb2020 @0 to 97 (new reds at seeds 5 and 55). The other two
are value-identical to what is already flowing in Java's bb2020 chain (`OLD_DEFENDER_STATE` is
published further up by `StepInitThrowTeamMate` there, and BB2020 has no bullseye) but only the
shared Rust chain publishes them, so suppressing them loses state Java still has. Only the
`IS_KICKED_PLAYER` publish is gated.

### Red 2, seed 91 i=226 — the serious-injury table was hardcoded to BB2025

Dice were **identical for the whole game** up to the very last MVP draw, yet the state hash diverged
at i=227. The differing token was one player's armour: Java `h01:12,7,Standing,5/5/4/**11**` against
Rust's `5/5/4/**10**`. i=227 is the kickoff after a touchdown, its kickoff event is Cheering Fans, and
the winner's Prayer to Nuffle differed:

```
JAVA_PRAYERPICK side=home shuffled=[12,1,13,9,16,2,11,5,14,10,3,8,15,7,6] pick=4    (Iron Man, +1 AV)
RUST_PRAYERPICK home=true shuffled=[6,8,4,5,16,3,14,2,1,11,13,9,12,15,7]  pick=10
```

Same 16-element list, different permutation ⇒ the two shared `java.util.Collections` streams were at
different positions. Simulating the seeded `JavaRandom` showed Rust's permutation is the one at
**zero** prior draws and Java's is the one at **four** — exactly one `Collections.shuffle` of a
5-element list.

That shuffle is `bb2020/RollMechanic.mapSIRoll`'s
`Collections.shuffle(injuriesWithReduceableStats)`: when the rolled serious injury cannot reduce a
stat the defender still has (a Snotling is already ST 1, so DISLOCATED_SHOULDER is out), Java remaps
it to a random reduceable one. Rust never got there, because
`step/util_server_injury.rs::evaluate_injury_context` called a **local BB2025-only helper**
(`serious_injury_kind_bb2025(casualty_roll[0])`) where Java calls
`rollMechanic.interpretSeriousInjuryRoll(game, injuryContext)` — the *edition* mechanic. A BB2020
game therefore never used the BB2020 SI table, never applied casualty modifiers to it, and never
made those four draws.

**Fix** (`step/util_server_injury.rs`): call `roll_mechanic_for(game.rules)` and its
`interpret_serious_injury_roll` / `interpret_serious_injury_roll_decay(.., true)`, and delete the
local helper. Regression test
`bb2020_serious_injury_uses_the_bb2020_mechanic_and_its_shared_shuffle` asserts both the table and
the four shared-stream draws; verified to fail with the old helper restored.

### Conclusions from the previous iteration that were WRONG

* **"PRIME SUSPECT: the scatter START coordinate / `game.pass_coordinate` is `None` for a KTM."**
  False. A probe printed `pass=Some(FieldCoordinate { x: 7, y: 1 })` at seed 73 i=231 and
  `start_coord` took it. The guarded `if let Some(pc) = game.pass_coordinate` is not the bug.
* **"`kick_player` is probably not the missing piece."** Correct, but for a better reason than the
  one recorded: in BB2020 that branch is *unreachable in Java*, not merely equivalent.
* **The ledger's earlier per-die table for seed 73** attributed the divergence to a landing injury on
  the pitch (`d=[1,4] → 5 Stunned → KO`). Both engines actually put the player out of bounds; the KO
  came from the dice-free `InjuryTypeKTMCrowd`, not from a landing roll.
* The comment introduced by the earlier `use_ktm_crowd` fix (ogre bb2020 seed 7 i=111) claimed Java
  knocks a kicked Snotling out via `InjuryTypeKTMCrowd`. It cannot: that arm is dead in bb2020. The
  `use_ktm_crowd` code is kept, because it is a faithful port of Java's line 251-252 — it is simply
  dead now, as it is in Java. Whatever really fixed seed 7 was something else in that change set.

### Known remaining 1:1 gap (not a parity red)

Java's `mapSIRoll` also does `injuryContext.setOriginalSeriousInjury(originalInjury)` before the
remap. Rust's `map_si_roll_bb2020` takes `&InjuryContext` and cannot. Nothing in the nine gates or
the random controls sees it; noted here rather than fixed blind.

## Gate (2026-09-06, this iteration's binary; jar unchanged)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** (was 98) | **100** |
| bb2025 | **100** | **100** | **100** |

Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`): bb2016 **100/100**,
bb2020 **100/100**, bb2025 **100/100**.

`cargo test -p ffb-engine`: **7424 passed, 0 failed** (7422 + the two new regression tests).

No Java tree was touched this iteration, so no jar rebuild and no `check_java_trees.py` run was
needed.

Timing: `rust_total=` 28.9–53.5s per 100-seed gate (bb2016 @1.0 → bb2020 @0).

Closed-roster regressions, bb2025 @1.0 seeds 1-100, all **100/100**: nurgle, necromantic, nippon,
lizardman, khemri, human, goblin, amazon, chaos, dwarf. Because both fixes are in edition-shared code
that BB2020 also runs, bb2020 @1.0 was regressed as well, all **100/100**: dwarf, goblin, chaos,
undead, norse.

Coverage harvested ×3 → `docs/EVENT_COVERAGE_ogre_{bb2016,bb2020,bb2025}.md`.
