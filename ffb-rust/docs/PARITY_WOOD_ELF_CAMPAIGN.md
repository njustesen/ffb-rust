# Parity campaign — wood_elf (heuristic agent)

Mirror matchup `--home wood_elf --away wood_elf`, tier 3, seeds 1-100, `--heur-classes all`,
all three editions × scales `1.0` / `0` / `1e6`.

## 🏁 CLOSED 2026-09-06 — GREEN AT BASELINE, no engine change

Baseline was taken on `c47d6516c` (the commit that closed `underworld`). All nine parity gates and
all three random controls were **100/100 on the first measurement**. No Rust engine file was edited
for this race; there is therefore no fix, no regression test and no Java harness change in this
entry. wood_elf is the **last race of the alphabetical heuristic sweep** — with it, **all 30 races
are closed**.

### Gate numbers actually run

| Edition | @1.0 | @0 | @1e6 | random control |
|---|---|---|---|---|
| bb2016 | 100/100 | 100/100 | 100/100 | 100/100 * |
| bb2020 | 100/100 | 100/100 | 100/100 | 100/100 |
| bb2025 | 100/100 | 100/100 | 100/100 | 100/100 |

\* the bb2016 random control prints the `…, but required coverage items are MISSING` trailer — that
is the tier-3 coverage checklist, not parity, and is a PASS for the parity half. The three
**heuristic** harvest runs all print `Result: ALL REQUIRED ITEMS PRESENT`.

`TIMING … rust_total=` at @1.0: **33.4 s** (bb2016) / **39.0 s** (bb2020) / **45.8 s** (bb2025),
against `java_total=` 75.1 / 77.5 / 80.4 s. At @0 (the slowest scale): 47.4 / 55.3 / 69.1 s rust
against 117.5 / 125.4 / 128.4 s java. At @1e6: 40.0 / 37.0 / 42.3 s.

`cargo test -p ffb-engine`: **7433 passed, 0 failed**, 15 ignored.
`python scripts/check_java_trees.py`: trees agree (no Java edit was made).

Closed-roster regressions, bb2025 @1.0 seeds 1-100, heuristic `--heur-classes all`, all **100/100**:
nurgle, necromantic, nippon, lizardman, khemri, human, goblin, amazon, chaos, dwarf, norse, ogre,
orc, renegades, skaven, slann. (Confirmatory only — nothing in the engine changed. No
edition-shared code was touched, so no bb2016/bb2020 carriage run was owed.)

Coverage harvested ×3, each run alone: `docs/EVENT_COVERAGE_wood_elf_bb2016.md`, `_bb2020.md`,
`_bb2025.md`.

## The roster, read from the JSON (not from the brief)

`data/rosters/{bb2016,bb2020,bb2025}/roster_wood_elf.json`:

| Position | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Lineman | — | — | — |
| Thrower | Pass | Pass | Pass, **Safe Pair of Hands** |
| Catcher | Catch, Dodge | Catch, Dodge | Catch, Dodge, **Sprint** |
| Wardancer | Block, Dodge, **Leap** | Block, Dodge, **Leap** | Block, Dodge, **Leap** |
| Treeman | Loner, Mighty Blow, Stand Firm, Strong Arm, **Take Root**, Thick Skull, Throw Team-Mate | same | same |

Two corrections to the brief, both from the JSON: the Treeman has **no Timmm-ber!** in any edition
(`grep -i timmm crates/ffb-engine/src/` also returns nothing), and no wood_elf position carries
**Right Stuff** in any edition — which turns out to matter (below).

Drafted squads (`data/teams/*/team_wood_elf.json`):

- **bb2016** — 1 Treeman, 2 Wardancers, 3 Catchers, 1 Thrower, 4 Linemen. No stars.
- **bb2020** — 1 Treeman, 1 Wardancer, 1 Catcher, 1 Thrower, 8 Linemen. No stars.
- **bb2025** — 1 Treeman, 2 Wardancers, 2 Catchers, 2 Throwers, 4 Linemen, **+ stars Rodney and
  Swiftvine**.

## Three coverage findings, each measured

### 1. Take Root is live in all three editions, and the event stream cannot see it

`takeRoot` / `confusionRoll` is **absent from all three harvests**. That is the exact fingerprint
the sweep has twice mistaken for a dead path, so it was measured rather than reasoned about.

