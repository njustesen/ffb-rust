# Team-Parity Matrix — BB2025 (hand-drafted teams)

Run 2026-08-08 — mirror matchups, tier 3, seeds 1-100,
teams from `data/teams/bb2025/` (see docs/TEAM_DRAFTS_BB2025.md), Java XMLs from scripts/gen_java_parity_data.py.
Reds are RECORDED, not fixed (scope of the 2026-08-08 team-creation task).

| Roster | Result | First divergence | Notes |
|---|---|---|---|
| `lineman` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `amazon` | 🟢 100/100 | fixed 2026-08-09: StepPassBlock port + agent window mirroring (commits b8aa79e8, 90f6f0cb) ||
| `chaos` | 🟢 100/100 | fixed 2026-08-10: Arm Bar armour-or-injury + ARM_BAR player choice (commits 0f15b607, 32ce6fb5) ||
| `chaos_dwarf` | 🟢 100/100 |  |  |
| `chaos_pact` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `dark_elf` | 🟢 100/100 | fixed 2026-08-10: Punt wait prompt + harness EndTurn abort (commit 0d48c04a) ||
| `dark_elf_league_fumbbl` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `dwarf` | 🟢 100/100 | fixed 2026-08-10: Bribery-and-Corruption argue re-roll — secret-weapon (82dc503a) + foul-ejection (485183bc) ||
| `elf` | 🔴 19/100 | seed 2, step 71, java 2d5efcb973581ce7 vs rust 2fc28391f6b5d205 |  |
| `goblin` | 🔴 34/100 | seed 3, step 149, java 47203bf4bd16a61d vs rust 0146a2651766fe1f |  |
| `halfling` | 🔴 88/100 | seed 6, step 218, java bb2815e6e646f5d3 vs rust e1af7418e782aba7 |  |
| `high_elf` | 🔴 19/100 | seed 3, step 165, java b984f4a0f8596bdc vs rust 0a69c723db8571a6 |  |
| `human` | 🔴 95/100 | seed 13, step 255, java e5fe283217c0feed vs rust 25607a9b2b190492 |  |
| `khemri` | 🔴 99/100 | seed 21, step 44, java 4c93a4d43e204750 vs rust c38d8eb4d2ded874 |  |
| `khemri_fumbbl` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `lizardman` | 🔴 0/100 | seed 1, step 22, java 7da84dba50abc1f5 vs rust 29081bc3e6ec4d48 |  |
| `necromantic` | 🔴 97/100 | seed 37, step 152, java 1ebdc097bb1705f1 vs rust cdd55e7326275e0e |  |
| `nippon` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `norse` | 🔴 87/100 | seed 1, step 170, java a1e292cf8db01160 vs rust 1be574caceb260d5 |  |
| `nurgle` | 🔴 5/100 | seed 1, step 11, java 482cd04ec2bf04ef vs rust ef202019fc3be926 |  |
| `ogre` | 🔴 94/100 | seed 12, step 75, java 98818cdc80d7791a vs rust 0b018aaab59832d3 |  |
| `orc` | 🔴 0/100 | seed 1, step 1, java 9d04af23c3b5728d vs rust 9c55510376cb6887 |  |
| `renegades` | 🟢 100/100 |  |  |
| `skaven` | 🔴 92/100 | seed 24, step 200, java 13ab375167193665 vs rust 63c555ead50ea9cc |  |
| `slann` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `slann_fumbbl` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `undead` | 🟢 100/100 |  |  |
| `underworld` | 🔴 92/100 | seed 4, step 44, java 7571510943221c00 vs rust e57de2f1c09f2a64 |  |
| `vampire` | 🟢 100/100 |  |  |
| `wood_elf` | 🟢 100/100 |  |  |

**16 green / 14 red of 30.** (amazon 08-09; chaos, dark_elf, dwarf 08-10)
