# Parity coverage — requirements and the honest matrix

Written 2026-09-07 after the alphabetical heuristic sweep was reported "30/30 complete" and that
claim was found to be **scoped to the repo's drafted teams rather than to the game**. This document
is the source of truth for what coverage means and what is actually covered.

## 1. Cell semantics (the requirement)

For a given (roster, ruleset) cell:

| icon | meaning |
|---|---|
| 🟢 | The two engines agree on per-step state-hash parity, **using a roster that belongs to that ruleset**. |
| 🔴 | Anything else. Includes: no roster data; no tests; failing tests; **or a roster whose provenance for that ruleset is not established**. |
| N/A | The team does not exist in that ruleset. Nothing to cover. |

Two consequences that are easy to miss and were both missed before:

1. **Untested is red, not N/A.** A team that exists in a ruleset and has no test is a gap, not an
   exemption.
2. **Parity cannot see a wrong roster.** Both engines are fed the SAME generated data
   (`scripts/gen_java_parity_data.py`). If a ruleset's roster silently carries another ruleset's
   stats or skills, the engines agree *perfectly* and the gate is green while the input is wrong.
   Green therefore requires roster provenance as well as engine agreement. Roster rules change
   between rulesets, so a same-named team is a DIFFERENT roster per ruleset.

## 2. Sources of truth

| ruleset | official team list | teams |
|---|---|---|
| bb2016 | `rules/bb2016/teams/*.md` | **24** |
| bb2020 | `rules/bb2020/teams/*.md` | **29** |
| bb2025 | `rules/teams/*.md` | **30** |

## 3. Roster provenance — the blocking issue

| ruleset | provenance | audit path | draft doc | verdict |
|---|---|---|---|---|
| bb2025 | audited against official team pages | `audit_rosters.py --edition bb2025` | `TEAM_DRAFTS_BB2025.md` | established |
| bb2016 | "cleaned FUMBBL LRB6/CRP data" | `--edition bb2016`, **cleanup only** | `TEAM_DRAFTS_BB2016.md` | NOT established — a negative check removes BB2020 contamination but never confirms a roster matches LRB6/CRP |
| bb2020 | **none stated anywhere** | **none** — the script only accepts bb2016/bb2025 | **none** | NOT established |

`audit_rosters.py` hard-codes `choices=["bb2016","bb2025"]`. `rules/bb2020/teams/` holds 29
authoritative pages that nothing reads.

**Under §1 this makes every bb2016 and bb2020 cell RED on provenance, whatever the parity numbers
say.** 27 of 29 rosters do differ per edition (so nobody copy-pasted wholesale — only the two
FUMBBL imports are identical across editions, which is defensible for a league import), but
"differs" is not "verified".

## 4. Rosters we test that are not official teams

These have passing gates, but the gates are not coverage of those rulesets:

| roster | official in | note |
|---|---|---|
| `chaos_pact` | bb2016 only | tested in all three; bb2020/bb2025 cells are N/A |
| `slann` | bb2016 only | tested in all three; bb2020/bb2025 cells are N/A |
| `nippon` | **no ruleset** | a FUMBBL/custom team; N/A everywhere |
| `dark_elf_league_fumbbl`, `khemri_fumbbl`, `slann_fumbbl` | **no ruleset** | FUMBBL league imports; N/A everywhere |

## 5. Missing rosters (no data at all)

| team | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Black Orc | N/A | 🔴 | 🔴 |
| Gnome | N/A | 🔴 | 🔴 |
| Imperial Nobility | N/A | 🔴 | 🔴 |
| Khorne | N/A | 🔴 | 🔴 |
| Old World Alliance | N/A | 🔴 | 🔴 |
| Snotling | N/A | 🔴 | 🔴 |
| Bretonnian | N/A | N/A | 🔴 |

**13 red cells from absent data.** They need: a roster authored from the official page, a
rule-legal parity team drafted, `gen_java_parity_data.py` re-run, then nine gates.

## 6. Ordered work list

1. **Add a `bb2020` mode to `audit_rosters.py`** reconciling `data/rosters/bb2020/*` against
   `rules/bb2020/teams/*.md`. One script; unblocks a whole column of greens. Highest value per
   unit of work.
2. **Turn the bb2016 audit from a cleanup into a reconciliation** against an authoritative LRB6/CRP
   source, so bb2016 provenance can be asserted rather than assumed.
3. **Draft the 7 missing teams** (6 for bb2020+bb2025, Bretonnian for bb2025 only), then gate them.
4. **Decide the status of the 6 non-official rosters.** Either keep them as extra confidence and
   mark them N/A in the official matrix, or retire the gates that are not coverage.
5. Then, and only then, re-issue the coverage matrix.

## 7. What the heuristic sweep did establish

Not nothing — it is just a narrower claim than "30/30". Across 30 drafted matchups the sweep took
the engines to nine-gate parity and, in doing so, found and fixed a long list of real engine bugs
(BB2020 SI table dispatch, the shared-step edition-blindness family, Foul Appearance on a pass,
the trap-door chain, Treacherous, the Diving-Tackle clamp, and six more behind the unreachable
Leap declaration). Engine agreement on the rosters we have is genuinely strong. Roster
*correctness* for bb2016/bb2020, and coverage of 7 official teams, are the open questions.

## 8. Roster / squad drafting requirements

Section 1 says a green cell needs "a roster that belongs to that ruleset". This section says what
the drafted SQUAD must look like. The distinction matters: `data/rosters/<ed>/` is the team's
*roster definition* (which positionals exist, their stats and costs); `data/teams/<ed>/` is the
*drafted squad* we actually field in a parity game. A correct roster definition proves nothing if
the squad never fields half of it.

### R1 — Realistic and rule-legal
A squad must be one a coach could actually field: positional quantity limits respected, budget
respected, sensible re-rolls / dedicated fans / apothecary. Not a degenerate stack.

### R2 — Every positional must be fielded
Every positional in the ruleset's roster definition must appear in the squad, so that every
position's stat line and starting skills are exercised. A positional that is never fielded has NO
parity evidence, exactly like an unreachable skill: the gate is green because the code never ran.

### R3 — Variants where R2 is impossible
Some rulesets forbid fielding every positional at once. The BB2025 pages carry an explicit
restriction — "*A <team> team may have a single Big Guy, chosen from the following*" — for:

| team | big-guy options | allowed at once | variants required |
|---|---:|---:|---:|
| Chaos Chosen | 3 (Troll, Ogre, Minotaur) | 1 | **3** |
| Underworld Denizens | 2 | 1 | **2** |
| Old World Alliance | 2 (Ogre, Altern Forest Treeman) | 1 | **2** |

Where a restriction makes R2 unsatisfiable in one squad, draft **one variant per mutually exclusive
option**, so that the UNION of the variants covers every positional. R2 is therefore a requirement
on the variant SET, not on any single squad. Each variant is gated like any other matchup.

### R4 — The union must be complete
For every (roster, ruleset) cell, the union of that cell's squads must field 100% of the ruleset's
positionals. Anything less is 🔴 under §1, because the unfielded positional has no parity evidence.

### Measured state of R2/R4 today (2026-09-07)

**10 drafted squads do not field every positional — 13 positional slots have NO parity evidence**,
and most of the misses are big guys, i.e. precisely the positions carrying the negatraits and
Throw/Kick Team-Mate chains that this campaign spent weeks fixing:

| ruleset | race | never fielded |
|---|---|---|
| bb2016 | orc | lineman |
| bb2020 | chaos | chaosogre, chaostroll |
| bb2020 | chaos_pact | renegadetroll |
| bb2020 | dwarf | trollslayer |
| bb2020 | renegades | 37730 |
| bb2020 | undead | skeleton |
| bb2020 | underworld | underworldsnotling, underworldtroll |
| bb2025 | chaos | ogre, troll |
| bb2025 | renegades | 37733 |
| bb2025 | underworld | 37844 |

