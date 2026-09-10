# BB2025 Parity Team Drafts

Re-drafted **2026-09-10** by `scripts/draft_all_squads.py` under the Matched Play drafting
rules (`rules/core_rules/04_drafting_a_blood_bowl_team.md` + `06_matched_play.md`). Superseded the hand drafts of 2026-08-08, which were legal only against
a checker calibrated on themselves -- see `docs/PARITY_COVERAGE_REQUIREMENTS.md` §19 (R6)
for the seven defects that survived it.

Budget 1,100,000 gold, **spent in full** (in this play format unspent gold is lost, and
the drafter proves nothing cheaper remained buyable). Dedicated Fans from 1 up to 3, at 5,000 each, not in TV. Team re-rolls 0-8 at the
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

Specs: `data/teams/bb2025/team_<cell>.json` (home and away are identical builds).
TV per the Java `UtilTeamValue.findTeamValue`.

## `amazon`

| Purchase | Qty | Cost |
|---|---|---|
| Jaguar Warrior | 2 | 220k |
| Piranha Warrior | 2 | 180k |
| Python Warrior | 2 | 160k |
| Eagle Warrior | 6 | 300k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `black_orc`

| Purchase | Qty | Cost |
|---|---|---|
| Trained Troll | 1 | 115k |
| Black Orc | 6 | 540k |
| Goblin Bruiser | 6 | 270k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→2 | +1 | 5k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1095k.

## `bretonnian`

| Purchase | Qty | Cost |
|---|---|---|
| Grail Knight | 2 | 190k |
| Bretonnian Knight Catcher | 2 | 170k |
| Bretonnian Knight Thrower | 2 | 160k |
| Bretonnian Squire | 7 | 350k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k.

## `chaos`

| Purchase | Qty | Cost |
|---|---|---|
| Minotaur | 1 | 150k |
| Chaos Chosen | 4 | 400k |
| Beastman Lineman | 7 | 385k |
| Team re-rolls @50k | 2 | 100k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1085k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `chaos.troll`, `chaos.ogre` -- fielded by this cell's other squad.

## `chaos_dwarf`

| Purchase | Qty | Cost |
|---|---|---|
| Minotaur | 1 | 150k |
| Bull Centaur | 2 | 260k |
| Flamesmith | 2 | 160k |
| Chaos Dwarf Blocker | 1 | 70k |
| Sneaky Stabba | 1 | 60k |
| Hobgoblin Lineman | 5 | 200k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `chaos_ogre`

