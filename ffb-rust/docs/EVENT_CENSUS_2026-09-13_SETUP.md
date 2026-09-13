# Event census of the heuristic-SETUP sweep — 2026-09-13

**What this is.** The coverage half of the setup change (docs/HEURISTIC_AGENT.md §6.21, BACKLOG
§H.43–§H.48). `docs/SWEEP_2026-09-13_SETUP.txt` says whether the two engines agreed (sweep 2, then
the six red gates re-run on the final binary: **329/330**, the one remaining red is the stock Java
crash JD-002 on goblin bb2025 @1.0 seed 67). This file says **what was on the pitch** while they
agreed, measured against the 2026-09-12 baseline sweep with the same census code
(`scripts/sweep_census/agg.py` → `out_rd` / `out_st`, `compare.py`).

## Headline

| | baseline (canonical setup) | new (heuristic setup) |
|---|---:|---:|
| games / events | 33,000 / 27.50 M | 33,000 / 27.37 M |
| **touchdowns** | 25,357 | **25,856 (+2.0%)** |
| TD/game @0 (argmax) | 1.887 | 1.854 (−1.8%) |
| TD/game @1.0 (sampled) | 0.364 | **0.436 (+19.9%)** |
| TD/game @1e6 (uniform) | 0.055 | **0.061 (+11.3%)** |
| block rolls / dodge rolls / fouls | 641k / 297k / 52.9k | **729k (+13.6%) / 346k (+16.4%) / 63.0k (+19.1%)** |
| KO / casualty / dead | 65,962 / 54,756 / 6,862 | 69,222 (+4.9%) / 58,554 (+6.9%) / 7,359 (+7.2%) |
| hand-offs / catch rolls / pass rolls | 33,214 / 87,928 / 41,523 | **25,854 (−22%)** / 71,778 (−18%) / 39,204 (−5.6%) |
| skills raising a SkillUse event (of 98 fielded) | 12 | **19** |
| `GameEvent` / `PlayerAction` variants | 76 / 14 | 76 / 14 (none gained, none lost) |
| gates short on the harness checklist | 30 | **69** (all but one: `action HandOver`) |

**Reading the TD row.** These are MIRROR matches — both sides got the new setup, so TD/game here
measures game dynamics, not relative strength. Relative strength is the A/B in BACKLOG §H.44 (one
side new, one side canonical, 24,000 games): **+28% at argmax, +27% sampled**, positive in every
race and edition. The mirror numbers say the sampled and uniform arms now score more (the setup
puts receivers where the ball lands), while argmax is flat: two well-set-up teams cancel out.

**Reading the contact rows.** More blocks, dodges, fouls and injuries: the formations put players
on the LOS in contact and close to the halfway line, so the game is more physical. Hand-offs and
catches fell: with a handler placed deep and the ball fielded cleanly, the give chain fires less.

**Coverage.** Seven skills raised their first SkillUse event: Diving Catch, Eye Gouge, Safe Pass,
Side Step, Steady Footing, Sure Hands, Taunt. No `GameEvent` or `PlayerAction` variant appeared or
vanished. The checklist regression is real and narrow: 69 gates now miss `action HandOver`
(baseline 30), the direct consequence of the hand-off drop; 11 baseline-short gates are complete
now and 39 new ones are short. Nothing else on the checklist moved.

**What the sweep also found.** Six latent engine gaps the canonical formation never reached
(§H.44–§H.47) and one stock Java crash (JD-002). Parity-wise the setup change is therefore a net
gain: six 1:1 ports the old matrix could not see are now in.

---

# Census comparison — out_rd (baseline) vs out_st (new)

