# BB2020 roster provenance — reconciliation against the official team pages

Written 2026-09-07. Closes item 1 of `docs/PARITY_COVERAGE_REQUIREMENTS.md` §6
("Add a `bb2020` mode to `audit_rosters.py`"). Before this, `rules/bb2020/teams/`
held 29 authoritative pages that **nothing in the repo read**, so every bb2020
cell was red on provenance regardless of its parity number.

## 1. The mode

```bash
python scripts/audit_rosters.py --edition bb2020 --report            # per-team, per-position diff
python scripts/audit_rosters.py --edition bb2020 --report --verbose  # + the page-row -> position pairing
python scripts/audit_rosters.py --edition bb2020 --selftest          # validate the CHECKER itself
python scripts/audit_rosters.py --edition bb2020 --apply             # rewrite the roster JSONs
```

It reconciles `data/rosters/bb2020/*.json` against `rules/bb2020/teams/*.md` the
way `audit_bb2025()` reconciles against `rules/teams/`: per team, per position,
on **quantity, cost, MA/ST/AG/PA/AV, starting skills (name *and* numeric value),
and primary/secondary skill categories**, plus the team-level **re-roll cost**
and **apothecary** flag.

`--apply` is implemented and dry-run verified (§6) but **was NOT RUN**: at the
time of writing another workflow was concurrently writing `data/rosters/**` and
`data/teams/**`. Nothing under `data/` was touched by this work
(`git status --porcelain data` is empty).

### Deliberate scope differences from the bb2025 path

| field | bb2025 | bb2020 | why |
|---|---|---|---|
| position `type` (Regular / Big Guy) | diffed | **not diffed** | the bb2025 pages carry a `_(Lineman, Big Guy)_` keyword parenthetical; the bb2020 pages have no keyword column at all, so there is no page-side truth. bb2020 marks its big guys only with a `*` footnote ("*An X team may include a single Big Guy*"). |
| `special_rules` | diffed | **informational** | the bb2020 roster JSONs carry no `special_rules` key. The page's rules are printed as `[cosmetic]` so the data is visible without manufacturing 29 phantom mismatches. |
| `max_rerolls` | forced to 8 | untouched | not stated on the page. |
| string skill values | diffed | **representation note** | see §3. |

### Page-format differences the parser handles

`parse_team_page(path, edition)` is now edition-aware. The bb2020 Positionals
table has the same 11 columns in the same order, but:

* the position cell is bare (`Eagle Warrior Linewoman`), with no
  `_(Keyword, Keyword)_` italic suffix;
* skill anchors are `#dodge`, not bb2025's `#dodge-active`, and each entry is
  bullet-prefixed (`• [Dodge](...)`) — neither affects the skill regex;
* the armour column header is `Ar`, not `AV` (parsing is positional, so this is
  cosmetic);
* a trailing `*` on the position name marks a mutually-limited Big Guy choice and
  is stripped.

## 2. Team-name mapping (NOT identity)

`OFFICIAL_BB2020` maps the repo race key to the official page slug. Eight are not
identity:

| race key | bb2020 page |
|---|---|
| `chaos` | `Chaos_Chosen` |
| `khemri` | `Tomb_Kings` |
| `elf` | `Elven_Union` |
| `undead` | `Shambling_Undead` |
| `underworld` | `Underworld_Denizens` |
| `renegades` | `Chaos_Renegades` |
| `lizardman` | `Lizardmen` |
| `necromantic` | `Necromantic_Horror` |

The other 21 are identity-after-snake-casing (`black_orc` → `Black_Orc`,
`high_elf` → `High_Elf`, `imperial_nobility` → `Imperial_Nobility`,
`old_world_alliance` → `Old_World_Alliance`, …).

**All 29 official bb2020 teams are now mapped and audited.** Six rosters we test
are not official bb2020 teams and are deliberately left untouched:
`chaos_pact`, `slann`, `nippon`, `dark_elf_league_fumbbl`, `khemri_fumbbl`,
`slann_fumbbl` (their bb2020 cells are N/A per `PARITY_COVERAGE_REQUIREMENTS.md`
§4, not green).

## 3. Skill-name mappings