R3 variant of the `chaos` cell (roster `chaos.lrb6`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Ogre | 1 | 140k |
| Chaos Chosen | 4 | 400k |
| Beastman Lineman | 7 | 385k |
| Team re-rolls @50k | 2 | 100k |
| Apothecary | 1 | 50k |
| Assistant coaches | 1 | 10k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1085k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `chaos.troll`, `chaos.minotaur` -- fielded by this cell's other squad.

## `chaos_pact`

| Purchase | Qty | Cost |
|---|---|---|
| Minotaur | 1 | 150k |
| Chaos Ogre | 1 | 140k |
| Chaos Troll | 1 | 110k |
| Dark Elf Renegade | 1 | 70k |
| Marauder | 7 | 350k |
| Skaven Renegade | 1 | 50k |
| Goblin Renegade | 1 | 40k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k.

## `chaos_troll`

R3 variant of the `chaos` cell (roster `chaos.lrb6`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Troll | 1 | 115k |
| Chaos Chosen | 4 | 400k |
| Beastman Lineman | 7 | 385k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `chaos.ogre`, `chaos.minotaur` -- fielded by this cell's other squad.

## `dark_elf`

| Purchase | Qty | Cost |
|---|---|---|
| Witch Elf | 2 | 220k |
| Dark Elf Blitzer | 1 | 105k |
| Dark Elf Assassin | 1 | 90k |
| Dark Elf Runner | 2 | 160k |
| Dark Elf Lineman | 5 | 325k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

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
| Dedicated Fans 1→2 | +1 | 5k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1095k.

## `dwarf`

| Purchase | Qty | Cost |
|---|---|---|
| Deathroller | 1 | 170k |
| Dwarf Blitzer | 2 | 200k |
| Troll Slayer | 1 | 95k |
| Dwarf Runner | 2 | 160k |
| Dwarf Lineman | 5 | 350k |
| Team re-rolls @60k | 2 | 120k |
| Dedicated Fans 1→2 | +1 | 5k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1095k.

## `elf`

| Purchase | Qty | Cost |
|---|---|---|
| Elf Blitzer | 2 | 230k |
| Elf Catcher | 2 | 200k |
| Elf Thrower | 1 | 75k |
| Elf Lineman | 6 | 390k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→2 | +1 | 5k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1095k.

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
| Assistant coaches | 2 | 20k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

16 players, TV 1090k.

## `goblin`

| Purchase | Qty | Cost |
|---|---|---|
| Trained Troll | 2 | 230k |
| Pogoer | 1 | 75k |
| Fanatic | 1 | 70k |
| Doom Diver | 1 | 65k |
| 'Ooligan | 1 | 60k |
| Bomma | 1 | 45k |
| Goblin Lineman | 7 | 280k |
| Looney | 1 | 40k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→2 | +1 | 5k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

15 players, TV 1095k.

## `halfling`

| Purchase | Qty | Cost |
|---|---|---|
| Athel Forest Treeman | 2 | 240k |
| Halfling Catcher | 2 | 110k |
| Halfling Hefty | 2 | 100k |
| Halfling Hopeful | 10 | 300k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Assistant coaches | 6 | 60k |
| Cheerleaders | 5 | 50k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

16 players, TV 1090k.

## `high_elf`

| Purchase | Qty | Cost |
|---|---|---|
| Dragon Prince | 2 | 220k |
| White Lion | 1 | 110k |
| Phoenix Warrior | 2 | 180k |
| High Elf Lineman | 6 | 390k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

## `human`

| Purchase | Qty | Cost |
|---|---|---|
| Ogre | 1 | 140k |
| Human Blitzer | 2 | 170k |
| Human Catcher | 2 | 150k |
| Human Thrower | 2 | 150k |
| Human Lineman | 5 | 250k |
| Halfling Hopeful | 1 | 30k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1090k.

## `imperial_nobility`

| Purchase | Qty | Cost |
|---|---|---|
| Ogre | 1 | 140k |
| Noble Blitzer | 2 | 180k |
| Bodyguard | 4 | 340k |
| Imperial Thrower | 1 | 75k |
| Imperial Retainer | 4 | 180k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1085k.

## `khemri`

| Purchase | Qty | Cost |
|---|---|---|
| Tomb Guardian | 4 | 460k |
| Tomb Kings Blitzer | 2 | 170k |
| Tomb Kings Thrower | 2 | 130k |
| Skeleton Lineman | 4 | 160k |
| Team re-rolls @60k | 3 | 180k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `khemri_fumbbl`

| Purchase | Qty | Cost |
|---|---|---|
| Tomb Guardian | 4 | 400k |
| Anointed Blitzer | 2 | 180k |
| Anointed Thrower | 2 | 140k |
| Skeleton Lineman | 4 | 160k |
| Team re-rolls @70k | 3 | 210k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `khorne`

| Purchase | Qty | Cost |
|---|---|---|
| Bloodspawn | 1 | 160k |
| Bloodseeker | 3 | 315k |
| Khorngor | 2 | 140k |
| Bloodborn Marauder | 6 | 300k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1085k.

## `lizardman`

| Purchase | Qty | Cost |
|---|---|---|
| Kroxigor | 1 | 140k |
| Saurus Blocker | 3 | 270k |
| Chameleon Skink | 2 | 140k |
| Skink Lineman | 6 | 360k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `necromantic`

| Purchase | Qty | Cost |
|---|---|---|
| Werewolf | 2 | 240k |
| Flesh Golem | 2 | 220k |
| Wraith | 2 | 170k |
| Ghoul Runner | 1 | 75k |
| Zombie Lineman | 6 | 240k |
| Team re-rolls @70k | 2 | 140k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

13 players, TV 1085k.

## `nippon`

| Purchase | Qty | Cost |
|---|---|---|
| Ninja | 2 | 180k |
| Samurai | 4 | 340k |
| Warrior Monk | 2 | 140k |
| Ashigaru | 4 | 200k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `norse`

| Purchase | Qty | Cost |
|---|---|---|
| Yhetee | 1 | 140k |
| Ulfwerener | 2 | 210k |
| Valkyrie | 2 | 190k |
| Norse Berserker | 2 | 180k |
| Norse Raider | 3 | 150k |
| Beer Boar | 2 | 40k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Assistant coaches | 1 | 10k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `nurgle`

| Purchase | Qty | Cost |
|---|---|---|
| Rotspawn | 1 | 140k |
| Bloater | 4 | 440k |
| Pestigor | 2 | 140k |
| Rotter | 5 | 200k |
| Team re-rolls @60k | 3 | 180k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

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
| Dwarf Blitzer | 1 | 100k |
| Troll Slayer | 1 | 95k |
| Human Blitzer | 1 | 85k |
| Dwarf Runner | 1 | 80k |
| Human Catcher | 1 | 75k |
| Human Thrower | 1 | 75k |
| Dwarf Lineman | 1 | 70k |
| Human Lineman | 2 | 100k |
| Halfling Hopeful | 3 | 90k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `oldworldalliance.altern_forest_treeman` -- fielded by this cell's other squad.

## `old_world_alliance_treeman`

R3 variant of the `old_world_alliance` cell (roster `oldworldalliance.lrb6`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Altern Forest Treeman | 1 | 120k |
| Dwarf Blitzer | 1 | 100k |
| Troll Slayer | 1 | 95k |
| Human Blitzer | 1 | 85k |
| Dwarf Runner | 1 | 80k |
| Human Catcher | 1 | 75k |
| Human Thrower | 1 | 75k |
| Dwarf Lineman | 2 | 140k |
| Human Lineman | 1 | 50k |
| Halfling Hopeful | 3 | 90k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `oldworldalliance.ogre` -- fielded by this cell's other squad.

## `orc`

| Purchase | Qty | Cost |
|---|---|---|
| Troll | 1 | 115k |
| Big Un Blocker | 2 | 190k |
| Orc Blitzer | 2 | 170k |
| Orc Thrower | 2 | 150k |
| Orc Lineman | 4 | 200k |
| Goblin Lineman | 1 | 40k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→2 | +1 | 5k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1095k.

## `renegades`

| Purchase | Qty | Cost |
|---|---|---|
| Minotaur | 1 | 150k |
| Ogre | 1 | 140k |
| Troll | 1 | 115k |
| Renegade Human Thrower | 1 | 75k |
| Renegade Dark Elf | 1 | 65k |
| Renegade Human | 4 | 200k |
| Renegade Orc | 1 | 50k |
| Renegade Skaven | 1 | 50k |
| Renegade Goblin | 1 | 40k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| Assistant coaches | 1 | 10k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1085k. The official page allows 3 Big Guys; this squad has 3. Held back by that cap: `37733` -- fielded by this cell's other squad.

## `renegades_37733`

R3 variant of the `renegades` cell (roster `1050157`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Rat Ogre | 1 | 150k |
| Ogre | 1 | 140k |
| Troll | 1 | 115k |
| Renegade Human Thrower | 1 | 75k |
| Renegade Dark Elf | 1 | 65k |
| Renegade Human | 4 | 200k |
| Renegade Orc | 1 | 50k |
| Renegade Skaven | 1 | 50k |
| Renegade Goblin | 1 | 40k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| Assistant coaches | 1 | 10k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

12 players, TV 1085k. The official page allows 3 Big Guys; this squad has 3. Held back by that cap: `37732` -- fielded by this cell's other squad.

## `skaven`

| Purchase | Qty | Cost |
|---|---|---|
| Rat Ogre | 1 | 150k |
| Skaven Blitzer | 2 | 180k |
| Gutter Runner | 2 | 170k |
| Skaven Thrower | 2 | 160k |
| Skaven Clanrat | 5 | 250k |
| Team re-rolls @50k | 2 | 100k |
| Apothecary | 1 | 50k |
| Assistant coaches | 3 | 30k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

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
| Pump Wagon | 2 | 200k |
| Fungus Flinga | 2 | 60k |
| Fun-hoppa | 2 | 40k |
| Stilty Runna | 2 | 40k |
| Snotling Lineman | 6 | 90k |
| Team re-rolls @70k | 3 | 210k |
| Apothecary | 1 | 50k |
| Assistant coaches | 6 | 60k |
| Cheerleaders | 6 | 60k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1050k** |
| Gold lost (unspendable) | | 50k |

16 players, TV 1040k.

## `undead`

| Purchase | Qty | Cost |
|---|---|---|
| Mummy | 2 | 250k |
| Wight Blitzer | 2 | 190k |
| Ghoul Runner | 2 | 150k |
| Skeleton Lineman | 6 | 240k |
| Zombie Lineman | 1 | 40k |
| Team re-rolls @70k | 3 | 210k |
| Assistant coaches | 1 | 10k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1090k.

## `underworld`

| Purchase | Qty | Cost |
|---|---|---|
| Troll | 1 | 115k |
| Skaven Blitzer | 1 | 90k |
| Gutter Runner | 1 | 85k |
| Skaven Thrower | 1 | 80k |
| Skaven Clanrat | 3 | 150k |
| Goblin Lineman | 7 | 280k |
| Snotling Lineman | 2 | 30k |
| Team re-rolls @70k | 3 | 210k |
| Apothecary | 1 | 50k |
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

16 players, TV 1090k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `37844` -- fielded by this cell's other squad.

## `underworld_37844`

R3 variant of the `underworld` cell (roster `underworld.lrb6`), fielding the Big Guy the base squad may not.

| Purchase | Qty | Cost |
|---|---|---|
| Rat Ogre | 1 | 150k |
| Skaven Blitzer | 1 | 90k |
| Gutter Runner | 1 | 85k |
| Skaven Thrower | 1 | 80k |
| Skaven Clanrat | 3 | 150k |
| Goblin Lineman | 6 | 240k |
| Snotling Lineman | 3 | 45k |
| Team re-rolls @70k | 3 | 210k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

16 players, TV 1100k. The official page allows 1 Big Guy; this squad has 1. Held back by that cap: `underworld.troll.warpstone` -- fielded by this cell's other squad.

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
| Dedicated Fans 1→3 | +2 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1090k.

## `wood_elf`

| Purchase | Qty | Cost |
|---|---|---|
| Wardancer | 1 | 130k |
| Loren Forest Treeman | 1 | 120k |
| Wood Elf Catcher | 1 | 90k |
| Wood Elf Thrower | 2 | 170k |
| Wood Elf Lineman | 6 | 390k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

