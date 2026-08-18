# Parity backlog

Ordered work queue for the "find unported / never-run Java code" tier. Worked **one item at a time**,
top to bottom. Each item states its own done-condition. Update this file as items complete — tick the
box, add the commit hash, and record anything learned that the next item needs.

**Standing rules for every item**

- Root-cause in the **Rust** engine. `ffb-common` / `ffb-server` Java is engine code and off-limits;
  `ffb-ai/ParityRunner.java` is co-editable harness.
- No hacks. A regression test per fix.
- Commit ONLY at `bb2016 30/30, bb2020 30/30, bb2025 30/30`:
  `python scripts/run_team_matrix.py --edition all --seeds 1-100 --parallel 3`
- Limited CPU: `--parallel 3`, `PARITY_JVM_CORES=1`, never 8. Never build or run while a gate is active.
- `--seeds N` means 1..N — write `N-N` for a single seed.
- Remove every probe before gating, then `git diff` the file and read **every** `-` line.

---

## 1. Land the interception fidelity fixes — ✅ DONE 2026-08-18

Gate `gate5` came back **bb2016 30/30, bb2020 30/30, bb2025 30/30, zero reds**. Workspace 14,536/0.

- [x] Read the gate result — 30/30/30.
- [x] Commit the seven fixes — `4677499b`.
- [x] `docs/DEAD_STEP_INVENTORY.md` updated — `0ac9ed89`. Cloud Burster recorded as "plumbing correct,
      blocked on BB2020 deflection fidelity"; dropped from the bb2025 row; `driver.rs` glob routing
      written down (bb2016/bb2020 pass twins are dead).
- [x] Memory updated (`parity_tier_ttm.md`, `MEMORY.md`).

**For the next item:** nothing is in flight, the tree is clean, and everything is pushed
(`e2c646c1..c7b2e158`). The seven fixes are in; the eighth bug (BB2020 deflection) is §2 below.

**The seven fixes in this commit**

| # | Fix |
|---|-----|
| 1 | `generator/bb2025/pass.rs` splices the PASS_INTERCEPT hook by `params.rules`, not a hard-coded `Rules::Bb2025` (BB2016 → SafeThrow, BB2020 → CloudBurster, BB2025 → none) |
| 3 | BB2020 deflection semantics inside the shared `step_intercept.rs` / `step_resolve_pass.rs` |
| 5 | Per-edition interception minimum roll — BB2016 `max(2, 7 − min(AG,6) + 2 + mods)` vs BB2020/25 AG-based |
| 6 | `DiceInterpreter.isSkillRollSuccessful` — natural 6 always succeeds, natural 1 always fails |
| 7 | Java's `preventCatch` filter in `find_interceptors`, and removal of Rust's extra thrower-square / target-square exclusion |
| 8 | `step_resolve_pass.rs` gates on the per-edition SUCCESS FLAG, never on `interceptor_id.is_some()`; `StepIntercept` clears `InterceptorId` on failure |

---

## 2. BB2020 deflection fidelity, then re-enable interception — ✅ DONE 2026-08-18

The only thing between this campaign and a live Cloud Burster. Diagnosed down to one function.

**Order correction (2026-08-18):** the boxes were originally written fix-then-re-enable, but the fix
cannot be MEASURED while the interception attempt is off — no interception means no deflection means
the `Deflected` arm never runs. Re-enable FIRST (the tree returns to nine known reds, which is the
expected intermediate state), then fix, then green the seeds. Do not gate or commit in between.

- [x] **Re-enabled 2026-08-18, uncommitted.** Rust `AgentPrompt::Interception` arm in
      `random_agent.rs` answers `SelectPlayer`; `ParityRunner.java` has `case INTERCEPTION:` +
      `sendInterceptorChoice(Game, GameState)`; jar rebuilt. Coordinate-sorted candidates from the
      engine's own `UtilPassing.findInterceptors`, one `actionRng` draw each. The decline test was
      swapped back for `interception_picks_a_coordinate_sorted_candidate_with_one_action_draw` +
      `interception_with_no_candidates_declines` (22 agent tests pass).
      **Correction:** the note above said to recover the code from `4677499b`'s parent — that is
      WRONG, the enabled version was never committed (its parent `e2c646c1` holds the *old* decline).
      It was rewritten from the spec. Verified back in the expected intermediate state:
      dark_elf bb2020 seed 21 is RED again. Do NOT gate or commit until the next box lands.