| | baseline | new | change |
|---|---:|---:|---:|
| gates | 330 | 330 | |
| games | 33000 | 33000 | |
| events | 27501468 | 27373954 | -0.5% |
| GameEvent variants | 76 | 76 | |
| PlayerAction variants | 14 | 14 | |
| skills raising SkillUse | 10 | 15 | |
| touchdowns | 25357 | 25856 | +2.0% |
| hand-offs | 33214 | 25854 | -22.2% |
| pass rolls | 41523 | 39204 | -5.6% |
| block rolls | 641309 | 728619 | +13.6% |
| fouls | 52903 | 63033 | +19.1% |
| kick-offs | 88397 | 89234 | +0.9% |
| throw-ins | 3710 | 3535 | -4.7% |
| interception rolls | 0 | 0 | n/a |
| catch rolls | 87928 | 71778 | -18.4% |
| pickup rolls | 125034 | 120543 | -3.6% |
| dodge rolls | 296930 | 345711 | +16.4% |
| rush rolls | 1148814 | 1165655 | +1.5% |
| ball scatters | 200016 | 193864 | -3.1% |
| touchbacks | 0 | 0 | n/a |
| injuries: ko | 65962 | 69222 | +4.9% |
| injuries: cas | 54756 | 58554 | +6.9% |
| injuries: dead | 6862 | 7359 | +7.2% |

## Touchdowns per game, by sampling scale

| scale | baseline games | baseline TD/game | new games | new TD/game | change |
|---|---:|---:|---:|---:|---:|
| 0 | 11000 | 1.887 | 11000 | 1.854 | -1.8% |
| 1.0 | 11000 | 0.364 | 11000 | 0.436 | +19.9% |
| 1e6 | 11000 | 0.055 | 11000 | 0.061 | +11.3% |

## Required-item checklist failures

baseline: 30 gates short; new: 69 gates short

new-only short gates:
  amazon bb2025 @1.0: action HandOver
  black_orc bb2020 @1e6: action HandOver
  black_orc bb2025 @1e6: action HandOver
  chaos bb2020 @1e6: action HandOver
  chaos_dwarf bb2016 @1.0: action HandOver
  chaos_dwarf bb2025 @1e6: action HandOver
  dark_elf bb2020 @1.0: action HandOver
  dark_elf_league_fumbbl bb2020 @1.0: action HandOver
  dark_elf_league_fumbbl bb2020 @1e6: action HandOver
  dark_elf_league_fumbbl bb2025 @1e6: action HandOver
  dwarf bb2020 @1.0: action HandOver
  elf bb2020 @1.0: action HandOver
  elf bb2025 @1e6: action HandOver
  gnome bb2020 @1e6: action HandOver
  gnome bb2025 @1e6: action HandOver
  goblin bb2025 @1e6: action HandOver
  halfling bb2016 @1e6: action HandOver
  halfling bb2020 @1.0: action HandOver
  halfling bb2020 @1e6: action HandOver
  high_elf bb2025 @1.0: action HandOver
  human bb2025 @1e6: action HandOver
  imperial_nobility bb2025 @1.0: action HandOver
  khemri bb2020 @1.0: action HandOver
  khemri bb2025 @1.0: action HandOver
  khemri bb2025 @1e6: action HandOver
  lizardman bb2016 @1e6: action HandOver
  lizardman bb2025 @1e6: action HandOver
  nippon bb2016 @1e6: action HandOver
  nippon bb2020 @1e6: action HandOver
  norse bb2016 @1.0: action HandOver
  norse bb2016 @1e6: action HandOver
  norse bb2020 @1e6: action HandOver
  norse bb2025 @1e6: action HandOver
  nurgle bb2020 @0: action Pass
  nurgle bb2020 @1.0: action HandOver
  nurgle bb2025 @1.0: action HandOver
  nurgle bb2025 @1e6: action HandOver
  ogre bb2020 @1.0: action HandOver
  ogre bb2020 @1e6: action HandOver
  ogre bb2025 @1.0: action HandOver
  old_world_alliance_ogre bb2020 @1e6: action HandOver
  old_world_alliance_treeman bb2025 @1.0: action HandOver
  old_world_alliance_treeman bb2025 @1e6: action HandOver
  skaven bb2016 @1e6: action HandOver
  skaven bb2020 @1e6: action HandOver
  skaven bb2025 @1e6: action HandOver
  slann bb2020 @1e6: action HandOver
  slann bb2025 @1e6: action HandOver
  slann_fumbbl bb2016 @1e6: action HandOver
  slann_fumbbl bb2020 @1.0: action HandOver
  slann_fumbbl bb2025 @1.0: action HandOver
  slann_fumbbl bb2025 @1e6: action HandOver
  snotling bb2025 @1e6: action HandOver
  undead bb2020 @1.0: action HandOver
  undead bb2020 @1e6: action HandOver
  undead bb2025 @1e6: action HandOver
  underworld bb2025 @1e6: action HandOver
  underworld_underworldtroll bb2020 @1e6: action HandOver
  vampire bb2016 @1.0: action HandOver
  vampire bb2020 @1e6: action HandOver
  vampire bb2025 @1.0: action HandOver

