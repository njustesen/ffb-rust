# khemri_fumbbl — heuristic-agent parity campaign

**Goal**: khemri_fumbbl v khemri_fumbbl, HeuristicAgent both sides, per-step state-hash parity
100/100, seeds 1-100, tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus
random controls. Procedure: `.claude/commands/amz-iter.md` with `MATCHUP=khemri_fumbbl`.
Started 2026-09-05, after khemri (`b9693dcaa`).

## Surface

The FUMBBL "Tomb Kings" league import (roster id **55051**), 12 players. Same skill surface as
`khemri` — Regeneration on every player, Decay on the Tomb Guardians, Thick Skull, Block, Pass,
Sure Hands — with one difference that matters for the campaign's open items: the FUMBBL Tomb
Guardian carries **only Decay + Regeneration, no Brawler**. BACKLOG §E8 therefore does not apply to
this variant.

**The FUMBBL roster-alias trap was checked FIRST, before any green was trusted.** These league
imports carry numeric position ids (`2892`…`2895`) and generic names ("Skeleton Lineman"), matching
neither the CLI key nor the bb2025 canonical name, and `make_team` once fell back silently to an
all-lineman AG3 team that diverged from Java's real roster at the first dodge. The alias is wired
(`runner.rs:1041`, `"khemri_fumbbl" => "55051"`) and its regression tests pass:
`fumbbl_khemri_and_slann_build_real_rosters` plus the four others, 5/5 green. The roster loads as
Tomb Kings with the real Anointed Thrower / Skeleton Lineman / Tomb Guardian / Anointed Blitzer
positions, not the fallback.

## Baseline (2026-09-05, measured on `beb1e0873`, nine gates, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** | **100** |
| bb2025 | **100** | **100** | **100** |

**All nine gates green at baseline, zero reds, no engine change** — the second race in a row to open
fully closed, for the same structural reason as khemri: no negatraits, no Throw Team-Mate, and an
injury/Regeneration surface already hardened by earlier campaigns.

Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`): bb2016 **100/100**,
bb2020 **100/100**, bb2025 **100/100**.

## Not vacuous

bb2025 @1.0 `rust_total=40.9s` (khemri 38.3s, human 40.3s) and **94,603 GameEvents** over the 100
seeds: 2,273 injuries, 1,569 blocks, **151 regenerationRoll**, 199 pass rolls, 107 hand-offs,
21 touchdowns. The race's headline skill fires and the ball moves.

Note the FUMBBL variant scores markedly more than stock khemri — 21 TDs vs 6, 107 hand-offs vs 58 —
which is the different position mix, not a different engine path.

## Coverage

Harvested ×3 → `docs/EVENT_COVERAGE_khemri_fumbbl_bb2016.md`, `_bb2020.md`, `_bb2025.md`.
`skillUse` is **zero**, and that is correct rather than a gap: the five emitting sites are
block-result Dodge, Dump Off, Horns, Juggernaut and Wrestle, and this roster carries none of them
(BACKLOG §E6b/§E6c). Regeneration, Decay, Thick Skull, Block, Pass and Sure Hands are all
exercised-but-unevented; parity plus the injury/Regeneration event counts are the proof they run.

**🏁 khemri_fumbbl CLOSED at baseline.** Frontier empty; no carry-over of its own (§E8 does not
apply — no Brawler on this roster).
