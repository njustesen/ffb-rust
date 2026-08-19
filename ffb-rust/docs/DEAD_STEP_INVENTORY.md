# Never-dispatched step inventory — RE-MEASURED 2026-08-19

Method identical to the 2026-08-18 sweep below: `FFB_DRIVE_TRACE=1 --uniform --all-rosters
--all-editions --seeds 1-3 --no-abort` (261 games, Rust-only), `DRIVE step=` lines vs the 199
`StepId` variants. Run AFTER the bomb-chain campaign (`7fa7b7ad`) and the ACTIVE-bit hash
(`04aef090`), and after fixing two uniform-agent gaps (a THROW_BOMB declaration with no receiver
— the exact TTM bug shape from Finding 1 below — and a BombRethrow answer of EndTurn, which the
`thrower==null` early return can never accept).

| | 2026-08-18 | **2026-08-19** |
|---|---|---|
| StepId variants | 199 | 199 |
| reached by the uniform sweep | 130 | **136** |
| never reached | 69 | **63** |

Newly reached: the ENTIRE bomb family (`Bombardier`, `InitBomb`, `ResolveBomb`, `EndBomb`,
`SpecialEffect`, `RecheckExplodeSkill`) plus `Intercept` and `SafeThrow` — all made live by the
bomb/interception campaigns. Headline from the parity matrices: **1,238 THROW_BOMB activations
per 300 goblin games** (three editions × 100 seeds) where every prior matrix had exactly zero.

## The 63 still-dead steps, classified

**Needs a SCORING agent (the standing blocked tier):**
`AssignTouchdowns InitPunt EndPunt PuntDirection PuntDistance` — Punt needs a turn-start ball
carrier; AssignTouchdowns needs a touchdown.

**Uniform-agent gaps (same shape as the TTM/bomb findings — the agent never *declares* it):**
`HitAndRun InitKickTeamMate EndKickTeamMate KickTeamMate KickTeamMateDoubleRolled` — Hit-and-Run
and Kick Team-Mate are LIVE, parity-verified mechanics under the parity agent; the uniform agent
does not drive them. Candidate quick wins if the uniform tool's coverage number matters.

**Needs a specific star/skill/inducement the parity teams do not draft:**
`AllYouCanEat AutoGazeZoat BalefulHex BlackInk CatchOfTheDay DispatchDumpOff DoubleStrength
EatTeamMate HailMaryPass LookIntoMyEyes InitLookIntoMyEyes PileDriver QuickBite RaidingParty
ThenIStartedBlastin EndThenIStartedBlastin ThrowARock ThrowKeg EndThrowKeg Treacherous
WeatherMage WisdomOfTheWhiteDwarf Pro` — star-player specials, Dump Off, Hail Mary, Pro, kegs,
cards. Unreachable until the drafted teams change.

**Needs the Multiple Block skill (no parity roster drafts it):**
`MultipleBlockFork BlockRollMultiple FoulAppearanceMultiple ApothecaryMultiple
DauntlessMultiple StateMultipleRolls ReportStabInjury`

**Needs an inducement/card/prayer path the harness declines:**
`PlayCard Wizard MasterChef FanFactor PrayerRoll` (the harness buys no cards/wizards; FanFactor
is a bb2016 start-step variant the shared start path does not dispatch).

**Furious Outburst family (bb2025 star):**
`InitFuriousOutburst FirstMoveFuriousOutburst SecondMoveFuriousOutburst EndFuriousOutburst`

**Blitz/gaze SELECT sub-chain (edition/protocol shape — the folded agents bypass it):**
`SelectBlitzTarget SelectBlitzTargetEnd SelectGazeTarget SelectGazeTargetEnd` — the harnesses
declare blitzes as BLITZ (folded target), so the BLITZ_MOVE→BLITZ_SELECT dialog chain never runs;
bb2020 gaze equivalents likewise (the parity path uses CLIENT_GAZE / GazeSelect instead).

**Plumbing/no-op ids the driver never dispatches by that name:**
`ConsumeParameter DropActingPlayer EndPlayerAction NoOp RevertEndTurn SetActingPlayerAndTeam
KickoffScatterRollAskAfter Bombardier2` — sequence-internal or superseded ids (EndPlayerAction is
a GENERATOR name; its sequence dispatches EndTurn/other ids).

