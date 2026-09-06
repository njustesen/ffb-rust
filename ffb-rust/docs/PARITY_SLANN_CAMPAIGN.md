# Slann — heuristic-agent parity campaign

Mirror matchup `--home slann --away slann`, tier 3, seeds 1-100, `--agent heuristic
--heur-classes all`, scales 1.0 / 0 / 1e6, in bb2016 / bb2020 / bb2025.

## Surface — read from `data/rosters/*/roster_slann.json`, not from the brief

| position | ma/st/ag/pa/av | skills (identical in all three editions except the Bone Head spelling) |
|---|---|---|
| Lineman | 6/3/3/4/8 | **Leap**, **Very Long Legs** |
| Catcher | 7/2/4/4/7 | **Diving Catch**, Leap, Very Long Legs |
| Blitzer | 7/3/3/4/8 | **Diving Tackle**, **Jump Up**, Leap, Very Long Legs |
| Kroxigor | 6/5/**1**/0/9 | Bone-Head (bb2016) / Bone head (bb2020+bb2025), **Loner**, Mighty Blow, **Prehensile Tail**, Thick Skull |

The team spec (`data/teams/*/team_slann.json`) fields 1 Kroxigor, 4 Blitzers, 3 Catchers,
3 Linemen, 2 re-rolls, **no star players**.

Two roster facts turned out to be what the campaign was actually about, and neither is Leap:

* **Diving Tackle** — `grep -rln "Diving Tackle" data/rosters/` returns only slann,
  slann_fumbbl (all editions) and bb2025 dwarf. slann is the first race in the sweep to put
  four Diving Tacklers on the pitch in **bb2020**.
* **AG 1 on the Kroxigor** — the only value in the sweep for which `Math.max(2, agility + …)`
  actually clamps.

## Baseline (measured on `c6f83644d`, the commit that closed skaven)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 100/100 | 100/100 | 100/100 |
| bb2020 | 53/100 | 87/100 | 55/100 |
| bb2025 | 82/100 | 97/100 | 91/100 |

bb2016 green at baseline was the first real clue: whatever was wrong lived in code bb2016
does not run. `driver::make_step_for` routes `StepId::MoveDodge` to the bb2016 twin for
bb2016 and to the **bb2025** file for both bb2020 and bb2025.

## ITER1 — the Diving-Tackle what-if minimum was clamped twice (`f7aeee8d1`)

**Where it showed.** bb2025 @0 seed 43, first state divergence at step index 85; the
resolving activation is i=85, `Activate(home_01, MOVE)` — the Kroxigor. `FFB_TRACE`'s
`RUST_STEP`/`JSTEP` state strings differ in exactly one field at i=86:

```
J  … r2,2 …      rng_calls 75 → 77   (BoneHead d6, dodge d6=3)
R  … r1,2 …      rng_calls 75 → 79   (BoneHead d6, dodge d6=3, Loner d6, fresh dodge d6=5)
```

Rust spent a team re-roll Java never offered. `RDODGEMIN` shows Rust's dodge itself
SUCCEEDED (`roll=3 min=2 ok=true`) and was then re-rolled anyway — the pre-emptive
Diving-Tackle offer.

**Root cause.** Java bb2025 `StepMoveDodge:415-441` builds the what-if by adding the +2
`DIVING_TACKLE` `DodgeModifier` to the modifier **set** and re-entering
`AgilityMechanic.minimumRollDodge`, which is `Math.max(2, agility + sum(modifiers))`. The
clamp is applied **once**, to agility + everything. Rust added `+2` to the already-clamped
minimum. The two functions agree whenever the clamp does not bind — which is why 26 closed
races never saw it — and differ exactly when `agility + modifiers < 2`: an AG1 dodger whose
destination square has no tacklezone. Java: `max(2, 1+2) = 3`, a roll of 3 succeeds, nothing
is asked. Rust: `max(2,1) + 2 = 4`, the 3 "fails", a team re-roll is offered, the heuristic
takes it, and two extra dice enter the stream.

The Break Tackle + Diving Tackle rescue minimum a few lines below had the identical defect
(Java: `minimumRollDodge(withDtAndBt, btStat)`) and is ported the same way.

**Fix.** `crates/ffb-engine/src/step/bb2025/move_/step_move_dodge.rs` — carry the UNCLAMPED
modifier sum and the agility out of the modifier block alongside the clamped minima, and
compute `min_with_dt = (agility + total + 2).max(2)` / `min_with_dt_bt =
(agility + total + 2 + bt).max(2)`.

**Tests.** `diving_tackle_what_if_minimum_is_clamped_once_for_ag1` (verified to FAIL on the
old expression) plus `diving_tackle_what_if_still_offers_the_reroll_when_the_clamp_does_not_bind`,
which pins the AG3 case that must not change.

**Measured.** bb2025 82/97/91 → **100/100/100**. bb2016 unchanged at 100/100/100.
bb2020 53/87/55 → 56/87/56 — a real but small gain, so bb2020 had a second, dominant defect.

## ITER2 — BB2020 has no pre-emptive Diving-Tackle re-roll at all

**Where it showed.** bb2020 @0 seed 3, `first_state_divergence.sh`: Rust ends the home turn
after i=18 while Java carries on with the Kroxigor. The state strings at i=19 differ in one
player and the re-roll bank:

```
J   h01: 14,7 Standing      r3,3   rng_calls 27 → 28  (one dodge d6=5)
R   h01: 12,4 Prone         r2,3   rng_calls 27 → 31  (dodge d6=5, fresh dodge d6=1, armour, injury)
```

