# BB2020 Parity Team Drafts

Re-drafted **2026-09-10** by `scripts/draft_all_squads.py` under the Exhibition Play drafting
rules (`rules/bb2020/core_rules/07_league_and_exhibition_play.md`). Superseded the hand drafts of 2026-08-08, which were legal only against
a checker calibrated on themselves -- see `docs/PARITY_COVERAGE_REQUIREMENTS.md` §19 (R6)
for the seven defects that survived it.

Budget 1,100,000 gold, **spent in full** (in this play format unspent gold is lost, and
the drafter proves nothing cheaper remained buyable). Dedicated Fans from 0 up to 6, at 10,000 each, not in TV. Team re-rolls 0-8 at the
roster's cost; assistant coaches and cheerleaders 0-6 each at 10,000; apothecary 50,000
where the roster allows one. **No star players**: a star is an Inducement, needing gold
and Skill Points, and a mirror match has no inducement gold.

Purchase order: one of every positional the caps allow; then more players dearest-first
while 11 bodies, 3 re-rolls and an apothecary stay reachable; then the 11th and 12th
bodies; 2 re-rolls; apothecary; a 3rd re-roll; a 13th body; fans; assistant coaches and
cheerleaders; then players to 16 and re-rolls to 8 with anything left.

Jerseys are numbered round-robin over the positions, so one of every positional takes a
shirt inside the first 11 and starts on the pitch -- both harnesses field the first 11 by
number, and a positional on the bench has no parity evidence.

Specs: `data/teams/bb2020/team_<cell>.json` (home and away are identical builds).
TV per the Java `UtilTeamValue.findTeamValue`.

## `amazon`

| Purchase | Qty | Cost |
|---|---|---|
| Jaguar Warrior Blocker | 2 | 220k |
| Piranha Warrior Blitzer | 2 | 180k |
| Python Warrior Thrower | 2 | 160k |
| Eagle Warrior Linewoman | 6 | 300k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `black_orc`

| Purchase | Qty | Cost |
|---|---|---|
| Trained Troll | 1 | 115k |
| Black Orc | 6 | 540k |
| Goblin Bruiser Lineman | 6 | 270k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

13 players, TV 1095k.

## `chaos`

| Purchase | Qty | Cost |
|---|---|---|
| Minotaur | 1 | 150k |
| Chosen Blocker | 3 | 300k |
| Beastman Runner Lineman | 8 | 480k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `chaos.chaostroll`, `chaos.chaosogre` -- fielded by this cell's other squad.

## `chaos_chaosogre`

