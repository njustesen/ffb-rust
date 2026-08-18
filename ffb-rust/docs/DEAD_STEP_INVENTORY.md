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

---

## Bucket (d) triaged against the DATA (2026-08-18, second pass)

The categories above were name-based guesses. This pass resolves them against what the parity teams
actually field: team specs list only `position_id`, so a skill is present only if a **drafted
position** carries it as a starting skill. (Grepping `data/teams/` for skill names proves nothing —
the specs contain no skill names at all. That mistake is why the first pass was only a hypothesis.)

Drafted-player starting skills: bb2016 57 distinct, bb2020 76, bb2025 88.

### (d1) Unreachable by DATA — no drafted player has the skill. Not bugs.

`Pro`, `Treacherous`, `Baleful Hex`, `Black Ink`, `Catch of the Day`, `Wisdom of the White Dwarf`,
`Then I Started Blastin`, `Raiding Party`, `All You Can Eat`, `Quick Bite`, `Pile Driver`,
`Furious Outburst`, `Look Into My Eyes`.

Same class as the BB2016 Kick-Team-Mate chain: the code is fine, nothing on the pitch can trigger it.
Reaching these means changing the drafted teams, which is a separate (and larger) decision.

### (d2) Reachable by data, yet the step never fires — the real targets

| skill | editions with a drafted carrier | dead steps |
|---|---|---|
| **Punt** | bb2025 | `InitPunt`, `EndPunt`, `PuntDirection`, `PuntDistance` |
| **Hit and Run** | bb2020, bb2025 | `HitAndRun` |
| **Hail Mary Pass** | bb2025 | `HailMaryPass` |
| **Cloud Burster** | bb2020, bb2025 | `CloudBurster` |
| Hypnotic Gaze | all three | `SelectGazeTarget`, `SelectGazeTargetEnd` (the `HypnoticGaze` step itself DOES run) |
| Dauntless | all three | `DauntlessMultiple` (`Dauntless` itself runs) |
| Bombardier | all three | `InitBomb`, `EndBomb`, `ResolveBomb`, `Bombardier2` (`Bombardier` itself runs) |

### Punt is the clearest next campaign — same shape as TTM and KTM

Both harnesses OFFER the action and then abort the target window, in lockstep:
- Rust `random_agent.rs`: `Some(AgentPrompt::PuntTarget { .. }) => Action::EndTurn`, with a comment
  noting Java has no INIT_PUNT handler.
- Java `ParityRunner`: `computeEligiblePlayers` adds `PlayerAction.PUNT` (BB2025, `canPunt`, ball
  carrier), but there is no `INIT_PUNT` step handler at all.

Because both abort together the matrices stay green — exactly how Throw Team-Mate and Kick Team-Mate
stayed green while never executing. A BB2025 roster fields a Punt carrier, so the mechanic is
reachable the moment both harnesses learn to drive the target window.

---

## Update 2026-08-18 — Hit and Run driven; the (d2) table needs a second column

`HitAndRun` now EXECUTES in bb2020 and bb2025 (commit 6c392fef). Both harnesses learned its move
window in lockstep — Rust answers `AgentPrompt::HitAndRun`, `ParityRunner` gained
`sendHitAndRunTarget` + a `HIT_AND_RUN` case — and the mechanic promptly exposed an engine bug that
had nothing to do with Hit and Run itself. See the commit message for the full chain; the part worth
carrying forward is this:

**`Pile Driver` was dead for the same reason and is now live too.** Java's `changeActingPlayer` keeps
an activated player in `MOVING` for the whole activation; Rust wrote `STANDING` on every stand-up.
`StepEndBlocking` gates BOTH `canMoveAfterBlock` (Hit and Run) and `canFoulAfterBlock` (Pile Driver)
on `base == MOVING`, so a single wrong state write disabled two skills at once — and the compared
state hash could not see it, because the activation-end `changeActingPlayer` reverts
`MOVING`->`STANDING` before the next comparison. **A mid-activation state that the hash cannot see is
the recurring shape of these bugs** (this is the third: the ACTIVE bit in the TTM campaign, the
`ttm_used`/`ktm_used` flags in KTM, and now the acting player's base).

### The (d2) table conflates two different things

"A drafted position carries the skill" is NOT the same as "the trigger is reachable". Punt is the
counter-example, and it invalidates the "Punt is the clearest next campaign" section above:

**Punt's plumbing is now correct and dark_elf bb2025 measures 100/100 — and `InitPunt` still
dispatches ZERO times.** Punt requires the acting player to be holding the ball at TURN START, and
both agents take their eligible-action snapshot once per turn, so a player who picks the ball up
mid-turn is never offered it. No amount of harness work reaches this; it needs an agent that
deliberately scores, which is a different tier.

Read the table as:

| skill | drafted carrier | trigger reachable under the turn-start snapshot? |
|---|---|---|
| **Hit and Run** | bb2020, bb2025 | YES — driven, 30/30/30 |
| **Punt** | bb2025 | NO — needs the carrier holding the ball at turn start |
| **Hail Mary Pass** | bb2025 | unverified |
| **Cloud Burster** | bb2020, bb2025 | unverified — needs an OPPONENT long pass, check first |
| Hypnotic Gaze | all three | unverified |
| Dauntless | all three | unverified |
| Bombardier | all three | unverified |

Verify the trigger BEFORE building harness plumbing for the remaining rows — that check is what would
have saved the Punt round-trip.
