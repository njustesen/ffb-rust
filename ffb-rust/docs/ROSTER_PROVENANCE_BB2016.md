# BB2016 roster provenance — LRB6/CRP reconciliation

Closes work-list item 2 of `docs/PARITY_COVERAGE_REQUIREMENTS.md` §6 ("turn the bb2016 audit from
a cleanup into a reconciliation"). The bb2016 path of `scripts/audit_rosters.py` now performs a
**positive** reconciliation of `data/rosters/bb2016/*.json` against `rules/bb2016/teams/*.md`
(LRB6 / Competition Rules Pack, `LRB6.pdf` pp.55-62) instead of only stripping BB2020-era
contamination.

```
python scripts/audit_rosters.py --edition bb2016 --report      # the inventory below
python scripts/audit_rosters.py --edition bb2016 --selftest    # validate the checker first
python scripts/audit_rosters.py --edition bb2016 --report --verbose   # page-row -> position pairing
python scripts/audit_rosters.py --edition bb2016 --report --show-pa   # per-position [pa] notes
python scripts/audit_rosters.py --edition bb2016 --cleanup --apply    # the LEGACY negative pass
```

The reconciliation is deliberately **report-only** — `--apply` is refused. The LRB6 pages are PDF
text extractions, several rows needed reviewed repairs to parse at all, and at least one finding
below is a page defect rather than a data defect. Mismatches are resolved by hand.

---

## 1. The conversion — bare characteristics, IDENTITY mapping

Every bb2016 page carries this header:

> LRB6 prints bare characteristics; BB2020+ prints roll targets. AG 4 here is not AG 4+ there.

That is a warning about a **corruption hazard**, not an instruction to convert. Established before
the checker was written, by reading the pages against the JSON on several teams:

| source | Wood Elf Lineman | Ogre (Ogre team) | Dwarf Blocker | Wood Elf Treeman |
|---|---|---|---|---|
| LRB6 page (bare) | MA 7 ST 3 AG 4 AV 7 | MA 5 ST 5 AG 2 AV 9 | MA 4 ST 3 AG 2 AV 9 | MA 2 ST 6 AG 1 AV 10 |
| `data/rosters/bb2016/*.json` | `7 3 4 7` | `5 5 2 9` | `4 3 2 9` | `2 6 1 10` |

**`data/rosters/bb2016/*.json` already stores BARE characteristics.** Confirmed globally: bb2016
rosters hold `ag` values 1..5 with the LOW values on the big guys — a roll-target encoding would
invert that (a Treeman would be `ag: 6`, not `ag: 1`).

> ### ⇒ The conversion used for `ma` / `st` / `ag` / `av` is the IDENTITY. ⇐
> Page value is compared to stored value with **no arithmetic in either direction**.

The bare→target mapping (`target = 6 - AG`, `ag 4` → `2+`) is applied by the engine at **roll**
time, not at storage time — `dodge_target` in `crates/ffb-engine/src/agent/heuristic_agent.rs`
and the bb2016 mechanics. Writing a roll target into a bb2016 roster would corrupt all 24
reconciled rosters at once and both engines would then agree on the wrong number, so parity could
never catch it. The checker's selftest asserts this explicitly in both directions:

* positive — for `wood_elf`, `ogre`, `dwarf`, `vampire`, every paired row's bare `ma/st/ag/av` must
  already be **identical** to the page;
* negative — a copy of the wood elf roster with `ag` rewritten to `7 - ag` (i.e. roll-target
  encoded) **must** be flagged, proving the comparison is not silently normalising.

## 2. What `pa` means for a bb2016 roster

**LRB6 has no PA characteristic at all — passing is AG-based — and the pages have no PA column.**
There is therefore no page-side truth for `pa`, and it **cannot** be reconciled. Decision:

> `pa` on a bb2016 roster is a **carried-over BB2020+ artefact: inert, parity-neutral, and out of
> scope for LRB6 reconciliation.** It is reported as an informational note (`--show-pa`), never as
> a mismatch. It is *not* zeroed either, because zeroing it would be an unforced data change to
> files another workflow owns, with no LRB6 source authorising the new value.

Verified in the engine rather than assumed:

| site | finding |
|---|---|
| `ffb-mechanics/src/bb2016/pass_mechanic.rs` | pass minimum and `format_roll_requirement` read `agility_with_modifiers()`; `passing` is never read |
| `ffb-mechanics/src/bb2016/stats_mechanic.rs` | `draw_passing()` returns `false` |
| `ffb-engine/src/mechanic/bb2016/roll_mechanic.rs` | no `InjuryAttribute::PA` arm at all (bb2020/bb2025 both have one) |
| `ffb-engine/src/step/bb2016/ttm/step_throw_team_mate.rs` | TTM minimum is derived from the distance modifier, not from `passing` |
| `ffb-engine/src/step/bb2016/start/step_buy_inducements.rs:228,306` | **does** copy `position.pa` into `player.passing` |
| `ffb-parity/src/runner.rs:1309` | ships the same `pa` to the Java side in the team spec |

So the value exists on the player and is **symmetric across the two engines** (both are fed from
this same JSON), while no bb2016 roll consults it.

Latent-hazard check (the concern raised in `PARITY_COVERAGE_REQUIREMENTS.md` §10): the only shared
reads of `passing_with_modifiers()` outside `bb2020/`/`bb2025/` are
`modifiers/stat_based_roll_modifier_factory.rs` and `marking/marker_generator.rs`, both keyed on
`PlayerStatKey::PA`, which bb2016 never produces (no PA injury attribute, `draw_passing()` false;
the marker generator is GUI-only). **No live bb2016 read of `pa` found.**

Distribution across the 29 bb2016 rosters (133 positions): `pa` ∈ {0 ×24, 2 ×6, 3 ×10, 4 ×46,
5 ×31, 6 ×8, −1 ×8}. The `−1` sentinels are in `khemri_fumbbl` / `slann_fumbbl`.

## 3. Other fields with no page-side truth

| field | treatment |
|---|---|
| `apothecary` | The pages never mention it. Checked against the **LRB6 rule** instead (Khemri, Necromantic, Nurgle, Undead and Vampire may not hire one) and labelled `[LRB6 rule, not page text]` in the output. All 24 agree. |
| `type` (`Regular` / `Big Guy`) | LRB6 has no Big Guy keyword — the pages express it only through the skill list (Loner) and the category letters. Not diffed. See §6 for an internal inconsistency this hides. |
| `max_rerolls` | Taken from the page's `0-8 Re-roll counters:` line. All 24 agree (8). |
| `special_rules` | Not an LRB6 concept; the bb2016 JSONs have no such field. Not diffed. |

## 4. Mismatch inventory — 2026-09-07

Run: `python scripts/audit_rosters.py --edition bb2016 --report`, against
`data/rosters/bb2016/` and `rules/bb2016/teams/` at commit `f6967ece8`.

**24 of the 29 bb2016 rosters map to an LRB6/CRP page. 16 match their page; 8 differ, over 18
mismatch lines.** Nothing was applied — this workflow is read-only with respect to
`data/rosters/**`.

Notation: `field: stored -> page`.

### 4.1 Clean — 16 teams

`amazon`, `chaos_dwarf`, `chaos_pact`, `dark_elf`, `elf`, `high_elf`, `human`, `khemri`,
`lizardman`, `norse`, `nurgle`, `ogre`, `orc`, `skaven`, `slann`, `vampire`

### 4.2 Mismatches — 8 teams, 18 lines

| # | team | mismatch | classification |
|---|---|---|---|
| 1 | `chaos` | `chaos.beastman.quantity: 12 -> 16` | **data** — CRP Chaos is 0-16 Beastmen |
| 2 | `dwarf` | `reroll_cost: 40000 -> 50000` | **data** — CRP Dwarf re-rolls are 50k |
| 3 | `dwarf` | `dwarf.blocker.quantity: 12 -> 16` | **data** — CRP is 0-16 Blockers |
| 4 | `dwarf` | `dwarf.deathroller.skills` missing `Loner` | **data** — CRP Deathroller has Loner |
| 5 | `goblin` | `goblin.goblin.skills` has extra `Thick Skull`, `Animosity`, `Regeneration` | **data** — CRP Goblins are `Dodge, Right Stuff, Stunty` |
| 6 | `goblin` | `goblin.bombardier.skills` has extra `Regeneration` | **data** |
| 7 | `goblin` | `goblin.looney.skills` has extra `Animosity`, `Regeneration` | **data** |
| 8 | `goblin` | `goblin.pogoer.skills` has extra `Animosity`, `Regeneration` | **data** |
| 9 | `goblin` | `goblin.troll.skills` lists `Regeneration` **twice** | **data** — duplicate entry |
| 10 | `halfling` | `halfling.halfling.skills` missing `Stunty` | **data** — CRP Halflings are Stunty |
| 11 | `halfling` | `halfling.treeman.skills` has `Loner`, the page does not | ⚠ **page defect** — see §5 |
| 12 | `necromantic` | `necromantic.zombie.cost: 30000 -> 40000` | **data** — CRP Necromantic Zombie is 40k (the `undead` Zombie is already 40k) |
| 13 | `undead` | `undead.skeleton.skills` missing `Thick Skull` | **data** — CRP Undead Skeleton is `Regeneration, Thick Skull` |
| 14 | `underworld` | `- position NOT ON PAGE: 37844 ('Mutant Rat Ogre')` | **data** — CRP Underworld has no Rat Ogre; FUMBBL-only import, numeric id |
| 15 | `underworld` | `underworld.goblin.skills` has extra `Animosity` | **data** — in CRP, Animosity is on the Skaven positions only |
| 16 | `underworld` | `underworld.skaven.lineman.quantity: 16 -> 2` | **data** — CRP is 0-2 Skaven Linemen |
| 17 | `underworld` | `underworld.troll.warpstone.quantity: 2 -> 1` | **data** — CRP is 0-1 Warpstone Troll |
| 18 | `wood_elf` | `woodelf.catcher.skills` missing `Sprint` | **data** — CRP Wood Elf Catcher is `Catch, Dodge, Sprint` |

Pattern: the extra `Animosity` / `Regeneration` / `Thick Skull` on the goblin and underworld
positions, and the duplicated `Regeneration`, are exactly the class of contamination the legacy
`--cleanup` pass was written for — but they are not in its `BB2016_REMOVE_SKILLS` list, so a
negative check could never have found them. That is the argument for this reconciliation in one
paragraph.

### 4.3 Cosmetic — page spelling only, not mismatches

Reported as `[cosmetic]`, no action needed; both engines resolve skill names case- and
punctuation-insensitively:

* `Bone-Head` (stored, and the bb2016 canonical per `BB2016_CANONICAL_NAME`) vs `Bone-head` on the
  page — `human.ogre`, `lizardman.kroxigor`, `ogre.ogre`, `slann.kroxigor`, `chaospact.ogre.chaos`.
* `Throw Team-Mate` (stored) vs `Throw Team-mate` on the page — `chaospact.troll.chaos`,
  `chaospact.ogre.chaos`, `underworld.troll.warpstone`.

Two further page spellings are mapped in `SKILL_ALIASES_BB2016`, both verified against the engine
rather than guessed, because without them a roster storing the **canonical** name gets reported as
a data error for using it:

| page prints | engine-canonical (and stored) | evidence |
|---|---|---|
| `Ball & Chain` | `Ball and Chain` | `ffb-model/src/factory/skill_factory.rs` registers `"Ball & Chain"` as an explicit alias of `SkillId::BallAndChain`, mirroring Java `SkillFactory.forName` |
| `Claws` | `Claw` | `ffb-model/src/enums/skill_id.rs`: `SkillId::Claw => "Claw"`, and `"claw" \| "claws"` both resolve |

### 4.4 Not reconciled — 5 rosters with no LRB6/CRP page

| roster | reason |
|---|---|
| `nippon` | not an LRB6/CRP team |
| `renegades` | FUMBBL Chaos Renegades import (10 positions, numeric ids). CRP prints **Chaos Pact** instead, which the repo already has as `chaos_pact` (clean). Reconciling `renegades` against `Chaos_Pact.md` would produce nothing but phantom add/remove pairs. |
| `dark_elf_league_fumbbl` | FUMBBL import (numeric ids) |
| `khemri_fumbbl` | FUMBBL import (numeric ids) |
| `slann_fumbbl` | FUMBBL import (numeric ids) |

All 24 LRB6 pages have a roster; no page is unmapped.

## 5. Suspected page defect, NOT a data defect

**`rules/bb2016/teams/Halfling.md`: the Treemen row is missing `Loner`.** The page prints
`Mighty Blow, Stand Firm, Strong Arm, Take Root, Thick Skull, Throw Team-Mate`; the JSON also has
`Loner`. In CRP the Halfling Treeman does have Loner, and every other big guy on these same pages
(Wood Elf Treeman, Chaos Minotaur, Orc/Goblin Troll, Human/Chaos Pact Ogre, Kroxigor, Snow Troll,
Beast of Nurgle, Deathroller, Warpstone Troll) prints `Loner` first. The extraction dropped it.

**Recommendation: fix the page, not the roster.** Not done here — this workflow only touches
`scripts/` and `docs/`.

## 6. Recorded, not changed — observations outside page reconciliation

Per the concurrency constraint on this work, nothing under `data/rosters/**` or `data/teams/**` was
modified. Two items have no LRB6 page authority but should be looked at by whoever fixes §4.2:

1. **`type` is inconsistent for identical big guys.** `chaos_pact`'s `Chaos Troll`, `Chaos Ogre`
   and `Minotaur`, and `ogre`'s `Ogre`, are typed `Regular`, while the same creatures on
   `chaos`/`chaos_dwarf`/`goblin`/`orc`/`human`/`norse`/`skaven`/`underworld` are typed `Big Guy`.
   (`undead.mummy` as `Regular` is correct — Mummies are not big guys in LRB6.) The LRB6 pages
   cannot settle this, so the checker does not diff `type`.
2. **Any fix to §4.2 changes generated Java parity data.** `scripts/gen_java_parity_data.py`
   converts these rosters into the Java-side XML, so per `CLAUDE.md` it must be re-run after any
   roster edit, and the affected races re-gated. `dwarf`, `goblin`, `undead` and `underworld` all
   currently sit in green parity campaigns; changing `dwarf`'s re-roll cost or `goblin`'s starting
   skills changes team drafting and dice consumption, so those gates need re-running, not assumed.
   Neither the generator nor any gate was run by this workflow.

## 7. How the checker was validated

Two ad-hoc validators earlier in this campaign produced confident false alarms — one flagged 73
squads as illegal (it omitted the dedicated-fans cost), another reported 81 parity reds (a broken
shell comparison). **Both were tool bugs, not data bugs.** So this checker was not believed until
it passed `--edition bb2016 --selftest`, which is committed alongside it and reports **ALL PASS**
(89 assertions):

| group | what it proves |
|---|---|
| **parser** | all 24 LRB6 pages parse; and four **hand-read** rows (`wood_elf` Linemen, `halfling` Treemen, `chaos_dwarf` Chaos Dwarf Blockers, `underworld` Warpstone Troll — chosen to cover a plain row, a 4-line wrapped skill cell with a hyphen-broken word, a wrapped position title, and a scrambled row needing a fixup) come back **verbatim** against values transcribed from the page by eye |
| **no-conversion** | for four teams, every paired row's bare `ma/st/ag/av` is already identical to the page (positive), **and** a roll-target-encoded copy of the wood elf roster is flagged (negative) — so the comparison is bare-vs-bare and is not normalising |
| **pairing** | every page row on **all 24** teams pairs 1:1 with a distinct JSON position. This is the guard that matters most: an unpaired row emits a phantom `position MISSING from JSON` + `position NOT ON PAGE` pair that reads exactly like a data error. A "clean" verdict from a matcher that pairs nothing would look identical to a real clean verdict |
| **alias** | `necromantic` and `goblin` produce no `.skills` mismatch mentioning `Claw` / `Ball`, so the two verified page-spelling aliases are not masquerading as data errors |
| **known-good** | `human` yields zero real mismatches on the untouched real files, with all 5 rows paired, no orphans, and 1:1 pairing |
| **known-bad** | `dwarf`'s `reroll_cost` really is 40k against the page's 50k, and the message states `40000 -> 50000` |
| **mutations** | each field the checker claims to cover (`ma`, `st`, `ag`, `av`, `cost`, `quantity`, skill added, skill dropped, category added, category removed, position deleted, position invented, `max_rerolls`, `apothecary`) is perturbed **in memory** on the known-good pair and must be caught |
| **cosmetics** | lower-casing / de-hyphenating skill names, reordering skill categories, and rewriting every `pa` to 99 must **not** be reported as mismatches (and the skill respellings must surface as `[cosmetic]`) |

Two real tool bugs were caught this way before any number was reported, both of the exact
false-alarm shape the lesson above describes:

1. **`-ies` singularisation.** The LRB6 pages print positions in the plural and the rosters store
   them singular, so the matcher singularises. A single-guess `-ies -> -y` rule turned `Zombies`
   into `Zomby`, which matched nothing, and the first draft of the report claimed
   `+ position MISSING from JSON: 'Zombies'` / `- position NOT ON PAGE: necromantic.zombie` for
   **both** `necromantic` and `undead` — a phantom pair that also **masked a real finding**
   (`necromantic.zombie.cost: 30000 -> 40000`, item 12). Fixed by returning a candidate **set**
   (`Mummies -> Mummy`, `Zombies -> Zombie`) and accepting any match; the all-teams pairing
   assertion now covers the whole bug class.
2. **Missing skill-spelling aliases.** `Claws`/`Claw` and `Ball & Chain`/`Ball and Chain` were
   first reported as `.skills` mismatches on `necromantic`, `norse` and `goblin` — i.e. the
   rosters were accused of being wrong for storing the engine-canonical name. Fixed with
   `SKILL_ALIASES_BB2016`, each entry checked against `skill_factory.rs` / `skill_id.rs`.

### Page fixups
`BB2016_PAGE_FIXUPS` holds four exact, reviewable text repairs for `Underworld.md`, whose PDF
extraction scrambled three rows (qty/title and stat block on different lines, the wrapped half of a
title landing *after* the skills, and the Warpstone Troll's skill-category cell floating above its
own row). No value is invented; each fixup raises if it stops applying, so a page edit cannot
silently change what is being compared. The `underworld` Warpstone Troll is one of the four
hand-read rows for exactly this reason.