R3 variant of the `chaos` cell (roster `chaos.lrb6`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Chaos Ogre | 1 | 140k |
| Chosen Blocker | 3 | 300k |
| Beastman Runner Lineman | 8 | 480k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `chaos.chaostroll`, `chaos.minotaur` -- fielded by this cell's other squad.

## `chaos_chaostroll`

R3 variant of the `chaos` cell (roster `chaos.lrb6`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Chaos Troll | 1 | 115k |
| Chosen Blocker | 3 | 300k |
| Beastman Runner Lineman | 8 | 480k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→3 | +3 | 30k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1065k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `chaos.chaosogre`, `chaos.minotaur` -- fielded by this cell's other squad.

## `chaos_dwarf`

| Purchase | Qty | Cost |
|---|---|---|
| Minotaur | 1 | 150k |
| Bull Centaur Blitzer | 2 | 260k |
| Chaos Dwarf Flamesmith | 1 | 80k |
| Chaos Dwarf Blocker | 1 | 70k |
| Hobgoblin Sneaky Stabba | 1 | 70k |
| Hobgoblin Lineman | 7 | 280k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k.

## `chaos_pact`

| Purchase | Qty | Cost |
|---|---|---|
| Renegade Minotaur | 1 | 150k |
| Renegade Rat Ogre | 1 | 150k |
| Renegade Ogre | 1 | 140k |
| Renegade Troll | 1 | 115k |
| Renegade Dark Elf | 1 | 75k |
| Renegade Human Thrower | 1 | 75k |
| Renegade Human Lineman | 2 | 100k |
| Renegade Orc | 1 | 50k |
| Renegade Skaven | 1 | 50k |
| Renegade Goblin | 1 | 40k |
| Team re-rolls @70k | 2 | 140k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

11 players, TV 1085k.

## `dark_elf`

| Purchase | Qty | Cost |
|---|---|---|
| Witch Elf | 1 | 110k |
| Blitzer | 2 | 200k |
| Assassin | 1 | 85k |
| Runner | 1 | 80k |
| Dark Elf Lineman | 6 | 420k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

11 players, TV 1095k.

## `dark_elf_league_fumbbl`

| Purchase | Qty | Cost |
|---|---|---|
| Witch Elf | 1 | 110k |
| Blitzer | 2 | 200k |
| Assassin | 1 | 85k |
| Runner | 1 | 80k |
| Dark Elf Lineman | 6 | 420k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

11 players, TV 1095k.

## `dwarf`

| Purchase | Qty | Cost |
|---|---|---|
| Deathroller | 1 | 170k |
| Troll Slayer | 2 | 190k |
| Runner | 1 | 85k |
| Blitzer | 1 | 80k |
| Dwarf Blocker Lineman | 6 | 420k |
| Team re-rolls @50k | 2 | 100k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

11 players, TV 1095k.

## `elf`

| Purchase | Qty | Cost |
|---|---|---|
| Blitzer | 2 | 230k |
| Catcher | 2 | 200k |
| Thrower | 2 | 150k |
| Lineman | 6 | 360k |
| Team re-rolls @50k | 2 | 100k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `gnome`

| Purchase | Qty | Cost |
|---|---|---|
| Altern Forest Treeman | 2 | 240k |
| Gnome Beastmaster | 2 | 110k |
| Gnome Illusionist | 2 | 100k |
| Woodland Fox | 2 | 100k |
| Gnome Lineman | 8 | 320k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→3 | +3 | 30k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

16 players, TV 1070k.

## `goblin`

| Purchase | Qty | Cost |
|---|---|---|
| Trained Troll | 2 | 230k |
| Pogoer | 1 | 75k |
| Fanatic | 1 | 70k |
| 'Ooligan | 1 | 65k |
| Doom Diver | 1 | 60k |
| Bomma | 1 | 45k |
| Goblin Lineman | 7 | 280k |
| Looney | 1 | 40k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

15 players, TV 1095k.

## `halfling`

| Purchase | Qty | Cost |
|---|---|---|
| Altern Forest Treeman | 2 | 240k |
| Halfling Catcher | 2 | 110k |
| Halfling Hefty | 2 | 100k |
| Halfling Hopeful Lineman | 10 | 300k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Assistant coaches | 6 | 60k |
| Dedicated Fans 0→6 | +6 | 60k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

16 players, TV 1040k.

## `high_elf`

| Purchase | Qty | Cost |
|---|---|---|
| Blitzer | 2 | 200k |
| Thrower | 1 | 100k |
| Catcher | 2 | 180k |
| Lineman | 6 | 420k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

## `human`

| Purchase | Qty | Cost |
|---|---|---|
| Ogre | 1 | 140k |
| Blitzer | 4 | 340k |
| Thrower | 2 | 160k |
| Catcher | 2 | 130k |
| Human Lineman | 2 | 100k |
| Halfling Hopeful | 1 | 30k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `imperial_nobility`

| Purchase | Qty | Cost |
|---|---|---|
| Ogre | 1 | 140k |
| Noble Blitzer | 2 | 210k |
| Bodyguard | 2 | 180k |
| Imperial Thrower | 2 | 150k |
| Imperial Retainer Lineman | 5 | 225k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1095k.

## `khemri`

| Purchase | Qty | Cost |
|---|---|---|
| Tomb Guardian | 4 | 400k |
| Anointed Blitzer | 2 | 180k |
| Anointed Thrower | 2 | 140k |
| Skeleton Lineman | 4 | 160k |
| Team re-rolls @70k | 3 | 210k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `khemri_fumbbl`

| Purchase | Qty | Cost |
|---|---|---|
| Tomb Guardian | 4 | 400k |
| Anointed Blitzer | 2 | 180k |
| Anointed Thrower | 2 | 140k |
| Skeleton Lineman | 4 | 160k |
| Team re-rolls @70k | 3 | 210k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `khorne`

| Purchase | Qty | Cost |
|---|---|---|
| Bloodspawn | 1 | 160k |
| Bloodseeker | 3 | 330k |
| Khorngor | 2 | 140k |
| Bloodborn Marauder Lineman | 6 | 300k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `lizardman`

| Purchase | Qty | Cost |
|---|---|---|
| Kroxigor | 1 | 140k |
| Saurus Blocker | 4 | 340k |
| Chameleon Skink | 1 | 70k |
| Skink Runner Lineman | 6 | 360k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `necromantic`

| Purchase | Qty | Cost |
|---|---|---|
| Werewolf | 2 | 250k |
| Flesh Golem | 2 | 230k |
| Wraith | 2 | 190k |
| Ghoul Runner | 1 | 75k |
| Zombie Lineman | 5 | 200k |
| Team re-rolls @70k | 2 | 140k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1085k.

## `nippon`

| Purchase | Qty | Cost |
|---|---|---|
| Ninja | 2 | 180k |
| Samurai | 4 | 340k |
| Warrior Monk | 2 | 140k |
| Ashigaru | 4 | 200k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `norse`

| Purchase | Qty | Cost |
|---|---|---|
| Yhetee | 1 | 140k |
| Ulfwerener | 2 | 210k |
| Valkyrie | 2 | 190k |
| Norse Berzerker | 2 | 180k |
| Norse Raider Lineman | 3 | 150k |
| Beer Boar | 2 | 40k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→2 | +2 | 20k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1080k.

## `nurgle`

| Purchase | Qty | Cost |
|---|---|---|
| Rotspawn | 1 | 140k |
| Bloater | 4 | 460k |
| Pestigor | 2 | 150k |
| Rotter Lineman | 6 | 210k |
| Team re-rolls @70k | 2 | 140k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k.

## `ogre`

| Purchase | Qty | Cost |
|---|---|---|
| Ogre Runt Punter | 1 | 145k |
| Ogre Blocker | 4 | 560k |
| Gnoblar Lineman | 9 | 135k |
| Team re-rolls @70k | 3 | 210k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

14 players, TV 1100k.

## `old_world_alliance_ogre`

R3 variant of the `old_world_alliance` cell (roster `oldworldalliance.lrb6`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Ogre | 1 | 140k |
| Old World Dwarf Troll Slayer | 1 | 95k |
| Old World Human Blitzer | 1 | 90k |
| Old World Dwarf Runner | 1 | 85k |
| Old World Dwarf Blitzer | 1 | 80k |
| Old World Human Thrower | 1 | 80k |
| Old World Dwarf Blocker | 2 | 150k |
| Old World Human Catcher | 1 | 65k |
| Old World Human Lineman | 1 | 50k |
| Old World Halfling Hopefuls | 2 | 60k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1085k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `oldworldalliance.altern_forest_treeman` -- fielded by this cell's other squad.

## `old_world_alliance_treeman`

R3 variant of the `old_world_alliance` cell (roster `oldworldalliance.lrb6`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Altern Forest Treeman | 1 | 120k |
| Old World Dwarf Troll Slayer | 1 | 95k |
| Old World Human Blitzer | 1 | 90k |
| Old World Dwarf Runner | 1 | 85k |
| Old World Dwarf Blitzer | 1 | 80k |
| Old World Human Thrower | 1 | 80k |
| Old World Dwarf Blocker | 1 | 75k |
| Old World Human Catcher | 1 | 65k |
| Old World Human Lineman | 3 | 150k |
| Old World Halfling Hopefuls | 2 | 60k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1090k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `oldworldalliance.ogre` -- fielded by this cell's other squad.

## `orc`

| Purchase | Qty | Cost |
|---|---|---|
| Untrained Troll | 1 | 115k |
| Big Un | 4 | 360k |
| Blitzer | 3 | 240k |
| Thrower | 1 | 65k |
| Orc Lineman | 2 | 100k |
| Goblin | 1 | 40k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `renegades`

| Purchase | Qty | Cost |
|---|---|---|
| Renegade Minotaur | 1 | 150k |
| Renegade Rat Ogre | 1 | 150k |
| Renegade Ogre | 1 | 140k |
| Renegade Human Thrower | 1 | 75k |
| Renegade Dark Elf | 1 | 75k |
| Renegade Human Lineman | 3 | 150k |
| Renegade Orc | 1 | 50k |
| Renegade Skaven | 1 | 50k |
| Renegade Goblin | 1 | 40k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→3 | +3 | 30k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1070k. The official page allows 3 Big Guys; this squad has 3. Held back by that cap: `37730` -- fielded by this cell's other squad.

## `renegades_37730`

R3 variant of the `renegades` cell (roster `1050157`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Renegade Rat Ogre | 1 | 150k |
| Renegade Ogre | 1 | 140k |
| Renegade Troll | 1 | 115k |
| Renegade Human Thrower | 1 | 75k |
| Renegade Dark Elf | 1 | 75k |
| Renegade Human Lineman | 4 | 200k |
| Renegade Orc | 1 | 50k |
| Renegade Skaven | 1 | 50k |
| Renegade Goblin | 1 | 40k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1085k. The official page allows 3 Big Guys; this squad has 3. Held back by that cap: `37732` -- fielded by this cell's other squad.

## `skaven`

| Purchase | Qty | Cost |
|---|---|---|
| Rat Ogre | 1 | 150k |
| Blitzer | 2 | 180k |
| Gutter Runner | 3 | 255k |
| Thrower | 1 | 85k |
| Skaven Clanrat Lineman | 5 | 250k |
| Team re-rolls @50k | 2 | 100k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→3 | +3 | 30k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1070k.

## `slann`

| Purchase | Qty | Cost |
|---|---|---|
| Kroxigor | 1 | 140k |
| Blitzer | 2 | 220k |
| Catcher | 3 | 240k |
| Lineman | 5 | 300k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

## `slann_fumbbl`

| Purchase | Qty | Cost |
|---|---|---|
| Kroxigor | 1 | 140k |
| Blitzer | 2 | 220k |
| Catcher | 3 | 240k |
| Lineman | 5 | 300k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

## `snotling`

| Purchase | Qty | Cost |
|---|---|---|
| Trained Troll | 2 | 230k |
| Pump Wagon | 2 | 210k |
| Fungus Flinga | 2 | 60k |
| Fun-Hoppa | 2 | 40k |
| Stilty Runna | 2 | 40k |
| Snotling Lineman | 6 | 90k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Assistant coaches | 6 | 60k |
| Cheerleaders | 6 | 60k |
| Dedicated Fans 0→6 | +6 | 60k |
| **Total spent** | | **1080k** |
| Gold lost (unspendable) | | 20k |

16 players, TV 1020k.

## `undead`

| Purchase | Qty | Cost |
|---|---|---|
| Mummy | 2 | 250k |
| Wight Blitzer | 2 | 180k |
| Ghoul Runner | 4 | 300k |
| Skeleton Lineman | 3 | 120k |
| Zombie Lineman | 1 | 40k |
| Team re-rolls @70k | 3 | 210k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `underworld`

| Purchase | Qty | Cost |
|---|---|---|
| Mutant Rat Ogre | 1 | 150k |
| Skaven Blitzer | 1 | 90k |
| Gutter Runner | 1 | 85k |
| Skaven Thrower | 1 | 85k |
| Skaven Clanrat | 3 | 150k |
| Underworld Goblin Lineman | 6 | 240k |
| Underworld Snotling | 2 | 30k |
| Team re-rolls @70k | 3 | 210k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

15 players, TV 1090k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `37844.underworldtroll` -- fielded by this cell's other squad.

## `underworld_underworldtroll`

R3 variant of the `underworld` cell (roster `underworld.lrb6`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Underworld Troll | 1 | 115k |
| Skaven Blitzer | 1 | 90k |
| Gutter Runner | 1 | 85k |
| Skaven Thrower | 1 | 85k |
| Skaven Clanrat | 3 | 150k |
| Underworld Goblin Lineman | 7 | 280k |
| Underworld Snotling | 2 | 30k |
| Team re-rolls @70k | 3 | 210k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

16 players, TV 1095k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `37844` -- fielded by this cell's other squad.

## `vampire`

| Purchase | Qty | Cost |
|---|---|---|
| Vargheist | 1 | 150k |
| Vampire Blitzer | 2 | 220k |
| Vampire Thrower | 1 | 110k |
| Vampire Runner | 1 | 100k |
| Thrall Lineman | 7 | 280k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Dedicated Fans 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `wood_elf`

| Purchase | Qty | Cost |
|---|---|---|
| Wardancer | 1 | 125k |
| Loren Forest Treeman | 1 | 120k |
| Thrower | 2 | 190k |
| Catcher | 1 | 90k |
| Wood Elf Lineman | 6 | 420k |
| Team re-rolls @50k | 2 | 100k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

11 players, TV 1095k.

