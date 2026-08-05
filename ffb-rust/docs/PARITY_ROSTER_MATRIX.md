# Roster Parity Matrix — bb2025, tier 3

Mirror matchups (`--home X --away X`), `--edition bb2025 --tier 3 --seeds 1-100`, per-step
state-hash parity Rust vs stock Java. Run in PARALLEL (`xargs -P 10`, each JVM capped to 2 cores).
The harness stops at the first failing seed, so a FAIL row means seeds 1..(seed-1) passed and
`seed` is the first miss. "harness gap" = ParityRunner can't drive that step (not a Rust engine bug).

| # | Roster (X vs X) | Result | First divergence / cause |
|---|---|---|---|
| 1 | lineman | GREEN 100/100 | — |
| 2 | amazon | GREEN 100/100 | — |
| 3 | chaos | GREEN 100/100 | — |
| 4 | chaos_dwarf | GREEN 100/100 | — |
| 5 | chaos_pact | GREEN 100/100 | — |
| 6 | dark_elf | FAIL 54/100 | seed 55, step 0 — first divergence |
| 7 | dark_elf_league_fumbbl | FAIL 0/100 | seed 1, step 9 — first divergence |
| 8 | dwarf | GREEN 100/100 | — |
| 9 | elf | GREEN 100/100 | — |
| 10 | goblin | FAIL 0/100 | seed 1, step 9 — first divergence |
| 11 | halfling | FAIL 0/100 | seed 1, step 20 — first divergence |
| 12 | high_elf | FAIL 13/100 | seed 14, step 138 — first divergence |
| 13 | human | GREEN 100/100 | — |
| 14 | khemri | FAIL 39/100 | seed 40, step 185 — first divergence |
| 15 | khemri_fumbbl | FAIL 0/100 | seed 1, step 9 — first divergence |
| 16 | lizardman | GREEN 100/100 | — |
| 17 | necromantic | FAIL 0/100 | seed 1, step 1 — first divergence |
| 18 | nippon | GREEN 100/100 | — |
| 19 | norse | FAIL 73/100 | seed 74, step 144 — first divergence |
| 20 | nurgle | FAIL 0/100 | seed 1, step 1 — first divergence |
| 21 | ogre | FAIL 0/100 | seed 1, step 143 — first divergence |
| 22 | orc | GREEN 100/100 | — |
| 23 | renegades | FAIL 1/100 | seed 2, step 1 — harness gap (STUCK_STEP: ANIMAL_SAVAGERY) |
| 24 | skaven | FAIL 87/100 | seed 88, step 62 — first divergence |
| 25 | slann | FAIL 2/100 | seed 3, step 112 — first divergence |
| 26 | slann_fumbbl | FAIL 0/100 | seed 1, step 9 — first divergence |
| 27 | undead | FAIL 0/100 | seed 1, step 84 — first divergence |
| 28 | underworld | FAIL 1/100 | seed 2, step 1 — harness gap (STUCK_STEP: ANIMAL_SAVAGERY) |
| 29 | vampire | FAIL 0/100 | seed 1, step 7 — first divergence |
| 30 | wood_elf | FAIL 0/100 | seed 1, step 49 — first divergence |

**Summary:** 11 green, 19 not green (of 30 matchups). Whole matrix ran in 222.9s wall (parallel).
