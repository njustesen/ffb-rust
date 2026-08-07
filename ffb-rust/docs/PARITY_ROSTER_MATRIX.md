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
| 6 | dark_elf | GREEN 100/100 | both agents DECLINE Dump Off (Rust random_agent SkillUse{DumpOff}→use=false; Java ParityRunner SKILL_USE declines DumpOff) → no DUMP_OFF/INIT_PASSING → no Java NPE / Rust stall |
| 7 | dark_elf_league_fumbbl | FAIL 0/100 | seed 1, step 9 — first divergence |
| 8 | dwarf | GREEN 100/100 | — |
| 9 | elf | GREEN 100/100 | — |
| 10 | goblin | FAIL 0/100 | seed 1, step 9 — first divergence |
| 11 | halfling | GREEN 100/100 | TTM fixes: pass modifiers + Strong Arm + null-target Block no-stand-up + fumbled-carrier ball-bounce/turnover (wood_elf Take Root retired the old blitz-negatrait deferral) |
| 12 | high_elf | GREEN 100/100 | — |
| 13 | human | GREEN 100/100 | — |
| 14 | khemri | GREEN 100/100 | — |
| 15 | khemri_fumbbl | FAIL 0/100 | seed 1, step 9 — first divergence |
| 16 | lizardman | GREEN 100/100 | — |
| 17 | necromantic | GREEN 100/100 | stand-up rush move-square fix (set going_for_it before update_move_squares) |
| 18 | nippon | GREEN 100/100 | — |
| 19 | norse | GREEN 100/100 | — |
| 20 | nurgle | GREEN 100/100 | — |
| 21 | ogre | FAIL 0/100 | seed 1, step 143 — first divergence |
| 22 | orc | GREEN 100/100 | — |
| 23 | renegades | FAIL 1/100 | seed 2, step 1 — harness gap (STUCK_STEP: ANIMAL_SAVAGERY) |
| 24 | skaven | GREEN 100/100 | — |
| 25 | slann | GREEN 100/100 | agent declines the diving-catch declaration prompt (mislabeled AgentPrompt::SwarmingPlayers from StepCatchScatterThrowIn) via SelectPlayer{empty} |
| 26 | slann_fumbbl | FAIL 0/100 | seed 1, step 9 — first divergence |
| 27 | undead | GREEN 100/100 | roll-to-stand-up success now sets STANDING state (was left PRONE) |
| 28 | underworld | FAIL 1/100 | seed 2, step 1 — harness gap (STUCK_STEP: ANIMAL_SAVAGERY) |
| 29 | vampire | GREEN 100/100 | Bloodlust (min-roll, failed-action routing, feed, suffering-move, free-stand-up, reroll-decline suffering) + guard used-skills reset on genuine player change |
| 30 | wood_elf | GREEN 100/100 | Take Root (old_player_state + dodging-clear) + agent phase-2 pre-draw for prone/rooted movers w/ uncapped neighbour list |

**Summary:** 16 green, 14 not green (of 30 matchups). Whole matrix ran in 222.9s wall (parallel).