Note **chaos** in bb2020 AND bb2025 fields only the Minotaur, so the Chaos Troll's Always Hungry and
the Chaos Ogre's Bone Head have never been exercised in those rulesets — while chaos was reported
green. That is not a roster-definition error (the official page does list all three at 0-1 with a
single-Big-Guy cap); it is a missing VARIANT, and it is the clearest example of why R3 exists.

Of the 16 (roster, ruleset) cells whose roster offers 2+ big guys, **7 field only some of them**:
bb2020 chaos / chaos_pact / renegades / underworld, and bb2025 chaos / renegades / underworld.

### Consequence for the work list
§6 item 3 ("draft the 7 missing teams") is amended: each new team must satisfy R1-R4, which for
Old World Alliance means **2 variants** (Ogre and Treeman) and for Snotling however many its own
Big Guy rule requires. A new §6 item is added: **re-draft or add variants for the 10 squads above**,
which is cheaper than drafting a new team and closes 13 slots of missing evidence on races already
reported green.

### CLOSED 2026-09-07 — all 13 slots covered

`python scripts/validate_teams.py` now prints **0 R1 violations, 0 unfielded positional slots**
for all three editions. The draft is `scripts/draft_r2_squads.py` (purchase table + arithmetic +
invariants, same shape as `draft_coverage_teams.py`).

Two shapes of fix, decided per cell by the official page:

| ruleset | race | slot(s) | fix |
|---|---|---|---|
| bb2016 | orc | `orc.lineman` | **AMENDED** — 1 Black Orc Blocker traded for 1 Orc Lineman |
| bb2020 | dwarf | `dwarf.trollslayer` | **AMENDED** — Troll Slayer added, re-rolls 3→2 to pay for it |
| bb2020 | undead | `undead.skeleton` | **AMENDED** — 1 Zombie traded for 1 Skeleton (same cost) |
| bb2020 | underworld | `37844.underworldsnotling` | **AMENDED** — 2 Snotlings added (treasury 30k→0) |
| bb2020 | chaos | `chaos.chaosogre`, `chaos.chaostroll` | variants `team_chaos_chaosogre`, `team_chaos_chaostroll` |
| bb2020 | chaos_pact | `chaospact.renegadetroll` | variant `team_chaos_pact_renegadetroll` |
| bb2020 | renegades | `37730` | variant `team_renegades_37730` |
| bb2020 | underworld | `37844.underworldtroll` | variant `team_underworld_underworldtroll` |
| bb2025 | chaos | `chaos.ogre`, `chaos.troll` | variants `team_chaos_ogre`, `team_chaos_troll` |
| bb2025 | renegades | `37733` | variant `team_renegades_37733` |
| bb2025 | underworld | `37844` | variant `team_underworld_37844` |

The variants exist because the positional CANNOT join the base squad: Chaos Chosen and Underworld
Denizens pages say "may have a single Big Guy" in **both** editions, and the Chaos Renegades page
says "up to three Big Guys" while the roster offers four — the base renegades / chaos_pact squads
were already at that cap. R2 is met by the UNION (R4); the base squads are untouched, so their
existing gates still describe the squad they were run on.

**Re-gate list.** The four AMENDED squads are different teams from the ones their earlier gates
measured, so `bb2016 orc`, `bb2020 dwarf`, `bb2020 undead` and `bb2020 underworld` must be
re-gated. The nine variants are new matchups and need their first gate.

**Non-fallback proof.** `make_team()` falls back to an ALL-LINEMAN team on a bad roster_id /
position_id with a `log::warn!` only, and a fallback team still gates green. Every one of the 19
squads in this cell set (13 new/changed + the base squads of the variant cells) is therefore
asserted in `runner::coverage_squad_tests` to build its real roster: exact fielded `position_id`
multiset, every player's stat line equal to its roster position's, and a fingerprint positional
carrying a fingerprint starting skill (Bone Head on the Ogres, Always Hungry on the Trolls,
Animal Savagery on the Rat Ogres, Unchannelled Fury on the Minotaurs, Frenzy on the Troll Slayer,
Right Stuff on the Snotling, Regeneration on the Skeleton). All 13 matchups then run
`--tier 3 --seeds 5` at **5/5 games match**, which is the Java-side proof as well: a per-step
state-hash match against a Rust team known to contain the Chaos Troll means Java fielded it too.

**One generator bug fixed on the way.** `gen_java_parity_data.py` built the Java `<team id=..>`
from the squad's `race` field, but `ffb-parity` asks Java for `java_team_id(<CLI matchup name>)`
= the FILE STEM in PascalCase. For an R3 variant the two differ, so every variant of a cell got
the SAME team id — `team_old_world_alliance_{ogre,treeman}_parity25_home.xml` both declared
`teamOldWorldAllianceParity25Home`, an id ffb-parity never asks for. The id now comes from the
stem; all four Old World Alliance variants re-verified at 5/5.

## 9. bb2016 characteristics — the conversion, stated correctly

`rules/bb2016/teams/*.md` warn: *"LRB6 prints bare characteristics; BB2020+ prints roll targets.
AG 4 here is not AG 4+ there."* That warning has been misread once already, so the facts, verified
against the engine and the data:

**The LRB6 agility TABLE, not an off-by-one.** BB2016 resolves an agility test through
`dodge_target` (`crates/ffb-engine/src/agent/heuristic_agent.rs:180`):

```rust
Rules::Bb2016 => ((7 - ag.min(6)) - 1 + tz_on_dest).max(2),   // bb2016: bare AG -> target
_             => (ag + tz_on_dest).max(2),                     // bb2020+: ag IS the target
```

So the bare-characteristic to roll-target mapping is `target = 6 - AG`:

| bare AG (LRB6) | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| equivalent roll target | 5+ | 4+ | 3+ | **2+** | 1+ (clamped 2+) | 0+ (clamped 2+) |

**AG 4 becomes 2+, NOT 4+.** An earlier note in this campaign said "AG 4 -> 4+"; that was wrong.

**Therefore no storage conversion is required.** `data/rosters/bb2016/*.json` already stores the
BARE characteristic exactly as the page prints it — verified: the LRB6 Amazon Blitzer/Catcher/
Thrower print AG 3 and the JSON stores `ag: 3`. The edition formula above does the conversion at
roll time. A bb2016 reconciliation must therefore compare **bare page value against bare stored
value, with NO arithmetic**. Applying any conversion during the audit would corrupt all 22 bb2016
rosters.

**Open question for the audit, do not assume:** LRB6 has no PA characteristic at all (passing is
AG-based), yet the bb2016 JSONs carry e.g. `pa: 5`. Establish what that field means for bb2016
(schema filler? derived?) and whether it should be excluded from the reconciliation, rather than
"fixing" it against a page that never prints it.

## 10. R5 — one roster per ruleset, never shared

**Requirement: every (team, ruleset) pair gets its OWN roster, authored from that ruleset's own
source. A roster is never reused across rulesets, even when the team name is identical.**

Rationale: roster rules change between rulesets — positionals, stats, costs, quantities and skills
all move, and the characteristic CONVENTION itself changes (§9). A shared roster is therefore wrong
in at least one ruleset by construction. And it is invisible to parity: both engines read the same
generated data, so a roster that belongs to another ruleset produces perfect engine agreement on a
wrong input. `data/rosters/<ed>/` already provides the structure; R5 says the CONTENT must be
independently derived too.

### Measured violations (2026-09-07)

**R5a — 4 races reuse a roster across rulesets** (byte-identical position/stat/cost/skill content,
i.e. not authored per ruleset):

| race | shared across |
|---|---|
| `dark_elf_league_fumbbl` | bb2016 == bb2020 == bb2025 |
| `khemri_fumbbl` | bb2016 == bb2020 == bb2025 |
| `nippon` | bb2020 == bb2025 |
| `slann` | bb2020 == bb2025 |

