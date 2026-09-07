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
