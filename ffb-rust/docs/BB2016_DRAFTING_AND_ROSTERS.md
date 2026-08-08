# BB2016 Team Drafting Rules & Roster Data — Source of Truth

This document is the in-repo source of truth for **BB2016-edition team creation** (the
`Rules::Bb2016` ruleset, FUMBBL "LRB6" — i.e. the CRP/Competition Rules Pack as updated by
the 2016 boxed edition and its Death Zone supplements, exactly as implemented by the Java
reference engine). The BB2025 equivalent lives in `rules/core_rules/04_drafting_a_blood_bowl_team.md`
plus the per-team pages in `rules/teams/`.

## 1. Drafting rules (CRP)

| Rule | Value |
|---|---|
| Team draft budget | 1,000,000 gold pieces (this project's parity teams use **1,100,000** — see `docs/TEAM_DRAFTS_BB2016.md`) |
| Players | min 11, max 16, within each position's `0-N` quantity limit |
| Team re-rolls | 0-8, at the roster's listed re-roll cost (double cost if bought after creation) |
| Fan Factor | 0-9, at 10,000 each, bought at creation (BB2016 keeps CRP-style purchasable FF; contrast BB2025's Dedicated Fans) |
| Assistant coaches | 10,000 each |
| Cheerleaders | 10,000 each |
| Apothecary | 0-1, 50,000, only if the roster allows one (Undead/Khemri/Necromantic/Nurgle/Vampire do not) |

**Team Value** (as computed by the reference engine, `ffb-common .../util/UtilTeamValue.java`):

```
TV = rerolls × roster_reroll_cost
   + fan_factor × 10,000
   + assistant_coaches × 10,000
   + cheerleaders × 10,000
   + apothecaries × 50,000
   + Σ player position costs (+ earned-skill costs, 0 for rookies)
```

Note: BB2016 TV **includes** Fan Factor; BB2025 TV excludes Dedicated Fans.

## 2. Roster data source

The normative BB2016 roster data is `data/rosters/bb2016/roster_*.json`. It derives from the
reference engine's own FUMBBL LRB6 roster exports (`ffb-server/rosters/roster_*.xml`), which are
CRP-faithful for costs, quantities and stats (verified samples: Amazon 50k re-rolls, Dwarf 40k
re-rolls, both correct LRB6 values). AG/AV use the **old scale** (AV 8 means "roll > 8 breaks
armour" per the BB2016 `StatsMechanic`); the `pa` field is a BB2020-era artifact that BB2016
mechanics do not read.

### 2.1 Contamination cleaned by the audit

The FUMBBL exports mix in BB2020-era data that does not exist in BB2016. The audit
(2026-08-08) removes, for every `data/rosters/bb2016/*.json` position:

1. **Skills that are not in the BB2016 skill set**: `Plague Ridden`, `Multiple Block` (as a
   roster starting skill), `Projectile Vomit`, `Animal Savagery`, `Safe Pair of Hands`,
   `On The Ball`, `Hit and Run`, `Defensive`, `Unchannelled Fury`.
2. **Parameterized skill values** (BB2020 syntax): `Mighty Blow(2)` → `Mighty Blow`,
   `Loner(4)` → `Loner`, `Secret Weapon(5)` → `Secret Weapon`, `Animosity(all)`/`Animosity(goblin)`
   → `Animosity`, `Bloodlust(3)` → `Blood Lust`, `Hatred(...)` → removed (not a BB2016 skill).
3. **Cross-edition dual spellings** — each edition's JSON keeps only its own canonical name:
   BB2016 keeps `Bone-Head`, `Claw` (the bb2016 Java classes); drops `bone head`, `Bone Head`,
   `Claws` duplicates (bb2020/25 spellings).
4. **Dead entries**: positions with `quantity: 0` (e.g. `human.catcher.old`).
5. **BB2016 spelling normalization** to the Java `SkillFactory.forName` canon for the BB2016
   skill classes (e.g. `Bone-Head`, `Throw Team-Mate`, `Blood Lust`).

Non-goblin examples cleaned: goblin (extras on every position), halfling (`Hatred`, `Multiple
Block`), norse snow troll & necromantic werewolf (`Claw`+`Claws` dual), human ogre
(`bone head`+`bone-head` dual), vampire (`Bloodlust(3)`).

### 2.2 Rosters with no BB2016-official definition (FUMBBL-legacy)

These matrix rosters are **not** part of CRP/BB2016 and are kept as-is (minus mechanical
cleanup) purely for FUMBBL-compat parity runs; they are flagged "FUMBBL-legacy" in the
BB2016 matrix report:

| CLI key | Why |
|---|---|
| `nippon` | FUMBBL custom roster (id 5681), never official |
| `renegades` | The stock XML is FUMBBL's **BB2020** Chaos Renegade roster (id 1050157, Animal Savagery etc.); the BB2016-official equivalent is `chaos_pact` |
| `dark_elf_league_fumbbl` (4959) | FUMBBL BB2020 league variant |
| `khemri_fumbbl` (55051) | FUMBBL BB2020 league variant |
| `slann_fumbbl` (744258) | FUMBBL BB2020 league variant |
| `lineman` | Synthetic parity fixture, not a real roster |

`chaos_pact`, `underworld` and `slann` **are** legitimate CRP-era rosters (CRP includes Chaos
Pact, Underworld and Slann) and are audited normally.

### 2.3 BB2016 roster reference tables

After cleanup, the 24 audited BB2016 rosters match the CRP/LRB6 published values. The tables
below are the audited reference (quantity / cost / MA / ST / AG / AV / starting skills; re-roll
cost and apothecary in the header). Any future change to `data/rosters/bb2016/` must be
reconciled against these tables.

*(Generated from the audited `data/rosters/bb2016/*.json` — see `scripts/audit_rosters.py
--edition bb2016 --tables` which regenerates this section; a mismatch between this doc and the
data is a test failure.)*

<!-- BB2016_TABLES_START -->
<!-- Regenerated by scripts/audit_rosters.py --edition bb2016 --tables -->

#### Amazon (`amazon`)
Re-rolls 50k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-4 | Blitzer | 90k | 6 | 3 | 3 | 7 | Dodge, Block |
| 0-2 | Catcher | 70k | 6 | 3 | 3 | 7 | Dodge, Catch |
| 0-2 | Thrower | 70k | 6 | 3 | 3 | 7 | Dodge, Pass |
| 0-16 | Linewoman | 50k | 6 | 3 | 3 | 7 | Dodge |

#### Chaos (`chaos`)
Re-rolls 60k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-12 | Chaos Beastman | 60k | 6 | 3 | 3 | 8 | Horns |
| 0-4 | Chaos Warrior | 100k | 5 | 4 | 3 | 9 |  |
| 0-1 | Minotaur | 150k | 5 | 5 | 2 | 8 | Loner, Frenzy, Horns, Mighty Blow, Thick Skull, Wild Animal |

#### Chaos Dwarf (`chaos_dwarf`)
Re-rolls 70k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Hobgoblin | 40k | 6 | 3 | 3 | 7 |  |
| 0-6 | Chaos Dwarf Blocker | 70k | 4 | 3 | 2 | 9 | Block, Tackle, Thick Skull |
| 0-2 | Bull Centaur | 130k | 6 | 4 | 2 | 9 | Sprint, Sure Feet, Thick Skull |
| 0-1 | Minotaur | 150k | 5 | 5 | 2 | 8 | Loner, Frenzy, Horns, Mighty Blow, Thick Skull, Wild Animal |

#### Chaos Pact (`chaos_pact`) *(FUMBBL-legacy)*
Re-rolls 70k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-12 | Marauder | 50k | 6 | 3 | 3 | 8 |  |
| 0-1 | Goblin Renegade | 40k | 6 | 2 | 3 | 7 | Animosity, Dodge, Right Stuff, Stunty |
| 0-1 | Skaven Renegade | 50k | 7 | 3 | 3 | 7 | Animosity |
| 0-1 | Dark Elf Renegade | 70k | 6 | 3 | 4 | 8 | Animosity |
| 0-1 | Chaos Troll | 110k | 4 | 5 | 1 | 9 | Always Hungry, Loner, Mighty Blow, Really Stupid, Regeneration, Throw Team-Mate |
| 0-1 | Chaos Ogre | 140k | 5 | 5 | 2 | 9 | Bone-Head, Loner, Mighty Blow, Thick Skull, Throw Team-Mate |
| 0-1 | Minotaur | 150k | 5 | 5 | 2 | 8 | Frenzy, Horns, Loner, Mighty Blow, Thick Skull, Wild Animal |

#### Dark Elf (`dark_elf`)
Re-rolls 50k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Lineman | 70k | 6 | 3 | 4 | 8 |  |
| 0-4 | Blitzer | 100k | 7 | 3 | 4 | 8 | Block |
| 0-2 | Runner | 80k | 7 | 3 | 4 | 7 | Dump-Off |
| 0-2 | Assassin | 90k | 6 | 3 | 4 | 7 | Shadowing, Stab |
| 0-2 | Witch Elf | 110k | 7 | 3 | 4 | 7 | Dodge, Frenzy, Jump Up |

#### Dark Elf (`dark_elf_league_fumbbl`) *(FUMBBL-legacy)*
Re-rolls 50k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-4 | Blitzer | 100k | 7 | 3 | 2 | 9 | Block |
| 0-12 | Dark Elf Lineman | 70k | 6 | 3 | 2 | 9 |  |
| 0-2 | Assassin | 85k | 7 | 3 | 2 | 8 | Shadowing, Stab |
| 0-2 | Runner | 80k | 7 | 3 | 2 | 8 | Dump-off |
| 0-2 | Witch Elf | 110k | 7 | 3 | 2 | 8 | Dodge, Frenzy, Jump Up |

#### Dwarf (`dwarf`)
Re-rolls 40k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-12 | Blocker | 70k | 4 | 3 | 2 | 9 | Block, Tackle, Thick Skull |
| 0-2 | Blitzer | 80k | 5 | 3 | 3 | 9 | Block, Thick Skull |
| 0-2 | Runner | 80k | 6 | 3 | 3 | 8 | Sure Hands, Thick Skull |
| 0-2 | Troll Slayer | 90k | 5 | 3 | 2 | 8 | Block, Dauntless, Frenzy, Thick Skull |
| 0-1 | Deathroller | 160k | 4 | 7 | 1 | 10 | Break Tackle, Dirty Player, Juggernaut, Mighty Blow, No Hands, Secret Weapon, Stand Firm |

#### Elf (`elf`)
Re-rolls 50k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Lineman | 60k | 6 | 3 | 4 | 7 |  |
| 0-4 | Catcher | 100k | 8 | 3 | 4 | 7 | Catch, Nerves of Steel |
| 0-2 | Thrower | 70k | 6 | 3 | 4 | 7 | Pass |
| 0-2 | Blitzer | 110k | 7 | 3 | 4 | 8 | Block, Side Step |

#### Goblin (`goblin`)
Re-rolls 60k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Goblin | 40k | 6 | 2 | 3 | 7 | Dodge, Right Stuff, Stunty, Thick Skull, Animosity, Regeneration |
| 0-2 | Troll | 110k | 4 | 5 | 1 | 9 | Always Hungry, Loner, Mighty Blow, Really Stupid, Regeneration, Throw Team-Mate, Regeneration |
| 0-1 | Bombardier | 40k | 6 | 2 | 3 | 7 | Bombardier, Dodge, Secret Weapon, Stunty, Regeneration |
| 0-1 | Pogoer | 70k | 7 | 2 | 3 | 7 | Dodge, Leap, Stunty, Very Long Legs, Animosity, Regeneration |
| 0-1 | Looney | 40k | 6 | 2 | 3 | 7 | Chainsaw, Secret Weapon, Stunty, Animosity, Regeneration |
| 0-1 | Fanatic | 70k | 3 | 7 | 3 | 7 | Ball and Chain, No Hands, Secret Weapon, Stunty |

#### Halfling (`halfling`)
Re-rolls 60k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-2 | Treeman | 120k | 2 | 6 | 1 | 11 | Loner, Mighty Blow, Stand Firm, Strong Arm, Take Root, Thick Skull, Throw Team-Mate |
| 0-16 | Halfling | 30k | 5 | 2 | 3 | 6 | Dodge, Right Stuff |

#### High Elf (`high_elf`)
Re-rolls 50k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Lineman | 70k | 6 | 3 | 4 | 8 |  |
| 0-2 | Thrower | 90k | 6 | 3 | 4 | 8 | Pass, Safe Throw |
| 0-2 | Blitzer | 100k | 7 | 3 | 4 | 8 | Block |
| 0-4 | Catcher | 90k | 8 | 3 | 4 | 7 | Catch |

#### Human (`human`)
Re-rolls 50k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Human Lineman | 50k | 6 | 3 | 3 | 8 |  |
| 0-4 | Human Blitzer | 90k | 7 | 3 | 3 | 8 | Block |
| 0-2 | Human Thrower | 70k | 6 | 3 | 3 | 8 | Pass, Sure Hands |
| 0-4 | Human Catcher | 70k | 8 | 2 | 3 | 7 | Catch, Dodge |
| 0-1 | Ogre | 140k | 5 | 5 | 2 | 9 | Loner, Bone-Head, Mighty Blow, Thick Skull, Throw Team-Mate |

#### Khemri (`khemri`)
Re-rolls 70k · Apothecary no

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-4 | Tomb Guardian | 100k | 4 | 5 | 1 | 9 | Decay, Regeneration |
| 0-2 | Blitz-Ra | 90k | 6 | 3 | 2 | 8 | Block, Regeneration |
| 0-2 | Thro-Ra | 70k | 6 | 3 | 2 | 7 | Pass, Regeneration, Sure Hands |
| 0-16 | Skeleton | 40k | 5 | 3 | 2 | 7 | Regeneration, Thick Skull |

#### Tomb Kings (`khemri_fumbbl`) *(FUMBBL-legacy)*
Re-rolls 70k · Apothecary no

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-2 | Anointed Thrower | 70k | 6 | 3 | 2 | 7 | Pass, Regeneration, Sure Hands |
| 0-16 | Skeleton Lineman | 40k | 5 | 3 | 2 | 7 | Regeneration, Thick Skull |
| 0-4 | Tomb Guardian | 100k | 4 | 5 | 1 | 9 | Decay, Regeneration |
| 0-2 | Anointed Blitzer | 90k | 6 | 3 | 2 | 8 | Block, Regeneration |

#### Lizardman (`lizardman`)
Re-rolls 60k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-1 | Kroxigor | 140k | 6 | 5 | 1 | 9 | Loner, Bone-Head, Mighty Blow, Prehensile Tail, Thick Skull |
| 0-6 | Saurus | 80k | 6 | 4 | 1 | 9 |  |
| 0-16 | Skink | 60k | 8 | 2 | 3 | 7 | Dodge, Stunty |

#### Necromantic (`necromantic`)
Re-rolls 70k · Apothecary no

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-2 | Werewolf | 120k | 8 | 3 | 3 | 8 | Claw, Frenzy, Regeneration |
| 0-2 | Flesh Golem | 110k | 4 | 4 | 2 | 9 | Regeneration, Stand Firm, Thick Skull |
| 0-2 | Wight | 90k | 6 | 3 | 3 | 8 | Block, Regeneration |
| 0-2 | Ghoul | 70k | 7 | 3 | 3 | 7 | Dodge |
| 0-16 | Zombie | 30k | 4 | 3 | 2 | 8 | Regeneration |

#### Nippon (`nippon`) *(FUMBBL-legacy)*
Re-rolls 60k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Ashigaru | 50k | 6 | 3 | 3 | 9 |  |
| 0-4 | Samurai | 85k | 7 | 3 | 3 | 9 | Block |
| 0-2 | Warrior Monk | 70k | 7 | 3 | 3 | 8 | Claw |
| 0-2 | Ninja | 90k | 8 | 2 | 2 | 8 | Dodge, Leap |

#### Norse (`norse`)
Re-rolls 60k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Lineman | 50k | 6 | 3 | 3 | 7 | Block |
| 0-2 | Thrower | 70k | 6 | 3 | 3 | 7 | Block, Pass |
| 0-2 | Runner | 90k | 7 | 3 | 3 | 7 | Block, Dauntless |
| 0-2 | Berserker | 90k | 6 | 3 | 3 | 7 | Block, Frenzy, Jump Up |
| 0-2 | Ulfwerener | 110k | 6 | 4 | 2 | 8 | Frenzy |
| 0-1 | Snow Troll | 140k | 5 | 5 | 1 | 8 | Claw, Disturbing Presence, Frenzy, Loner, Wild Animal |

#### Nurgle (`nurgle`)
Re-rolls 70k · Apothecary no

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Rotter | 40k | 5 | 3 | 3 | 8 | Decay, Nurgle's Rot |
| 0-4 | Pestigor | 80k | 6 | 3 | 3 | 8 | Horns, Nurgle's Rot, Regeneration |
| 0-4 | Nurgle Warrior | 110k | 4 | 4 | 2 | 9 | Disturbing Presence, Foul Appearance, Nurgle's Rot, Regeneration |
| 0-1 | Beast of Nurgle | 140k | 4 | 5 | 1 | 9 | Disturbing Presence, Foul Appearance, Loner, Mighty Blow, Nurgle's Rot, Really Stupid, Regeneration, Tentacles |

#### Ogre (`ogre`)
Re-rolls 70k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Snotling | 20k | 5 | 1 | 3 | 5 | Dodge, Right Stuff, Side Step, Stunty, Titchy |
| 0-6 | Ogre | 140k | 5 | 5 | 2 | 9 | Bone-Head, Mighty Blow, Thick Skull, Throw Team-Mate |

#### Orc (`orc`)
Re-rolls 60k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-1 | Troll | 110k | 4 | 5 | 1 | 9 | Loner, Always Hungry, Mighty Blow, Really Stupid, Regeneration, Throw Team-Mate |
| 0-4 | Orc Blitzer | 80k | 6 | 3 | 3 | 9 | Block |
| 0-4 | Black Orc Blocker | 80k | 4 | 4 | 2 | 9 |  |
| 0-2 | Orc Thrower | 70k | 5 | 3 | 3 | 8 | Pass, Sure Hands |
| 0-4 | Goblin | 40k | 6 | 2 | 3 | 7 | Right Stuff, Dodge, Stunty |
| 0-16 | Orc Lineman | 50k | 5 | 3 | 3 | 9 |  |

#### Chaos Renegade (`renegades`)
Re-rolls 70k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-1 | Renegade Rat Ogre | 150k | 6 | 5 | 4 | 9 | Frenzy, Loner, Mighty Blow, Prehensile Tail |
| 0-1 | Renegade Ogre | 140k | 5 | 5 | 4 | 10 | Bone-Head, Loner, Mighty Blow, Thick Skull, Throw Team-mate |
| 0-1 | Renegade Troll | 115k | 4 | 5 | 5 | 10 | Always Hungry, Loner, Mighty Blow, Really Stupid, Regeneration, Throw Team-mate |
| 0-1 | Renegade Dark Elf | 75k | 6 | 3 | 2 | 9 | Animosity |
| 0-1 | Renegade Human Thrower | 75k | 6 | 3 | 3 | 9 | Animosity, Pass |
| 0-1 | Renegade Orc | 50k | 5 | 3 | 3 | 10 | Animosity |
| 0-1 | Renegade Skaven | 50k | 7 | 3 | 3 | 8 | Animosity |
| 0-1 | Renegade Goblin | 40k | 6 | 2 | 3 | 8 | Animosity, Dodge, Right Stuff, Stunty |
| 0-12 | Renegade Human Lineman | 50k | 6 | 3 | 3 | 9 |  |
| 0-1 | Renegade Minotaur | 150k | 5 | 5 | 4 | 9 | Frenzy, Horns, Loner, Mighty Blow, Thick Skull |

#### Skaven (`skaven`)
Re-rolls 60k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-1 | Rat Ogre | 160k | 6 | 5 | 2 | 8 | Loner, Frenzy, Mighty Blow, Prehensile Tail, Wild Animal |
| 0-2 | Blitzer | 90k | 7 | 3 | 3 | 8 | Block |
| 0-4 | Gutter Runner | 80k | 9 | 2 | 4 | 7 | Dodge |
| 0-2 | Thrower | 70k | 7 | 3 | 3 | 7 | Pass, Sure Hands |
| 0-16 | Lineman | 50k | 7 | 3 | 3 | 7 |  |

#### Slann (`slann`) *(FUMBBL-legacy)*
Re-rolls 50k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-16 | Lineman | 60k | 6 | 3 | 3 | 8 | Leap, Very Long Legs |
| 0-4 | Catcher | 80k | 7 | 2 | 4 | 7 | Diving Catch, Leap, Very Long Legs |
| 0-4 | Blitzer | 110k | 7 | 3 | 3 | 8 | Diving Tackle, Jump Up, Leap, Very Long Legs |
| 0-1 | Kroxigor | 140k | 6 | 5 | 1 | 9 | Bone-Head, Loner, Mighty Blow, Prehensile Tail, Thick Skull |

#### Slann (`slann_fumbbl`) *(FUMBBL-legacy)*
Re-rolls 50k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-4 | Blitzer | 110k | 7 | 3 | 3 | 8 | Diving Tackle, Jump Up, Leap, Very Long Legs |
| 0-4 | Catcher | 80k | 7 | 2 | 4 | 7 | Diving Catch, Leap, Very Long Legs |
| 0-1 | Kroxigor | 140k | 6 | 5 | 1 | 9 | Bone-Head, Loner, Mighty Blow, Prehensile Tail, Thick Skull |
| 0-16 | Lineman | 60k | 6 | 3 | 3 | 8 | Leap, Very Long Legs |

#### Undead (`undead`)
Re-rolls 70k · Apothecary no

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-2 | Mummy | 110k | 3 | 5 | 1 | 9 | Mighty Blow, Regeneration |
| 0-2 | Wight | 90k | 6 | 3 | 3 | 8 | Block, Regeneration |
| 0-4 | Ghoul | 70k | 7 | 3 | 3 | 7 | Dodge |
| 0-16 | Zombie | 40k | 4 | 3 | 2 | 8 | Regeneration |
| 0-16 | Skeleton | 30k | 5 | 3 | 2 | 7 | Regeneration |

#### Underworld (`underworld`)
Re-rolls 70k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-2 | Mutant Rat Ogre | 150k | 6 | 5 | 4 | 9 | Frenzy, Loner, Mighty Blow, Prehensile Tail |
| 0-12 | Underworld Goblin | 40k | 6 | 2 | 3 | 7 | Dodge, Right Stuff, Stunty, Animosity |
| 0-16 | Skaven Lineman | 50k | 7 | 3 | 3 | 7 | Animosity |
| 0-2 | Skaven Thrower | 70k | 7 | 3 | 3 | 7 | Animosity, Pass, Sure Hands |
| 0-2 | Skaven Blitzer | 90k | 7 | 3 | 3 | 8 | Animosity, Block |
| 0-2 | Warpstone Troll | 110k | 4 | 5 | 1 | 9 | Always Hungry, Loner, Mighty Blow, Really Stupid, Regeneration, Throw Team-Mate |

#### Vampire (`vampire`)
Re-rolls 70k · Apothecary no

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-6 | Vampire | 110k | 6 | 4 | 4 | 8 | Blood Lust, Hypnotic Gaze, Regeneration |
| 0-16 | Thrall | 40k | 6 | 3 | 3 | 7 |  |

#### Wood Elf (`wood_elf`)
Re-rolls 50k · Apothecary yes

| Qty | Position | Cost | MA | ST | AG | AV | Skills |
|---|---|---|---|---|---|---|---|
| 0-1 | Treeman | 120k | 2 | 6 | 1 | 10 | Loner, Mighty Blow, Stand Firm, Strong Arm, Take Root, Thick Skull, Throw Team-Mate |
| 0-2 | Wardancer | 120k | 8 | 3 | 4 | 7 | Block, Dodge, Leap |
| 0-2 | Thrower | 90k | 7 | 3 | 4 | 7 | Pass |
| 0-4 | Catcher | 90k | 9 | 2 | 4 | 7 | Catch, Dodge |
| 0-16 | Lineman | 70k | 7 | 3 | 4 | 7 |  |
<!-- BB2016_TABLES_END -->