**R5b — ALL 29 bb2016 rosters carry a PA value, and LRB6 has no PA characteristic.**
In the old rules **AG is the passing stat**, which is exactly why no PA is printed. The engine
already encodes this: `bb2016/PassMechanic` has no PA conjunct ("LRB6 passing is AG-based, the stat
may be 0 for everyone"). So every `pa` in a bb2016 roster is a BB2020+ artefact — the precise
contamination the page header warns about, which the existing bb2016 *cleanup* mode does not strip.
Values range 2-6, and `khemri_fumbbl` / `slann_fumbbl` carry `pa: -1` sentinels.

This makes the bb2016 provenance 🔴 substantively true, not merely procedural: 29 of 29 rosters
contain a field their ruleset does not define.

### Latent hazard this creates
A bb2016 `pa` is inert ONLY IF every read of it is edition-gated. This campaign has repeatedly
found shared steps reading the wrong edition's field (the BB2025-hardcoded serious-injury helper
that blinded every bb2020 game; `StepFallDown`'s Safe Pair of Hands argument; the BB2025-only
Diving-Tackle re-roll running under bb2020). An ungated `passing_with_modifiers()` read in shared
code would make bb2016 behave as though PA existed — and both engines would agree on it. The audit
should therefore report bb2016 PA as contamination to REMOVE, not as a value to reconcile.

### Work implied
1. Strip `pa` from all 29 bb2016 rosters (or set it null) and grep for ungated reads.
2. Re-author the 4 shared rosters per ruleset, or delete the ones whose team is not official in that
   ruleset (`nippon` is official in none; `slann` and `chaos_pact` are official in bb2016 only).
3. Extend `validate_teams.py` with an R5 check: fail if two rulesets' rosters for one team are
   byte-identical, and fail if a bb2016 roster carries PA.

## 11. `scripts/validate_teams.py` — the squad legality checker

R1-R5 above are now machine-checked. Nothing else in the repo checks a drafted SQUAD:
`scripts/check_skill_names.py` and `all_roster_starting_skills_resolve` only check skill
SPELLING in the roster *definitions*, and parity is structurally blind to a bad squad — both
engines read the same generated data, so an illegal squad produces a perfectly MATCHED green.

```
python scripts/validate_teams.py                 # all three editions
python scripts/validate_teams.py --edition bb2025
python scripts/validate_teams.py --r5            # + the roster-sharing / bb2016-PA check
python scripts/validate_teams.py --selftest      # validate the CHECKER
```

Exit 0 = clean, 1 = violations, 2 = selftest failed. The script is **read-only with respect to
`data/`** — there is deliberately no `--apply`.

### What it checks, per `data/teams/<ed>/team_*.json`

| id | check | why |
|---|---|---|
| C1 | `roster_id` resolves in `data/rosters/<ed>/` | **CRITICAL.** `make_team()` (`crates/ffb-parity/src/runner.rs`) ends in `.unwrap_or_else(\|e\| { log::warn!(..); make_lineman_team(..) })`. A typo'd id is a warning and an ALL-LINEMAN team — and a green gate on a team that was never fielded. |
| C2 | every `players[].position_id` is in that roster | same failure mode |
| C3 | 11..16 rostered players; players + fielded stars ≤ 16 | R1 |
| C4 | per-position count ≤ that position's `quantity` | R1 |
| C5 | spend reconciles (formula below) | R1 |
| C6 | `rerolls ≤ max_rerolls`; apothecary only if the roster allows one; `dedicated_fans` 1..6 (bb2020+) / `fan_factor` 0..9 (bb2016), and neither field set in the edition that has no such concept | R1 |
| C7 | squad numbers unique and in 1..16 | the harness indexes activation snapshots into the nr-sorted player list |
| C8 | every `stars[].star_id` resolves in `data/star_players/all_editions.json` | R1 |
| BG | a roster whose official page says "*may have a single Big Guy*" fields at most one | R1/R3 |
| R2/R4 | per **(edition, race) cell**, the union of that cell's squads fields every positional in the roster | R2/R4 |
| R5 | (`--r5`) no roster content-identical across two rulesets; no bb2016 roster carries PA | §10 |

Variants are grouped by the spec's own `race` field, **not** by filename prefix: filenames
collide (`team_dark_elf.json` and `team_dark_elf_league_fumbbl.json` are different cells, while
`team_old_world_alliance_{ogre,treeman}.json` are one cell).

### The C5 money formula, derived from the data

An earlier ad-hoc validator reported **73 false violations** because it omitted the fan cost.
The formula below was therefore derived empirically and reproduces the declared
`team_value` / `spent` / `treasury` for **100% of the squads in the tree**, all three editions:

```
players = sum(position.cost)          # star players are NOT counted: they are inducements,
                                      # and no declared team_value includes them
staff   = rerolls * roster.reroll_cost + apothecaries * 50_000

bb2016:   team_value = players + staff + fan_factor * 10_000    # LRB6 Fan Factor costs 10k
          spent      = team_value                               # and DOES count in TV

bb2020/   team_value = players + staff                          # Dedicated Fans are NOT TV;
bb2025:   spent      = team_value + (dedicated_fans - 1) * 5_000  # the first one is free

all:      spent + treasury == 1_100_000                         # the drafting budget
```

### How the checker itself is validated

`--selftest` copies `data/teams/bb2025` to a temp dir and asserts **0 violations** on that
known-good tree, then breaks one squad nine ways (C1-C8, BG, and a dropped positional for
R2) and asserts each check fires. It never writes to `data/`.

### Current state (measured 2026-09-07, 102 squads: 29 bb2016 / 36 bb2020 / 37 bb2025)

**0 R1 violations.** Every drafted squad — including the seven teams added after §5 was
written — resolves its roster, is inside its quantity caps and budget, and reconciles to the
gold piece.

**13 unfielded positional slots in 10 cells**, exactly the §8 list, independently reproduced:

| ruleset | cell | never fielded |
|---|---|---|
| bb2016 | orc | `orc.lineman` |
| bb2020 | chaos | `chaos.chaostroll`, `chaos.chaosogre` |
| bb2020 | chaos_pact | `chaospact.renegadetroll` |
| bb2020 | dwarf | `dwarf.trollslayer` |
| bb2020 | renegades | `37730` |
| bb2020 | undead | `undead.skeleton` |
| bb2020 | underworld | `37844.underworldsnotling`, `37844.underworldtroll` |
| bb2025 | chaos | `chaos.troll`, `chaos.ogre` |
| bb2025 | renegades | `37733` |
| bb2025 | underworld | `37844` |

`--r5` reports **4 shared rosters and 29 bb2016 PA carriers**, also matching §10's independent
count. Those are data changes and are NOT applied by this script.

### Re-measured the same day, after the R3 variant drafting landed

The squad tree was being re-drafted while the checker was written, so the run above is a
snapshot. Re-run on the finished tree — 111 squads: 29 bb2016 / 41 bb2020 / 41 bb2025 —
the result is:

```
=== bb2016: 29 squads, 0 R1 violations, 0 cells with unfielded positionals ===
=== bb2020: 41 squads, 0 R1 violations, 0 cells with unfielded positionals ===
=== bb2025: 41 squads, 0 R1 violations, 0 cells with unfielded positionals ===
TOTAL: 0 R1 violations, 0 unfielded positional slots
```

**All 13 slots are closed and R1/R2/R4 are clean.** The nine new files are exactly the R3
variants this document called for — bb2020 `team_chaos_chaosogre`, `team_chaos_chaostroll`,
`team_chaos_pact_renegadetroll`, `team_renegades_37730`, `team_underworld_underworldtroll`;
bb2025 `team_chaos_ogre`, `team_chaos_troll`, `team_renegades_37733`,
`team_underworld_37844` — plus in-place re-drafts for bb2016 `orc`, bb2020 `dwarf` and
bb2020 `undead`. Variants are matched by their `race` field, so each set unions into its
own cell.

`--r5` is unchanged: 4 shared rosters, 29 bb2016 PA carriers. That work is still open.

## 11. The coverage matrix — 2026-09-07

