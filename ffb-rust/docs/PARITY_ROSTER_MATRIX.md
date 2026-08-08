# Roster Parity Matrix — bb2025, tier 3

> **SUPERSEDED (2026-08-08).** This matrix ran on the legacy first-11-by-(quantity,cost)
> teams. The current matrices use hand-drafted rule-legal teams and audited per-edition
> roster data: see `docs/TEAM_MATRIX_BB2025.md` and `docs/TEAM_MATRIX_BB2016.md`
> (teams in `data/teams/`, drafting docs `docs/TEAM_DRAFTS_*.md`). Kept for history.

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
| 7 | dark_elf_league_fumbbl | **GREEN 100/100** | FUMBBL roster-id lookup: id "4959"/name "Dark Elf" matched neither CLI key → make_team fell back to an all-generic-lineman team (AG3, no skills) → first dodge diverged (Witch Elf AG2 vs fallback AG3). Fix: alias fumbbl CLI names to numeric roster ids in make_team_from_roster |
| 8 | dwarf | GREEN 100/100 | — |
| 9 | elf | GREEN 100/100 | — |
| 10 | goblin | **GREEN 100/100** | Ball&Chain/secret-weapon/TTM/chainsaw: 6 fixes — GettingEven push+roster keywords; end-of-game secret-weapon ban roll; variable Mighty Blow value; eaten-player canUseApo skips Regeneration; Pitch-Invasion B&C stun rolls-but-discards chain injury; mbStacksAgainstChainsaw baseline option (fallen Looney's chainsaw+MB armour break) |
| 11 | halfling | GREEN 100/100 | TTM fixes: pass modifiers + Strong Arm + null-target Block no-stand-up + fumbled-carrier ball-bounce/turnover (wood_elf Take Root retired the old blitz-negatrait deferral) |
| 12 | high_elf | GREEN 100/100 | — |
| 13 | human | GREEN 100/100 | — |
| 14 | khemri | GREEN 100/100 | — |
| 15 | khemri_fumbbl | **GREEN 100/100** | fixed once the FUMBBL numeric-id alias built the real roster (shared with dark_elf_league_fumbbl); re-verified 100/100 |
| 16 | lizardman | GREEN 100/100 | — |
| 17 | necromantic | GREEN 100/100 | stand-up rush move-square fix (set going_for_it before update_move_squares) |
| 18 | nippon | GREEN 100/100 | — |
| 19 | norse | GREEN 100/100 | — |
| 20 | nurgle | GREEN 100/100 | — |
| 21 | ogre | **GREEN 100/100** | no-target FOUL deselects (agent mirrors ParityRunner.sendFoulAction) instead of ending the turn |
| 22 | orc | GREEN 100/100 | — |
| 23 | renegades | **GREEN 100/100** | AS lash-out + prone-TTM stand-up + SafePairOfHands PLACE_BALL harness decline |
| 24 | skaven | GREEN 100/100 | — |
| 25 | slann | GREEN 100/100 | agent declines the diving-catch declaration prompt (mislabeled AgentPrompt::SwarmingPlayers from StepCatchScatterThrowIn) via SelectPlayer{empty} |
| 26 | slann_fumbbl | **GREEN 100/100** | Kroxigor's trait is spelled "Bone-head" (hyphen) in FUMBBL roster 744258, but Java's bb2025 SkillFactory keys the skill by its canonical "Bone Head" (space) and `forName` does an exact case-insensitive match → the hyphen spelling resolves to null, so Java's Kroxigor has NO Bone Head. Rust's lenient resolver kept it, rolling a per-activation negatrait d6 Java never rolled → dice-stream desync (seed 1 step 9: the Kroxigor's dodge failed → turnover). Fix: bb2025 drops the hyphen-spelled "bone-head" during roster build (bb2016's canonical IS "Bone-Head", so kept there) |
| 27 | undead | GREEN 100/100 | roll-to-stand-up success now sets STANDING state (was left PRONE) |
| 28 | underworld | **GREEN 100/100** | Cheering-Fans additional-assist turn-lifecycle clear (acting-team turn_nr>=1, not turn_started) |
| 29 | vampire | GREEN 100/100 | Bloodlust (min-roll, failed-action routing, feed, suffering-move, free-stand-up, reroll-decline suffering) + guard used-skills reset on genuine player change |
| 30 | wood_elf | GREEN 100/100 | Take Root (old_player_state + dodging-clear) + agent phase-2 pre-draw for prone/rooted movers w/ uncapped neighbour list |

**Summary:** 30 green, 0 not green (of 30 matchups) — ALL rosters GREEN.
Both former fails (khemri_fumbbl / slann_fumbbl) first needed dark_elf_league_fumbbl's FUMBBL
roster-id alias fix (numeric `id` + generic `name` → all-lineman fallback). khemri_fumbbl then
verified 100/100; slann_fumbbl needed one more fix — Java's bb2025 SkillFactory drops the FUMBBL
Kroxigor's hyphen-spelled "Bone-head" (canonical is "Bone Head"), which Rust's lenient resolver
had kept (extra negatrait d6 → dice desync). Both now re-verified 100/100.