**Known seed-depth stragglers:** `Swoop` (live and parity-verified at deeper seeds; 3-seed sweep
misses it), `CloudBurster` (live in bb2020 interceptions; needs a Cloud Burster carrier passing
long — see the interception campaign).

---

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

`Swoop` remains unreached by the uniform sweep at 3 seeds/matchup — **VERIFIED 2026-08-18 as seed
depth, not a second gap.** Under the PARITY agent it dispatches on goblin bb2020 at seeds 1-20
(`Swoop: 1`, matchup 20/20 green), so the step is live and parity-checked. Two corrections to the
sentence this replaces: it does NOT need a *kicked* Doom Diver — that run had `KickTeamMate: 0` and
`ThrowTeamMate: 13`, so a THROWN Doom Diver produced it — and it is no longer "not yet verified".

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

---

## Update 2026-08-18 (2) — Cloud Burster: plumbing correct, mechanic still blocked

`CloudBurster` is **not** driven. Correcting the (d2) table again: it listed Cloud Burster for
**bb2025**, but Java has no `bb2025/CloudBursterBehaviour` at all — only bb2020 registers a step for
the PASS_INTERCEPT hook point, so there is nothing to drive in bb2025. Only `high_elf` carries the
skill (`highelf.thrower`, one drafted in the bb2020 parity team).

| skill | drafted carrier | trigger reachable? |
|---|---|---|
| **Cloud Burster** | bb2020 only (NOT bb2025 — no behaviour exists) | plumbing correct; blocked on BB2020 deflection fidelity |

**What now works:** the PASS_INTERCEPT hook splices correctly per edition (it had been hard-coded to
`Rules::Bb2025` inside the SHARED generator, making the call an unconditional no-op), and both
harnesses *can* drive the interception window — candidates from the engine's own
`UtilPassing.findInterceptors`, coordinate-sorted, one `actionRng` draw each.

**Why it is off:** switching the attempt on exposed eight Rust fidelity bugs. Seven are fixed
(commit 4677499b). The eighth is the BB2020 **deflection** chain below `StepResolvePass`: a deflected
ball that Java leaves on the deflector ends on the receiver in Rust. Until that is ported, the attempt
is reverted in both harnesses — see `docs/BACKLOG.md` §2 for the diagnosis (dark_elf bb2020 seed 21,
narrowed to the `Deflected` arm of `bb2025/shared/step_catch_scatter_throw_in.rs`).

**Routing note that cost real time twice this session:** `driver.rs` globs
`use crate::step::bb2025::pass::*` and `use crate::step::bb2025::shared::*`, so the **bb2016 AND
bb2020 pass twins are DEAD** — every edition runs the BB2025 steps. Always check the glob imports in
`driver.rs` before probing a per-edition file; a probe added to `step/bb2016/pass/step_intercept.rs`
produced no output and was briefly credited with a fix it could not have caused.

### The five recurring shapes

Every target in this tier has been one of these:

1. A **per-edition rule hard-coded to one edition inside a SHARED file** — five instances now.
2. **Both harnesses declining the same dialog in lockstep**, keeping the matrices green while the
   mechanic underneath is dead.