Cell semantics are §1, tightened by the user on 2026-09-07: **🟢 = the two engines agree
100/100 × 9 gates using a roster from *that* ruleset that follows the R1–R5 roster
requirements.** Anything else is 🔴 — including a roster that reconciles clean but whose
squad has been re-drafted since it was last gated. N/A = the team does not exist in that
ruleset, so there is nothing to cover.

A cell is 🟢 only if **all four** hold:

| | condition | source |
|---|---|---|
| a | a roster exists for that ruleset | `data/rosters/<ed>/` |
| b | its squad(s) satisfy R1–R4 | `scripts/validate_teams.py` — 0 violations, 0 unfielded slots |
| c | the roster reconciles clean against that ruleset's official pages | `scripts/audit_rosters.py --edition <ed> --report` |
| d | nine gates 100/100 **on the current squad** | per-race ledger `docs/PARITY_<RACE>_CAMPAIGN.md` |

### 11.1 Official teams — 32 distinct, 83 real cells

| team | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| amazon | 🟢 | 🔴 c | 🟢 |
| black_orc | N/A | 🔴 d | 🔴 d |
| bretonnian | N/A | N/A | 🔴 d |
| chaos | 🔴 c | 🔴 d | 🔴 d |
| chaos_dwarf | 🟢 | 🟢 | 🟢 |
| chaos_pact | 🟢 | N/A | N/A |
| dark_elf | 🟢 | 🟢 | 🟢 |
| dwarf | 🔴 c | 🔴 c d | 🟢 |
| elf | 🟢 | 🟢 | 🟢 |
| gnome | N/A | 🔴 d | 🔴 d |
| goblin | 🔴 c | 🟢 | 🟢 |
| halfling | 🔴 c | 🟢 | 🟢 |
| high_elf | 🟢 | 🟢 | 🟢 |
| human | 🟢 | 🟢 | 🟢 |
| imperial_nobility | N/A | 🟢 | 🟢 |
| khemri | 🟢 | 🟢 | 🟢 |
| khorne | N/A | 🔴 d | 🔴 d |
| lizardman | 🟢 | 🔴 c | 🟢 |
| necromantic | 🔴 c | 🟢 | 🟢 |
| norse | 🟢 | 🟢 | 🟢 |
| nurgle | 🟢 | 🟢 | 🟢 |
| ogre | 🟢 | 🟢 | 🟢 |
| old_world_alliance | N/A | 🔴 d | 🔴 d |
| orc | 🔴 d | 🟢 | 🟢 |
| renegades | N/A | 🔴 d | 🔴 d |
| skaven | 🟢 | 🔴 c | 🟢 |
| slann | 🟢 | N/A | N/A |
| snotling | N/A | 🔴 d | 🔴 d |
| undead | 🔴 c | 🔴 d | 🟢 |
| underworld | 🔴 c | 🔴 d | 🔴 d |
| vampire | 🔴 d | 🔴 c d | 🔴 d |
| wood_elf | 🔴 c | 🟢 | 🟢 |

`c` = condition (c) fails, the roster does not reconcile against that ruleset's pages.
`d` = condition (d) fails, no valid nine-gate result stands on the current squad.

| | bb2016 | bb2020 | bb2025 | total |
|---|---|---|---|---|
| 🟢 | 14 | 15 | 20 | **49** |
| 🔴 | 10 | 14 | 10 | **34** |
| N/A | 8 | 4 | 1 | 13 |
| cells | 24 | 29 | 30 | 83 |

### 11.2 Why each red is red

**Provenance only (`c`) — 13 cells.** The roster carries numbers the ruleset's own pages
contradict. Measured live at `f9e1e45d6`, not inferred:

* bb2020, 5 teams / 6 numbers: `amazon`/`dwarf`/`lizardman` `reroll_cost` too cheap,
  `skaven`/`vampire` too expensive, `vampire.apothecary` false → true. Zero positional
  mismatches over 155 positions. `dwarf` 40k and `skaven` 60k are LRB6 numbers never
  updated for BB2020.
* bb2016, 8 teams / 18 lines (§4.2). The `goblin` and `underworld` extras — `Animosity`,
  `Regeneration`, `Thick Skull`, and a `Regeneration` listed twice — are BB2020-era
  contamination that the legacy negative `--cleanup` pass could not find, because those
  skills are not in its `BB2016_REMOVE_SKILLS` list.

**No standing gate (`d`) — 21 cells.** Three distinct causes:

1. **Never gated — 7 teams, 13 cells.** `black_orc`, `gnome`, `imperial_nobility`,
   `khorne`, `old_world_alliance`, `snotling` (bb2020+bb2025) and `bretonnian` (bb2025).
   Rosters and squads exist as of `8bf1e2cfe`; no parity run has ever been made.
2. **Squad re-drafted in place since its gate — 4 cells.** bb2016 `orc`, bb2020 `dwarf`,
   bb2020 `undead`, bb2020 `underworld`. Each was gated 100/100 × 9, but on a *different*
   squad; the R4 re-draft voided that result. These are the cheapest reds on the board —
   re-run, no engine work expected.
3. **R3 variant not gated — `chaos`, `chaos_pact`, `renegades`, `underworld`.** R4 asks the
   *union* of a cell's squads to cover every positional, so the base squad's green does not
   cover the cell once a variant supplies the rest of the coverage. Nine new variant squads
   are still uncommitted.

**`vampire` is the only genuine engine red.** Nine gates measured
bb2020 100/98/100, bb2025 99/95/100, **bb2016 0/100** — the frontier is the bb2016 re-pick
sampler (`docs/PARITY_VAMPIRE_CAMPAIGN.md` §Frontier).

### 11.3 One deviation carried by every bb2016 cell

All 29 bb2016 rosters store a `pa` value. **LRB6 defines no PA characteristic — passing is
AG-based — and the pages have no PA column** (§2). The reconciliation deliberately neither
reports nor zeroes it, and it was *verified inert*: bb2016 never produces
`PlayerStatKey::PA`, and `draw_passing()` is false on that path, so it cannot move a parity
result.

Under a strict reading of R5 ("a roster for each ruleset, faithful to that ruleset") every
bb2016 🟢 above is provisional until `pa` is stripped. It is listed here rather than folded
into the dots because it is provably unable to change any gate outcome — but it is the
user's call, and stripping it is cheap.

### 11.4 Non-official rosters — not part of the matrix

Five repo rosters map to no official page in any edition and are excluded from §11.1
(§4 covers them): `nippon`, `dark_elf_league_fumbbl`, `khemri_fumbbl`, `slann_fumbbl`, and
bb2016 `renegades` (a FUMBBL Chaos Renegades import; CRP prints Chaos Pact, which the repo
already has as `chaos_pact`). All five are gated 100/100 × 9 and green as *engine* parity;
they simply have no ruleset page to be faithful to.

## 12. The measured gate matrix — 2026-09-07, the honest numbers

This section is a **measurement**, not a claim of completion. Every number below comes from a run
that printed `PARITY: …` and did not panic; nothing is rounded, nothing is omitted, and the red
cells are reported exactly as they came out.

### 12.1 What was run

```
./target/release/ffb-parity --home <M> --away <M> --edition <E> --tier 3 \
    --seeds 1-100 --no-abort --agent heuristic --heur-scale <S> --heur-classes all
```

**120 gates**, each 100 seeds: every new matchup (the 7 new teams and all 9 R3 variants) at
`1.0` / `0` / `1e6` in each edition it exists; all four AMENDED races at nine gates each; and a
15-race regression sample at bb2025 `@1.0`.

Provenance of the binaries under test:

