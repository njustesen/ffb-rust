# Dead-step inventory — re-measured 2026-08-25

Supersedes the 2026-08-19 inventory (136/199), which predated the bomb chain, the star
batches (§§9-11) and the blitz-select chain (§12).

**167 of 200 `StepId` variants reached; 33 dead.**

## How this was measured — and why the obvious method is wrong

Two sweeps, unioned:

1. `FFB_DRIVE_TRACE=1 ffb-parity --uniform --all-rosters --all-editions --seeds 1-3 --no-abort`
   (Rust-only, 270 games) → 147 reached.
2. The **random** (parity-tier) agent over all 30 rosters x 3 editions x seeds 1-10, driven by a
   loop over `--home K --away K --edition E --tier 3` → 161 reached.

Union: 167. Three traps, all of which produced a wrong number first:

- **The uniform agent under-reports star mechanics.** It references the star specials 3 times
  against `random_agent`'s 22 - it does not *declare* most of them, so they cannot dispatch no
  matter what is on the pitch (the sweep had 261 star references). A uniform-only sweep is not an
  inventory.
- **`--all-rosters` / `--all-editions` are silently ignored without `--uniform`.** A run that looks
  like a full sweep quietly measures lineman-vs-lineman only.
- **Seed count dominates the result.** `CatchOfTheDay` fires **21 times in 30 seeds** of wood_elf
  bb2025 and **zero** times in seeds 1-2. Going from 2 to 10 seeds moved 11 steps out of "dead".
  10 seeds is the floor used here and is still not proof of death for a rare mechanic - see
  category E.

**A "dead" list is only as good as its sampling.** Verify per-mechanic on the roster that carries
it before concluding anything is unreachable.

## The 33, classified

### A. Vestigial in BOTH engines — not parity gaps (6)
`EndPlayerAction` `NoOp` `RevertEndTurn` `Bombardier2` `SelectGazeTarget` `SelectGazeTargetEnd`

No Java **generator** pushes any of these (checked against
`ffb-server/.../step/generator/`). In Rust, `StepId::EndPlayerAction` is referenced only from
`step/sequences.rs` and `step/engine.rs` - the pre-driver legacy modules; the live
`EndPlayerAction` generator pushes a sequence of *other* steps and never itself. The gaze twins
were closed on 2026-08-18: `canGazeDuringMove` is registered only by `skill/bb2016/HypnoticGaze`,
so no BB2020 player can declare GAZE.

These are dead code in Java too. Deleting them is a cleanup question, not a fidelity one.

### B. Unreachable by DATA (6)
`InitKickTeamMate` `KickTeamMate` `EndKickTeamMate` `KickTeamMateDoubleRolled`
`PileDriver` `DropActingPlayer`

- The KTM sequence generator is **BB2016-only** in Java (`generator/bb2016/KickTeamMate.java`);
  BB2020/BB2025 route kicks through the shared TTM steps. **No bb2016 roster carries
  "Kick Team-Mate"** (only the bb2020/bb2025 ogre Runt Punter does), so the BB2016 twins cannot
  run. The MECHANIC is live: ogre bb2025 dispatches `ThrowTeamMate` 364 times over 30 seeds and
  the agent declares KICK_TEAM_MATE. This is **not** a regression of commit `60131597`.
- `PileDriver` appears nowhere in `data/` under either name ("Pile Driver" or the older
  "Piling On") - no roster, no star, no drafted team. `DropActingPlayer` is pushed by exactly one
  Java generator, `mixed/PileDriver.java`, so it is the same item.

Reaching either means changing drafted team data - a separate and larger decision (same category
as `DauntlessMultiple`, closed 2026-08-18).

### C. Scoring-gated — BLOCKED on a user tier decision (5)
`AssignTouchdowns` `InitPunt` `EndPunt` `PuntDirection` `PuntDistance`

Out of scope until the agent is allowed to score. Unchanged.

### D. Inducements — parity teams purchase none (6)
`MasterChef` `WeatherMage` `Wizard` `PlayCard` `FanFactor` `PrayerRoll`

The existing "inducement purchasing (7 ids)" backlog item. Needs both harnesses taught to buy
inducements in lockstep - the same shape as the TTM/KTM/interception/bomb campaigns.

### E. Star specials — live in §§9-11, not sampled at 10 seeds (6)
`Treacherous` `LookIntoMyEyes` `InitLookIntoMyEyes` `Swoop` `ThrowARock` `QuickBite`

Recorded LIVE 100/100 in the star campaigns. `CatchOfTheDay` was in this list at 2 seeds and left
it at 10, which is direct evidence the category is a sampling artifact rather than a set of gaps.
**Verify each on its own star's roster with 30+ seeds before treating any as dead.**

### F. Unclassified — the actual new frontier (4)
`EatTeamMate` `ReportStabInjury` `StateMultipleRolls` `KickoffScatterRollAskAfter`

`EatTeamMate` has 8 Rust push/reference sites (vampire feeding); `ReportStabInjury` and
`StateMultipleRolls` are multiblock leftovers from §10, whose family otherwise reached at 10
seeds. These four are the only entries here that are neither explained nor blocked, and are where
the next dead-step iteration should start.

## Summary

Of the 33, **17 are closed** (A vestigial + B data-blocked + C blocked-on-decision), **6 are
almost certainly sampling artifacts** (E), **6 are one known campaign** (D inducements), and
**4 are genuinely open** (F).
