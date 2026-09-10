# BB2016 Parity Team Drafts

Re-drafted **2026-09-10** by `scripts/draft_all_squads.py` under the CRP tournament drafting
rules (`docs/BB2016_DRAFTING_AND_ROSTERS.md`). Superseded the hand drafts of 2026-08-08, which were legal only against
a checker calibrated on themselves -- see `docs/PARITY_COVERAGE_REQUIREMENTS.md` §19 (R6)
for the seven defects that survived it.

Budget 1,100,000 gold, **spent in full** (in this play format unspent gold is lost, and
the drafter proves nothing cheaper remained buyable). Fan Factor 0-9 at 10,000 each, which **counts in Team Value**. Team re-rolls 0-8 at the
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

Specs: `data/teams/bb2016/team_<cell>.json` (home and away are identical builds).
TV per the Java `UtilTeamValue.findTeamValue`.

## `amazon`

| Purchase | Qty | Cost |
|---|---|---|
| Blitzer | 4 | 360k |
| Catcher | 2 | 140k |
| Thrower | 2 | 140k |
| Linewoman | 5 | 250k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| Fan Factor 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k.

## `chaos`

| Purchase | Qty | Cost |
|---|---|---|
| Minotaur | 1 | 150k |
| Chaos Warrior | 3 | 300k |
| Chaos Beastman | 8 | 480k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `chaos_dwarf`

| Purchase | Qty | Cost |
|---|---|---|
| Minotaur | 1 | 150k |
| Bull Centaur | 2 | 260k |
| Chaos Dwarf Blocker | 3 | 210k |
| Hobgoblin | 7 | 280k |
| Team re-rolls @70k | 2 | 140k |
| Apothecary | 1 | 50k |
| Fan Factor 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k.

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

## `dark_elf`

| Purchase | Qty | Cost |
|---|---|---|
| Witch Elf | 1 | 110k |
| Blitzer | 2 | 200k |
| Assassin | 1 | 90k |
| Runner | 1 | 80k |
| Lineman | 6 | 420k |
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
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

11 players, TV 1095k.

## `dwarf`

| Purchase | Qty | Cost |
|---|---|---|
| Deathroller | 1 | 160k |
| Troll Slayer | 1 | 90k |
| Blitzer | 1 | 80k |
| Runner | 1 | 80k |
| Blocker | 7 | 490k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

## `elf`

| Purchase | Qty | Cost |
|---|---|---|
| Blitzer | 2 | 220k |
| Catcher | 3 | 300k |
| Thrower | 2 | 140k |
| Lineman | 4 | 240k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

## `goblin`

| Purchase | Qty | Cost |
|---|---|---|
| Troll | 2 | 220k |
| Fanatic | 1 | 70k |
| Pogoer | 1 | 70k |
| Bombardier | 1 | 40k |
| Goblin | 10 | 400k |
| Looney | 1 | 40k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Fan Factor 0→3 | +3 | 30k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

16 players, TV 1100k.

## `halfling`

| Purchase | Qty | Cost |
|---|---|---|
| Treeman | 2 | 240k |
| Halfling | 14 | 420k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Assistant coaches | 6 | 60k |
| Cheerleaders | 6 | 60k |
| Fan Factor 0→9 | +9 | 90k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

16 players, TV 1100k.

## `high_elf`

| Purchase | Qty | Cost |
|---|---|---|
| Blitzer | 2 | 200k |
| Catcher | 2 | 180k |
| Thrower | 1 | 90k |
| Lineman | 6 | 420k |
| Team re-rolls @50k | 3 | 150k |
| Apothecary | 1 | 50k |
| Fan Factor 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

## `human`

| Purchase | Qty | Cost |
|---|---|---|
| Ogre | 1 | 140k |
| Human Blitzer | 4 | 360k |
| Human Catcher | 4 | 280k |
| Human Thrower | 1 | 70k |
| Human Lineman | 2 | 100k |
| Team re-rolls @50k | 2 | 100k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `khemri`

| Purchase | Qty | Cost |
|---|---|---|
| Tomb Guardian | 4 | 400k |
| Blitz-Ra | 2 | 180k |
| Thro-Ra | 2 | 140k |
| Skeleton | 4 | 160k |
| Team re-rolls @70k | 3 | 210k |
| Fan Factor 0→1 | +1 | 10k |
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
| Fan Factor 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `lizardman`

| Purchase | Qty | Cost |
|---|---|---|
| Kroxigor | 1 | 140k |
| Saurus | 6 | 480k |
| Skink | 5 | 300k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Fan Factor 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `necromantic`

