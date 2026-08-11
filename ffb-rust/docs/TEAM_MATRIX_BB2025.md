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
| `elf` | 🟢 100/100 | fixed 2026-08-10: auto-use Sidestep pushback (commit f44cecf0) ||
| `goblin` | 🟢 100/100 |  | 2026-08-10: Swoop TTM decline + StepFollowup Fend/Taunt dialogs + Ball&Chain Fanatic wander (InjuryTypeCrowdPush publish + B&C fall injury) |
| `halfling` | 🟢 100/100 |  | 08-10: claws-aware armour recalc uses armour-with-modifiers (seed 38, dbd21667); TTM hit-player injury deferred not applied inline → clears rooted (seeds 34/65, fb7ffaf8) |
| `high_elf` | 🟢 100/100 |  | 2026-08-10: Steady Footing faller-resolution + Wrestle Both-Down ball bounce + changeActingPlayer computed hasActed() + My Ball carrier PASS exclusion (ParityRunner harness) |
| `human` | 🟢 100/100 |  | 08-10: Tackle cancels the Dodge skill re-roll on a failed dodge (seeds 13/33/56/62/65, dd12cf97) |
| `khemri` | 🟢 100/100 |  | 08-10: apply casualty modifiers (Decay +1 / nigglings) to the d16 tier → SI triggers Getting Even (seed 21, 3f4cc69d) |
| `khemri_fumbbl` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `lizardman` | 🟢 100/100 |  | 08-11: cleared by the human Tackle-cancels-Dodge-reroll fix (dd12cf97) |
| `necromantic` | 🟢 100/100 |  | 08-11: StepTouchback KICKOFF-not-REGULAR (seed 37, 7ffbcd2f); SecureTheBall Unsteady harness (seed 83, f9e0544ea); bb2025 Regeneration doesn't preventRaiseFromDead (seed 89, a4e68430) |
| `nippon` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `norse` | 🟢 100/100 |  | 08-11: StepPickMeUp emits PICK_ME_UP dialog instead of stalling (Beer Boar, 11 seeds); Strip Ball drop-on-pushback in StepBlockChoice.initPushback (seeds 28/30) |
| `nurgle` | 🟢 100/100 |  | 08-11: cleared by earlier engine fixes (stale 08-08 red re-verified 100/100) |
| `ogre` | 🔴 94/100 | seed 12, step 75, java 98818cdc80d7791a vs rust 0b018aaab59832d3 |  |
| `orc` | 🟢 100/100 |  | 08-11: cleared by earlier engine fixes (stale 08-08 red re-verified 100/100) |
| `renegades` | 🟢 100/100 |  |  |
| `skaven` | 🟢 100/100 |  | 08-11: Strip Ball drop-on-pushback on the dodged-stumble path (shared util_block_sequence.init_pushback; seed 52) |
| `slann` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `slann_fumbbl` | 🟢 100/100 |  | FUMBBL-legacy roster |
| `undead` | 🟢 100/100 |  |  |
| `underworld` | 🟢 100/100 |  | 08-11: cleared by earlier engine fixes (stale 08-08 red re-verified 100/100) |
| `vampire` | 🟢 100/100 |  |  |
| `wood_elf` | 🟢 100/100 |  |  |

**29 green / 1 red of 30.** (amazon 08-09; chaos, dark_elf, dwarf, elf 08-10; high_elf, goblin, halfling, human, khemri 08-10; lizardman, necromantic, norse, nurgle, orc, skaven, underworld 08-11). Only `ogre` remains red (DEFERRED per campaign scope).
