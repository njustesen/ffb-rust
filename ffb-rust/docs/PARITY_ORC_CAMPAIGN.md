# Orc — heuristic-agent parity campaign

**🏁 CLOSED 2026-09-06 at BASELINE (ITER1).** Nine gates 100/100 and three random controls 100/100
with **no engine change**. Measured on `8fbcfabdd` (the commit that closed ogre). Started after
norse + ogre; alphabetically the next unstarted race in the sweep.

## Surface — what orc actually carries (the briefing was wrong)

The iteration brief predicted "Block, Grab, Dirty Player, Right Stuff, Stunty, and (bb2025) Brawler
on the Black Orc". Read from `data/rosters/*/roster_orc.json`, the real surface is:

| position | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Lineman | — | Animosity | — |
| Thrower | Pass, Sure Hands | Animosity, Pass, Sure Hands | Pass, Sure Hands |
| Blitzer | Block | Animosity, Block | Block, **Break Tackle** |
| Black Orc / Big Un | — (`orc.blackorc`) | Animosity (`orc.bigun`) | Mighty Blow, **Taunt**, Thick Skull, **Unsteady** |
| Goblin | Dodge, Right Stuff, Stunty | same | same |
| **Troll** | Always Hungry, Loner, Mighty Blow, Really Stupid, Regeneration, **Throw Team-Mate** | + **Projectile Vomit** | + Projectile Vomit |

**There is no Grab, no Dirty Player and no Brawler on any orc roster in any edition.**
`grep -rn Brawler data/rosters/` returns exactly three hits — bb2020 chaos_dwarf, bb2025 chaos_dwarf,
bb2025 khemri — so BACKLOG **§E8** ("khemri's Tomb Guardian is the only Brawler in the sweep") is
*still accurate for bb2025 except for chaos_dwarf*, and orc does not touch it. Orc therefore produces
**no Brawler parity evidence**, exactly as the brief anticipated, but for the simpler reason that the
skill is not on the roster at all — not because the agents cannot reach it. E8 stands unchanged.

What orc *does* bring that is worth the run: a **Troll** in all three editions — Really Stupid
(negatrait), Always Hungry + Throw Team-Mate (the throw chain), Regeneration, Loner — throwing
**Right Stuff / Stunty goblins**. That is the TTM surface that cost ten engine bugs on
`parity_tier_ttm.md`, and bb2020 adds Animosity on four positions.

## Baseline = final (seeds 1-100, tier 3, `--agent heuristic --heur-classes all`)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100/100** | **100/100** | **100/100** |
| bb2020 | **100/100** | **100/100** | **100/100** |
| bb2025 | **100/100** | **100/100** | **100/100** |

Random controls (`FFB_PARITY_ROOT=parity_random --agent random`, seeds 1-100):
bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.

`cargo test -p ffb-engine`: **7424 passed, 0 failed**, 15 ignored.

Timing (batched JVM, 100 seeds): bb2016 `rust_total=` 28.97 / 58.11 / 32.02 s at @1.0 / @0 / @1e6;
bb2020 35.26 / 63.66 / 27.75 s; bb2025 39.01 / 65.00 / 28.48 s.

## Regressions (bb2025 @1.0, seeds 1-100, all **100/100**)

nurgle, human, goblin, dwarf, amazon, chaos, necromantic, nippon, lizardman, khemri, norse, ogre.
No engine code changed for this race, so these are a confirmation that the tree is where ogre left
it rather than a test of an orc fix.

## Not vacuous — coverage proves the Troll paths ran

Harvested ×3 with `MATCHUP=orc sh scripts/harvest_coverage.sh <edition> 1.0`, each run alone
(`docs/EVENT_COVERAGE_orc_bb2016.md` / `_bb2020.md` / `_bb2025.md`).

Player actions declared, bb2016 / bb2020 / bb2025:
**ThrowTeamMate 304 / 268 / 279**, Foul 255 / 268 / 263, PassMove 175 / 164 / 159,
HandOverMove 115 / 71 / 95.

bb2025 GameEvent catalog (94,610 events over 100 games) includes
**confusionRoll 1583** (Really Stupid), **alwaysHungry 20**, **throwTeamMateRoll 17**,
**rightStuffRoll 14**, **regenerationRoll 3**, dodgeRoll 832, `skillUse` 38 (all Dodge).
bb2016 records 2 Dodge skillUses and bb2020 47.

So the Troll's whole negatrait → Always Hungry → TTM → Right Stuff landing chain executed in every
edition, in both engines, with identical per-step state hashes.

## Process note (recorded because it is a rule violation, not a finding)

While launching the regression batches I briefly started **two bb2025 nurgle-vs-nurgle runs at the
same time** — the campaign's "never two runs of one edition+matchup at once" rule. I killed the
duplicate within seconds. The surviving nurgle run reported 100/100 and nurgle is a known-green
race, so the number is not load-bearing; recorded here rather than quietly dropped.

## What was NOT verified

- No `--agent random` run at scales other than default, and no bb2016/bb2020 regression sweep of the
  closed roster set (only bb2025 @1.0) — justified only by there being **zero code change** this
  iteration.
- Projectile Vomit, Taunt, Unsteady, Break Tackle and Animosity emit no `skillUse` event, so the
  coverage harvest gives **no positive evidence** that any of them was exercised. Parity is genuine
  either way (both engines agree), but "green" is not "exercised" for these five.

**🏁 orc CLOSED. Frontier empty.** Next new race alphabetically: **renegades**
(then skaven, slann, slann_fumbbl, underworld, wood_elf; undead and vampire are out of this sweep).