The live step is `crates/ffb-engine/src/step/bb2025/shared/step_take_root.rs` — `driver.rs:217`
routes `StepId::TakeRoot` there through the `bb2025::shared::*` glob (line 64), and **neither** the
`Rules::Bb2020` nor the `Rules::Bb2016` arm of `make_step_for` overrides it, so all three editions
run that one file with the edition differences gated inside it. That file emits **no `GameEvent`**
at all; only the dead twin `step/bb2016/step_take_root.rs:115` emits `GameEvent::ConfusionRoll`.
The absence is an event-stream blindness, not a dead mechanism.

Proved with a temporary `FFB_TAKEROOT` probe at the roll site (added, measured, removed with a
targeted edit; `git diff --stat crates/` afterwards is **empty**), seeds 1-5 per edition:

| | bb2016 | bb2020 | bb2025 |
|---|---:|---:|---:|
| Take Root d6 rolls in 5 games | **51** | **49** | **6** |

The bb2025 number is an order of magnitude lower *and that is correct*: the edition gate inside the
step (its own comment, citing wood_elf seed 1) is that in bb2025 a **PRONE** player standing up does
**not** roll Take Root, while in bb2016/bb2020 it still does. The comments in that file name
wood_elf and bb2016 seed 1 by index — i.e. this step was already root-caused against this very race
in an earlier campaign, which is part of why the baseline was green.

**Recommendation (not done here):** give the shared step the `GameEvent::ConfusionRoll` its dead
bb2016 twin already emits, so the harvest stops being blind to a mechanism that fires ~10× per game.

### 2. `ThrowTeamMate` is declared hundreds of times and never once thrown — correctly

Declared actions: **254** (bb2016) / **201** (bb2020) / **66** (bb2025). `throwTeamMateRoll`,
`rightStuffRoll` and `alwaysHungry`: **zero everywhere**.

This is the same surface fingerprint as the BB2020 TTM bug (`docs/PARITY_TTM.md`: 7,235 declarations,
0 dispatches, caused by a harness filter disagreeing with the engine). It is **not** the same cause.
`grep -il "right stuff" data/rosters/*/roster_wood_elf.json` returns nothing: the wood elf roster has
**no throwable player in any edition**. The Treeman can declare the action and then has no legal
target, so both engines deselect identically — which the nine green gates prove. Squad composition,
not a filter bug.

It is, however, an **agent-quality** note: the heuristic spends ~250 declarations per 100 bb2016
games on an action that can never resolve on this roster. Cheap fix would be to require a non-empty
throwable-target list before scoring the declaration, on **both** agents with the cross-language
goldens re-blessed.

### 3. Leap is unreachable — BACKLOG E12, confirmed a second time

`jumpRoll` is **zero across all 300 games**, exactly as on slann. Every Wardancer carries Leap in
every edition and the agent has no jump *declaration* (only a `"JUMP"` entry in the re-roll-action
name list in `heuristic_agent.rs`), so the `StepJump` chain is never entered. This was expected and
is **not** a red; wood_elf is the second race to confirm E12 rather than the race that closes it.

### What the skill-use stream does and does not show

`skillUse` totals: 154 (bb2016) / 85 (bb2020) / 186 (bb2025), and the resolved names are **Dodge**
only in bb2016/bb2020, **Dodge + Wrestle** in bb2025. Block, Catch, Pass, Sprint, Safe Pair of
Hands, Mighty Blow, Stand Firm, Strong Arm and Thick Skull produce no distinct events — they are
passive modifiers on rolls the engine already reports. This is `EVENT_COVERAGE.md` findings F1–F5
again: the event stream is mostly blind to skill use, and **parity is the proof** that both engines
apply them identically.

The rolls those skills modify are all heavily exercised: bb2025 has 1229 dodge rolls, 299 catch
rolls, 200 pass rolls, 4020 GFI rolls, 1687 block rolls, 394 pickup rolls and 122 touchdowns.

## Frontier

Empty. **Sweep complete: 30 of 30 races closed.** (undead and vampire are outside this sweep.)

Open items this race leaves behind, none of them a parity red:

- **E12** (existing) — the agent cannot declare a Jump, so Leap / Very Long Legs / `StepJump` are
  unreachable from either harness.
- **New, agent quality** — `ThrowTeamMate` declared with no throwable target on the roster.
- **New, instrumentation** — the live shared `StepTakeRoot` emits no `GameEvent`, so ~10 rolls per
  game are invisible to every coverage harvest.
