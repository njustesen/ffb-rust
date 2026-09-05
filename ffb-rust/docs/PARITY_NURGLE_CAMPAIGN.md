# Nurgle — heuristic-agent parity campaign

**🏁 CLOSED 2026-09-05 at BASELINE.** Nine gates 100/100 and three random controls 100/100 with no
engine change. Started after necromantic (`e025e6f0d`).

## Surface — the richest roster in the sweep

- **Rotter**: Decay, Plague Ridden
- **Pestigor**: Horns, Plague Ridden, Regeneration, Steady Footing, Thick Skull
- **Nurgle Warrior**: Disturbing Presence, **Foul Appearance**, Plague Ridden, Regeneration,
  Stand Firm, Unsteady
- **Beast of Nurgle**: Disturbing Presence, **Foul Appearance**, Loner 4, Mighty Blow, Pick-me-up,
  Plague Ridden, **Really Stupid**, Regeneration, Tentacles

That is a negatrait (Really Stupid), two Foul Appearance carriers, Tentacles, Stand Firm, Decay,
Regeneration ×3 positions and Horns — several of which have been the source of fixes on other races.

## Baseline (measured on `e025e6f0d`, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** | **100** |
| bb2025 | **100** | **100** | **100** |

Random controls: bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.

## Why this one matters more than a clean baseline usually does

nurgle is the race that most heavily exercises **Foul Appearance**, and it was measured on the
binary carrying the fix that closed necromantic hours earlier — the one that stopped
`StepFoulAppearance` from rolling against a pass RECEIVER (see `PARITY_NECROMANTIC_CAMPAIGN.md`).
Two of nurgle's four positions carry the skill, so a green nine-gate here is a genuine independent
check on that fix rather than a race that happens to avoid the code.

Not vacuous either: coverage harvested ×3 shows **Horns firing 91 / 59 / 71 times** across
bb2016 / bb2020 / bb2025.

Since no code changed for this race, the closed-roster regression set is satisfied by construction.

**🏁 nurgle CLOSED.** Frontier empty.