3. A step returning a bare `cont()` with **no prompt** — breaks only once the other harness answers.
4. A **general Java rule simplified away** in Rust (`isSkillRollSuccessful`'s natural 6 / natural 1),
   invisible until a mechanic that can produce an out-of-range target starts running.
5. A **Java predicate re-implemented with a missing or an extra clause** (`preventCatch`; the extra
   thrower/target-square exclusion), harmless until the mechanic that uses it starts running.


---

## Update 2026-08-18 (3) — Cloud Burster is DRIVEN; interception is ON in both harnesses

**Supersedes Update (2) above**, which recorded Cloud Burster as "plumbing correct, blocked on BB2020
deflection fidelity". That blocker is gone. Agent-driven interception is now ENABLED in both
harnesses — each picks a COORDINATE-sorted candidate (never id-sorted; the two engines' player ids
differ) from the ENGINE's own `UtilPassing.findInterceptors`, with exactly one `actionRng` draw.

| skill | drafted carrier | trigger reachable? |
|---|---|---|
| **Cloud Burster** | bb2020 only (bb2025 has no `CloudBursterBehaviour` at all) | **YES — driven** |

Switching interception on exposed **twelve** Rust fidelity bugs in total. The seven shipped in
`4677499b` are listed in Update (2); the five that finished the job:

| # | Fix | Commit |
|---|-----|--------|
| 8 | `StepResolvePass` gates the ball-to-interceptor branch on the PER-EDITION success flag (BB2020 `isDeflectionSuccessful`, BB2016/BB2025 `isInterceptionSuccessful`) — never on `interceptor_id.is_some()` | `66607f9b` |
| 9 | The BB2020 deflected branch clears `CatcherId`, so the deflected catch resolves for the DEFLECTOR under the ball rather than the intended receiver | `494c68a0` |
| 10 | A FAILED interception is re-rolled from a SKILL source on the interceptor, recursing into `intercept()`; the lookup key is per-edition (BB2016 `CATCH`, BB2020/BB2025 `INTERCEPTION`) | `ef647683` |
| 11 | A successful Safe Throw clears the interception success flags as well as `InterceptorId` | `01da521e` |

### The parameter-outlives-its-sequence family — four instances

Java keeps pass state in a `PassState` object that is **re-created for each pass**. Rust threads the
same values as published step parameters, which **persist past the sequence that published them**.
Every one of these was the same bug wearing a different hat: `InterceptorId` surviving into a later
pass, `CatcherId` leaking into a deflected catch, the deflection flags outliving their pass, and the
Safe Throw flags not being cleared when the interception was cancelled.

**When adding any new published flag, ask what clears it.** That question would have caught all four.

### Technique note

`FFB_DICE_DEEP=1` makes Java print the FULL caller chain for every die. When two dice share a
`rollSkill:112` frame — as the pass, interception, Safe Throw and catch rolls all do — never infer
which is which from the values. It identified the culprit in one run for three consecutive bugs, and
one inference-based theory ("missing tacklezone modifiers") was flatly wrong and nearly produced a
bad fix to working code.

Also: `driver.rs` has a PER-EDITION OVERRIDE BLOCK (~line 434) that takes precedence over its glob
imports — BB2016 runs its own `StepCatchScatterThrowIn`. Checking the globs alone is not enough;
three probes went into dead files this session before that was understood.

---

## Update 2026-08-18 (4) — the gaze-target steps are vestigial in Java

`SelectGazeTarget` / `SelectGazeTargetEnd` were listed as "(d2) reachable by data, yet never fires,
all editions". Both halves of that were wrong.

**They are BB2020-only** — the Java steps are `@RulesCollection(BB2020)` and are pushed solely from
`bb2020/move/StepEndSelecting`. BB2025 has no gaze-target path at all (only `AutoGazeZoat`), and
BB2016 uses `bb2016/move/StepHypnoticGaze`, which already runs.

**They are unreachable in Java itself.** The push happens on `PlayerAction.GAZE_SELECT`, which
`bb2020/shared/StepInitSelecting:113` produces only from a client `GAZE_MOVE` declaration. A client
offers GAZE only when the player has `canGazeDuringMove` (`MoveLogicModule:362`; `ParityRunner:1995`
uses the same property) — and that property is registered **only by `skill/bb2016/HypnoticGaze`**.
bb2020's and bb2025's HypnoticGaze register just `inflictsConfusion`. So no BB2020 player can declare
GAZE, and the BB2020-only gaze-target steps can never run.

A Rust routing bug was fixed along the way (`a7e7da8f`): the push existed only in the dead
`step/bb2020/move_/step_end_selecting.rs`, so it would not have fired even if the action existed. That
fix is faithful but will never be exercised — recorded here so nobody re-measures it hoping otherwise.

**Method note.** This is the second target closed by checking the trigger before building plumbing
(after Punt). The check cost minutes; the alternative was teaching both harnesses to declare an action
Java's own eligibility rules never offer, i.e. fabricating behaviour and then "fixing" the engine until
the fabrication matched.
