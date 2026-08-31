# Parity campaign — HeuristicAgent on chaos vs chaos (all three rulesets)

Started 2026-08-31, immediately after the amazon campaign closed (nine gates 100/100, `68fa1851e`).
Next race alphabetically; same goal shape, same three-loop procedure.

**Command**: `/chaos-iter` — `.claude/commands/chaos-iter.md`. Read the TAIL of this file every
iteration for the frontier. The amazon campaign is the playbook and cautionary record:
`docs/PARITY_AMAZON_CAMPAIGN.md` (especially ITER24–30) and `.claude/commands/amz-iter.md`.

## Goal

Nine `PARITY: 100/100 games match` — chaos v chaos, tier 3, seeds 1-100, `--agent heuristic
--heur-classes all`, across bb2016 / bb2020 / bb2025 × `--heur-scale` 0 / 1.0 / 1e6 — **plus**:

- the agent uses the chaos skills in its scoring where they change costs and risks it already
  models, kept fast and simple;
- the event coverage is analysed and recorded (`MATCHUP=chaos scripts/harvest_coverage.sh`), with
  every on-pitch skill explained: agent gap, engine bug, unevented (BACKLOG §E6/E7), or dead path.

## Why chaos — what this roster reaches that amazon could not

All three editions field **Minotaur + 4 Chaos Warriors + 7 Beastmen**:

| | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Beastman | **Horns** | **Horns** | **Horns**, Thick Skull |
| Warrior | (none) | (none) | **Arm Bar** |
| Minotaur | Loner, **Frenzy**, Horns, **Mighty Blow**, Thick Skull, **Wild Animal** | Frenzy, Horns, **Loner 4**, MB(1), Thick Skull, **Unchannelled Fury** | same as bb2020 |

New surface, by mechanism:
- **Horns** — +1 ST on a blitz: block-dice counts change with the action, not just the matchup.
- **Frenzy** — mandatory follow-up + a second block after a pushback; the double-block chain.
- **Mighty Blow** — the armour/injury modifier CHOICE (bb2020+: spend on AV or on injury).
- **Loner (4+)** — the heuristic USES team re-rolls, so Loner gate rolls fire for the first time
  in any heuristic campaign.
- **Wild Animal (bb2016) / Unchannelled Fury (bb2020/25)** — activation negatraits on a player the
  agent wants to blitz with; the Select-sequence negatrait steps under an agent that moves.
- **Thick Skull** — stun-vs-KO boundary on the injury roll.
- ST5 Minotaur + ST4 Warriors — 3-die blocks and `block 3 dice` coverage, absent all campaign.

## Status

| chaos v chaos, `--heur-classes all`, seeds 1-100 | sampled (1.0) |
|---|---|
| bb2016 | 67/100 → **100/100** (ITER2) 🏁 |
| bb2020 | 60/100 → **92/100** (ITER1) |
| bb2025 | 74/100 → **98/100** (ITER1) |

Baseline 2026-08-31, binary `68fa1851e` (`rust_total=` 30–36 s). Every edition is red this time —
Horns/Frenzy/Wild Animal are new even in bb2016. Control: `--agent random` chaos was 100/100 ×3 in
the roster campaign, so the roster itself is parity-clean and every red belongs to the heuristic
path. Scales 0 / 1e6: not yet measured; gate them once 1.0 is green.

## Ledger

(one `## ITER<n>` per iteration below)

## ITER1 — Unchannelled Fury: a consumed re-roll must roll a FRESH negatrait d6 (bb2020/25)

**Seed**: bb2025 seed 8, first hash diff i=34 (resolving i=33, home_01 Minotaur MOVE).

**Road to the root cause** (three wrong turns, each killed by a better instrument):
1. Suspected the bb2025 StepInitMoving never publishes COORDINATE_FROM/TO — FALSE; the publish is
   there (my publisher grep was `head`-truncated; the trace-window-truncation trap again).
2. Suspected the Arm Bar search/handler — FALSE; a gated `RARMBAR` probe (kept, FFB_TRACE-gated in
   `step_move_dodge.rs::fail_dodge`) showed the single candidate `away_05` found and `ForSpp#away_05`
   dispatched; FFB_DRIVE_TRACE showed StepFallDown consuming its 2 armour dice at BOTH fail sites.