The bb2020 pages use the BB2020-canonical spelling for every skill on all 29
teams **except** the two below. Direction is page → engine-canonical, and note it
is *not* the same table as bb2025's: BB2025 renamed Side Step to "Sidestep", so
the two tables map in **opposite directions**. `SKILL_ALIASES_BB2020` is a
separate table from `SKILL_ALIASES_BB2025` for exactly that reason — do not
unify them.

| page spelling | engine-canonical (bb2020) | why |
|---|---|---|
| `Ball & Chain` | `Ball and Chain` | semantically required — Java `SkillFactory.forName` special-cases the ampersand; the skill class's canonical name is "Ball and Chain". |
| `Sidestep` | `Side Step` | the bb2020 rosters spell it "Side Step". Cosmetic for resolution (both engines match case- and punctuation-insensitively) but applied so `--apply` writes the roster-canonical spelling. |

Two skills appear on bb2020 pages and, before this session, in **no** bb2020
roster JSON: **`Arm Bar`** (Old World Alliance) and **`Swarming`** (Snotling).
Both exist in the engine (`data/skills/mixed_skills.json` → `ArmBar`,
`data/skills/bb2020_skills.json` → `Swarming`), so neither needs an alias; they
pass through and the newly-added rosters for those two teams reconcile clean.

### Spelling differences that are NOT mismatches

Both engines resolve skill names case- and punctuation-insensitively (Java
`SkillFactory.forName` is case-insensitive; Rust `SkillId::from_class_name`
strips non-alphanumerics), so the checker compares skills on a normalised key
(`skill_key()`) and reports case/punctuation-only differences as `[cosmetic]`.
Three observations that follow:

1. **The bb2020 roster JSONs are internally inconsistent on three skill names** —
   they contain **both** `Bone Head`/`Bone head`, `Dump-Off`/`Dump-off` and
   `Jump Up`/`Jump up`. Harmless to both engines, but worth normalising the next
   time those files are rewritten. **Recorded, not changed.**
2. **The skill-category name is spelled both `Mutation` and `Mutations`** across
   the bb2020 rosters. `CATEGORY_SYNONYMS` treats them as one, and categories are
   compared as **sets** because their order is meaningless in both engines.
   **Recorded, not changed.**
3. **String skill values are prose on the page and sometimes ids in the JSON.**
   `Animosity` is the only case. The pages say `(All)`,
   `(Underworld Goblin Linemen)`, `(dwarfs and halflings)`; `roster_underworld`
   and `roster_renegades` store the same prose, but
   `roster_old_world_alliance` stores a semicolon-separated list of position ids
   (`oldworldalliance.dwarf_blitzer;…`). Prose is not mechanically comparable to
   an id list, so **numeric** skill values (Loner 4+, Mighty Blow +1) are
   compared strictly while **string** values produce a labelled
   `[representation]` note. Deleting a string value entirely is still a
   mismatch (asserted in the selftest) — only prose-vs-encoding is excused.

## 4. The mismatch inventory (the deliverable)

**All 29 official bb2020 teams now have a roster JSON. 24 match their official
page exactly. 5 differ — and every one of the 5 differences is a team-level
staff cost, not a positional. Zero positional mismatches exist: not one
quantity, cost, MA, ST, AG, PA, AV, starting skill, numeric skill value or skill
category disagrees with the official page across all 29 rosters and their 155
positions.**

That is the headline for the coverage matrix: **the bb2020 positional data is
correct as authored**, and the column clears provenance after five single-number
corrections.

### 4a. The 5 mismatches (6 wrong numbers)

| race | page | field | JSON has | page says |
|---|---|---|---|---|
| `amazon` | Amazon | `reroll_cost` | 50 000 | **60 000** |
| `dwarf` | Dwarf | `reroll_cost` | 40 000 | **50 000** |
| `lizardman` | Lizardmen | `reroll_cost` | 60 000 | **70 000** |
| `skaven` | Skaven | `reroll_cost` | 60 000 | **50 000** |
| `vampire` | Vampire | `reroll_cost` | 70 000 | **60 000** |
| `vampire` | Vampire | `apothecary` | `false` | **`true`** |

All six were re-read by hand off the pages' `### Staff` sections (§6). Note
`skaven` and `vampire` are **too expensive** in the JSON while `amazon`,
`dwarf` and `lizardman` are **too cheap** — this is not one systematic offset,
it is five independent transcription errors.

