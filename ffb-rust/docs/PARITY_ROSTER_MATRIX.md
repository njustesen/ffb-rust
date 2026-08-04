# Roster Parity Matrix — bb2025, tier 3

Mirror matchups (`--home X --away X`), `--edition bb2025 --tier 3 --seeds 1-100`, per-step state-hash parity Rust vs stock Java.
Pass/fail and the first-fail seed/step are exact. The parenthetical `UNHANDLED_STEP` is the FIRST such
tag seen in the run and is only a hint — it may not be the actual fail cause (e.g. vampire's true cause
is a Rust agent panic, not INIT_PASSING). Timings omitted here — this batch ran 10-way parallel so they
were contention-inflated 3-8x; see the perf notes.

| # | Roster (X vs X) | Result | First divergence / cause |
|---|---|---|---|
| 1 | lineman | GREEN 100/100 | — |
| 2 | amazon | GREEN 100/100 | — |
| 3 | chaos | GREEN 100/100 | — |
| 4 | chaos_dwarf | GREEN 100/100 | — |
| 5 | chaos_pact | GREEN 100/100 | — |
| 6 | dark_elf | FAIL 0/100 | seed 1, step 36 (UNHANDLED_STEP: INIT_PASSING) |
| 7 | dark_elf_league_fumbbl | FAIL 0/100 | seed 1, step 9 (UNHANDLED_STEP: INIT_PASSING) |
| 8 | dwarf | FAIL 0/100 | seed 1, step 101 (UNHANDLED_STEP: INIT_PASSING) |
| 9 | elf | FAIL 37/100 | seed 38, step 265 |
| 10 | goblin | FAIL 0/100 | seed 1, step 9 (UNHANDLED_STEP: INIT_PASSING) |
| 11 | halfling | FAIL 0/100 | seed 1, step 20 (UNHANDLED_STEP: INIT_PASSING) |
| 12 | high_elf | FAIL 13/100 | seed 14, step 138 |
| 13 | human | GREEN 100/100 | — |
| 14 | khemri | FAIL 39/100 | seed 40, step 185 (UNHANDLED_STEP: INIT_PASSING) |
| 15 | khemri_fumbbl | FAIL 0/100 | seed 1, step 9 (UNHANDLED_STEP: INIT_PASSING) |
| 16 | lizardman | GREEN 100/100 | — |
| 17 | necromantic | FAIL 0/100 | seed 1, step 1 (UNHANDLED_STEP: INIT_PASSING) |
| 18 | nippon | GREEN 100/100 | — |
| 19 | norse | FAIL 1/100 | seed 2, step 151 (UNHANDLED_STEP: INIT_PASSING) |
| 20 | nurgle | FAIL 0/100 | seed 1, step 1 (UNHANDLED_STEP: INIT_PASSING) |
| 21 | ogre | FAIL 0/100 | seed 1, step 143 (UNHANDLED_STEP: INIT_PASSING) |
| 22 | orc | GREEN 100/100 | — |
| 23 | renegades | FAIL 0/100 | seed 1, step 93 (UNHANDLED_STEP: ANIMAL_SAVAGERY) |
| 24 | skaven | FAIL 87/100 | seed 88, step 62 (UNHANDLED_STEP: INIT_PASSING) |
| 25 | slann | FAIL 2/100 | seed 3, step 112 (UNHANDLED_STEP: INIT_PASSING) |
| 26 | slann_fumbbl | FAIL 0/100 | seed 1, step 9 (UNHANDLED_STEP: INIT_PASSING) |
| 27 | undead | FAIL 0/100 | seed 1, step 84 (UNHANDLED_STEP: INIT_PASSING) |
| 28 | underworld | FAIL 0/100 | seed 1, step 42 (UNHANDLED_STEP: ANIMAL_SAVAGERY) |
| 29 | vampire | FAIL (crash) | Rust random_agent panic: no handler for BloodlustAction prompt |
| 30 | wood_elf | FAIL 0/100 | seed 1, step 49 (UNHANDLED_STEP: INIT_PASSING) |

**Summary:** 9 green, 21 failing (of 30 matchups).

Failure clusters: most are `UNHANDLED_STEP: INIT_PASSING` (ParityRunner has no handler when a pass reaches the engine's INIT_PASSING wait — a harness gap, not a Rust engine bug; same as human seed 16). Two are `ANIMAL_SAVAGERY` (another missing ParityRunner handler: renegades, underworld). elf/high_elf are real per-step engine divergences. vampire is a Rust random_agent gap (BloodlustAction).