- [x] **Fixed 2026-08-18, uncommitted — and it was NOT in the `Deflected` arm.** That arm is faithful;
      the bug was upstream. Java's `StepPass.setParameter` returns TRUE for `CATCHER_ID`, i.e. it
      CONSUMES the key into `PassState.catcherId` so it never reaches `StepCatchScatterThrowIn`.
      Rust's `bb2025/pass/step_pass.rs` accepted the key without consuming it, so the intended
      receiver leaked downstream and `if catcher_id.is_none() { catcher_id = player_under_ball }`
      never fired — the deflected catch resolved for the RECEIVER instead of the deflector under the
      ball. Probe confirmed it exactly: `catcher_id=away_09` vs `under_ball=home_02`. Fix is a
      `consumes_parameter` override on StepPass; test
      This is the THIRD instance of the parameter-outlives-its-sequence shape (after `InterceptorId`
      and the pass-state ids), so treat "does Java's setParameter return true here?" as a standard check.
      **Second correction:** the first attempt made `StepPass` CONSUME `CatcherId` (mirroring Java's
      `setParameter` returning true). That over-reached and REGRESSED necromantic bb2020 seed 8 to a
      new red at step 5 — in Java the later steps read `catcherId` off the shared `PassState`, but
      Rust's `StepEndPassing` reads the PARAMETER, so consuming it at StepPass starves them. The
      shipped fix instead publishes `CatcherId(None)` from the BB2020 deflected branch of
      `step_resolve_pass.rs`, alongside the `Deflected` mode. Strictly better: it greens necromantic
      41 as well and causes no regression.
- [x] **All six original blocking seeds are GREEN.** The last of them needed one more fix: Java
      re-rolls a FAILED interception from a SKILL source on the interceptor, recursing into
      `intercept()` (`bb2016:182-196`, `bb2025:234-249`). The lookup key is PER-EDITION — BB2016 asks
      for a `CATCH` source (the Catch skill), BB2020/BB2025 for `INTERCEPTION`. Rust never re-rolled,
      so a Catch-carrying interceptor stayed one die behind Java for the rest of the game. Implemented
      in `bb2025/pass/step_intercept.rs` (`intercept` now takes `&mut self`); test
      `a_failed_bb2016_interception_is_rerolled_by_the_catch_skill` pins the die count at 1 without
      Catch and 2 with it. **Process note:** the previous iteration's "missing tacklezone modifiers"
      theory was WRONG and was flagged as unverified before any code changed. `FFB_DICE_DEEP=1` prints
      Java's FULL caller chain per die and identified the real culprit in one run — use it whenever
      two dice share a `rollSkill:112` caller instead of inferring from values.