fixed (short in baseline, complete now):
  amazon bb2020 @1e6: action HandOver
  amazon bb2025 @1e6: action HandOver
  chaos bb2020 @1.0: action HandOver
  chaos_pact bb2020 @1e6: action HandOver
  dark_elf bb2016 @1.0: action HandOver
  goblin bb2016 @1.0: action HandOver
  halfling bb2025 @1e6: action HandOver
  human bb2020 @1e6: action HandOver
  khorne bb2020 @1.0: action HandOver
  necromantic bb2020 @1e6: action HandOver
  necromantic bb2025 @1e6: action HandOver
  nurgle bb2016 @1e6: action HandOver
  nurgle bb2025 @0: action Pass
  ogre bb2025 @1e6: action HandOver
  orc bb2020 @1e6: action HandOver
  orc bb2025 @1.0: action HandOver
  renegades bb2016 @1e6: action HandOver
  renegades bb2025 @1e6: action HandOver
  snotling bb2020 @0: action Pass
  underworld bb2020 @1e6: action HandOver
  underworld_37844 bb2025 @1e6: action HandOver
  wood_elf bb2016 @1e6: throw-ins

still short in both:
  chaos_pact bb2025 @1e6: action HandOver
  dwarf bb2020 @1e6: action HandOver
  khemri_fumbbl bb2020 @0: action Pass, pass rolls
  khemri_fumbbl bb2025 @0: action Pass, pass rolls
  necromantic bb2016 @1e6: action HandOver
  slann_fumbbl bb2020 @0: action Pass, pass rolls
  slann_fumbbl bb2025 @0: action Pass, pass rolls
  underworld bb2016 @1e6: action HandOver

## GameEvent variants

new only: none
gone: none

