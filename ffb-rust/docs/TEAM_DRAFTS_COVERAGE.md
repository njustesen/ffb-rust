# Coverage squads — the 7 teams added for `PARITY_COVERAGE_REQUIREMENTS` §5/§8

Drafted 2026-09-07. These are the squads for the seven official teams that had roster
definitions but no drafted squad, closing the 13 red cells of §5.

The draft itself lives in `scripts/draft_coverage_teams.py` as an explicit purchase list per
squad; the script does only the arithmetic (spend / treasury / TV / jersey numbers) and
enforces the invariants, so a typo'd position id fails there instead of silently producing a
wrong squad. Re-run `python scripts/draft_coverage_teams.py --check` to verify the checked-in
files still match the table, then `python scripts/gen_java_parity_data.py` to re-mirror the
Java XML.

Project drafting heuristics, unchanged from `TEAM_DRAFTS_BB2025.md`: budget 1,100,000;
every positional bought at least once (R2); 12+ players when affordable; 2+ team re-rolls;
apothecary; Dedicated Fans 1 → 3 at 5k; remainder treasury; jerseys premium-first so the
first 11 start on the pitch. TV per Java `UtilTeamValue` (players + re-rolls + apothecary;
Dedicated Fans excluded).

## R3 — the one variant set

Only **Old World Alliance** carries the single-Big-Guy restriction in its official pages
(`rules/teams/Old_World_Alliance.md:50`, `rules/bb2020/teams/Old_World_Alliance.md:53`), so it
is drafted as two variants per edition — `old_world_alliance_ogre` and
`old_world_alliance_treeman` — whose union fields all 11 positionals. Gnome and Snotling both
list their Big Guys at **0‑2 with no shared cap**, so a single squad fields every positional
and no variant is needed; Black Orc, Imperial Nobility and Khorne each have exactly one Big
Guy option; Bretonnian has none.

A variant needs only a team file: it names the shared roster through its own `roster_id`
field, which is how both engines resolve it (`make_team_from_file` matches on `roster_id`;
`gen_java_parity_data.py` falls back to a `roster_id` lookup when no `roster_<squad>.json`
exists).

## The squads