| | |
|---|---|
| repo HEAD when the batch started | `279e59cc0` |
| Rust binary | built 13:30 from HEAD plus the then-uncommitted vampire **ITER1** edits (since committed as `3ff581f51`); frozen in a private `CARGO_TARGET_DIR` for the whole batch so a concurrent session's rebuilds could not move it |
| Java harness jar | `ffb-ai-jar-with-dependencies.jar` built 13:18 from the same ITER1 harness source. It was rebuilt at 14:54 by the concurrent vampire session, i.e. **mid-batch** — but `git diff --numstat --ignore-all-space -- ffb-java` over that window is **empty** and there is no committed `ffb-java` change after `3ff581f51`, so the two jars are behaviourally identical for these runs |
| data | `python scripts/gen_java_parity_data.py` re-run first: "wrote 666 XML files across 2 server dirs", with **no XML drift** against the committed tree. `python scripts/check_java_trees.py`: *trees agree* |
| bb2016 caveat | vampire ITER2–ITER4 later edited bb2016 `step_end_moving` / `step_end_selecting` / `step_init_passing` / `framework.rs`. The bb2016 numbers below predate those edits |

Pre-flight checks, all clean: `scripts/validate_teams.py` → **0 R1 violations, 0 unfielded
positional slots** (29 bb2016 / 41 bb2020 / 41 bb2025 squads); `scripts/check_skill_names.py` →
**0 unresolvable skill names**; `cargo test -p ffb-model all_roster_starting_skills_resolve` →
1 passed.

### 12.2 (a) The new matchups — 24 cells, 72 gates

| edition | matchup | @1.0 | @0 | @1e6 | cell |
|---|---|---:|---:|---:|---|
| bb2020 | black_orc | **97/100** | 100/100 | **98/100** | 🔴 |
| bb2020 | gnome | **38/100** | **53/100** | **58/100** | 🔴 |
| bb2020 | imperial_nobility | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2020 | khorne | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2020 | old_world_alliance_ogre | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2020 | old_world_alliance_treeman | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2020 | snotling | **0/100** | **1/100** | **0/100** | 🔴 |
| bb2020 | chaos_chaosogre *(R3 variant)* | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2020 | chaos_chaostroll *(R3)* | 100/100 | 100/100 | 100/100 \* | 🟢 |
| bb2020 | chaos_pact_renegadetroll *(R3)* | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2020 | renegades_37730 *(R3)* | **99/100** | 100/100 | 100/100 | 🔴 |
| bb2020 | underworld_underworldtroll *(R3)* | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2025 | black_orc | **99/100** | 100/100 | **98/100** | 🔴 |
| bb2025 | gnome | **35/100** | **10/100** | **51/100** | 🔴 |
| bb2025 | imperial_nobility | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2025 | khorne | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2025 | old_world_alliance_ogre | 100/100 | 100/100 | **99/100** | 🔴 |
| bb2025 | old_world_alliance_treeman | 100/100 | 100/100 | **99/100** | 🔴 |
| bb2025 | snotling | **0/100** | **0/100** | **0/100** | 🔴 |
| bb2025 | bretonnian | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2025 | chaos_ogre *(R3)* | 100/100 | 100/100 | 100/100 \* | 🟢 |
| bb2025 | chaos_troll *(R3)* | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2025 | renegades_37733 *(R3)* | 100/100 | 100/100 | 100/100 | 🟢 |
| bb2025 | underworld_37844 *(R3)* | 100/100 | 100/100 | 100/100 | 🟢 |

\* prints the `…, but required coverage items are MISSING` trailer — that is the tier-3 coverage
checklist, not parity, and is a PASS for the parity half (as established in the wood_elf ledger).

**New-matchup cells: 14 🟢 / 10 🔴.** All **nine R3 variants** are green except `renegades_37730`,
which loses one seed. Of the 7 new teams, `khorne` and `bretonnian` are green at every gate they
have, and both Old World Alliance variants are green in bb2020; `snotling` is the worst red on the
board and `gnome` the second.

### 12.3 (b) The AMENDED squads re-gated — 12 cells, 36 gates

Their earlier gates were void: those squads were re-drafted in place by the §8 R2/R4 closure
(bb2016 `orc`, bb2020 `dwarf`, bb2020 `undead`, bb2020 `underworld`). Nine gates were re-run per
race — all three editions, because these races share code paths across editions.

| race | bb2016 @1.0 / @0 / @1e6 | bb2020 @1.0 / @0 / @1e6 | bb2025 @1.0 / @0 / @1e6 |
|---|---|---|---|
| orc | 100/100 · 100/100 · 100/100 | 100/100 · 100/100 · 100/100 \* | 100/100 · 100/100 · 100/100 |
| dwarf | 100/100 \* · 100/100 · 100/100 | 100/100 · 100/100 · 100/100 | 100/100 · 100/100 · 100/100 |
| undead | 100/100 · 100/100 · 100/100 | 100/100 · 100/100 · 100/100 | 100/100 · 100/100 · 100/100 |
| underworld | 100/100 · 100/100 · 100/100 \* | 100/100 · 100/100 · 100/100 | 100/100 · 100/100 · 100/100 |

**36 of 36 gates at 100/100.** The in-place re-drafts cost nothing: the Orc Lineman traded in, the
Troll Slayer added, the Skeleton swapped for a Zombie and the two Snotlings added are all at parity
on their first measurement.

### 12.4 (c) Regression sample — 15 races, bb2025 @1.0

Run to show the §8 data change and the `gen_java_parity_data.py` team-id fix disturbed no race that
nobody touched:

| race | result | | race | result |
|---|---|---|---|---|
| nurgle | 100/100 | | ogre | 100/100 |
| necromantic | 100/100 | | orc | 100/100 |
| khemri | 100/100 | | skaven | 100/100 |
| human | 100/100 | | slann | 100/100 |
| goblin | 100/100 | | wood_elf | 100/100 |
| amazon | 100/100 | | undead | 100/100 |
| chaos_dwarf | 100/100 | | dwarf | 100/100 |
| norse | 100/100 | | | |

**15 of 15 at 100/100.** (`orc`, `dwarf` and `undead` are the §12.3 runs of the same cell — one
measurement reported under both headings, not run twice.)

### 12.5 Totals

| | gates | 100/100 | below 100 |
|---|---:|---:|---:|
| (a) new matchups | 72 | 51 | **21** |
| (b) amended re-gate | 36 | 36 | 0 |
| (c) regression sample | 12 (+3 shared with b) | 12 | 0 |
| **total** | **120** | **99** | **21** |

No run panicked (`panics=0` on all 120). Cell verdicts over the 36 cells measured:
**26 🟢 / 10 🔴**.

### 12.6 Every red, classified from its own log

Classification only — **no engine fix was attempted in this phase**, by instruction.

| gate | fails | of which STALLS (`rust=None`) | first failing seed / step | shape |
|---|---:|---:|---|---|
| snotling bb2020 @1.0 | 100 | 0 | seed 1, **step 0** | every seed, first step |
| snotling bb2020 @0 | 99 | 0 | seed 1, **step 0** | " |
| snotling bb2020 @1e6 | 100 | 0 | seed 1, **step 0** | " |
| snotling bb2025 @1.0 | 100 | 0 | seed 1, **step 0** | " |
| snotling bb2025 @0 | 100 | 0 | seed 1, **step 0** | " |
| snotling bb2025 @1e6 | 100 | 0 | seed 1, **step 0** | " |
| gnome bb2025 @0 | 90 | **80** | seed 1, step 135 | mostly Rust STALLS |
| imperial_nobility bb2025 @1e6 | 67 | 1 | seed 4, step 43 | state divergence |
| gnome bb2025 @1.0 | 65 | 16 | seed 1, step 87 | mixed |
| gnome bb2020 @1.0 | 62 | 13 | seed 2, step 86 | mixed |
| gnome bb2025 @1e6 | 49 | 11 | seed 1, step 59 | mixed |
| gnome bb2020 @0 | 47 | 2 | seed 2, step 242 | state divergence |
| gnome bb2020 @1e6 | 42 | 10 | seed 4, step 38 | mixed |
| imperial_nobility bb2025 @1.0 | 16 | 0 | seed 3, step 67 | state divergence |
| black_orc bb2020 @1.0 | 3 | 0 | seed 14, step 71 (seeds 14, 33, 37) | state divergence |
| black_orc bb2020 @1e6 | 2 | 0 | seed 34, step 35 (seeds 34, 40) | state divergence |
| black_orc bb2025 @1e6 | 2 | 0 | seed 3, step 45 (seeds 3, 24) | state divergence |
| black_orc bb2025 @1.0 | 1 | 0 | seed 33, step 79 | state divergence |
| renegades_37730 bb2020 @1.0 | 1 | 0 | seed 33, step 93 | state divergence |
| old_world_alliance_ogre bb2025 @1e6 | 1 | 0 | seed 68, step 89 | state divergence |
| old_world_alliance_treeman bb2025 @1e6 | 1 | 0 | seed 93, step 119 | state divergence |