| event | baseline | new | change |
|---|---:|---:|---:|
| playerMoved | 14665160 | 14217803 | -3.1% |
| playerAction | 5346515 | 5243598 | -1.9% |
| goForItRoll | 1148814 | 1165655 | +1.5% |
| turnEnd | 1138141 | 1139215 | +0.1% |
| injury | 755511 | 846778 | +12.1% |
| block | 665270 | 755064 | +13.5% |
| blockRoll | 641309 | 728619 | +13.6% |
| pushback | 455756 | 506983 | +11.2% |
| playerFellDown | 434385 | 490869 | +13.0% |
| confusionRoll | 407938 | 377193 | -7.5% |
| dodgeRoll | 296930 | 345711 | +16.4% |
| scatterBall | 200016 | 193864 | -3.1% |
| pickupRoll | 125034 | 120543 | -3.6% |
| apothecaryRoll | 104843 | 110255 | +5.2% |
| kickoffResultEvent | 88397 | 89234 | +0.9% |
| kickoffScatter | 88397 | 89234 | +0.9% |
| skillUse | 50217 | 75500 | +50.3% |
| ballPickedUp | 77492 | 74217 | -4.2% |
| catchRoll | 87928 | 71778 | -18.4% |
| startHalf | 65999 | 66000 | +0.0% |
| mvpRoll | 65998 | 65996 | -0.0% |
| winningsRoll | 65998 | 65996 | -0.0% |
| foul | 52903 | 63033 | +19.1% |
| refereeSpotsFoul | 52903 | 63033 | +19.1% |
| passRoll | 41523 | 39204 | -5.6% |
| bloodLustRoll | 44052 | 38079 | -13.6% |
| animalSavagery | 34328 | 32833 | -4.4% |
| reRoll | 12655 | 26653 | +110.6% |
| touchdown | 25357 | 25856 | +2.0% |
| handOver | 33214 | 25854 | -22.2% |
| throwTeamMateRoll | 27867 | 16358 | -41.3% |
| argueTheCall | 11654 | 13598 | +16.7% |
| playerEjected | 10999 | 12707 | +15.5% |
| foulAppearanceRoll | 8114 | 12397 | +52.8% |
| standUpRoll | 7705 | 12396 | +60.9% |
| weatherChange | 11706 | 11897 | +1.6% |
| cheeringFans | 10667 | 10742 | +0.7% |
| playerNote | 9885 | 9979 | +1.0% |
| kickoffExtraReRoll | 9733 | 9911 | +1.8% |
| scatterPlayer | 11876 | 8968 | -24.5% |
| spellEffectRoll | 8088 | 8733 | +8.0% |
| lonerRoll | 6031 | 8202 | +36.0% |
| biteSpectator | 8704 | 7375 | -15.3% |
| quickSnapRoll | 7014 | 7114 | +1.4% |
| kickoffExtraReRollBb2016 | 6428 | 6563 | +2.1% |
| rightStuffRoll | 12425 | 5832 | -53.1% |
| regenerationRoll | 5117 | 5748 | +12.3% |
| solidDefenceRoll | 5547 | 5353 | -3.5% |
| alwaysHungry | 8063 | 4672 | -42.1% |
| kickoffPitchInvasionStun | 4231 | 4611 | +9.0% |
| prayerRoll | 4333 | 4380 | +1.1% |
| throwIn | 3710 | 3535 | -4.7% |
| kickoffTimeout | 3177 | 3219 | +1.3% |
| passDeviate | 3682 | 3089 | -16.1% |
| blitzRoll | 2912 | 2878 | -1.2% |
| animosityRoll | 2715 | 2568 | -5.4% |
| kickoffPitchInvasion | 2130 | 2270 | +6.6% |
| dodgySnackRoll | 1722 | 1771 | +2.8% |
| kickoffDodgySnack | 1507 | 1554 | +3.1% |
| kickoffOfficiousRef | 1421 | 1474 | +3.7% |
| dauntlessRoll | 331 | 1360 | +310.9% |
| kickoffThrowARockBb2016 | 1308 | 1285 | -1.8% |
| kickoffRiot | 1226 | 1271 | +3.7% |
| hitAndRun | 1836 | 993 | -45.9% |
| passBlock | 926 | 850 | -8.2% |
| escapeRoll | 1377 | 820 | -40.5% |
| jumpRoll | 407 | 664 | +63.1% |
| kickoffPitchInvasionBb2016 | 455 | 454 | -0.2% |
| throwAtStallingPlayer | 367 | 412 | +12.3% |
| trapDoor | 302 | 340 | +12.6% |
| bombOutOfBounds | 221 | 336 | +52.0% |
| playerAdded | 217 | 234 | +7.8% |
| kickTeamMateFumble | 183 | 181 | -1.1% |
| chainsawRoll | 49 | 144 | +193.9% |
| swoopPlayer | 115 | 62 | -46.1% |
| safeThrowRoll | 2 | 4 | +100.0% |

## PlayerAction variants

new only: none
gone: none

| action | baseline | new | change |
|---|---:|---:|---:|
| Move | 4457356 | 4272305 | -4.2% |
| Block | 398110 | 455763 | +14.5% |
| BlitzMove | 177767 | 207725 | +16.9% |
| ThrowTeamMate | 114809 | 101367 | -11.7% |
| Blitz | 67006 | 72864 | +8.7% |
| Foul | 54334 | 65134 | +19.9% |
| PassMove | 24271 | 21634 | -10.9% |
| HandOverMove | 28559 | 21411 | -25.0% |
| ThrowBomb | 8128 | 8847 | +8.8% |
| Pass | 7101 | 6214 | -12.5% |
| HandOver | 6148 | 5563 | -9.5% |
| KickTeamMate | 2806 | 4566 | +62.7% |
| Punt | 104 | 183 | +76.0% |
| HailMaryPass | 16 | 22 | +37.5% |

## Skill uses (SkillUse events, used=true)

new only: ['128', '152', '154', '17', '88']
gone: none