At least two are BB2016-era values never updated for BB2020 (40k dwarf and 60k
skaven re-rolls are the LRB6/CRP numbers). That is precisely the failure mode
`PARITY_COVERAGE_REQUIREMENTS.md` §1.2 describes: both engines were fed the same
wrong number, so the gates were green while the input was wrong.

Vampire's `apothecary: false` is the most consequential of the six — it is not a
price but a **capability**, so no bb2020 Vampire parity game has ever exercised
the apothecary path for that team.

**Do these 5 cells flip for free?** No. They need the roster corrections above,
then `gen_java_parity_data.py`, then a re-gate — because a re-roll cost and the
apothecary flag change what a legal drafted squad looks like. The squads in
`data/teams/bb2020/` for these five races were budgeted against the old numbers,
so they may now be over-budget (amazon/dwarf/lizardman got more expensive) or
under-spent (skaven/vampire got cheaper). **The other 24 clear provenance for
free**, subject only to the §5 caveats.

### 4b. The 24 that match their page exactly

`black_orc`, `chaos`, `chaos_dwarf`, `dark_elf`, `elf`, `gnome`, `goblin`,
`halfling`, `high_elf`, `human`, `imperial_nobility`, `khemri`, `khorne`,
`necromantic`, `norse`, `nurgle`, `ogre`, `old_world_alliance`, `orc`,
`renegades`, `snotling`, `undead`, `underworld`, `wood_elf`

Note in particular that `skaven`'s and `vampire`'s **positions** all match too —
only their team-level numbers are wrong. So the positional-data claim above
covers all 29, not just these 24.

Every page row paired 1:1 with a distinct JSON position on an **exact
display-name match** for all 29 teams — 155 pairings, no fuzzy match needed
anywhere, and zero unmatched JSON positions. That matters: a matcher that paired
nothing would also have printed "MATCHES PAGE". Run `--verbose` to see the
pairing.

## 5. What this does NOT establish

Provenance of the roster *definition* is only one of the requirements in
`PARITY_COVERAGE_REQUIREMENTS.md`. Still open for the bb2020 column:

1. **§8 R2/R4 — squads that do not field every positional.** Independent of
   roster correctness. The requirements doc already records the bb2020 misses:
   `chaos` (chaosogre, chaostroll), `chaos_pact` (renegadetroll),
   `dwarf` (trollslayer), `renegades` (37730), `undead` (skeleton),
   `underworld` (underworldsnotling, underworldtroll). Those positionals have no
   parity evidence even though their stat lines are now verified correct.
2. **Big Guy typing is unverified.** The bb2020 pages have no keyword column, so
   `type` has no page-side truth and is not diffed. The pages *do* carry the `*`
   footnote naming the mutually-exclusive Big Guy choices — e.g. underworld's
   "An Underworld team may include a single Big Guy" marks Underworld Troll and
   Mutant Rat Ogre — which is the §8 R3 **variant** requirement, not a roster
   error.