3. The armour values differed → suspected `is_armour_broken` — FALSE; the dice STREAMS were offset.

**Actual root cause**: index-base alignment (Java DICE_TRACE pos is 1-based, Rust 0-based) proved
the streams identical from kickoff until the Minotaur's activation: Java rolls UF 3 (fail) → Loner
4 (pass) → **fresh UF re-roll 5 (pass)** → dodge 2 (fail) → ForSpp armour 2+6, +1 Arm Bar breaks
AV9 → injury 10 → cas d16 → Badly Hurt. Rust rolled UF 3 → Loner 4 → **no fresh die** → consumed
the 5 as its dodge, sat one draw behind for the rest of the game, and the same dodge fail armoured
5+2 = held → Prone.

Java `UnchannelledFuryBehaviour.java:66-84`: `boolean doRoll = true`; the `hasUnusedSkill` test is
the ELSE branch, taken only when NO re-roll is in flight. After a CONSUMED `useReRoll` Java rolls a
fresh d6 even though the first roll `markSkillUsed`-ed. Rust applied the used-skills gate on every
path, so the re-entry found UF already used and skipped the roll (`StepAction::NextStep`, 0 dice).

**Fix** (`step/mixed/step_unchannelled_fury.rs`): `do_roll = rerolling || (has_uf_skill &&
!used_skills.contains(UF))`. Test `consumed_team_reroll_rolls_a_fresh_fury_die` (Java-derived).
Note: `step_animal_savagery.rs` already has the correct Java shape; BoneHead/ReallyStupid delegate
to behaviours (bb2025 BoneHeadBehaviour was fixed earlier) — the frontier will say if any sibling
still carries the misport.

**Gate**: chaos bb2016 67 (UF absent — unchanged), bb2020 60→92, bb2025 74→98. Standing: amazon ×3
100/100, lineman ×3 100/100, random controls chaos+amazon+lineman ×3 editions 100/100 (isolated
root), cargo test -p ffb-engine 7366/0. 20-seed probes first: bb2025 20/20, bb2020 16/20, bb2016 12/20.

**Remaining reds**: bb2025 seeds {57 (LENGTH/stall), 1 other}; bb2020 8 seeds incl. seed 18
HandOffMove-vs-PASS_MOVE declaration divergence; bb2016 33 seeds (Wild Animal familia untouched).

## ITER2 — bb2016 Loner minimum is a FIXED 4, not the (absent) skill value

**Seed**: bb2016 seed 10, first hash diff idx 8 (resolving idx 7... actually i=8, the Minotaur's
MOVE after the home_02 blitz). Purpose-diffing the dice streams (Rust DRIVE rng-deltas vs Java
Fortuna callers, 0- vs 1-based aligned) put the first mismatch at pos 33: Java
`RollMechanic.useReRoll:286` (the Loner die) where Rust attributed the position to a fresh GFI.

**Root cause**: Java bb2016 `RollMechanic.minimumLonerRoll` returns a FIXED 4 (LRB6 Loner has no
printed value); bb2020/25 read the skill value. Rust's shared `loner_roll` read
`get_skill_int_value` in every edition — a valueless bb2016 "Loner" yields 0, so
`is_skill_roll_successful(3, 0)` auto-passed every Loner check. Same die drawn, opposite verdict:
Java's Minotaur loses the GFI re-roll and falls; Rust re-rolls the GFI and runs on.

**Fix** (`step/util_server_re_roll.rs::loner_roll`): bb2016 → fixed 4, else the value read. Test
`bb2016_loner_minimum_is_a_fixed_four`.

**Gate**: chaos bb2016 67→**100/100** 🏁 (one fix, all 33 reds were this), bb2020 92 (same 8 seeds),
bb2025 98 (same 2). Standing: amazon ×3 + lineman ×3 100/100, random chaos bb2016 100/100,
ffb-engine 7367/0. Probe first: bb2016 20/20.

**Remaining reds**: bb2020 {4,10,17,18,26,70,76,85} — family: agent-choice divergence on matched
hashes (Rust RSUM shows ZERO home_01/Pass candidates at seed 4 k=15 where Java declares PASS_MOVE).
bb2025 {28,77}.