Two findings worth recording before anyone picks a target:

1. **`snotling` is not an agent disagreement — it is a PRE-GAME divergence.** All 600 snotling
   games fail at **step 0**, with the two `state_hash` values already different *before the first
   activation is applied*, in both editions and at all three scales. The distinctive roster feature
   is **`Swarming`** on the Snotling Lineman (it changes how many players are set up), plus **four
   secret-weapon players** (2 Fungus Flingas, 2 Pump Wagons) and `Pogo Stick` / `Projectile Vomit`.
   A setup-time difference is the hypothesis the log supports; it has **not** been verified and must
   not be assumed.
2. **`gnome` is dominated by Rust STALLS, not by wrong decisions.** 80 of the 90 bb2025 `@0`
   failures carry `rust=None` — Rust ran out of steps while Java kept playing. That is the
   "shorter Rust log with no state mismatch" fingerprint the heuristic campaign documents as a
   *dead action*, not a subtle divergence. The gnome roster's distinctive skills are `My Ball`,
   `Trickster`, `Timmm-ber!` and `Take Root`, and its Woodland Fox is a PA-0 positional.

### 12.7 How each new matchup was proved NOT to be the silent lineman fallback

`make_team()` ends in `.unwrap_or_else(|e| { log::warn!(..); make_lineman_team(side, roster_name) })`
and the fallback team keeps the *requested* `roster_id`, so a fallback is invisible in the ids and
still gates green. Three independent proofs were used, and all three hold for all 30 squads in this
batch (13 new/changed plus the base squads of the variant cells):

1. **Rust side, on the live path, asserted per squad.** `cargo test -p ffb-parity coverage_squad`
   → **2 passed, 0 failed** on the binary under test.
   `coverage_squads_build_their_real_roster_not_the_lineman_fallback` calls the same
   `make_team(squad, side, edition)` a gate calls, for **home and away of all 30 squads**, and
   asserts: the exact fielded `position_id` multiset against the squad JSON; every player's
   `(ma, st, ag, av)` equal to its roster position's (**the fallback is 6/3/3/8**); and a fingerprint
   positional carrying a fingerprint starting skill — Bone Head on the Ogres, Always Hungry on the
   Trolls, `My Ball` on the Woodland Fox, `Unchannelled Fury` on Bloodspawn and Minotaur, `Take Root`
   on the Altern Forest Treeman, `Dauntless` on the Grail Knight, `Right Stuff` on the Snotling,
   `Regeneration` on the Skeleton, `Frenzy` on the Troll Slayer, `Animal Savagery` on the Rat Ogres.
   A fallback lineman has **no starting skills at all**, so the last assertion alone kills it.
   `coverage_cells_field_every_positional` re-checks R2/R4 over the same set.
2. **Java side, read out of the runs themselves.** `gen_java_parity_data.py` derives the Java
   `<team id=..>` from the squad **file stem**, so the player ids Java prints name the team XML it
   actually loaded. Harvested from this batch's own logs: `teamBlackOrcParity20/25`,
   `teamGnomeParity20/25`, `teamImperialNobilityParity25`, `teamKhorneParity25`,
   `teamOldWorldAllianceOgreParity25`, `teamOldWorldAllianceTreemanParity25`,
   `teamSnotlingParity20/25`, `teamBretonnianParity25`, `teamRenegades37730Parity20`,
   `teamChaosOgreParity25`, `teamChaosTrollParity25`, `teamRenegades37733Parity25`,
   `teamUnderworld37844Parity25`, `teamOrcParity25`, `teamDwarfParity25`, `teamUndeadParity25`,
   `teamUnderworldParity25`. Note that **each R3 variant prints its OWN id** — the generator bug
   fixed in `279e59cc0` staying fixed. `grep -rl teamLineman logs/` over the whole batch returns
   **nothing**. (A fully green log that never rolls a pickup prints no player id at all; that is an
   absence of evidence in the log, not evidence of a fallback, and proof 1 covers those cells.)
3. **Transitively, for every green cell.** A per-step state-hash match over 100 seeds against a Rust
   team proven by (1) to hold the real roster means Java fielded the same team — a fallback on one
   side only cannot produce 100/100. For the red cells the divergence itself, plus (2), rules the
   fallback out.

### 12.8 A measurement trap this batch fell into, and the guard added

The first attempt ran 7 lanes concurrently. Two things went wrong, and both produced
*plausible-looking* results:

* **Resource exhaustion looks like a gate.** With 7 JVMs the machine ran out of memory, `fork()`
  began failing, and jobs exited `rc=127` after 2–4 s having written only their banner line. Those
  rows had **no `PARITY:` line** — which is exactly why the "a sweep counts only if it prints
  `PARITY: N/M`" rule exists; counting the absence of `PARITY FAIL` would have scored them green.
  Concurrency was cut to 3 lanes and the runner now retries a job up to 3× when no `PARITY:` line
  appears.
* **A CRLF job file silently changed `--heur-scale`.** The second round's job list was written by a
  Python script in text mode, so every line ended `\r`. `--heur-scale "1.0\r"` hits
  `raw[i + 1].parse().unwrap_or(0.0)` in `crates/ffb-parity/src/main.rs:131` and becomes **0.0** —
  so 70 runs labelled `@1.0` / `@0` / `@1e6` had **all three run at scale 0**, and the only giveaway
  was that a cell's three scales printed *identical* numbers. Those results were re-scoped to `@0`
  (27 usable, the 43 duplicates discarded) and every `@1.0` / `@1e6` gate was re-run. Where a
  round-1 `@0` result and a re-scoped round-2 `@0` result both existed for a cell they **agreed in
  every case** — a free determinism check. The runner now reads the scale back out of the run's own
  banner (`scale=1` / `scale=0` / `scale=1000000`) and stamps each row `scale-ok` or
  `SCALE-MISMATCH`; only `scale-ok` rows are counted, and every row in §12.2–12.4 is `scale-ok`
  (the round-1 rows were verified retrospectively by grepping all 108 of their banners).

*Lesson, in its general form: an argument the CLI parses with `unwrap_or(<default>)` cannot report a
bad value, so a typo in a harness script silently becomes a different experiment. Read the setting
back out of the run; do not trust the flag you passed.* A third, smaller trap: the lane loop fed its
job list on stdin and the first `ffb-parity` consumed the rest of it, so each lane silently ran
exactly one job — the runner now reads the list into an array and gives the binary `< /dev/null`.

### 12.9 What §11.1 now owes

§11.1 marked 21 cells red for "no standing gate (`d`)". This batch settles the parity half for the
36 cells above and leaves the roster-provenance half (`c`) exactly where it was. Re-measured live
for this section: `audit_rosters.py --edition bb2025 --report` → **0 rosters differ**;
`--edition bb2020` → **24 of 29 match their page, 5 differ** (amazon, dwarf, lizardman, skaven,
vampire — and all seven NEW bb2020 rosters are in the clean list); `--edition bb2016` → **16 of 24
match, 8 differ**. `validate_teams.py --r5` is unchanged: 4 shared rosters, 29 bb2016 PA carriers.

So §11.1 should become: the four AMENDED cells and the nine R3 variant cells lose their `d`;
`khorne`, `bretonnian` and the two bb2020 Old World Alliance variants become 🟢; and `black_orc`,
`gnome`, `imperial_nobility`, `snotling` and the two bb2025 Old World Alliance cells keep `d` on the
numbers above. That edit is deliberately **not** made here — §11.1's green also requires (c), and
rewriting it belongs with the provenance work, not with a measurement.

