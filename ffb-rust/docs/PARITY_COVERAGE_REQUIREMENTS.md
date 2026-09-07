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