3. **`special_rules` are page-only.** The pages list them; the bb2020 roster
   JSONs have no such field (unlike bb2025's). Whether that matters depends on
   whether any bb2020 mechanic keys off a team special rule. Recorded, not
   changed.
4. **Three cosmetic data inconsistencies** are recorded in §3 and left alone:
   the dual `Bone Head`/`Bone head`, `Dump-Off`/`Dump-off`, `Jump Up`/`Jump up`
   spellings; the dual `Mutation`/`Mutations` category name; and Old World
   Alliance's id-list Animosity encoding.
5. **`old_world_alliance` and `snotling` rosters appeared mid-audit**, written by
   the concurrent workflow. They are included above and reconcile clean, but
   their numbers were audited against a moving target — re-run the report once
   that workflow finishes.

## 6. How the checker was validated

Two ad-hoc validators earlier in this campaign produced false alarms — 73
"illegal" squads, 81 "parity reds" — and **both were tool bugs, not data bugs**.
So this checker was not believed until it was tested, and testing it mattered:
**the first run of this mode reported 11 teams with phantom "position missing /
position not on page" pairs and 13 phantom category/skill mismatches, all of
which were bugs in the checker**, not in the data:

* it was reusing bb2025's `POSITION_ID_ALIASES`, which maps BB2025 page names to
  BB2025 position ids — so goblin's "Bomma" was sent to `goblin.bombardier`,
  which does not exist in the bb2020 JSON, while the real `goblin.bomma` sat
  unmatched. Fixed by giving bb2020 its own `match_position_id_bb2020()` that
  deliberately does not consult that table;
* it compared skill categories as ordered lists and treated `Mutation` vs
  `Mutations` as different;
* it printed skills without their values, so a value-only difference rendered as
  the nonsense `['Animosity'] -> ['Animosity']`.

Had that first output been trusted, this document would have reported ~11 false
roster errors. `--selftest` is now a permanent part of the script and reports
**ALL PASS** (27 checks):

* **Known-good.** `human` reports zero mismatches on the real files — and the
  same test asserts all 6 page rows paired, no JSON position was left over, and
  the pairing is 1:1, so a "clean" verdict cannot be vacuous. The Human page's
  six rows and `roster_human.json` were *also* compared **by hand**, field by
  field, and agree. The same hand-comparison was done for Underworld (8 rows,
  including the `*` Big Guy footnote and two parameterised skills).
* **Known-bad.** `amazon`'s re-roll cost really is 50k in the JSON against 60k on
  the page; the selftest asserts the checker prints exactly `50000 -> 60000`. All
  five §4a findings were re-read by hand out of the pages' `### Staff` sections
  (`grep "Re-roll"` → 60K / 50K / 70K / 50K / 60K) and out of the JSONs
  (50000 / 40000 / 60000 / 60000 / 70000), and Vampire's page `### Staff` does
  list `[Apothecary] - 50K` against the JSON's `false`.
* **Mutation testing.** For every field the checker claims to cover, the
  known-good pair is perturbed **in memory** (nothing written) and the
  perturbation must be caught: `ma`, `st`, `ag`, `pa`, `av`, `cost`, `quantity`,
  a dropped skill, an added skill, a changed numeric skill value, an added
  category, a removed category, a deleted position, an invented position,
  `reroll_cost`, `apothecary`, and a deleted string skill value. 17/17 caught.
  * One selftest case initially FAILED and it was the **test** that was wrong,
    not the checker: the skill mutations were applied to human's Lineman, which
    has no starting skills, so "drop a skill" was a no-op measuring nothing. The
    selftest now targets a position that actually has skills. Same class of bug
    as the two false alarms — worth recording.
* **Cosmetic-blindness.** Lower-casing every skill name, reversing category order
  while rewriting `Mutation`→`Mutations`, and rewriting a string skill value must
  each produce **zero** mismatches (the last one producing a `[representation]`
  note instead). All pass — this is what keeps the report from drowning in
  100+ harmless spelling diffs, which is the opposite failure mode from a false
  alarm and just as bad.
* **No regression of the existing modes.** The bb2025 and bb2016 reports were
  generated from the `HEAD` version of the script and from the new one; both
  diffs are **byte-identical**.
* **`--apply` dry-run.** `apply_bb2020()` was run in memory over all 29 teams
  (results discarded, no file written). For each: re-diffing the result against
  the page yields **clean**, the set of position ids is unchanged, the set of
  top-level JSON keys is unchanged, and no per-position key is dropped. So
  `--apply` is safe to run once `data/` has a single writer — it will change
  exactly the six numbers in §4a and nothing else.

## 7. Recommended next steps

1. Re-run `--edition bb2020 --report` once the concurrent `data/` writer is done;
   its edits may change this inventory (it already added two rosters mid-audit).
2. Run `--edition bb2020 --apply` to fix the six numbers in §4a.
3. Re-check the five affected drafted squads in `data/teams/bb2020/` for
   rule-legality against the corrected re-roll costs and Vampire's now-available
   apothecary.
4. Re-run `scripts/gen_java_parity_data.py`, then re-gate `amazon`, `dwarf`,
   `lizardman`, `skaven`, `vampire`.
5. Optionally normalise the §3/§5.4 cosmetic inconsistencies.
6. Update `PARITY_COVERAGE_REQUIREMENTS.md` §3 to state bb2020 provenance as
   *established via `audit_rosters.py --edition bb2020`*, and mark §6 item 1 done.