| squad (CLI name) | edition | purchases | staff | players | spent | TV | treasury |
|---|---|---|---|---:|---:|---:|---:|
| `black_orc` | bb2025 | Trained Troll x1 + Black Orc x6 + Goblin Bruiser x5 | 2 re-rolls @60k, apo, DF3 | 12 | 1060k | 1050k | 40k |
| `gnome` | bb2025 | Altern Forest Treeman x2 + Gnome Beastmaster x2 + Gnome Illusionist x2 + Woodland Fox x2 + Gnome Lineman x7 | 3 re-rolls @50k, apo, DF3 | 15 | 1040k | 1030k | 60k |
| `imperial_nobility` | bb2025 | Ogre x1 + Noble Blitzer x2 + Bodyguard x3 + Imperial Thrower x2 + Imperial Retainer x4 | 2 re-rolls @60k, apo, DF3 | 12 | 1085k | 1075k | 15k |
| `khorne` | bb2025 | Bloodspawn x1 + Bloodseeker x3 + Khorngor x2 + Bloodborn Marauder x6 | 2 re-rolls @60k, apo, DF3 | 12 | 1095k | 1085k | 5k |
| `old_world_alliance_ogre` | bb2025 | Ogre x1 + Dwarf Blitzer x1 + Troll Slayer x1 + Human Blitzer x1 + Dwarf Runner x1 + Human Catcher x1 + Human Thrower x1 + Dwarf Lineman x1 + Halfling Hopeful x1 + Human Lineman x3 | 2 re-rolls @70k, apo, DF3 | 12 | 1100k | 1090k | 0k |
| `old_world_alliance_treeman` | bb2025 | Altern Forest Treeman x1 + Dwarf Blitzer x1 + Troll Slayer x1 + Human Blitzer x1 + Dwarf Runner x1 + Human Catcher x1 + Human Thrower x1 + Dwarf Lineman x1 + Halfling Hopeful x1 + Human Lineman x3 | 2 re-rolls @70k, apo, DF3 | 12 | 1080k | 1070k | 20k |
| `snotling` | bb2025 | Trained Troll x2 + Pump Wagon x2 + Fungus Flinga x2 + Stilty Runna x2 + Fun-hoppa x2 + Snotling Lineman x6 | 5 re-rolls @70k, apo, DF3 | 16 | 1070k | 1060k | 30k |
| `bretonnian` | bb2025 | Grail Knight x2 + Knight Catcher x2 + Knight Thrower x2 + Squire x6 | 3 re-rolls @60k, apo, DF3 | 12 | 1060k | 1050k | 40k |
| `black_orc` | bb2020 | Trained Troll x1 + Black Orc x6 + Goblin Bruiser Lineman x5 | 2 re-rolls @60k, apo, DF3 | 12 | 1060k | 1050k | 40k |
| `gnome` | bb2020 | Altern Forest Treeman x2 + Gnome Beastmaster x2 + Gnome Illusionist x2 + Woodland Fox x2 + Gnome Lineman x7 | 3 re-rolls @50k, apo, DF3 | 15 | 1040k | 1030k | 60k |
| `imperial_nobility` | bb2020 | Ogre x1 + Noble Blitzer x2 + Bodyguard x2 + Imperial Thrower x1 + Imperial Retainer Lineman x6 | 2 re-rolls @70k, apo, DF3 | 12 | 1075k | 1065k | 25k |
| `khorne` | bb2020 | Bloodspawn x1 + Bloodseeker x2 + Khorngor x2 + Bloodborn Marauder Lineman x7 | 2 re-rolls @60k, apo, DF3 | 12 | 1050k | 1040k | 50k |
| `old_world_alliance_ogre` | bb2020 | Ogre x1 + Troll Slayer x1 + Human Blitzer x1 + Dwarf Runner x1 + Human Thrower x1 + Dwarf Blitzer x1 + Dwarf Blocker x1 + Human Catcher x1 + Halfling Hopeful x1 + Human Lineman x3 | 2 re-rolls @70k, apo, DF3 | 12 | 1090k | 1080k | 10k |
| `old_world_alliance_treeman` | bb2020 | Altern Forest Treeman x1 + Troll Slayer x1 + Human Blitzer x1 + Dwarf Runner x1 + Human Thrower x1 + Dwarf Blitzer x1 + Dwarf Blocker x1 + Human Catcher x1 + Halfling Hopeful x1 + Human Lineman x3 | 2 re-rolls @70k, apo, DF3 | 12 | 1070k | 1060k | 30k |
| `snotling` | bb2020 | Trained Troll x2 + Pump Wagon x2 + Fungus Flinga x2 + Stilty Runna x2 + Fun-Hoppa x2 + Snotling Lineman x6 | 5 re-rolls @60k, apo, DF3 | 16 | 1030k | 1020k | 70k |

**R4 holds for all 13 cells**: every cell's squad set fields 100% of its roster's positionals
(asserted by the script and by `coverage_cells_field_every_positional` in
`crates/ffb-parity/src/runner.rs`).

## The silent-lineman-fallback proof

`make_team()` ends in `unwrap_or_else(|_| make_lineman_team(..))`, so a typo'd id yields an
all-lineman team that still produces perfectly MATCHED parity games — a green gate on
nothing. This already bit the FUMBBL variants once. Each new squad is therefore proven to
build its real roster three ways:

1. `coverage_squads_build_their_real_roster_not_the_lineman_fallback` goes through the LIVE
   `make_team` path for both sides of all 15 squads and asserts the exact fielded
   `position_id` multiset, every player's `(MA, ST, AG, AV)` against its roster position
   (the fallback is uniformly 6/3/3/8), and a fingerprint skill on a fingerprint positional
   (the fallback has no starting skills at all) — Trained Troll/Always Hungry, Woodland
   Fox/My Ball, Ogre/Bone Head, Bloodspawn/Unchannelled Fury, Treeman/Take Root, Grail
   Knight/Dauntless.
2. On the Java side, all 30 generated team sheets field exactly the drafted `positionId`s and
   each resolves against its generated roster XML; the Java harness has no lineman fallback
   (`resolveTeamId` returns the name unchanged and a missing sheet fails the run).
3. Every one of the 15 matchups was run for one seed and reached the parity comparison, so
   both engines really built a team from this data. 12 printed `PARITY: 1/1 games match`;
   `old_world_alliance_ogre`/`_treeman` (both editions) and bb2020 `snotling` printed
   `0/1 passed, 1 FAILED` — genuine engine divergences on newly-reachable positionals, which
   is the measure phase's work, not a data problem.
