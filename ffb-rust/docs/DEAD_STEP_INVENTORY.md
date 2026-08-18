# Never-dispatched step inventory (2026-08-18)

Method: extract every `StepId` variant (199), then collect every `step=` line from
`FFB_DRIVE_TRACE=1` over two independent agents, and subtract.

- **Uniform agent** (`--uniform --all-rosters --all-editions`, 87 matchups x 3 seeds, Rust-only,
  ~5.7M dispatch lines) — the tool built to maximise mechanic coverage.
- **Parity agent** (the random agent the matrices gate on): ogre bb2020 and goblin bb2025, 3 seeds
  each — chosen because they carry Throw/Kick Team-Mate and Swoop.

| | count |
|---|---|
| StepId variants | 199 |
| reached by the uniform agent | 121 -> **130** after the Finding-1 fix |
| reached by the parity agent (2 matchups only) | 117 |
| reached by either | 130 |
| **never reached by either** | **69** |

## Finding 1 — the coverage tool had a blind spot (FIXED)

Nine steps are reached by the PARITY agent but never by the UNIFORM agent, even though the uniform
sweep covers all 87 matchups including ogre:

    AlwaysHungry  DispatchScatterPlayer  EndScatterPlayer  EndThrowTeamMate
    InitScatterPlayer  InitThrowTeamMate  RightStuff  Swoop  ThrowTeamMate

That is the entire Throw-Team-Mate family. `step=ThrowTeamMate` appeared **zero times** in 5.7M
uniform dispatch lines. The uniform agent exists precisely to answer "how much of the mechanic
surface does random play exercise?", so a whole action it never declared undercut every coverage
number it reported.

ROOT CAUSE: `uniform_agent.rs`'s target-selection `match` had arms for Block/Blitz, Foul, HandOff and
Pass, then `_ => None` — no arm for Throw/Kick Team-Mate. The declaration therefore went out with no
thrown player, `StepInitSelecting` deselected it, and the action never resolved. Exactly the shape of
the Kick Team-Mate bug in the parity agent found the same day: an agent declaring an action it never
supplies a target for.

FIXED — the uniform agent now picks a thrown player from `legal_throw_team_mate_targets`, the same
primitive the parity agent uses. Re-measured over the same sweep: the uniform agent now reaches
**130 of 199** step-ids, up from 121, gaining eight of those nine
(`AlwaysHungry`, `DispatchScatterPlayer`, `EndScatterPlayer`, `EndThrowTeamMate`,
`InitScatterPlayer`, `InitThrowTeamMate`, `RightStuff`, `ThrowTeamMate`). The sweep alone now matches
what previously took both agents together.

`Swoop` remains unreached by the uniform sweep at 3 seeds/matchup — it needs a kicked Doom Diver in
BB2016/BB2020 specifically, so this may be seed depth rather than a second gap. Not yet verified.

## Finding 2 — 69 steps no agent reaches

Not all of these are bugs; they need triage into four buckets. The categories below are
**hypotheses from the names**, not verified reachability claims — confirm each before acting.

**(a) Framework pseudo-steps — expected, not mechanics.**
`NextStep`, `NoOp`, `ConsumeParameter`, `SetActingPlayerAndTeam`, `RevertEndTurn`,
`StateMultipleRolls`, `MultipleBlockFork`, `SpecialEffect`.

**(b) Needs inducements, cards or a wizard — the parity teams buy none.**
`PlayCard`, `BuyCards`-adjacent leaves, `Wizard`, `WeatherMage`, `MasterChef`, `RaidingParty`,
`ThrowARock`, `ThrowKeg`/`EndThrowKeg`, `InitBomb`/`EndBomb`/`ResolveBomb`/`Bombardier2`,
`RecheckExplodeSkill`.

**(c) Needs a touchdown — the agent never scores.**
`AssignTouchdowns`. This is the same gap the "make the agent score" tier targets.

**(d) Reachable mechanics that simply never fire — the richest bucket, and the same shape as the
two campaigns that just finished (BB2020 Throw Team-Mate, Kick Team-Mate).**
`Pro` (a common re-roll skill), `HitAndRun`, `HailMaryPass`, `DauntlessMultiple`,
`FoulAppearanceMultiple`, `BlockRollMultiple`, `ApothecaryMultiple`, `SelectBlitzTarget`,
`SelectBlitzTargetEnd`, `SelectGazeTarget`, `SelectGazeTargetEnd`, `LookIntoMyEyes`,
`InitLookIntoMyEyes`, `AutoGazeZoat`, `BalefulHex`, `BlackInk`, `CatchOfTheDay`, `CloudBurster`,
`Treacherous`, `WisdomOfTheWhiteDwarf`, `ThenIStartedBlastin`/`EndThenIStartedBlastin`,
`QuickBite`, `AllYouCanEat`, `EatTeamMate`, `PileDriver`, `DoubleStrength`, `DropActingPlayer`,
`DispatchDumpOff`, `FumbleTtmPass`, `PrayerRoll`, `FanFactor`, `EndPlayerAction`,
`KickoffScatterRollAskAfter`, `PuntDirection`/`PuntDistance`/`InitPunt`/`EndPunt`,
`FirstMoveFuriousOutburst`/`SecondMoveFuriousOutburst`/`InitFuriousOutburst`/`EndFuriousOutburst`,
`ReportStabInjury`.

**Known-good exception:** `InitKickTeamMate`, `KickTeamMate`, `KickTeamMateDoubleRolled` and
`EndKickTeamMate` are the **BB2016** Kick Team-Mate chain (its generator exists only at
`generator/bb2016/KickTeamMate.java`). No bb2016 roster carries the skill, so they are unreachable
by data, not by bug — BB2020/BB2025 kicks go through the ThrowTeamMate sequence and ARE exercised
(see the Kick Team-Mate campaign in PARITY_TTM.md).

## Why this matters

Both campaigns finished today started from exactly this question and each yielded six to ten real
engine bugs. A green matrix can mean the harness never exercised the mechanic at all — the 69 steps
above are where that is provably true today.