- [ ] **Gate `gate6` came back bb2020 30/30, bb2025 30/30, bb2016 29/30 — ONE red left:**
      `high_elf bb2016 seed 90`, first `rng_calls` divergence at i=269 (Rust 88, Java 89), i.e. the
      extra die falls in activation i=268 `Activate(home_08, PASS)`. Green this, then re-gate and commit.

      Java's four dice there, from `FFB_DICE_DEEP=1`:

          #86 StepIntercept.intercept            d6=6  <- interception SUCCEEDS (natural 6)
          #87 StepSafeThrow.executeStep:125      d6=4  <- Safe Throw cancels it
          #88 StepPass.executeStep               d6=4  <- the pass
          #89 StepCatchScatterThrowIn.catchBall  d6=2  <- the catch

      Rust spends only three. Probes confirm Rust DOES reach both earlier rolls: `ST entry
      interceptor=Some("away_03") thrower=Some("home_08") has_st=Some(true)` and `ST rolling
      calls_before=86`, so the interception (#86) and the Safe Throw (#87) both happen, and #88 is the
      pass. **The missing roll is the CATCH (#89).** So after a Safe-Throw-cancelled interception the
      pass completes but Rust never rolls the receiver's catch. Look at what the bb2016
      `StepCatchScatterThrowIn` receives in that path — `catcher_id`, the player under the ball, and
      the incoming `CatchScatterThrowInMode` — and compare with Java's. Note this is a THIRD variant
      of the same family already fixed twice (`InterceptorId` leaking, `CatcherId` leaking), so check
      whether a parameter is missing or stale here too.

      **DONE (`01da521e`).** The cause was a consequence of fix 8: a successful Safe Throw CANCELS
      the interception (Java publishes `INTERCEPTOR_ID = null`), but the interception success flags
      that `StepResolvePass` now gates on were left set, so ResolvePass took the interceptor branch,
      published no catch mode, and the bb2016 `CatchScatterThrowIn` returned early on `mode == None`.
      Safe Throw's success path now clears all three. Test
      `a_successful_safe_throw_clears_the_interception_flags`.

**SECTION COMPLETE.** Gate `gate7`: bb2016 30/30, bb2020 30/30, bb2025 30/30 at seeds 1-100 with
agent-driven interception ENABLED. Workspace 14,539 pass / 0 fail. Cloud Burster is a live mechanic.
Twelve Rust fidelity bugs came out of this campaign; see `docs/DEAD_STEP_INVENTORY.md` Update (3) for
the full table and the two technique notes worth carrying forward (`FFB_DICE_DEEP=1`, and the
parameter-outlives-its-sequence family). After the narrowed CatcherId fix:
      **green** — dark_elf bb2020 21, dark_elf_league_fumbbl bb2020 21, amazon bb2020 75,
      necromantic bb2020 41 (and necromantic 8, the regression the first attempt caused);
      **still red** — elf bb2016 83, high_elf bb2016 24. Both bb2016, so the BB2020 deflection work
      is done and these are a different cause.

**Frontier — high_elf bb2016 seed 24.** State strings first differ at i=67 (Java home turn 4, Rust
away turn 5 — Rust ended the home turn early; `h02` Prone in Rust, Standing in Java). But `rng_calls`
diverge EARLIER, at i=56: Java 37 vs Rust 36. So the extra Java call happens during the i=55
activation `Activate(away_03, PASS)`, where Java spends four calls (33→37) and Rust three (33→36).
Narrowed further (2026-08-18). Both engines roll the SAME VALUES at calls #34-#37 (`2, 4, 4, 2`).
Rust spends three of them and Java four:

- Rust `#34` = the interception, `roll=2 min=6 ok=false ag=4` (probe `IC`), then `#35` = the pass roll,
  then `#36` = something that is NOT a catch. Its `DRIVE` window for i=55 is
  `Pass -> ResolvePass -> GotoLabel -> CatchScatterThrowIn -> EndPassing`.
- **`catch_ball` is NEVER CALLED in the whole game** — a probe at its entry printed nothing. So after
  a FAILED bb2016 interception Rust never rolls the receiver's catch, while Java rolls `#36 d6=4` and
  then `#37 d6=2`, which looks like a catch followed by a **Catch-skill re-roll** (high elves have
  Catch). That missing catch is the bug; find why `CatchScatterThrowIn` reaches neither
  `handle_regular_catch`'s roll nor `catch_ball` — most likely `catcher_id` is None AND nobody is
  registered under the ball, so it takes an empty-square branch.
- This seed was ALREADY red in gate4 (step 66), so it is a pre-existing bug, not something the
  interception work introduced.

**RESOLVED to a single wrong number (2026-08-18).** The catch IS rolled — the earlier "catch_ball is
never called" reading was from probing the WRONG FILE. `driver.rs` has an explicit per-edition
override block (~line 434) that takes precedence over its glob imports, and BB2016 maps
`StepId::CatchScatterThrowIn` to its OWN `crate::step::bb2016::StepCatchScatterThrowIn`. **Checking
the globs is not enough — check the override block too.** (Third wrong-file probe this session.)

With the right file probed:

    IC      id=home_03 roll=2 min=6 ok=false calls=34 ag=4      <- interception fails, both engines
    CATCH16 id=away_02 roll=4 min=2 ok=true  calls=36 ag=4 mode=CatchAccuratePass

Rust's catch target is **2** and it SUCCEEDS on the 4. Java rolls the same 4, FAILS, and spends `#37`
on the Catch-skill re-roll (high elves have Catch) — so Java's target is **5 or more**. The formula
itself is right (`minimum_roll_catch_edition`, BB2016 = `(7 - AG) + modifiers` floor 2; AG4 -> base 3),
so the divergence is the MODIFIER TOTAL: Rust ends at 3 + (-1 accurate pass) = 2, Java at >= 5, a gap
of about +3 that looks like missing TACKLEZONE modifiers on the catcher.

- [ ] Finish this. Measured so far:

          CMODS total=-1 ["Accurate Pass=-1"] tz=0 at=Some((12, 7)) ball=Some((12, 7))

      Rust applies ONLY `Accurate Pass=-1` and counts **zero** tacklezones on the catcher `away_02`
      at (12,7) — even though `home_03` stands at (12,8), directly adjacent. The plumbing all looks
      right: Rust's bb2016 `CatchModifierCollection` extends the base collection (which does carry
      `1..8 Tacklezone` and the disturbing-presence entries, exactly like Java's), the factory's
      `find_applicable` does the count-based TACKLEZONE selection, and `UtilPlayer::find_tacklezones`
      -> `find_tacklezone_players` correctly takes the OTHER team relative to the given player and
      the player's own coordinate. So the next question is why the count is 0: most likely `home_03`
      has no tacklezones at that moment (prone/stunned), which would ALSO mean Java counts none.

      **CAUTION — an unverified assumption underpins the "Java target >= 5" reading.** It assumes
      Java's 4th die (`#37 d6=2`) is a Catch-skill re-roll after a FAILED catch on `#36 d6=4`. That
      has NOT been confirmed; `#36` and `#37` are both `rollSkill:112` and the die name cannot tell
      them apart. If instead Java's catch SUCCEEDED on `#36` (min <= 4) then `#37` is something else
      entirely and the whole tacklezone theory is wrong. **Verify what Java's `#37` actually is
      before changing any modifier code** — e.g. by checking whether Java's ReportCatchRoll for this
      catch says rerolled, or by finding which Java call site consumes call #37. Do not fix on the
      assumption.
      Then re-check `elf bb2016 83`, which may share the cause.

PROBES CURRENTLY IN THE TREE for this investigation — remove before gating, then `git diff` and read
every `-` line: `FFB_P2` prints `IC` in `bb2025/pass/step_intercept.rs`, and `CS16` + `CATCH16` in
`bb2016/step_catch_scatter_throw_in.rs`. (The bb2025 shared-file probes were already removed — that
file is not the one BB2016 runs.)

**Blocking seeds** (four are BB2020 deflections)

| edition | roster | seed | step |
|---|---|---|---|
| bb2020 | dark_elf | 21 | 95 |
| bb2020 | dark_elf_league_fumbbl | 21 | 95 (same cause) |
| bb2020 | amazon | 75 | 152 |
| bb2020 | necromantic | 41 | 134 |
| bb2016 | elf | 83 | 99 |
| bb2016 | high_elf | 24 | 66 |

**What is already established for dark_elf 21** — do not re-derive. Activation i=95
`Activate(away_02, PASS)`, first differing state i=96. With interception enabled, Rust's interception
matches Java exactly: same candidate `home_02`, same call #49, `roll=6 min=6 ok=true`. The BB2020
deflection branch in `step_resolve_pass.rs` fires (`took=true`) and places the ball on the deflector.
Java ends with the ball at 12,7 (the deflector caught it); Rust ends at 22,7 (the receiver / pass
coordinate). So the bug is strictly below `ResolvePass`.

**Leading suspect, found while ticking item 1.** `handle_regular_catch` does
`if self.catcher_id.is_none() { self.catcher_id = player_under_ball.clone(); }` — the same shape as
Java's `if (!StringTool.isProvided(fCatcherId))`. So the structure matches and the divergence must be
the VALUE of `catcher_id` on entry: if a stale `CatcherId` (the intended receiver, published upstream)
survives into this step, the deflected catch is resolved for the RECEIVER instead of the deflector,
and the ball lands on the receiver's square — exactly the 22,7 observed. This is the same
parameter-outlives-its-sequence bug as fix 8 (`InterceptorId`); `step_resolve_pass.rs` already carries
a comment that Rust's `CatcherId` delivery "STOPS at the first consumer". Confirm by probing
`catcher_id` vs `player_under_ball` at the `Deflected` arm, then decide whether the deflected path
should clear `catcher_id` or whether the upstream publish should not reach here at all.

---

## 3. Audit the bare skill-roll comparisons — ✅ DONE 2026-08-18 (`be482da4`)

`crates/ffb-engine/src/dice_interpreter.rs` already has the correct rule, yet ~25 sites still compare
`roll >= min_roll` directly (bone head, really stupid, wild animal, blood lust, take root, dauntless,
jump up, chainsaw, GFI, shadowing, …). One of them was a real bug this session (fix 6 above).

- [x] Audited. The ~55 sites were SPLIT rather than swept:
      - **Fixed targets** (`>= 2/3/4/5`) left alone — inside 2..6 the natural-6/natural-1 rule and a
        bare `>=` are provably identical (a natural 6 clears any target ≤ 6; a natural 1 fails any
        target ≥ 2), so there is nothing to fix.
      - **Variable targets** cross-referenced against the Java classes that actually call
        `isSkillRollSuccessful` (`grep -rl isSkillRollSuccessful ffb-server/.../server/`). That
        membership test replaced guesswork. **23 sites** changed: bone head (×3 editions), really
        stupid (×3), wild animal, blood lust (×4), take root (×2), jump up, shadowing (×2),
        tentacles, unchannelled fury, animal savagery, go-for-it (×3), and the dead bb2020 intercept
        twin (kept faithful).
      - **Deliberately NOT changed:** chainsaw and chomp — Java's `StepBlockChainsaw` uses a literal
        `roll >= minimumRoll` and neither is in the helper list; and **dauntless**, whose roll could
        not be located in `StepDauntless.java`. Only CONFIRMED mismatches were touched.
      - Gate 30/30/30, workspace 14,539/0.

**Residual risk, recorded honestly:** this class of mismatch is INVISIBLE until a target leaves 2..6,
so the green gate does NOT prove the untouched sites are correct — it only proves these 23 changed
nothing observable. The verification that counted was reading the Java call sites. If a future red
seed implicates a roll, re-check its Java helper before assuming the site is fine.

**Do not blanket-change the remainder.** A sweeping edit is the "measured worse" trap this project has
hit twice.

---

## 4. Remaining dead-step targets

Work in this order; each needs the trigger verified as reachable *before* any harness plumbing.

- [x] `SelectGazeTarget` / `SelectGazeTargetEnd` — **DONE 2026-08-18: routing fixed (`a7e7da8f`),
      mechanic confirmed UNREACHABLE IN JAVA. Closed, not driven.** Correction to the line above: these are **BB2020-ONLY**, not "all editions" —
      the Java steps are `@RulesCollection(BB2020)` and are pushed solely from
      `bb2020/move/StepEndSelecting`; BB2025 has no gaze-target path at all (only `AutoGazeZoat`).

      1. **Dead-twin routing.** The only Rust code that pushes the sequence is
         `step/bb2020/move_/step_end_selecting.rs`, which is DEAD: BB2020 and BB2025 both run the
         glob'd `bb2025::shared::step_end_selecting` (no gaze push), and BB2016 gets its own via the
         per-edition override block. So the sequence is never pushed in any edition. Fix by
         edition-gating the push INSIDE the shared step — never by routing to the dead twin.
      2. **The trigger needs a declaration neither harness makes.** Java pushes the sequence on
         `PlayerAction.GAZE_SELECT`, and `GAZE_SELECT` is produced ONLY by
         `bb2020/shared/StepInitSelecting:113` when the client declares **`GAZE_MOVE`** with no target
         selection state. Both harnesses declare plain `GAZE` (`ParityRunner:1996` adds
         `PlayerAction.GAZE`), so `GAZE_SELECT` never arises. Driving this therefore needs both
         harnesses taught to declare `GAZE_MOVE` **in lockstep** — the same shape as the TTM, KTM and
         interception campaigns, and the same risk profile: expect it to expose real engine bugs.

      **Blocker 1 FIXED (`gate9` 30/30/30).** The `GAZE_SELECT` arm now lives in the LIVE shared
      `bb2025/shared/step_end_selecting.rs`, gated to BB2020, test
      `gaze_select_pushes_the_gaze_target_sequence_in_bb2020`. It is **LATENT** — vampire bb2020 still
      measures 10/10 with ZERO `SelectGazeTarget` dispatches, exactly as blocker 2 predicts. The steps
      can now be reached; they are not yet driven.
      **Blocker 2 RESOLVED — the steps are UNREACHABLE IN JAVA ITSELF. Do not build harness
      plumbing for them.** The chain requires a `GAZE_MOVE` declaration, which the client only offers
      when the player has `canGazeDuringMove` (`MoveLogicModule:362`, and `ParityRunner:1995` uses the
      same property). That property is registered **only by `skill/bb2016/HypnoticGaze`** — bb2020's
      and bb2025's HypnoticGaze register just `inflictsConfusion`. So no BB2020 player can ever
      declare GAZE, `GAZE_SELECT` is never produced, and the BB2020-only `SelectGazeTarget` /
      `SelectGazeTargetEnd` can never run. They are **vestigial in Java**, not merely unported.
      BB2016 does offer GAZE, but its gaze path is `bb2016/move/StepHypnoticGaze`, which already runs.
      Teaching the harness to declare `GAZE_MOVE` under BB2020 would FABRICATE an action Java's own
      eligibility rules never offer — that is inventing behaviour, not fixing parity.
      Note this also means blocker 1's fix (`a7e7da8f`) is correct but will never fire in practice;
      it is kept as faithful routing, not as a live mechanic.
- [x] `DauntlessMultiple` — **CLOSED 2026-08-18: unreachable by DATA, no code change needed.**
      It is pushed only from `StepMultipleBlockFork` (both engines, live bb2025 path), so it requires a
      MULTIPLE BLOCK. **No roster in any edition carries "Multiple Block"** — the only occurrence
      anywhere in `data/` is on a STAR PLAYER (`data/star_players/all_editions.json`), and the drafted
      parity teams hire none: 0 star entries across all 1,049 drafted player rows. So
      `MultipleBlockFork` never runs and `DauntlessMultiple` cannot either. This is the (d1) category
      — "the code is fine, nothing on the pitch can trigger it" — and reaching it would mean changing
      the drafted teams, which is a separate and larger decision. `Dauntless` itself runs normally.
- [ ] The bomb chain — `InitBomb`, `EndBomb`, `ResolveBomb`, `Bombardier2`.
      **TRIGGER VERIFIED 2026-08-18: REACHABLE, and it is the same lockstep-abandonment shape as TTM,
      KTM and interception — the fourth of its kind. This one is worth driving.**
      - Carrier IS drafted: `goblin.bombardier`, 1 in the bb2025 goblin squad (Bombardier appears in
        the goblin roster of all three editions).
      - BOTH harnesses OFFER the action — Rust `legal_actions/mod.rs:194` pushes
        `PlayerActionChoice::ThrowBomb`; Java `ParityRunner:1949` adds `PlayerAction.THROW_BOMB`.
      - BOTH then ABANDON it in lockstep: `isHandledActingAction` (`ParityRunner:2191`) and Rust's
        `is_handled_acting_action` (`random_agent.rs:85`) both omit THROW_BOMB / HAIL_MARY_BOMB, so
        the pick is deselected with no step logged (`UNHANDLED_ACTING_ACTION_AT_PICK`). That is
        exactly why every matrix stays green while the whole bomb chain is dead.
      **PLAN:** add THROW_BOMB to both handled-sets IN LOCKSTEP, then teach both harnesses to answer
      the bomb-target prompt with the same candidate list from the ENGINE's own predicate, the same
      ordering, and ONE `actionRng` pick — and check the coordinate frame by reading the target step's
      `handle_command` (`StepInitThrowTeamMate` un-mirrors, `StepHitAndRun` does not; do not assume).
      Expect reds and real engine bugs, as with the previous three: interception surfaced twelve.
      Measure after EACH change and isolate halves. Use `--home goblin --away goblin --edition bb2025`.
- [ ] `HailMaryPass`.
- [ ] `Swoop` — unreached by the uniform sweep at 3 seeds/matchup; confirm whether it is genuinely dead.

**Method per target**

1. Resolve which drafted positions carry the skill. `data/teams/` specs hold only `position_id` — resolve
   positions to roster starting skills; never grep the specs for skill names.
2. **Verify the trigger is reachable** before building anything. Java's `UNHANDLED_DIALOG` stderr lines
   answer "is this dialog ever raised?" in one grep; a `step=` count from `FFB_DRIVE_TRACE` answers "does
   this step ever run?". This is the check Punt failed.
3. Read the Java step for the client command it waits on.
4. Teach both harnesses in lockstep: same candidate list from the **engine's own** predicate, coordinate
   ordering, a single `actionRng` pick, correct coordinate frame (read the target step's `handle_command`
   to see whether it un-mirrors).
5. Measure after **each** change; isolate halves rather than shipping them together.

---

## Blocked — needs a tier decision from the user

- **Punt.** Plumbing is correct and dark_elf bb2025 is 100/100, but `InitPunt` dispatches zero times:
  Punt needs the carrier holding the ball at *turn start*, which the turn-start snapshot makes
  unreachable. Belongs to the "make the agent score" tier.
- **Widen the state hash to include the ACTIVE bit.** Three bugs this session hid in state the hash
  cannot see (the ACTIVE bit, `ttm_used`/`ktm_used`, and the acting player's MOVING base).

---

## The five recurring bug shapes

Every target in this tier has been one of these. Check them first.

1. A **per-edition rule hard-coded to one edition inside a SHARED file** — five instances so far.
2. **Both harnesses declining the same dialog in lockstep**, which keeps the matrices green while the
   mechanic underneath is dead.
3. A step returning a bare `cont()` with **no prompt**, which only breaks once the other harness starts
   answering that dialog.
4. A **general Java rule simplified away** in Rust — invisible until a mechanic that can produce an
   out-of-range value starts running.
5. A **Java predicate re-implemented with a missing or an extra clause** — harmless until the mechanic
   that uses it starts running.

## Debugging recipe

The one that produced all eight findings of the interception campaign:

- Reproduce the single seed.
- Compare `RUST_STEP i=N rng_calls=` with `JSTEP i=N rng_calls=` (`FFB_TRACE=1`) to find the diverging
  activation.
- Diff the two state strings token by token. Rust's live in
  `parity/<edition>/<matchup>/seed_N_rust.jsonl` (flat JSON, written only under `FFB_TRACE` or
  `--verbose`); Java's come from its `JSTEP i=... state=...` stderr lines. State-string player labels are
  **positional indices**, not ids (`h01` = home_02, `a08` = away_09).
- `FFB_DRIVE_TRACE=1` gives Rust's `DRIVE step=` sequence for the activation.
- The RNG stream is **positional**: equal values at equal positions do *not* mean the same event. To
  settle "which die is this?", print the call count at the roll site on both sides — Rust `rng.call_count`,
  Java `gameState.getDiceRoller().getCallCount()` from the harness.
- Probe the **live** file: check `driver.rs` glob imports first. Per-edition twins are usually dead.
- Piping the parity binary's stdout through `grep` can lose the `PARITY: n/m` line — redirect to a file
  first, then grep the file.