`RDODGEMIN pid=home_02 roll=5 min=4 … ok=true` followed by `roll=1 min=4 … ok=false`: the
same pre-emptive Diving-Tackle re-roll, this time re-rolling a Blitzer's *successful* dodge
into a fall and a turnover.

**Root cause.** The pre-emptive offer is a **BB2025 addition**. The BB2020 Java
`StepMoveDodge`'s success branch is bare `status = ActionStatus.SUCCESS`; `grep
findEligibleDivingTacklers` on `ffb-server/.../step/bb2020/move/StepMoveDodge.java` returns
nothing, and `DIVING_TACKLE` appears there only in the `fUsingDivingTackle` modifier path.
Because `make_step_for` has no `Rules::Bb2020` arm for `MoveDodge`, bb2020 games were running
the BB2025 mechanism.

**Fix.** Edition-gate that one block to `Rules::Bb2025` inside the shared step — the
`handle_cheering_fans` pattern — rather than routing bb2020 to its staler twin.

**Test.** `bb2020_never_pre_emptively_offers_the_diving_tackle_reroll`.

**Measured.** bb2020 @1.0 56 → **100/100**.

## What I got wrong along the way (recorded deliberately)

* I first diffed the two engines' whole `DICE_TRACE` streams positionally and "found" a
  divergence at pos 83 where Java rolled a d8 scatter and Rust a d6. That was **wrong**: the
  Rust trace covers every `GameRng` draw (190 entries) and the Java one only `Fortuna`
  (177), so the streams are not positionally alignable. The campaign's own rule — never diff
  two probe streams positionally unless both emit the same number of entries — applies here
  and I broke it. The real instrument was `FFB_TRACE`'s state strings plus `rng_calls`
  deltas per activation, and `FFB_DIE_AT` for attribution.
* I expected **Leap / Very Long Legs** to be the surface that mattered (the brief said so,
  and it is the rarest skill on the roster). Neither appears in either defect. The whole
  race came down to Diving Tackle and to AG 1.

## Final gates — 🏁 slann CLOSED 2026-09-06

Nine gates, `--home slann --away slann --tier 3 --seeds 1-100 --no-abort --agent heuristic
--heur-classes all`, measured on `933c2cefb`:

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100/100** | **100/100** | **100/100** |
| bb2020 | **100/100** | **100/100** | **100/100** |
| bb2025 | **100/100** | **100/100** | **100/100** |

Random controls (`FFB_PARITY_ROOT=parity_random --agent random`, seeds 1-100), all re-measured
on the final binary: bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.

`cargo test -p ffb-engine`: **7433 passed, 0 failed**, 15 ignored.

Timing at @1.0 (batched JVM, 100 seeds): `rust_total=` 34.4 s (bb2016) / 39.2 s (bb2020) /
38.2 s (bb2025).

### Closed-roster regressions, all **100/100**

bb2025 @1.0: dwarf, nurgle, necromantic, nippon, lizardman, khemri, human, goblin, amazon,
chaos, norse, ogre, orc, renegades, skaven.

The fix is edition-shared, so per the "choose regression races by who carries the skill" rule
(`grep -rln "Diving Tackle" data/rosters/` → slann, slann_fumbbl, bb2025 dwarf) the bb2025
**dwarf** run above is the load-bearing one, plus bb2020 @1.0 **dwarf**, **human** and
**chaos_pact** — all 100/100.

## Coverage — harvested x3, each run alone

`MATCHUP=slann sh scripts/harvest_coverage.sh <edition> 1.0` →
`docs/EVENT_COVERAGE_slann_bb2016.md` / `_bb2020.md` / `_bb2025.md`, each from its own
100/100 run.

bb2025, 96,705 events over 100 games: **confusionRoll 1539** (the Kroxigor's Bone Head fired
in almost every activation it had), dodgeRoll 929, blockRoll 1490, goForItRoll 3876,
catchRoll 171, passRoll 142, pickupRoll 393, touchdown 33, foul 213. bb2020 is comparable
(98,658 events, confusionRoll 1651, dodgeRoll 890). Both defects fixed here live on the
dodge path, and the dodge path ran ~900 times per edition in both engines with identical
per-step state hashes.

### What was NOT exercised — say it plainly

**`jumpRoll` count is ZERO in all three editions**, i.e. across 300 games. `JumpRoll` is a
real `GameEvent` (`ffb-model/src/events/game_event.rs:39`) and the whole Leap / Very Long
Legs surface — the thing the iteration brief expected to matter — **never executed**. The
heuristic agent has no Leap arm at all: `grep -ri leap crates/ffb-engine/src/agent/` returns
nothing, and the only `JUMP` in the agent is the re-roll-action name list, so it can answer a
jump re-roll prompt but never declares a jump — the `StepJump` chain is unreachable from this
harness. Green here is therefore NOT evidence about Leap in any edition.

Also unevidenced: `skillUse` is 0 in every run (that event has only five emitting sites —
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle — none of which slann carries), so
**Diving Catch, Jump Up, Prehensile Tail, Thick Skull and Mighty Blow** produce no positive
event trace either. Prehensile Tail at least shows up indirectly: `RDODGEMIN` lines carry
`"1 for being marked with Prehensile Tail"` on the dodges next to a Kroxigor, and those
dodges are hash-identical between the engines.

Filed for the sweep: **the agent cannot Leap.** That is an agent-coverage gap, not a parity
red, and it will not be closed by any race whose roster carries Leap (slann, slann_fumbbl,
wood_elf) until the agent learns to declare a jump.

Sweep: **27 races closed**. Remaining: slann_fumbbl, underworld, wood_elf (undead and vampire
are outside this sweep).