| Purchase | Qty | Cost |
|---|---|---|
| Werewolf | 2 | 240k |
| Flesh Golem | 2 | 220k |
| Wight | 2 | 180k |
| Ghoul | 1 | 70k |
| Zombie | 6 | 240k |
| Team re-rolls @70k | 2 | 140k |
| Fan Factor 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k.

## `nippon`

| Purchase | Qty | Cost |
|---|---|---|
| Ninja | 2 | 180k |
| Samurai | 4 | 340k |
| Warrior Monk | 2 | 140k |
| Ashigaru | 4 | 200k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| Fan Factor 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `norse`

| Purchase | Qty | Cost |
|---|---|---|
| Snow Troll | 1 | 140k |
| Ulfwerener | 2 | 220k |
| Berserker | 1 | 90k |
| Runner | 1 | 90k |
| Thrower | 2 | 140k |
| Lineman | 5 | 250k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `nurgle`

| Purchase | Qty | Cost |
|---|---|---|
| Beast of Nurgle | 1 | 140k |
| Nurgle Warrior | 4 | 440k |
| Pestigor | 1 | 80k |
| Rotter | 7 | 280k |
| Team re-rolls @70k | 2 | 140k |
| Fan Factor 0→2 | +2 | 20k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k.

## `ogre`

| Purchase | Qty | Cost |
|---|---|---|
| Ogre | 5 | 700k |
| Snotling | 7 | 140k |
| Team re-rolls @70k | 3 | 210k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `orc`

| Purchase | Qty | Cost |
|---|---|---|
| Troll | 1 | 110k |
| Black Orc Blocker | 4 | 320k |
| Orc Blitzer | 3 | 240k |
| Orc Thrower | 1 | 70k |
| Orc Lineman | 1 | 50k |
| Goblin | 2 | 80k |
| Team re-rolls @60k | 3 | 180k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `renegades`

| Purchase | Qty | Cost |
|---|---|---|
| Renegade Minotaur | 1 | 150k |
| Renegade Rat Ogre | 1 | 150k |
| Renegade Ogre | 1 | 140k |
| Renegade Troll | 1 | 115k |
| Renegade Human Thrower | 1 | 75k |
| Renegade Dark Elf | 1 | 75k |
| Renegade Human Lineman | 2 | 100k |
| Renegade Orc | 1 | 50k |
| Renegade Skaven | 1 | 50k |
| Renegade Goblin | 1 | 40k |
| Team re-rolls @70k | 2 | 140k |
| Fan Factor 0→1 | +1 | 10k |
| **Total spent** | | **1095k** |
| Gold lost (unspendable) | | 5k |

11 players, TV 1095k.

## `skaven`

| Purchase | Qty | Cost |
|---|---|---|
| Rat Ogre | 1 | 150k |
| Blitzer | 2 | 180k |
| Gutter Runner | 4 | 320k |
| Thrower | 1 | 70k |
| Lineman | 4 | 200k |
| Team re-rolls @60k | 2 | 120k |
| Apothecary | 1 | 50k |
| Fan Factor 0→1 | +1 | 10k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

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

## `undead`

| Purchase | Qty | Cost |
|---|---|---|
| Mummy | 2 | 240k |
| Wight | 2 | 180k |
| Ghoul | 4 | 280k |
| Skeleton | 3 | 120k |
| Zombie | 1 | 40k |
| Team re-rolls @70k | 3 | 210k |
| Fan Factor 0→3 | +3 | 30k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

12 players, TV 1100k.

## `underworld`

| Purchase | Qty | Cost |
|---|---|---|
| Warpstone Troll | 1 | 110k |
| Skaven Blitzer | 2 | 180k |
| Skaven Thrower | 2 | 140k |
| Skaven Lineman | 2 | 100k |
| Underworld Goblin | 7 | 280k |
| Team re-rolls @70k | 3 | 210k |
| Apothecary | 1 | 50k |
| Fan Factor 0→3 | +3 | 30k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

14 players, TV 1100k.

## `vampire`

| Purchase | Qty | Cost |
|---|---|---|
| Vampire | 6 | 660k |
| Thrall | 7 | 280k |
| Team re-rolls @70k | 2 | 140k |
| Fan Factor 0→2 | +2 | 20k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

13 players, TV 1100k.

## `wood_elf`

| Purchase | Qty | Cost |
|---|---|---|
| Treeman | 1 | 120k |
| Wardancer | 1 | 120k |
| Catcher | 3 | 270k |
| Thrower | 1 | 90k |
| Lineman | 5 | 350k |
| Team re-rolls @50k | 2 | 100k |
| Apothecary | 1 | 50k |
| **Total spent** | | **1100k** |
| Gold lost (unspendable) | | 0k |

11 players, TV 1100k.