## 13. The matrix, updated from §12's measurements — 2026-09-07

§11 was computed when the 7 new teams, the 9 R3 variants and the 4 amended squads were all
UNGATED. §12 measured them. Three groups of cells move:

* **The 4 amended squads are re-gated 36/36 at 100/100** (§12b), so bb2016 `orc`, bb2020 `undead`
  and bb2020 `underworld` clear condition (d). bb2020 `dwarf` stays 🔴 on provenance (c).
* **All 9 R3 variants are green bar `renegades_37730`** (bb2020, 99/100 @1.0), so `chaos` and
  `underworld` clear in both editions and bb2025 `renegades` clears; bb2020 `renegades` does not.
* **The 7 new teams measured 14 green / 10 red cells**, per-cell in §12a.

| team | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| amazon | 🟢 | 🔴 c | 🟢 |
| black_orc | N/A | 🔴 97/100/98 | 🔴 99/100/98 |
| bretonnian | N/A | N/A | 🟢 |
| chaos | 🔴 c | 🟢 | 🟢 |
| chaos_dwarf | 🟢 | 🟢 | 🟢 |
| chaos_pact | 🟢 | N/A | N/A |
| dark_elf | 🟢 | 🟢 | 🟢 |
| dwarf | 🔴 c | 🔴 c | 🟢 |
| elf | 🟢 | 🟢 | 🟢 |
| gnome | N/A | 🔴 38/53/58 | 🔴 35/10/51 |
| goblin | 🔴 c | 🟢 | 🟢 |
| halfling | 🔴 c | 🟢 | 🟢 |
| high_elf | 🟢 | 🟢 | 🟢 |
| human | 🟢 | 🟢 | 🟢 |
| imperial_nobility | N/A | 🟢 | 🔴 84/100/33 |
| khemri | 🟢 | 🟢 | 🟢 |
| khorne | N/A | 🟢 | 🟢 |
| lizardman | 🟢 | 🔴 c | 🟢 |
| necromantic | 🔴 c | 🟢 | 🟢 |
| norse | 🟢 | 🟢 | 🟢 |
| nurgle | 🟢 | 🟢 | 🟢 |
| ogre | 🟢 | 🟢 | 🟢 |
| old_world_alliance | N/A | 🟢 | 🔴 both variants 99 @1e6 |
| orc | 🟢 | 🟢 | 🟢 |
| renegades | N/A | 🔴 variant 99/100 | 🟢 |
| skaven | 🟢 | 🔴 c | 🟢 |
| slann | 🟢 | N/A | N/A |
| snotling | N/A | 🔴 0/1/0 | 🔴 0/0/0 |
| undead | 🔴 c | 🟢 | 🟢 |
| underworld | 🔴 c | 🟢 | 🟢 |
| vampire | 🔴 46/33/67 | 🔴 c + 100/98/100 | 🔴 99/95/100 |
| wood_elf | 🔴 c | 🟢 | 🟢 |

| | bb2016 | bb2020 | bb2025 | total |
|---|---|---|---|---|
| 🟢 | 15 | 20 | 24 | **59** |
| 🔴 | 9 | 9 | 6 | **24** |
| N/A | 8 | 4 | 1 | 13 |

**59 🟢 / 24 🔴**, up from §11's 49/34. The 13 provenance-only reds (`c`) are unchanged — no
roster data was corrected, so those cells cannot move until the bb2020 6 numbers and the bb2016 18
lines are applied.

### Caveat on the bb2016 column

§12 records that its bb2016 numbers **predate** the vampire ITER2-ITER4 edits to bb2016
`step_end_moving` / `step_end_selecting` / `step_init_passing` / `framework.rs`. bb2016 `orc`'s
🟢 therefore rests on a pre-edit measurement and should be re-gated before it is trusted; the same
caveat applies to every bb2016 🟢 carried over from §11.

### The two remaining engine faults worth naming

* **`snotling` diverges at STEP 0 of every seed, both editions** — a pre-game divergence, not an
  agent disagreement. Cheapest red on the board to diagnose: nothing has happened yet.
* **`gnome` failures are dominated by Rust STALLS** (80 of 90 at bb2025 @0).

### Two measurement traps §12 recorded, both worth carrying forward

1. **7 concurrent JVMs exhausted RAM**, producing `rc=127` runs with no PARITY line. Concurrency
   here is bounded by MEMORY, not cores.
2. **A CRLF job file made `--heur-scale "1.0\r"` parse to 0.0 via `unwrap_or(0.0)`**, silently
   running 70 gates at the WRONG scale. A parse fallback that swallows a malformed value is worse
   than a crash: every one of those gates would have been reported against the wrong column. The
   batch runner now reads the scale back out of each run's own banner. **`ffb-parity`'s
   `unwrap_or(0.0)` on the scale argument should reject a bad value instead** — filed as follow-up.

## 14. The matrix after the R1-R5 roster corrections — 2026-09-07

84 gates re-measured on the corrected data (`scripts/gate_batch.ps1`: one gate at a time, pinned to
4 of 16 CPUs, per-gate `FFB_PARITY_ROOT`). Every bb2016 race was re-gated at all three scales
because the PA strip touched all 29 of its rosters; the 5 changed bb2020 races were re-gated too.
The bb2025 column was not touched by these changes and carries forward from §13.

**Stripping PA is behaviour-neutral, measured not assumed.** Three pure PA-only controls (human,
orc, khemri) and every other unchanged bb2016 race came back 100/100 at all three scales.

| | bb2016 | bb2020 | bb2025 | total |
|---|---|---|---|---|
| 🟢 | 21 | 24 | 24 | **69** |
| 🔴 | 3 | 5 | 6 | **14** |
| N/A | 8 | 4 | 1 | 13 |

Against §13's 59/24 and §11's 49/34. Ten cells flipped green on the provenance fixes alone:
bb2016 `chaos` `dwarf` `goblin` `halfling` `undead` `wood_elf`, bb2020 `amazon` `dwarf`
`lizardman` `skaven`.

### The corrections exposed TWO new reds, which is the point

Both were green before and are red on rule-legal data. Neither is a regression: the old squads were
illegal (over budget, or fielding a position that is not on the CRP page), and an illegal squad
cannot reach the paths a legal one does.

1. **`necromantic` bb2016 — 98/100 @1.0, 99/100 @0, 99/100 @1e6.** Its Zombie cost was 30k against
   the page's 40k, which had kept the squad 60,000 OVER budget. Both @1.0 failures (seeds 47, 56)
   diverge at the **first activation of half 2**.
2. **`underworld` bb2016 @0 — 99/100 (seed 100).** Its roster listed a Mutant Rat Ogre that is not
   on the CRP page at all, and its squad fielded two of them plus 3 skaven linemen (cap 2) and 2
   warpstone trolls (cap 1). On the re-drafted legal squad the divergence is at `i=68`, half 1
   turn 5, resolving a **Blitz at i=67** into a **Foul** declaration. @1.0 and @1e6 are 100/100.

### A reading note on this batch

Seven gates print `100/100 games match, but required coverage items are MISSING` and exit 1. That
is the **tier-3 coverage checklist**, not parity: those cells are parity-GREEN and counted as such
here, consistent with how §12 counted the same trailer.

## 15. underworld bb2016 CLOSED -- 70 green / 13 red, 2026-09-08

`underworld` bb2016 was the newest red (§14), exposed when its off-page Mutant Rat Ogre was removed
and its illegal squad re-drafted. **It is now GREEN at all three scales** (100/100 at 1.0, 0 and
1e6) via two defects in the bb2016 `StepPushback` crowd branch -- see commit for the full account:

1. the crowd-push victim was `game.defender_id` (the block's ORIGINAL defender) where Java uses
   `state.defender`, the CURRENT occupant of the square being pushed into;
2. the crowd branch RETURNED where Java sets `doPush = true` and falls through to the block that
   pops the pushback stack and moves the queued players.

Fixing only (1) moved the frontier instead of closing it, which is what exposed (2) -- the
half-fix left the crowd victim correct but the rest of the chain frozen.

| | bb2016 | bb2020 | bb2025 | total |
|---|---|---|---|---|
| 🟢 | 22 | 24 | 24 | **70** |
| 🔴 | 2 | 5 | 6 | **13** |
| N/A | 8 | 4 | 1 | 13 |

Remaining bb2016 reds: `necromantic` (98/99/99, unchanged by this fix) and `vampire` (46/33/67).

No regression: bb2016 `chaos` `orc` `dwarf` `undead` `goblin` `khemri` 100/100 @1.0 and `amazon`
`human` 100/100 @0. ffb-engine 7457/0.

## 16. renegades bb2020 CLOSED -- 71 green / 12 red, 2026-09-08

`renegades` bb2020 was red on ONE seed of its R3 variant (`renegades_37730` 99/100 @1.0). Both the
base squad and the variant are now 100/100 at all three scales.

**Cause: an edition twin the driver never runs.** bb2020's `StepAlwaysHungry` sets
`setPassUsed(true)` where bb2025 sets `setTtmUsed(true)`, but `driver.rs` builds the bb2025 file
for bb2020 (`use crate::step::bb2025::ttm::*` plus the default `StepId::AlwaysHungry` arm) -- so
the correct bb2020 twin was DEAD CODE and bb2020 wrote the bb2025 flag. `pass_used` is hashed by
`state_string`; `ttm_used` is not. Every bb2020 Always Hungry throw therefore diverged the hash one
step later.

Fixed by edition-gating INSIDE the shared step, the pattern the TTM campaign already established
(`[[parity_tier_ttm]]`: routing an edition to its dead twin does not work).

**What made it findable:** at seed 33 the boards were identical (`FFB_IDSTATE`, 0 diffs), the
declarations were identical, and the ONLY difference in the entire state string was the flag block
-- `f0001` against `f0000`, the `pass_used` digit of `blitz/foul/hand_over/pass`. The id-state dump
cannot see turn flags, so the state STRING is the instrument that names this class of bug.

| | bb2016 | bb2020 | bb2025 | total |
|---|---|---|---|---|
| 🟢 | 22 | 25 | 24 | **71** |
| 🔴 | 2 | 4 | 6 | **12** |
| N/A | 8 | 4 | 1 | 13 |

No regression: goblin, `underworld_underworldtroll`, `chaos_pact` and ogre bb2020 all 100/100
@1.0 (every other Always Hungry carrier reachable here), and goblin bb2025 + bb2016 100/100 @1.0
confirming the untouched editions. ffb-engine 7458/0.

## 17. vampire bb2020 CLOSED -- 72 green / 11 red, 2026-09-08

`vampire` bb2020 measured **100/100 at all three scales on one build** (`CELL: GREEN`). Its
recorded red was 100/98/100, with two failures at @0.

### No new engine work closed this one, and the attribution matters

Neither engine fix from this session can explain it:

* the bb2016 chain crowd-push fix is bb2016-only;
* the Always Hungry edition gate needs a carrier with `mightEatPlayerToThrow`, and the vampire
  roster has none (Thrall, Runner, Blitzer, Thrower, Vargheist -- no Big Guy);
* F0 only touches argument parsing.

The apothecary correction is NOT the mechanism either: the squad fields `apothecaries: 0`, so the
roster's `apothecary` flag changes nothing that is played. What remains is the `reroll_cost`
70k -> 60k correction (spend 1,080,000 -> 1,050,000) and, more likely, that **the 98/100 was
measured on an older binary** -- §12 records its build as predating engine edits that are now
committed. Without bisecting the two I cannot say which, so this cell is recorded as **verified,
not fixed**.

### The generalisable point: some reds may be stale rather than real

Every remaining red except `necromantic` bb2016 and `underworld`/`renegades` (both now closed)
traces to the §12 batch, measured on a binary built before several committed engine changes. A red
number is only as current as the build that produced it. **Re-measure a red before designing a fix
for it** -- the same lesson vampire bb2016 taught in ITER2, now paid twice.

| | bb2016 | bb2020 | bb2025 | total |
|---|---|---|---|---|
| 🟢 | 22 | 26 | 24 | **72** |
| 🔴 | 2 | 3 | 6 | **11** |
| N/A | 8 | 4 | 1 | 13 |

Remaining: bb2016 `necromantic` `vampire`; bb2020 `black_orc` `gnome` `snotling`; bb2025
`black_orc` `gnome` `imperial_nobility` `old_world_alliance` `snotling` `vampire`. The bb2020 and
bb2025 entries all carry §12-vintage numbers and should be re-measured first.

## 18. THE TRUSTWORTHY MATRIX -- 333 gates, one build, 2026-09-08

The whole matrix re-measured end to end on a single binary: every squad in every edition at all
three scales, base squads AND the nine R3 variants. **333 gates, 0 without a verdict.** Ran as 4
sharded workers on 8 of 16 cores (`scripts/sweep_worker.ps1`), sharded by (edition, matchup) so no
two runs of one matchup ever overlapped. 0.93 gates/min, ~6h.

Previous tables (§11 §13 §14 §16 §17) were each honest per number but MIXED BUILDS, so they are
superseded. This is the first table where every cell comes from the same engine.

| | bb2016 | bb2020 | bb2025 | total |
|---|---|---|---|---|
| 🟢 | 23 | 26 | 25 | **74** |
| 🔴 | 1 | 3 | 5 | **9** |
| N/A | 8 | 4 | 1 | 13 |

### The 9 remaining red cells, with their current numbers (@1.0 / @0 / @1e6)

| edition | team | @1.0 | @0 | @1e6 | note |
|---|---|---|---|---|---|
| bb2016 | necromantic | 98 | 99 | 99 | exposed by the Zombie-cost correction; both @1.0 seeds diverge at the FIRST ACTIVATION OF HALF 2 |
| bb2020 | black_orc | 98 | 100 | 98 | |
| bb2020 | gnome | 38 | 53 | 58 | failures dominated by Rust STALLS |
| bb2020 | snotling | 0 | 1 | 0 | diverges at STEP 0 of every seed -- pre-game |
| bb2025 | black_orc | 99 | 100 | 98 | |
| bb2025 | gnome | 35 | 10 | 51 | STALLS (80 of 90 at @0) |
| bb2025 | imperial_nobility | 84 | 100 | 33 | green at argmax, worst at uniform => the option SET, not the weights |
| bb2025 | old_world_alliance | 100 | 100 | 99 | BOTH variants 99 at @1e6 only; one seed each |
| bb2025 | snotling | 0 | 0 | 0 | pre-game, as bb2020 |

### What the sweep settled that spot-checks could not

* **The chain crowd-push fix held across all of bb2016.** Every bb2016 cell except `necromantic` is
  green on the current build, including `vampire` (was 46/33/67) and `underworld`. I had only
  gated 9 races by hand; the edition-wide regression check arrived here, clean.
* **`vampire` is now green in ALL THREE editions.** Its bb2016 number was the largest red on the
  board and F1 in the queue; the fix for underworld closed it. The give-chain symptom
  ("`prompt_after=None` after a HandOffMove") was the frozen pushback chain, not a separate bug.
* **Six of the nine reds are the newly drafted teams** (`black_orc`, `gnome`, `snotling` -- never
  exercised before this campaign), plus `imperial_nobility` and `old_world_alliance`. The
  long-established races are green everywhere except `necromantic` bb2016.
* **Cross-build seed counts are not comparable even when a cell's colour is unchanged**:
  `black_orc` bb2020 read 97 at @1.0 on the old build and 98 here. Only same-build numbers can be
  differenced.

### Non-official rosters (not part of the 83 official cells)

All green on this build: `nippon`, `slann`, `chaos_pact` (bb2020/bb2025), `dark_elf_league_fumbbl`,
`khemri_fumbbl`, `slann_fumbbl`.
