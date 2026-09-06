# Parity campaign — underworld (heuristic agent)

Mirror matchup `--home underworld --away underworld`, tier 3, seeds 1-100, `--heur-classes all`,
all three editions × scales `1.0` / `0` / `1e6`.

## 🏁 CLOSED 2026-09-06 — GREEN AT BASELINE, no engine change

Baseline was taken on `73891f3e6` (the commit that closed `slann_fumbbl`). All nine parity gates
and all three random controls were **100/100 on the first measurement**. No Rust engine file was
edited for this race; there is therefore no fix, no regression test, and no Java harness change in
this entry. This is the second race in the sweep (after `slann_fumbbl`) to need zero engine work.

### Gate numbers actually run

| Edition | @1.0 | @0 | @1e6 | random control |
|---|---|---|---|---|
| bb2016 | 100/100 | 100/100 | 100/100 | 100/100 |
| bb2020 | 100/100 | 100/100 | 100/100 | 100/100 |
| bb2025 | 100/100 | 100/100 | 100/100 | 100/100 * |

\* `rand25` and `s6_16` print the `…, but required coverage items are MISSING` trailer — that is the
tier-3 coverage checklist, not parity, and is a PASS for the parity half.

`TIMING … rust_total=` at @1.0: **31.1 s** (bb2016) / **39.0 s** (bb2020) / **40.1 s** (bb2025),
against `java_total=` 80.5 / 90.0 / 89.1 s.

`cargo test -p ffb-engine`: **7433 passed, 0 failed**, 15 ignored.

Closed-roster regressions, bb2025 @1.0 seeds 1-100, heuristic, all **100/100**:
nurgle, necromantic, nippon, lizardman, khemri, human, goblin, amazon, chaos, dwarf, norse, ogre,
orc, renegades, skaven. (Confirmatory only — nothing in the engine changed.)

Coverage harvested ×3: `docs/EVENT_COVERAGE_underworld_bb2016.md`, `_bb2020.md`, `_bb2025.md`.

### Why underworld needed no work, stated honestly

The race's surface is the TTM + Right Stuff + negatrait chain plus Animosity. Every one of those
mechanisms is *exercised* here (numbers below), and every one of them was already load-bearing for
a race closed earlier in this sweep: ogre and goblin for Throw Team-Mate / Right Stuff / Always
Hungry, orc and chaos_pact and renegades for Really Stupid / Animal Savagery / Animosity, skaven
for the Skaven positionals. Underworld is the first roster that puts them on **one** team, and that
combination produced no new divergence.

The drafted squads differ sharply per edition, and that is what shapes coverage:

- **bb2016** (`data/teams/bb2016/team_underworld.json`): 2 Mutant Rat Ogres, 2 Warpstone Trolls,
  2 Blitzers, 1 Thrower, 3 Linemen, **1** Goblin.
- **bb2020**: 1 Mutant Rat Ogre, Blitzer, Thrower, Gutter Runner, 3 Clanrats, 6 Goblins.
  **No Troll** — so the bb2020 squad has **no Throw Team-Mate carrier at all**.
- **bb2025**: 1 Warpstone Troll, Blitzer, Gutter Runner, Thrower, 3 Linemen, 1 Goblin,
  6 Snotling Linemen. **No Rat Ogre** — so bb2025 has no Animal Savagery carrier.

Measured event counts that prove the real roster is being fielded (not a lineman fallback):

| | bb2016 | bb2020 | bb2025 |
|---|---:|---:|---:|
| `confusionRoll` (Really Stupid) | 2777 | 0 | 1811 |
| `animalSavagery` | 0 | 1511 | 0 |
| `throwTeamMateRoll` | 22 | 0 | 28 |
| `ThrowTeamMate` (action) | 546 | 0 | 276 |
| `rightStuffRoll` | 0 | 0 | 18 |
| `alwaysHungry` | 0 | 0 | 33 |
| `animosityRoll` | 0 | 0 | 11 |
| `regenerationRoll` | 21 | 0 | 2 |

Every zero in that table is explained by the squad composition above, not by a dead code path —
except the two bb2016 ones, which are the known edition-wide bb2016 coverage hole (below).

### A conclusion of mine that was WRONG — recorded on purpose

Mid-iteration I read `0 confusionRoll` in the bb2020 harvest, noticed that
`crates/ffb-engine/src/skill_behaviour/registry.rs::build_bb2020()` registers **no**
`AnimalSavageryBehaviour` while Java has
`ffb-server/…/skillbehaviour/bb2020/AnimalSavageryBehaviour.java` (`@RulesCollection(Rules.BB2020)`),
and concluded that BB2020 Animal Savagery was dead in Rust — a ported-but-unregistered behaviour of
exactly the family the procedure warns about.

**That was wrong, on two counts, and both were caught by measuring instead of reasoning:**

1. Rust does not use the behaviour registry for this skill at all. Both editions' hooks are folded
   into the step itself — `crates/ffb-engine/src/step/mixed/shared/step_animal_savagery.rs`, whose
   own header comment says so ("…precedent rather than the generic dispatch registry"). The missing
   registry entry is deliberate, not a gap.
2. The mechanism is demonstrably live in bb2020. `FFB_STEPTRACE=1` on bb2020 seed 1 prints eight
   `RSTATE step=AnimalSavagery` lines with `prompt=ReRollOffer` and `prompt=PlayerChoice`, on
   `ap=H1` / `ap=A1` — the Rat Ogre failing its roll and being offered a lash-out target. The
   harvest reports it under the event name **`animalSavagery` (1511)**, not `confusionRoll`; my
   grep alternation simply did not contain that word. `confusionRoll` is Really Stupid / Bone Head,
   and the bb2020 squad has neither.

The generalisable form: *an absent event name is not an absent mechanism until you have grepped the
event the code actually emits* (`grep -n "GameEvent::" <step>.rs`).

### Honest gaps (none is a parity red; both engines agree)

- **bb2016 emits no `playerMoved`, no `goForItRoll`, no `rightStuffRoll` and no `alwaysHungry`.**
  Pre-existing and edition-wide: the same four are absent from `EVENT_COVERAGE_ogre_bb2016.md`,
  `EVENT_COVERAGE_goblin_bb2016.md`, `EVENT_COVERAGE_slann_bb2016.md` and
  `EVENT_COVERAGE_skaven_bb2016.md`, all of which do show `throwTeamMateRoll`. bb2016 coverage
  counts are not comparable to bb2020/bb2025 ones. Recorded first in the slann_fumbbl entry.
- **No `stab` event anywhere, including bb2025 where the Gutter Runner carries a *permanent* Stab.**
  This is BACKLOG **E9**: neither harness offers the block-kind choice, so `BlockKind::STAB` is
  never selected (`crates/ffb-engine/src/agent/uniform_agent.rs:311` says exactly this). Underworld
  strengthens E9 — until now the only evidence was norse's *temporary* Stiletto-granted Stab, so
  E9 could be read as a prayer-only corner. It is not: 100 bb2025 games with a permanent Stab
  carrier on the pitch produced zero stabs.
- `Projectile Vomit`, `Titchy`, `Insignificant`, `Sidestep` and `Strip Ball` produce no distinct
  events in any harvest; they are passive modifiers, so their absence from the event stream says
  nothing either way. Parity is the proof that they are handled identically.

### Frontier

Empty. Sweep after this race: **29 closed**. Remaining: **wood_elf**.