| skill | baseline | new | change |
|---|---:|---:|---:|
| 127 | 21437 | 18841 | -12.1% |
| 10 | 12790 | 15672 | +22.5% |
| 15 | 3024 | 14041 | +364.3% |
| 7 | 754 | 4091 | +442.6% |
| 22 | 2584 | 3136 | +21.4% |
| 152 | 0 | 3098 | n/a |
| 128 | 0 | 2210 | n/a |
| 154 | 0 | 1214 | n/a |
| 36 | 645 | 993 | +54.0% |
| 0 | 166 | 772 | +365.1% |
| 133 | 333 | 540 | +62.2% |
| 18 | 183 | 213 | +16.4% |
| 16 | 14 | 106 | +657.1% |
| 17 | 0 | 11 | n/a |
| 88 | 0 | 9 | n/a |

## Re-roll rates

| roll | baseline rolls | baseline rerolled | new rolls | new rerolled |
|---|---:|---:|---:|---:|
| goForItRoll | 1148814 | 58700 (5.1%) | 1165655 | 58106 (5.0%) |
| blockRoll | 641309 | 90064 (14.0%) | 728619 | 96100 (13.2%) |
| confusionRoll | 407938 | 0 (0.0%) | 377193 | 0 (0.0%) |
| dodgeRoll | 296930 | 44936 (15.1%) | 345711 | 50756 (14.7%) |
| pickupRoll | 125034 | 12189 (9.7%) | 120543 | 11901 (9.9%) |
| catchRoll | 87928 | 7384 (8.4%) | 71778 | 5862 (8.2%) |
| refereeSpotsFoul | 52903 | 0 (0.0%) | 63033 | 0 (0.0%) |
| passRoll | 41523 | 2748 (6.6%) | 39204 | 2689 (6.9%) |
| bloodLustRoll | 44052 | 0 (0.0%) | 38079 | 0 (0.0%) |
| animalSavagery | 34328 | 0 (0.0%) | 32833 | 0 (0.0%) |
| argueTheCall | 11654 | 0 (0.0%) | 13598 | 0 (0.0%) |
| foulAppearanceRoll | 8114 | 0 (0.0%) | 12397 | 0 (0.0%) |
| standUpRoll | 7705 | 0 (0.0%) | 12396 | 0 (0.0%) |
| spellEffectRoll | 8088 | 0 (0.0%) | 8733 | 0 (0.0%) |
| lonerRoll | 6031 | 0 (0.0%) | 8202 | 0 (0.0%) |
| rightStuffRoll | 12425 | 0 (0.0%) | 5832 | 0 (0.0%) |
| regenerationRoll | 5117 | 0 (0.0%) | 5748 | 0 (0.0%) |
| alwaysHungry | 8063 | 0 (0.0%) | 4672 | 0 (0.0%) |
| animosityRoll | 2715 | 0 (0.0%) | 2568 | 0 (0.0%) |
| dauntlessRoll | 331 | 0 (0.0%) | 1360 | 0 (0.0%) |
| escapeRoll | 1377 | 0 (0.0%) | 820 | 0 (0.0%) |
| jumpRoll | 407 | 0 (0.0%) | 664 | 0 (0.0%) |
| throwAtStallingPlayer | 367 | 0 (0.0%) | 412 | 0 (0.0%) |
| trapDoor | 302 | 0 (0.0%) | 340 | 0 (0.0%) |
| chainsawRoll | 49 | 0 (0.0%) | 144 | 8 (5.6%) |
| safeThrowRoll | 2 | 0 (0.0%) | 4 | 0 (0.0%) |

## Kick-off results

| result | baseline | new |
|---|---:|---:|
| Blitz | 4463 | 4428 |
| BrilliantCoaching | 12493 | 12685 |
| Charge | 2968 | 2942 |
| CheeringFans | 14307 | 14492 |
| DodgySnack | 1507 | 1554 |
| GetTheRef | 2454 | 2485 |
| HighKick | 11081 | 11206 |
| OficiousRef | 1421 | 1474 |
| PerfectDefence | 2182 | 2170 |
| PitchInvasion | 2585 | 2724 |
| QuickSnap | 9972 | 10049 |
| Riot | 1226 | 1271 |
| SolidDefence | 5547 | 5353 |
| ThrowARock | 1308 | 1285 |
| TimeOut | 3177 | 3219 |
| WeatherChange | 11706 | 11897 |
