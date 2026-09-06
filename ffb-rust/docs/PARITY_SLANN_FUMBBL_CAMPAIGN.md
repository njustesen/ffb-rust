# Parity campaign — slann_fumbbl (heuristic agent)

Mirror matchup `--home slann_fumbbl --away slann_fumbbl`, HeuristicAgent driving BOTH engines,
per-step state hashes compared against stock Java.

## ITER1 (2026-09-06) — baseline was already green; no engine change

This race needed **no Rust engine fix**. Every one of the nine gates, and all three random
controls, came in at 100/100 on the binary built from `6a7cd4110` (the commit that closed
`slann`). The two dodge fixes that closed `slann` — the Diving-Tackle what-if minimum clamped
once (`f7aeee8d1`) and BB2020 having no pre-emptive Diving-Tackle re-roll (`933c2cefb`) — are the
fixes this race would otherwise have needed: `slann_fumbbl` is the FUMBBL import of the same
roster and carries Diving Tackle in all three editions
(`grep -rln "Diving Tackle" data/rosters/` → slann, slann_fumbbl, bb2025 dwarf).

### The FUMBBL alias trap — verified NOT active

FUMBBL league imports carry numeric position ids and generic display names that match neither the
CLI key nor the canonical roster name. `make_team` once fell back **silently** to an all-lineman
AG3 team, which still produced matched-but-meaningless games. Checked before trusting any green:

- `crates/ffb-parity/src/runner.rs` aliases `"slann_fumbbl" => "744258"`.
- `data/teams/bb2025/team_slann_fumbbl.json` names `roster_id: "744258"` and four **distinct**
  position ids (2993/2994/2995/2996) across 11 players — not a uniform lineman team.
- `cargo test -p ffb-parity fumbbl` → 5/5, including
  `fumbbl_khemri_and_slann_build_real_rosters` and
  `fumbbl_slann_kroxigor_has_no_bonehead_in_bb2025`.
- Independent live confirmation from the coverage harvest: **`confusionRoll` is 1557 in bb2016 and
  1627 in bb2020, and ZERO in bb2025.** That is exactly the shape the historical loader fix
  predicts — the FUMBBL Kroxigor's trait is spelled "Bone-head" (hyphen), which is canonical in
  bb2016 but which the bb2025 `SkillFactory` key "Bone Head" (space) does not match, so bb2025
  deliberately drops it (`ffb-model/src/data/loader.rs`). A fallback lineman team could not
  produce 1557 Bone Head rolls in bb2016.

## Gates — nine at 100/100

`./target/release/ffb-parity --home slann_fumbbl --away slann_fumbbl --edition <E> --tier 3 --seeds 1-100 --no-abort --agent heuristic --heur-scale <S> --heur-classes all`

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100/100** | **100/100** | **100/100** |
| bb2020 | **100/100** | **100/100** | **100/100** |
| bb2025 | **100/100** | **100/100** | **100/100** |

Random controls (`FFB_PARITY_ROOT=parity_random --agent random`, seeds 1-100):
bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.

`cargo test -p ffb-engine`: **7433 passed, 0 failed**, 15 ignored.

Timing at @1.0 (batched JVM, 100 seeds): `rust_total=` 33.6 s (bb2016) / 36.4 s (bb2020) /
28.7 s (bb2025).

## Coverage — harvested x3, each run ALONE

`MATCHUP=slann_fumbbl sh scripts/harvest_coverage.sh <edition> 1.0` →
`docs/EVENT_COVERAGE_slann_fumbbl_bb2016.md` / `_bb2020.md` / `_bb2025.md`, each from its own
100/100 run.

- bb2025, 94,738 events: dodgeRoll 900, blockRoll 1673, goForItRoll 3744, pickupRoll 411,
  catchRoll 177, passRoll 123, handOver 94, touchdown 43, foul 208, confusionRoll **0**.
- bb2020, 98,085 events: confusionRoll 1627.
- bb2016, 30,958 events: confusionRoll 1557, dodgeRoll 867, catchRoll 212.

### What was NOT exercised — say it plainly

- **`jumpRoll` is ZERO in all three editions.** Same finding as `slann`: the heuristic agent has
  no Leap arm at all, so the `StepJump` chain is unreachable from this harness. Green here is
  **not** evidence about Leap / Very Long Legs. Already filed as BACKLOG **E12**.
- **`skillUse` is zero in every run.** That event has only five emitting sites (block-result
  Dodge, Dump Off, Horns, Juggernaut, Wrestle), none of which the slann roster carries, so
  Diving Catch, Jump Up, Prehensile Tail, Very Long Legs and Leap leave no positive event trace.
  Parity of the underlying dodge/catch rolls is the evidence instead (BACKLOG E6).
- **bb2016 emits no `playerMoved` and no `goForItRoll` at all** — 30,958 events against ~95k in
  the other two editions. This is a **pre-existing, edition-wide** event-emission gap, not
  something slann_fumbbl introduced: the same two events are absent from
  `docs/EVENT_COVERAGE_slann_bb2016.md` and `docs/EVENT_COVERAGE_skaven_bb2016.md`. It does not
  affect parity (the per-step state hashes match), but it does mean bb2016 coverage counts are
  not comparable to bb2020/bb2025 ones.

## Conclusions of mine that were WRONG

None to report: no hypothesis was formed, because the baseline measured green on the first gate.
The one thing worth recording is that **the baseline was taken first and in full** rather than
assumed from `slann`'s green — the alias trap makes "same roster, therefore same result" an
unsafe inference for any `*_fumbbl` race.

## Closed-roster regressions — all **100/100**

bb2025 @1.0, seeds 1-100, heuristic, `--heur-classes all` (each batch under its own
`FFB_PARITY_ROOT` so concurrent runs could not share a jsonl directory):

nurgle, necromantic, nippon, lizardman, khemri, human, goblin, amazon, chaos, dwarf, norse, ogre,
orc, renegades, skaven, **slann**.

`dwarf` and `slann` are the load-bearing ones — they are the other two Diving Tackle carriers, the
skill behind both fixes that closed `slann` and therefore the ones this race depends on. No Rust
engine code changed in this iteration, so no bb2020/bb2016 regression pass was owed; the nine
slann_fumbbl gates plus the three random controls already re-measure those editions.
