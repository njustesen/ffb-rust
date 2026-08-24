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
- **Rust-only change? Gate with `--reuse-java`.** The gate runs BOTH engines and Java is ~10x the
  wall-clock (one matchup, 10 seeds: `java_total=6.588s` vs `rust_total=0.675s`), so the JVM is
  essentially the whole gate. `--reuse-java` skips it for any matchup whose cached logs provably came
  from the same jar and Java data, and prints REUSE / REUSE declined so it is never silent. Only omit
  it when `ParityRunner.java` was actually touched and the jar rebuilt — that invalidates the cache.
  (Gates 8 and 9 were Rust-only and re-ran the full JVM for nothing; do not repeat that.)

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

- [x] **RESOLVED — gate6 had one red left (high_elf bb2016 seed 90), fixed by the Safe Throw flag
      clearing (`01da521e`); the final gate7 was 30/30/30. Historical detail follows.** Original note: gate6 came back bb2020 30/30, bb2025 30/30, bb2016 29/30 — ONE red left:**
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

- [x] **RESOLVED — this bb2016 catch investigation was closed by the interception skill re-roll fix
      (`ef647683`), not by a catch-modifier change; the tacklezone theory below was disproved.**
      Historical measurements:

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
- [x] The bomb chain — `InitBomb`, `EndBomb`, `ResolveBomb`, `Bombardier2`. **CLOSED as a
      documented gap (not driven); one latent-stall fix kept.**
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

      **STEP 1 DONE (uncommitted): THROW_BOMB added to BOTH handled-sets in lockstep.**
      Rust `random_agent.rs` — `is_handled_acting_action` now accepts `ThrowBomb`, and the
      declaration picks its target with the SAME rule as a pass (`PlayerActionChoice::Pass |
      PlayerActionChoice::ThrowBomb` share the receiver arm). Java `ParityRunner` —
      `isHandledActingAction` accepts `THROW_BOMB`, and `sendConcreteAction` routes it to
      `sendPassAction`. Jar rebuilt. Justification: a bomb dispatches into the PASS sequence
      (`step_end_selecting`'s `Pass|HailMaryPass|ThrowBomb|HailMaryBomb|HandOver` arm) and
      `StepInitBomb` consumes only `CLIENT_USE_SKILL`, reading the pass coordinate for its target.

      **RESULT: goblin bb2025 0/10 — but NOT a content divergence.** `UNHANDLED_ACTING_ACTION_AT_PICK`
      is now 0 (the action is no longer deselected), and both engines declare the bomb IDENTICALLY:
      seed 1 i=3 `Activate(away_07, ThrowBomb)` in Rust and `Activate(...Away7, THROW_BOMB)` in Java.
      Then **BOTH logs stop at 3 steps** and the diff reports `java=None rust=None`. So both engines
      STALL right after the declaration — each is waiting for a command neither harness sends.
      Dispatch counts confirm nothing downstream ran: `InitBomb`/`ResolveBomb`/`EndBomb`/`Bombardier2`
      all 0, while `Bombardier` (already live) ran 3 times.

      **STALL ROOT-CAUSED (not `StepInitBomb` — that suspicion was wrong).** Both engines park in
      `StepInitPassing` (`step/mixed/pass/StepInitPassing.java`); Java logs
      `UNHANDLED_STEP: INIT_PASSING` 500 times and Rust's last `DRIVE step=` is `InitPassing`.
      That method has **NO else branch**: if none of its three branches matches it leaves the default
      `CONTINUE` and waits forever. Every bomb-capable branch requires
      `mechanic.findPassingDistance(game, throwerCoordinate, passCoordinate, false) != null` — the
      target must be IN RANGE. `sendPassAction` (and Rust's mirrored pass-receiver rule) picks any
      teammate square on the pitch with NO range filter, so an out-of-range bomb target matches no
      branch and the step never advances. `CLIENT_PASS` itself is handled fine: it sets
      `passCoordinate`, `throwerId = actingPlayer` and `throwerAction = THROW_BOMB`, so those
      conjuncts hold; only the distance one fails.

      **THE RANGE THEORY WAS WRONG — do not implement a range filter.** Measuring instead of assuming
      (as the caution demanded) showed the distance was perfectly valid:
      `IP action=Some(ThrowBomb) thrower_acting=true dist_valid=true tc=(19,6) pc=(22,6) raw=Some(QuickPass)`.
      The real Rust cause was a MISSING TARGET: `bb2025/shared/step_init_selecting.rs` threaded
      `TargetCoordinate` through for `Pass | HandOver` only, so a bomb reached `StepInitPassing` with
      no coordinate, `thrower_id`/`thrower_action` were never set, and the step parked on `cont()`
      forever. Adding `PlayerAction::ThrowBomb` to that arm fixed the RUST side.

      **OUTCOME 2026-08-18: SCOPED DOWN after three iterations. Harness change REVERTED in lockstep;
      one real fidelity fix KEPT. The bomb chain remains dead — recorded honestly rather than forced.**

      - **KEPT (gated green):** `bb2025/shared/step_init_selecting.rs` threaded `TargetCoordinate` for
        `Pass | HandOver` only, so a bomb reached `StepInitPassing` with no coordinate,
        `thrower_id`/`thrower_action` were never set, and the step parked on `cont()` forever.
        `PlayerAction::ThrowBomb` now joins that arm. INERT while no bomb is declared, but faithful and
        it removes a real latent stall.
      - **REVERTED (both sides together):** `THROW_BOMB` in `is_handled_acting_action` /
        `isHandledActingAction`, and Java's `sendConcreteAction` -> `sendPassAction` routing. With them
        in, Rust DID drive the bomb (`InitBomb: 1`, `EndBomb: 1`, goblin bb2025 seed 1) but Java parked
        at `UNHANDLED_STEP: INIT_PASSING` (2500 spins) and goblin bb2025 was 0/10. After the revert:
        goblin bb2025 back to 10/10.
      - **Two theories of mine were disproved by measurement — do not revisit.** (a) `StepInitBomb`'s
        `CLIENT_USE_SKILL` was NOT the stall. (b) The RANGE theory was wrong (`dist_valid=true …
        raw=Some(QuickPass)`), so do NOT add a passing-distance filter to either harness.

      **Why stopping here was right.** The remaining gap is HARNESS-AUTHORING, not an engine bug:
      Java's phase-2 visit never happens for a bomb, so `CLIENT_PASS` is never injected, so
      `StepInitSelecting` never publishes `TARGET_COORDINATE` (it publishes only inside its
      `CLIENT_PASS` case, and `StepInitPassing` reads that parameter to set the thrower). Closing it
      means reverse-engineering how the real client declares a bomb — open-ended, and no engine bug in
      sight.

      **To resume:** re-apply the two handled-set changes plus Java's `sendConcreteAction` case (all
      together), then answer one question — why phase 2 is not reached for THROW_BOMB. Precedent in the
      same file: BLITZ is declared as `BLITZ_MOVE` because `StepInitSelecting` dispatches it onward, so
      a bomb may need declaring as a different `PlayerAction`, or the harness may need to inject
      `CLIENT_PASS` at declaration time.

- [x] `HailMaryPass` — **CLOSED 2026-08-18: blocked by the SAME root cause as the bomb chain.**
      - Carrier IS drafted: `elf.thrower` carries Hail Mary Pass and TWO are in the bb2025 elf squad.
      - **Neither harness ever offers the action.** `ParityRunner` contains no `HAIL_MARY` occurrence
        at all; Rust's `legal_actions/mod.rs` has no `HailMaryPass` entry (the mention at
        `random_agent.rs:954` is only a display mapping).
      - **Same declaration route as the bomb:** `bb2025/move/StepInitMoving:215` dispatches it from the
        `CLIENT_PASS` handler and only when `actingPlayer.getPlayerAction() == HAIL_MARY_PASS` — so it
        must be declared at activation and then confirmed by `CLIENT_PASS`, exactly the path the bomb
        campaign proved is broken.
      - **This is a PREDICTION, not a measurement.** Deliberately not tested: the bomb chain had just
        been scoped down for this reason, and re-running the same experiment on a second action would
        spend an iteration to learn what is already known.
      **Resolving the phase-2 declaration question unlocks BOTH targets at once** — one follow-up, not
      two campaigns. Highest-value remaining item in this section.
- [x] `Swoop` — **CLOSED 2026-08-18: NOT dead. It is live and parity-verified.**
      Measured on goblin bb2020, seeds 1-20, parity agent: `Swoop: 1` dispatch, matchup **20/20 green**.
      Its absence from the uniform sweep was SEED DEPTH (3 seeds/matchup), not a second gap — exactly
      the alternative the inventory flagged as "not yet verified".
      **Correction to `docs/DEAD_STEP_INVENTORY.md`:** the note said Swoop "needs a kicked Doom Diver
      in BB2016/BB2020 specifically". The run that produced it had `KickTeamMate: 0` and
      `ThrowTeamMate: 13`, so it came from a THROWN Doom Diver; a kick is not required.

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

## 5. Bomb chain — ✅ COMPLETE 2026-08-19 (`7fa7b7ad` + ffb `bae96fcd2`), matrix 30/30/30

The user chose the "phase-2 declaration route". **It is answered**, and the answer was not what the
question assumed:

> **Phase 2 is never reached for THROW_BOMB because the bomb re-throw has no declaration at all.**
> After a bomb is caught, the ENGINE makes the catcher the acting player with `THROW_BOMB` in
> `TurnMode.BOMB_HOME`/`BOMB_AWAY`. Nothing goes through phase 1, so the phase-1/phase-2 activation
> loop is never entered for it. `StepInitPassing` then parks with a null thrower, and the harness
> spun on `UNHANDLED_STEP: INIT_PASSING` (2500x).

The decisive evidence was a harness-side probe printing `thrower/passCoord/actingPid/turnMode` at
the unhandled step: `thrower=null passCoord=null actingAction=THROW_BOMB turnMode=BOMB_HOME`.

**A decline is impossible here** — `StepInitPassing.executeStep` opens with
`if (thrower == null || throwerAction == null) return;`, which is BEFORE the `fEndTurn` check, so
`CLIENT_END_TURN` and a deselect both set their flag and are then swallowed. The only command that
advances the step is `CLIENT_PASS`. The bomb must actually be thrown. (A first attempt to decline
measured as a `STUCK_STEP` — the same stall wearing a different name.)

### Landed (uncommitted — gate NOT yet run)
- Harness (both sides, in lockstep): `THROW_BOMB` joins the handled-action set, `sendConcreteAction`
  routes it to `sendPassAction`, and a new `case INIT_PASSING` throws the re-thrown bomb with the
  SAME candidate rule and the SAME single `actionRng` draw as an ordinary pass.
- Rust `AgentPrompt::BombRethrow` surfaces the otherwise prompt-less park (the parity loop
  `break`s silently on `current_prompt().is_none()` — that is why the game just ended at the first
  bomb, with no error). Both agents answer it via `legal_pass_receivers`.
- **Engine bug 1** — `bb2016/move_/step_init_selecting.rs`: `ThrowBomb` was missing from the
  `Pass | HandOver` target-threading arm, so the bomb carried no `TargetCoordinate`.
- **Engine bug 2** — `bb2016/move_/step_end_selecting.rs`: Java groups
  `PASS | HAIL_MARY_PASS | THROW_BOMB | HAIL_MARY_BOMB | HAND_OVER`; **both bomb variants were
  missing**, so a declared bomb dispatched to nothing and the activation was a pure no-op
  (`post_hash == pre_hash`). Regression test `dispatch_bomb_actions_push_pass_sequence`.

Result on goblin bb2016 seed 1: first divergence moved **step 2 -> step 17**, and the bomb now
executes in BOTH engines (it had never executed in either). Workspace 14,540 pass / 0 fail.

### Next
- [x] **Step 17 SOLVED — engine bug 3: a fumbled BOMB scattered the real football.**
      `bb2025/pass/step_pass.rs` published `CatchScatterThrowInMode::ScatterBall` unconditionally in
      the fumble branch; every edition's Java `StepPass` publishes it ONLY in the non-bomb arm. The
      spurious d8 both moved the ball AND shifted every later die by one, so the armour roll landed
      on Java's next two dice — breaking AV and STUNNING the thrower where Java leaves him Prone.
      Tests `fumbled_bomb_does_not_publish_scatter_ball` + `fumbled_pass_still_publishes_scatter_ball`.
      Frontier moved **step 17 -> step 32**.
      **The wrong-file trap struck a 4th time**: `StepId::Pass` is NOT in the bb2016 override list, so
      bb2016 runs the SHARED bb2025 `StepPass`. The first fix went into `bb2016/pass/step_pass.rs`
      and changed nothing. ALWAYS grep the override block for the exact `StepId` before editing.
      (The bb2016 file kept the same faithful correction; it is currently unrouted/dead.)
- [x] **Two rejected hypotheses, recorded so they are not retried.** (a) Routing bb2016 to its own
      `StepSpecialEffect` twin — measured WORSE (frontier 17 -> 2): that twin does not apply the blast
      at all, while the shared step already matched. Reverted; `driver.rs` is untouched. (b) The
      bomb INJURY TYPE (`InjuryTypeBomb` vs `InjuryTypeBombWithModifier`) — bb2016 Java does use the
      plain type and the shared Rust step is edition-gated for it now, but it changed NOTHING
      observable on this seed. The gate is faithful so it stays, but it was not the bug.
- [x] **Step 32 SOLVED — engine bug 4: bb2016's bomb turnover was being swallowed.**
      Java bb2016's `StepSpecialEffect` ends the turn whenever a NON-fireball effect hits a player of
      the acting team — with **no `isStanding` gate and no `suppressEndTurn`**. The shared Rust step
      applied both bb2020+ gates to bb2016, so a bomb that caught the thrower's own already-PRONE
      team-mate never ended the turn (seed 1 step 32: Java ends the away turn, Rust kept activating).
      Fixed by edition-gating both. Test `bb2016_bomb_on_prone_own_team_player_still_publishes_end_turn`.
      **goblin bb2016 seed 1 is now GREEN (1/1).**
- [x] **goblin bb2016 = 100/100 GREEN.** Five more fixes got there (user directive: continue until
      ALL GREEN, then commit and push — the gate is `--edition all --seeds 1-100 --parallel 3` 30/30/30):
      - **Bug 5 (seed 9 step 5):** an accurate BOMB's catch got no accurate modifier — StepPass
        published `PassAccurate(!is_bomb)`, starving ResolvePass's `CatchAccurateBomb` branch (catch
        at 4+ instead of 3+). Now `PassAccurate(true)` for bombs too; the completion-SPP overcount
        that `!is_bomb` guarded against is gated in StepEndPassing exactly where Java gates it
        (`hasBall(catcher)` — never true for a bomb).
      - **Bug 6 (seed 9 step 61):** `handle_injury_by_name` had NO `"InjuryTypeBomb"` arm — the
        bb2016 injury-type gate fell through to the generic DropFall fallback, skipping the Stunty
        interpretation (injury 9 on a Stunty goblin is Badly Hurt, not KO).
      - **Bug 7 (seed 3 step 6):** the bomb RE-THROW sequence was built with the bb2025 generator
        (mixed StepEndBomb hardcodes it); bb2016 places MissedPass AFTER ResolvePass, bb2025 BEFORE,
        so ResolvePass ran after the scatter and dragged the bomb back to the original target
        square. Mixed EndBomb now pushes the bb2016 Pass sequence under bb2016 — Java uses the
        per-ruleset SequenceGeneratorFactory here.
      - **Bug 8 (seed 7 step 33):** `DropPlayerFromBombCommand` called the rng-LESS `drop_player`,
        silently skipping the Ball & Chain chain-injury roll (2d6) Java's dropPlayer makes for a
        `placedProneCausesInjuryRoll` player — the streams desynced invisibly (state hash blind).
        Now `drop_player_rng`.
      - **Bug 9 (seed 4 step 56):** a bomb CATCHER who re-threw was retired STANDING+INACTIVE at
        the end of the activation and could never activate again (the ACTIVE bit is invisible to
        the hash; found via JAVA_ACT_PICK/RUST_ACT_PICK pick-stream comparison — Rust rejected
        home_10 as inactive turn after turn). TWO faithful pieces: (a) mixed StepEndBomb now hands
        the acting slot back to `game.original_bombardier` before EndPlayerAction (the "PassState is
        a stub" comment was stale — the field exists and is set); (b) `change_player_action_to_none`
        now carries Java's THROW_BOMB carve-out (retire-inactive only when `!bombAction ||
        (hasBombSkill && bombSkillUsed)`).
      - **Bug 10 / harness (seed 5 step 74):** bb2016 Java's StepSpecialEffect publishes DIRECTLY
        (`INJURY_RESULT` first, dropPlayer's params after), so the B&C chain injury is published
        LAST and the apothecary applies the CHAIN result over the bomb's. The shared bb2025
        SteadyFooting shape publishes in the opposite order. bb2016 now publishes directly in the
        shared step (edition gate); bb2020+ keep the SteadyFootingContext shape (Java bb2025's own
        fail() order matches Rust's).
      - **Harness (seed 21 step 23):** the INIT_PASSING redraw is now gated on `thrower == null`
        (the true re-throw park). A park with the thrower SET is an OUT-OF-RANGE declared
        pass/bomb, whose established contract is END_TURN with zero dice (underworld seed 72);
        redrawing unconditionally made Java re-pick until in range.
- [x] **goblin bb2025 97/100, bb2020 10/10 (1-10), bb2016 100/100** — nine more fixes:
      - bb2025 `BOMB_BOUNCES_ON_EMPTY_SQUARES` read raw (`is_enabled`) instead of
        `get_option_with_default` — factory default is TRUE, so the empty-square bounce d8 never
        rolled (seed 3 step 6). One engine test re-pinned the option off.
      - `InjuryTypeBombWithModifier(+ForSpp)`: the old skip-armour-roll-for-B&C translation matched
        an EARLIER Java version — current Java rolls armour UNCONDITIONALLY and recomputes
        armorBroken (overwriting the B&C pre-break), placing a held-armour victim PRONE (seed 4).
      - bb2020 arm in the shared `StepSpecialEffect`: direct publish order (bomb first, dropPlayer
        chain last = applied), UNCONDITIONAL suppressEndTurn (no option gate — that's bb2025-only),
        active-restore, END_TURN stripped from dropPlayer params when suppressed.
      - bb2020: caught bomb explodes in hand on d6 4+ (`ReportBombExplodesAfterCatch`) — gated
        into the shared InitBomb; bb2020 has NO bounce (gated out).
      - The shared MissedPass's out-of-bounds ThrowIn publish is an invention (Java publishes
        NOTHING there; ResolvePass routes OOB per kind) — kept for the ball path (measured green),
        gated off for bombs (a wildly-inaccurate bomb OOB got the real ball thrown in, seed 10 bb2020).
      - `StepSpecialEffect` read `StepParameter::OriginalBombardier` that NOTHING publishes — the
        bomber-hit turnover reset never fired anywhere, and it CLOBBERED `game.original_bombardier`
        with None. Now reads the game field; bb2025 InitSelecting mirrors Java's
        `passState.setOriginalBombardier/reset()` at declaration.
      - Mixed EndBomb bomber-restore now captures the bomber's PlayerState BEFORE
        `change_player_action` (which stamps MOVING) — a bomber stunned by his own fumbled bomb
        stood back up at retire (seed 65).
      - bb2020 MightyBlow's injury predicate also excludes MB when the ARMOUR roll carried a
        `blocksLikeChainsaw` modifier (Troll blocks the Looney; seed 5 step 241 KO-vs-Bh).
      - `legal_activate_player_actions` lacked ParityRunner's ON-PITCH guard — a boxed/banned
        player at (-1,-1) with a Standing base (banned Secret Weapon Bombardier) padded the
        snapshot by one, shifting every pick modulo (seed 99; found via new JAVA_PPICK probe).
      - Mixed StepMoveBallAndChain missed Java's `setGoingForIt(isNextMoveGoingForIt)` before the
        compulsory block — an over-MA B&C block skipped the rush d6 (seed 87).
- [x] **GOBLIN 100/100 IN ALL THREE EDITIONS (2026-08-19).** Final fixes:
      - **Bomb interception is a pip harder** — StepIntercept's `is_bomb_flag` read the dead
        `OriginalBombardier` parameter; now reads `game.original_bombardier` (fixed seeds 33+52 at
        once). Edition-gated OFF for bb2016: its Java builds `InterceptionContext(..., false)`
        hard-false, and bb2016 lacks the InitSelecting reset so the game field goes stale there
        (this gate alone was the 100/100→15/100 collapse's second suspect — the real one was the
        BOUNCE, below).
      - **The InitBomb empty-square bounce is bb2025-ONLY** — bb2016's and bb2020's own Java
        classes have no bounce. The earlier "bb2016 measured green with the bounce" note was wrong:
        that measurement predated the option-default fix, when the bounce was dead everywhere.
      - **bb2020: a deflected (non-easy) interception must NOT hand the Bomb sequence a catcher** —
        Java bb2020 publishes INTERCEPTOR_ID only in its easyIntercept branch; Rust publishes the
        id for every success (its ResolvePass needs it), so EndPassing now gates the bomb-catcher
        choice on the new `interception_successful` flag (seed 45: a dropped deflection handed the
        deflector a free re-throw). One EndPassing test updated to set the flag.
      - **EndBomb bomber-restore refinements**: bb2016-gated write-back of a non-standing captured
        state (its apothecary applies the KO via AcceptInjury BEFORE EndBomb runs, so the MOVING
        stamp erased a fresh KO — seed 29); tightening the guard to other-holds-slot regressed
        bb2025/bb2020 and was REVERTED (Java's None-acting case runs the restore).
      - **Same-team block strength (Ball & Chain)**: Java's `findBlockStrength` same-team clauses
        ported — `ignoresAssists && sameTeam → bare strength` (bb2020/25 B&C) and
        `flipSameTeamOpponentToOtherTeam && sameTeam → assists marked by the OTHER team` (bb2016
        B&C). Reads MUST be `has_skill_property_in(game.rules, ..)` — the edition-agnostic union
        list makes the ignores clause swallow the bb2016 flip. Tests:
        `same_team_ball_and_chain_block_flips_assist_marking`, `eligible_players_excludes_boxed_standing_player`.
      All probes removed (diff-verified); workspace 14,546 / 0.
- [x] **FULL MATRIX GATE PASSED 2026-08-19: 30 green/0 red in bb2016, bb2020 AND bb2025 at seeds
      1-100.** COMMITTED AND PUSHED per the user's directive: ffb-rust `7fa7b7ad` (30 files,
      the whole §5 campaign); ffb repo `bae96fcd2` on branch `t3-phase2-wip` (ParityRunner only —
      the other 7 modified files there remain local tracing diffs, as before). §5 is COMPLETE:
      THROW_BOMB is a live, parity-verified mechanic in all three editions for the first time.
- [x] (was: gate running) (bash id `brkw294vp`,
      output `...\scratchpad\gate_final.txt` via tasks output). At 30/30/30: COMMIT AND PUSH per
      the user's directive — ffb-rust changes in one commit (or a few logical ones); the
      ParityRunner.java + Xoshiro/etc harness changes stay in the ffb repo (commit only
      intended files: ffb-ai ParityRunner.java; leave the long-standing local tracing diffs in
      the other 7 files uncommitted as before — check `git diff --stat` there first).
      If RED: diagnose per the recipe; the bomb changes touch every Bombardier roster
      (goblin/halfling/underworld/ogre/…) and the shared pass/injury/block paths touch EVERYONE.
- [x] (superseded) goblin bb2025 seeds 33/52/64 detail
- [x] OLD:  Seed 52 step 121 is traced: home_07's bomb →
      caught → re-thrown INACCURATE → 3 scatters → **StepIntercept deflection on the scattered
      bomb → catch → explode**; Java's chain ends the HOME turn 8 → away t8 has zero activations →
      half ends (argue-the-call bans, half-2 kickoff). Rust never ends home turn 8 (h1t88 vs
      h2t10). Suspect the deflected-bomb explosion path (DEFLECTED_BOMB mode) skips the END_TURN
      publish, or the deflection sub-chain diverges. bb2020 needs 11-100; bb2016 needs a re-verify
      after today's shared-file edits; then halfling/underworld (other Bombardier rosters) via the
      full gate.
- [ ] PROBES still in tree (all FFB_TRACE/FFB_ACT_TRACE-gated, remove before commit):
      RPASSEVAL (step_pass), RMISSED25 (missed_pass), RINITBOMB (init_bomb), RRESOLVEBOMB
      (resolve_bomb), RSTEADY/RSTEADY_SET (steady_footing), RAPO_SET (apothecary), RAPPLY/RAPPLY2
      (injury_result.rs + injury.rs), RMBFILTER (injury_type_block), RUST_ACT_PICK/RPASSPICK
      (random_agent — consider keeping, they mirror Java's DEBUG_ACT), JAVA_PPICK (ParityRunner —
      keep, DEBUG_ACT-gated).
- [x] (superseded detail)
- [x] OLD-1/10-FRONTIER:  Seed 3 (bb2025) is fully traced:
      - Step 4's bomb (Home7 → (12,7)) matches THROUGH the throw; note Rust offers a TRR
        `ReRollOffer{action=CATCH}` on the failed bomb catch that Java does not appear to surface —
        agent declines, verify it costs no dice.
      - **Step 6's INACCURATE bomb (away_07): Java rolls pass + 3 scatters + a 4th d8 (bb2025
        StepMissedPass line ~191, likely the occupied-square bounce) + 2 wizard rolls + armour —
        Rust stops after the 3 scatters: the bomb never explodes and the turn ownership forks**
        (J: h1t22 home / R: h1t12 away at i=7). Suspect the shared bb2025 missed-BOMB path never
        hands the landed bomb to StepSpecialEffect (same family as bb2016 bug 3, different site).
      - Old artifacts mislead: re-run before diffing — a stale seed_3_rust.jsonl cost three wrong
        theories this session. `RUST_STEP` lines (FFB_TRACE=1) are the live truth.
- [x] (superseded) goblin bb2016 seeds 1-10 frontier
- [x] SUPERSEDED-DETAIL:  First fails: **seed 9 step 5** (turn 2, the
      earliest — start here) and seed 8 step 53. Expected: the bomb chain has never run in either
      engine, so there is a backlog of these. Same method: `FFB_TRACE=1`, diff the `JSTEP ... state=`
      line against the jsonl `state` field, then read the rust_events chain around the bomb.
      Remember `StepId::Pass`/`SpecialEffect` are NOT bb2016-overridden — the SHARED bb2025 files are
      the live ones, and bb2016 differences belong behind an edition gate inside them.
- [x] (history) The step-17 diagnosis: the two differing fields were the ball square and
      Prone-vs-Stunned; Rust's re-throw target was VERIFIED IDENTICAL to Java's `(12,6)`,
      which ruled out the agent and pointed at the extra scatter die.
- [x] bb2020/bb2025 need NO equivalent fix. The driver override block is `if rules == Rules::Bb2016`
      ONLY, so bb2020 runs the shared bb2025 steps (`step/bb2020/*selecting*.rs` are DEAD), and the
      shared `bb2025/shared/step_end_selecting.rs:338` already groups all four
      `Pass | HailMaryPass | ThrowBomb | HailMaryBomb`. bb2016 was the lone gap.
- [ ] `HailMaryPass` rides the same route — re-check it once the bomb is green.
- [ ] `uniform_agent` answers `BombRethrow` with `EndTurn`, which cannot advance the step. Harmless
      for parity (coverage-only agent) but it will stall a coverage run — fix before using one.
- [ ] Do NOT commit until `--edition all --seeds 1-100 --parallel 3` is 30/30/30.

---

## 6. Widen the state hash — ✅ COMPLETE 2026-08-19 (`04aef090` + ffb `b776e1150`), gate 30/30/30

Goal: make the per-step hash see the state that hid 4+ bugs this session — above all the
player's **ACTIVE bit** (invisible re-activations, lost deactivations), evaluated for the
**MOVING base** (currently normalised?) too. The two builders MUST stay byte-identical:

- Java: `ParityRunner.stateString(game)` (+ `turnFlags`) — HARNESS, co-editable.
- Rust: the `state_string`/`state_hash` in ffb-parity (see `state_hash.rs` / runner.rs).

Constraints:
- ttm_used/ktm_used stay OUT unless Java's TurnData exposes an accessor (engine code is
  off-limits; the old note says it exposes none — RE-CHECK before assuming).
- Edit BOTH sides in lockstep, rebuild the jar, then re-gate the FULL matrix. NEW REDS ARE THE
  POINT: each one is a real, previously-invisible divergence. Fix them in the Rust engine per
  the standing rules; expect the ACTIVE bit to surface several.
- [x] Format DECIDED + IMPLEMENTED lockstep (uncommitted): each player part gains a trailing
      `,<active-bit>` — `h03:x,y,State,M/S/A/V,1`. Java `ps.isActive()`, Rust `state.is_active()`.
      Jar + binary rebuilt; ffb-model state_hash tests pass.
- [x] **All 8 goblin reds fixed with TWO engine bugs — goblin 100/100 ×3 with the widened hash:**
      1. `change_player_action` never retired the OLD acting player on a genuine change (Java's
         UtilActingPlayer does it on EVERY change: acted → STANDING+INACTIVE with the THROW_BOMB
         carve-out, standing-up → PRONE, else STANDING) — ported as `retire_old_acting_player`,
         evaluated BEFORE set_player resets the acting struct. Test
         `genuine_change_retires_old_acting_player_inactive`. (Fixed the bomber-active-after-bomb
         family: seeds 2/20/83/90 etc.)
      2. The turn-start refresh (`refresh_players_for_turn_start`) was gated `Regular && !new_half`
         — Java runs it UNCONDITIONALLY after the turn-flip handling, including at touchdowns and
         new halves, BEFORE the KO-recovery/fainting block. The gate left last-activation-inactive
         players fainting/boxing with a dead bit that survived into the next drive's Reserve state
         (seed 75 and the java=None/rust=None tails).
      RWATCH/RRETIRE probes removed (diff-verified). Workspace 14,547 / 0.
- [x] **GATE PASSED 30/30/30 ×3 with the widened hash** — no other roster surfaced a divergence
      beyond the two engine fixes. Pushed: ffb-rust `04aef090`, ffb `b776e1150`.
- [x] (was: burn-down list)
- [x] OLD:  (they are the previously-invisible
      ACTIVE-bit divergences this tier exists to surface):
      bb2016: seeds 2 (step 56), 20 (78), 52 (279 java=None rust=None — BOTH logs short?!),
      75 (119); bb2020: 57 (278 both-None), 83 (57), 90 (89); bb2025: 83 (112).
      **Seed 2 bb2016 traced**: after away_05's THROW_BOMB, positional a04 (= away_05, the
      BOMBER) is INACTIVE in Java, ACTIVE in Rust, same rng — the bomber's retire is losing the
      deactivation. Suspects: `change_player_action_to_none`'s THROW_BOMB carve-out
      (`bombardier_spent` — check the ENABLE_THROW_BOMB_ACTION property is registered/edition-
      aware and that StepBombardier really marked the skill used), or the stray-MOVING reset
      (which keeps ACTIVE) running instead of the retire. The `java=None rust=None` fails mean
      BOTH logs ended early — different game lengths, diff the last common step.
- [ ] Then the FULL matrix gate; expect more reds on other rosters (TTM'd players, stunned
      recoveries, pass-block movers all touch the ACTIVE bit).
- [ ] Gate 30/30/30 → commit+push both repos (ffb-rust + ParityRunner in ffb `t3-phase2-wip`).

## 7. New coverage sweep (USER-SELECTED 2026-08-19)

Goal: re-measure step/mechanic coverage against the CURRENT engine — the bomb chain,
interceptions, TTM/KTM/Hit-and-Run, and the ACTIVE-bit fixes changed what is reachable, so
`docs/DEAD_STEP_INVENTORY.md` (2026-08-18, uniform agent = 130/199 StepIds) is stale. Output: an
updated inventory + the next concrete backlog of dead-or-thin mechanics.

Method (per the old inventory + docs/COVERAGE_REPORT.md):
- `FFB_DRIVE_TRACE=1 ./target/release/ffb-parity.exe --uniform --all-rosters --all-editions
  --seeds 1-3 --no-abort` (Rust-only, no JVM), collect `DRIVE step=` lines → reached set;
  subtract from the 199 StepId variants.
- Also aggregate the `*_rust_events.jsonl` tallies for the PARITY matrices already on disk
  (playerAction/kickoff/injury spectra) per COVERAGE_REPORT.md.
- PREP DONE: the uniform agent now supplies a receiver for THROW_BOMB (same bug shape as its old
  TTM gap) and answers BombRethrow by re-throwing (its EndTurn answer could never advance the
  park). Without these the sweep would under-report the entire bomb family.
- [x] Run the uniform sweep (2026-08-19, 261 games): **136/199 reached (was 130), 63 dead
      (was 69)**. Newly reached exactly as predicted: the whole bomb family (Bombardier, InitBomb,
      ResolveBomb, EndBomb, SpecialEffect, RecheckExplodeSkill) + Intercept + SafeThrow.
      CloudBurster stayed dead at 3 uniform seeds (it IS live in the bb2020 parity matrices — 6
      dispatches — a seed-depth artifact, not a gap). T3_COVERAGE.md regenerated by the run.
- [x] docs/DEAD_STEP_INVENTORY.md updated: new 2026-08-19 section prepended with the 63 dead
      steps in 9 classified buckets. Largest: 23 star/skill/inducement-gated, 7 Multiple Block
      family, 8 plumbing/no-op ids never dispatched by name. Actionable buckets: 5 scoring-gated
      (Punt+AssignTouchdowns), 5 uniform-agent declaration gaps (HitAndRun + KTM family — LIVE
      under the parity agent, the uniform agent just never declares them).
- [x] Findings → next backlog: (a) the scoring tier unlocks Punt + AssignTouchdowns (already the
      standing Blocked item); (b) optional cheap win: teach the uniform agent HitAndRun/KTM
      declarations if its coverage number ever matters; (c) the 23-step star/inducement bucket
      needs a DRAFTING change (new tier-sized decision); (d) Multiple Block needs a roster that
      drafts it (none do). Parity-event headline: 1,238 THROW_BOMB actions per 300 goblin games
      across the three editions (was 0 forever). Goblin 3×100 smoke before commit: see below.

## 8. Uniform agent: HitAndRun + Kick Team-Mate (USER-SELECTED 2026-08-19)

Goal: the two "uniform-agent gap" buckets from the §7 inventory — Hit-and-Run and the KTM family
are LIVE, parity-verified mechanics that the uniform sweep cannot reach. Coverage-tool-only tier:
no parity-path changes expected, but uniform_agent.rs lives in ffb-engine, so the goblin 3×100
smoke gates the commit as usual.

Found so far:
- **HitAndRun root cause (confirmed in code):** uniform's `SkillUse` answer hardcodes
  `UseSkill { skill_id: SkillId::Block }`. `StepEndBlocking::handle_command` matches the answer's
  skill_id (`SkillId::HitAndRun` sets `use_hit_and_run`; anything else falls into the
  add-block-die arm), so the Hit-and-Run offer is never answered and the step re-prompts.
  Fix: resolve the prompt's `skill_name` via `SkillId::from_class_name` (the prompt sends the
  Debug name, which normalizes) and echo it; fall back to Block.
- **KTM hypothesis:** only ogre bb2020/bb2025 rosters carry Kick Team-Mate → a 3-seed sweep has
  6 candidate games. Deep ogre-only uniform run (bb2025, 30 seeds) in flight to decide
  agent-gap vs seed-depth-artifact before changing anything.

- [x] SkillUse echo fixed (resolve skill_name via `SkillId::from_class_name`, fall back Block);
      regression test `skill_use_answer_echoes_the_offered_skill`. Amazon bb2020 uniform 10
      seeds → 6 `HitAndRun` dispatches (was 0 everywhere, forever).
- [x] KTM resolved as a RECLASSIFICATION: the dedicated KTM StepIds are dispatched only by the
      bb2016 generator; bb2020/bb2025 kicks ride the shared TTM sequence with IsKickedPlayer
      (probe: 318 offers / 7 picks, kicks execute as InitThrowTeamMate; ogre bb2025 seed 1 parity
      events carry 12 KickTeamMate actions). No bb2016 roster drafts the skill →
      needs-specific-roster. Probes removed, diffs verified clean.
- [x] TTM/KTM staleness filters ported into uniform's live_actions (bb2025 ttm_used; bb2016/
      bb2020 also pass_used; kick: bb2016 blitz_used, else ktm_used).
- [x] Round-2 sweep: **137/199 reached, 62 dead** — HitAndRun newly reached, nothing lost.
      Inventory updated. Goblin 3×100 smoke + commit+push below.

## 9. Star/inducement drafting (USER-SELECTED 2026-08-19)

Goal: unlock the 23 star/skill/inducement-gated dead steps from the §7 inventory (AllYouCanEat,
AutoGazeZoat, BalefulHex, BlackInk, CatchOfTheDay, DispatchDumpOff, DoubleStrength, EatTeamMate,
HailMaryPass, LookIntoMyEyes×2, PileDriver, Pro, QuickBite, RaidingParty, ThenIStartedBlastin×2,
ThrowARock, ThrowKeg×2, Treacherous, WeatherMage, WisdomOfTheWhiteDwarf). Constraint: BOTH
engines must field identical teams — the Java sheets come from `scripts/gen_java_parity_data.py`
over `data/teams` + `data/rosters`; the audited standard rosters must NOT be altered.

Facts so far:
- Dump-off IS drafted (dark elf Runner, all editions) yet DispatchDumpOff is dead — either the
  trigger (carrier with Dump-off gets blitzed) never fires under random play or there is an
  engine/agent gap. Investigate separately: possibly a free fix with no drafting at all.
- No standard roster carries Hail Mary Pass, Pro, Pile Driver, or Multiple Block — those and the
  star specials need new fielded players.

- [x] Pipeline investigated, mechanism DECIDED (2026-08-19): draft stars as EXTRA ROSTERED
      PLAYERS, no inducement phase. `data/star_players/all_editions.json` holds 173 stars in
      exactly the roster-position shape (type "Star", full stat block, `available_for`);
      gen_java_parity_data.py currently SKIPS type Star/Infamous Staff when emitting roster XML,
      and the Rust loader skips the same. Plan: add an optional `"stars": ["<star_id>"]` list to
      a team spec; the gen script resolves it from the star file and emits the star as an extra
      <position> (type Star) + <player>; Rust make_team injects the identical player. Parity
      re-baselines that matchup (same teams both engines). Caveats: the star file is
      bb2016 (race.Name ids) + bb2020 (numeric FUMBBL ids) era — the bb2025 star specials
      (Blastin'/White Dwarf/Keg/etc.) have NO data yet and must be drafted from
      rules/star_players/*.md in a later batch; skill names must resolve in BOTH engines
      (check_skill_names discipline).
      Ready-made pilots: dwarf.Farblast (bb2016 dwarf, Hail Mary Pass), 39465 Helmut Wulf
      (bb2020 dark_elf/nippon/renegades, Pro), 54496 Kiroth (bb2020 dark_elf, Black Ink),
      39464 Hakflem (bb2020 renegades, Treacherous), darkelf.Horkon (bb2016, Multiple Block).
- [x] DispatchDumpOff investigated: NOT free and NOT star-gated — it is a MULTI-BLOCK-sequence
      step (only generator/bb2020+bb2025 multi_block.rs push it); the ordinary window is
      `StepId::DumpOff`, present in every block sequence and already reached. Moved to the
      Multiple Block bucket in the inventory (unlocked by drafting darkelf.Horkon).
- [x] `stars` spec plumbing implemented both sides (TeamFileJson.stars → STAR_PLAYERS lookup →
      Player::from_position; gen script resolves team["stars"], emits the star <position
      type=Star> + <player>). LESSON (new bug shape): the harness activation snapshots index by
      list position (idx % N), so BOTH engines must list players in the identical order — the
      spec edit put nr 13 before 12 and 5/10 seeds went red at the half-2 setup (pick=8 N=10 =
      Java nr 12 / Rust nr 13). Both sides now nr-sort; test star_drafting_injects_the_star_nr_sorted.
- [x] Pilot bb2016 dwarf + dwarf.Farblast at nr 11 (blocker 11→13): 10/10 GREEN vs Java — the
      star drafts, plays, and his Secret Weapon ban matches (both Bariks + Deathrollers benched
      in matching states). Seeds 1-100 running.
- [x] HAIL_MARY_PASS declaration route (2026-08-19): PAC::HailMaryPass variant; legal_actions
      offers it DIRECTLY AFTER Pass for a canPassToAnySquare carrier (order = snapshot contract);
      agents ride the Pass receiver arm + pass_used staleness; bb2016 InitSelecting folded arm,
      bb2025-shared InitSelecting deselect+target arms, bb2025-shared EndSelecting Pass-sequence
      arm, ffb-client encoder; ParityRunner: eligible builder + filterStaleActions +
      isHandledActingAction + sendConcreteAction→sendPassAction. dwarf bb2016 **100/100 GREEN
      with 4 live `step=HailMaryPass` dispatches** — first star special ever exercised. Tests:
      hail_mary_pass_offered_to_carrier_right_after_pass; legality test counts stars outside the
      1.1M budget (modeled induced star).
- [x] bb2020 stars DRAFTED and green (committed): dark_elf + Helmut 39465 (Pro) @10 + Kiroth
      54496 (Black Ink) @11; renegades + Hakflem 39464 (Treacherous) @11. Both matchups 10/10.
- [x] DECLARED-ACTION route for Treacherous/BlackInk tried and REVERTED — wrong architecture.
      Findings (hard-won, keep): (1) ParityRunner `getUnusedSkillWithProperty(Player,..)` returns
      Optional<Skill> — `!= null` is ALWAYS true; use .isPresent() (round-2 0/10: every player
      offered the specials). (2) The REAL client (bb2025 SelectLogicModule) never declares
      TREACHEROUS etc.: it declares a normal action and follows with `sendUseSkill(skill, true)`;
      Java StepInitSelecting:348-378 maps CLIENT_USE_SKILL by skill property →
      fDispatchPlayerAction (canStabTeamMateForBall→TREACHEROUS, canMoveOpenTeamMate→
      RAIDING_PARTY, canStealBallFromOpponent→LOOK_INTO_MY_EYES, canMakeOpponentMissTurn→
      BALEFUL_HEX, canGetBallOnGround→CATCH_OF_THE_DAY, canBlastRemotePlayer→
      THEN_I_STARTED_BLASTIN) + forceGotoOnDispatch — ONE channel unlocks the whole special
      family. BLACK_INK is NOT in that chain (different trigger — find it). (3) Rust's shared
      EndSelecting special arms push `.push_seq(special).push_seq(select)` and the SELECT seq ran
      first in the live trace (special drained by end-of-activation gotos) — the arm comment
      claims the opposite; when building the UseSkill route, verify/fix the push order against
      Java's LIFO (last pushSequence runs first → special first).
- [x] CLIENT_USE_SKILL channel implemented — **Treacherous LIVE: renegades bb2020 100/100 with
      3 StepTreacherous dispatches** (stab + post-stab pass continuation both match Java).
      Pieces: (a) skill_id.rs — the 8 special skills register their Java properties (they
      registered NONE); (b) shared EndSelecting: all 8 special arms push select FIRST, special
      LAST (pushes drain in order onto the LIFO stack → LAST push runs FIRST, matching Java —
      the old order silently DRAINED every special); (c) shared InitSelecting: UseSkill
      6-property dispatch chain (Java :354-378), CLIENT_PASS arm (Java :256-277, dispatching
      DIRECTLY past the folded-model no-defender deselect), start() continuation guard (acting
      player with PASS_MOVE → BombRethrow pass-window instead of the activation-retire clear),
      PAC::Treacherous bridging (= the client's ActingPlayer(PASS_MOVE)+UseSkill pair as one
      agent action); (d) legal_actions + ParityRunner offer TREACHEROUS after KTM under the
      client's isTreacherousAvailable rule; ParityRunner phase-1 double-inject. LESSONS: the
      CLIENT_PASS coordinate is RAW from Rust agents (transforming it threw the pass at the
      mirrored square, seed 85); regression seeds 52/85/91 all green; dark_elf/dwarf/goblin
      regressions green; 3 unit tests; workspace 14,258/0.
- [x] BLACK_INK LIVE — **dark_elf bb2020 100/100 with 130 StepBlackInk dispatches** (renegades
      100/100 w/ 3 stabs, dwarf/goblin/amazon regressions green). It WAS in the UseSkill chain
      (Java :399, later than the first read). Pieces: UseSkill chain += BlackInk/AutoGazeZoat;
      StepBlackInk (both editions) emits PlayerChoice{reason:BLACK_INK} at the victim dialog
      (was a bare cont() that silently ended the game); random_agent answers with the mandatory
      min-(x,y) SelectPlayer (ANIMAL_SAVAGERY rule; ParityRunner's PLAYER_CHOICE branch covers
      the mode); PAC::BlackInk bridging = ActingPlayer(MOVE)+UseSkill, and the post-ink Move
      CONTINUATION re-dispatches (start()'s !acted guard); Kiroth moved to nr 1 (LOS) so windows
      and adjacency actually occur. THREE window lessons: (a) Java declares the specials INSIDE
      the pass-block window — swallowing one forked used_skills, which the hash cannot see, and
      the snapshot N diverged 100 steps later; (b) a window BLITZ against the SUSPENDED THROWER
      re-fires CONFIRM_END_ACTION forever (Java 2M-iteration hang); (c) contract: in PASS_BLOCK
      mode both harnesses shrink the action list to MOVE+Treacherous+BlackInk (Rust agent filter
      + ParityRunner filterStaleActions early-return, PASS_BLOCK only).
- [x] **FULL GATE 30/30/30 in ALL THREE EDITIONS** (2026-08-19; bb2016 30/30, bb2020 30/30,
      bb2025 29/30 in the matrix run + elf re-measured 100/100 after two engine fixes the gate
      itself surfaced — the drafted stars + HMP route made bb2025 elf's roster-native Hail Mary
      Pass live for the first time):
      (a) Rust bb2025 StepHailMaryPass used the `roll==1` fumble shortcut; Java evaluates through
      the REAL pass mechanic (modified result ≤ 1 = FUMBLE — roll 5 under Very Sunny fumbles),
      with the Pass-skill/team reroll cascade and Java :188-217's ball placement (a fumbled HMP
      drops THE ball at the thrower even when he never carried it — the turn-start snapshot can
      offer HMP to a player who lost the ball).
      (b) Rust StepCatchScatterThrowIn accepted the published CATCHER_ID (Java's setParameter
      does NOT — fCatcherId comes from its own dialogs or playerUnderBall) — the stale intended
      receiver "caught" a scattered ball in a square it never visited (bug shape #5).
      Catch-path regressions green: elf 100/100, human/goblin bb2025 30/30, dwarf bb2016 30/30,
      dark_elf bb2020 30/30.
- [ ] Batch the remaining specials (needs bb2025 star DATA drafted from rules/star_players; Pro
      reroll route; Horkon Multiple Block) — NEXT-TIER DECISION for the user.

## 10. Pro + Multiple Block (USER-SELECTED 2026-08-19)

Goal: finish the drafted stars' remaining specials. Helmut Wulf (bb2020 dark_elf @10) carries
Pro + Old Pro; Horkon Heartripper (bb2016-id star) carries Multiple Block. Targets: StepId::Pro,
and the 8-step multiblock family (MultipleBlockFork BlockRollMultiple FoulAppearanceMultiple
ApothecaryMultiple DauntlessMultiple StateMultipleRolls ReportStabInjury DispatchDumpOff).

- [x] INVESTIGATED (2026-08-19): (a) the multiblock STEP family is bb2020/bb2025-only (Java
      generators; bb2016 multi-block = MULTI_BLOCK_DEFENDER_ID params on ordinary Block seqs —
      different mechanism, никогда dispatches the family). Horkon (darkelf.Horkon, the only
      Multiple Block star in the data) is hostable in bb2020 dark_elf — his skills all resolve
      in bb2020 (Multiple Block/Dodge/Leap/Shadowing/Stab/Loner) and the star plumbing is
      edition-agnostic. (b) Pro (bb2016+bb2020; bb2025 has a different Pro class): StepId.PRO
      dispatches from ONE site — StepHandleDropPlayerContext.handleCommand: a DROPPED player's
      skill whose InjuryContextModification requiresConditionalReRollSkill() (Helmut's Old Pro)
      is offered via a SkillUse dialog; USING it pushes the PRO step (the conditional d6).
      ParityRunner's SKILL_USE arm auto-uses. **Rust's bb2020 step_handle_drop_player_context.rs
      documents the gap**: 'requires Sequence/StepId::Pro wiring and a skill→modification lookup
      that don't exist'. dark_elf bb2020 is green only because Helmut never got dropped with the
      modification applicable in seeds 1-100 — a live landmine, not a safe stub. (c) the client
      declares MULTIPLE_BLOCK via sendActingPlayer(player, MULTIPLE_BLOCK, false); target
      selection runs through SynchronousMultiBlockLogicModule dialogs.
- [x] Multiple Block LIVE (2026-08-19): darkelf.Horkon drafted @nr2 bb2020 dark_elf (blitzer
      2→15, witchelf 1→11); **dark_elf bb2020 100/100** with BlockRollMultiple /
      FoulAppearanceMultiple / DispatchDumpOff / DauntlessMultiple / ApothecaryMultiple all
      dispatching. NINE fixes (uncommitted pending gate):
      [ffb-rust] (1) PAC::MultipleBlock plumbing + offer (legal_actions: CAN_BLOCK_TWO_AT_ONCE,
      >1 adjacent blockables) + two-phase declaration in shared InitSelecting (ActivatePlayer arm
      + Action::MultiBlock = Java CLIENT_SYNCHRONOUS_MULTI_BLOCK :308-316 publish BLOCK_TARGETS +
      dispatch) + MultiBlockTargets continuation window (start guard AND execute tail);
      (2) Action::EndPlayerAction deselect arm (stale-target infinite loop, seed 2);
      (3) one-roll-per-pass push_self next_step in BOTH step_block_roll_multiple twins — batching
      both eval sequences' pushes+publishes in one outcome parked the 2nd BlockChoice promptless
      (Java interleaves per roll; publish delivery is consume-scoped); (4) first_run captures each
      target's pre-block state into BlockRolls (BLOCK_TARGETS is id-only; the restore wrote
      unwrap_or_default() = PlayerState(0) → targets teleported to Reserve, seed 10);
      (5) bb2020 DropFallingPlayers publishes the DEFENDER's DropPlayerContext directly (Java
      PilingOnBehaviour :157; SteadyFooting is bb2025-only — the wrap only ever worked in
      sequences that CONTAIN a SteadyFooting step, which the MB eval seq does not) AND
      (6) publishes the attacker INJURY_RESULT for bb2016|bb2020 (:183), seed 5;
      (7) **MB dice-count folds ASSISTS** (seed 10 second red): the roll phase passed raw
      strengths to find_nr_of_block_dice — port of Java findNrOfBlockDice full path
      (getTotalAttackerStrength + findBlockStrength both sides, MB modifiers on base strength,
      using_multi_block=false to avoid double-apply — same shape as bb2016 bug #12). NOTE:
      Java's isValidAssist MB-target exclusion is a NO-OP in this flow — the synchronous command
      never calls addMultiBlockTarget, only the GUI's incremental SET_BLOCK_TARGET does;
      (8) **driver.rs constructed StepApothecaryMultiple with new(String::new())** (seed 53):
      team_id pre-set to Some("") disabled the acting-team resolution (guarded on is_none) → the
      retain filter compared "" to real team ids and silently DROPPED every acting-team injury —
      the MB attacker's KO was never applied and he stood up next turn (Java: KO box). Fixed to
      ::default() (team_id None). Bug shape: a bad FACTORY DEFAULT that poisons a
      resolve-on-first-run guard — invisible until the step first processes an acting-team injury.
      (9) 2 stale tests updated to the Java-faithful bb2020 no-SteadyFooting expectation.
      [ParityRunner] MB offer + isHandled + phase-2 sendSynchronousMultiBlock (2 actionRng picks
      idx%N, idx%(N-1) over coordinate-sorted blockables; BlockTarget(id, BLOCK, state)) +
      RE_ROLL_BLOCK_FOR_TARGETS handler (first needsSelection roll → die index 0; decline
      DialogReRollForTargetsParameter). Tests: multi_block_dice_count_folds_assists,
      acting_team_resolution_applies_attacker_ko. ffb-engine 7242/0.
- [x] Full 3-edition gate with the MB fixes: 30/30 ×3, committed `316c774a` + pushed. A follow-up
      wave the same day fixed three more reds the MB drafts surfaced (apothecary factory default,
      MB assist fold, bb2016 HMP natural-1 — see the commit).
- [x] Pro route LIVE (2026-08-20, dark_elf bb2020 100/100 + full gate 30/30 ×3). The whole
      InjuryContextModification pipeline now runs: registry `modification_for_skill` +
      `unused_injury_modification` (Java Player.getUnusedInjuryModification; 16 behaviours
      mapped), modification machinery converted u16→SkillId, and
      `modification_aware_handle_injury` executes Java :36-57/:70-75 (attacker lookup, defender
      fallback when is_chainsaw/is_vomit_like — Helmut's kickback; alternate context resolved via
      std::mem::swap with armour_roll(roll=false) re-evaluation — block/chainsaw/foul now
      re-interpret EXISTING dice instead of bailing; modify_injury re-interprets via the new
      `interpret_and_set_injury` split of do_injury_roll). Live HandleDropPlayerContext: report +
      SkillUse dialog (Continue), UseSkill → push Sequence[Pro] + push_self (agents send a
      PLACEHOLDER SkillId::Block — self.skill is authoritative), SUCCESSFUL_PRO stored as
      pending_pro and applied at re-execute (set_parameter has no game — the ApothecaryMultiple
      lesson again), swap_to_alternate_context ported onto injury::InjuryResult.
      **VACUOUS-GREEN LESSON**: the first "100/100 with Pro fielded" had ZERO OldPro events —
      Helmut at nr10 never swung; moved to nr3 (assassin→10) and the board LIT UP with 5 real
      engine bugs:
      (1) eligibleForPro used the bare `acting.has_acted` field — Java's
      hasActedIgnoringNegativeTraits is computed from moved/fouled/blocked/passed sub-flags;
      (2) StepPro's post-decline re-execute rolled a SECOND Pro die — Java's useReRoll(PRO)
      gates on `canRerollOncePerTurn && !hasUsedPro`;
      (3) OldPro modify_armour skipped Java's base-pipeline steps: DiceInterpreter-equivalent
      recalc (armour WITH modifiers + edition predicate — bb2020 breaks on >=) and the
      CLEAR-armour-modifiers before the reroll;
      (4) the transient usedPro BIT is cleared for every player at each TURN START
      (UtilPlayer.refreshPlayersForTurnStart :420-425) — Rust kept it for the game, so Old Pro
      offered once per game instead of once per turn-context;
      (5) **the SHARED Collections-shuffle stream is fed by list SIZES**: Rust's Stiletto
      selector passed added_skills=[Stab] where Java's RandomSelectionPrayerHandler.addedSkills
      is EMPTY (only bb2025 BadHabits overrides) → 7-element shuffle vs Java's 8 → every later
      shuffle-driven pick diverged (seed 31: the next Cheering-Fans prayer became IRON_MAN).
      Companion fixes: bb2020 mapSIRoll now Collections-shuffles the reduceable SI list
      (previously "first reduceable"); `game.collections_rng` is now RefCell<JavaRandom> so
      &Game-only paths (the SI remap) can draw. Debug tooling that cracked it: simulating
      java.util.Random + Collections.shuffle in python and brute-forcing the shuffle-size
      HISTORY that reproduces each engine's observed permutation ([16,8] Java vs [16,7] Rust).
      Tests: 3 lookup tests, modified_injury_context_offers_skill_use_then_pushes_pro_and_swaps;
      engine 7247/0, model 1160/0, mechanics 2796/0.
- [ ] Inventory update (multiblock family reached; ReportStabInjury/StateMultipleRolls status).
- [ ] Cosmetic: no `skillUse` GameEvent is emitted for the Old Pro dialog (coverage counts it
      as unexercised even when it fires); wire the event when coverage reporting needs it.

## 11. bb2025 star-special batch (USER-SELECTED 2026-08-20)

Goal: switch on the remaining star-special dead steps by ADDING bb2025 star data (the current
`data/star_players/all_editions.json` lacks the bb2025-only stars entirely) and drafting one
carrier per special into a hosting bb2025 team. Per star: add the data entry (skills must
resolve in BOTH engines — canonical Java names verified below), draft at/near the LOS, rerun
`scripts/gen_java_parity_data.py`, run the host matchup 1-100, **verify the special actually
FIRES (vacuous-green check: grep traces/events)**, fix divergences Rust-side, gate.

Carriers (from rules/star_players/*.md; Java canonical skill names in quotes):
- Batch A — CLIENT_USE_SKILL channel (plumbing exists from §9):
  - [x] Ivar Eriksson LIVE (2026-08-20, `bfb382c1` + harness commit): human bb2025 100/100 with
        RaidingParty events in 99/100 games; full gate 30/30 ×3. Surfaced the missing GUARD
        assist clause in find_block_strength (counts while marked). Port lessons: MoveSquare
        dodge/GFI flags are FUNCTIONAL (resetState must updateMoveSquares); activation prompt
        lists come from eligible_players_for_activation (its PAC→PA map too); a step whose next
        wait publishes only MoveSquares leaves the answered PLAYER_CHOICE dialog set — the
        harness clears it locally.
  - [x] Boa Kon'ssstriktr LIVE (2026-08-20): drafted @nr2 bb2025 lizardman, 100/100 on fresh
        Java logs, full gate 30/30 ×3. StepLookIntoMyEyes was already fully ported — only
        offers/declaration/data were dead. Vacuous-green check: the condition (activation while
        adjacent to the ball carrier) is genuinely RARE — 1 declaration in 100 games (seed 13)
        is the expected exposure, not a wiring failure. Harness lesson: the LIME failure notice
        is a DialogInformationOkay — ParityRunner's default routes unknown dialogs to the
        NON-SEEDED RandomStrategy, so INFORMATION_OKAY joined the clear-only group (Rust's
        agent acknowledges it with zero RNG draws; clearing is the deterministic mirror).
  - [x] Estelle la Veneaux LIVE (2026-08-20): drafted @nr2 bb2025 amazon (catcher 2→13),
        100/100 fresh Java logs, full gate 30/30 ×3, hex fires in 96/100 games (182
        declarations). THREE engine fixes: (1) Java skill enhancements can grant BARE
        PROPERTIES — hex success adds a hasToMissTurn temporary property to the target; the
        model gained Player.temporary_properties (Java Player.temporaryProperties) and the
        bb2020/bb2025 mechanic enhancement-removal sets were TODO-empty; (2) find_block_strength
        guard-cancel streams over ALL defensiveAssists INCLUDING THE DEFENDER — a Defensive
        defender cancels a marked Guard assist (only the marker COUNT excludes the defender);
        (3) refreshPlayersForTurnStart's playerOnTeamFromLastTurn is the ASYMMETRIC
        `team != home && isHomePlaying`, not `is_home != home_playing` — the symmetric form
        left a hexed HOME player's enhancement stuck across the half boundary. Harness lesson:
        --reuse-java after a draft change reds 90 seeds at step 1 — regenerate Java logs after
        ANY data change.
  - [x] Rodney Roachbait LIVE (2026-08-20): drafted @nr2 bb2025 wood_elf (wardancer 2→12),
        100/100 fresh Java logs, full gate 30/30 ×3, CotD fires in 53/100 games. Step was
        already a full port — only offers/declaration/data were dead; zero engine fixes
        needed (the RaidingParty/LIME/BalefulHex plumbing pattern applied cleanly).
  - [x] Zzharg Madeye LIVE (2026-08-21): drafted @nr2 bb2025 chaos_dwarf (bull centaur 2→12),
        100/100 fresh Java logs, full gate 30/30 ×3, Blastin' fires in 98/100 games. — BATCH A
        COMPLETE. Two engine fixes: (1) StepEndThenIStartedBlastin only PUBLISHED
        EndPlayerAction/EndTurn — nobody consumed them, so the star stayed ACTIVE after his
        shot; Java clears the stack and PUSHES the EndPlayerAction sequence. (2) Java
        ActingPlayer.markSkillUsed puts the skill in the ACTING PLAYER's used set — that set is
        what makes hasActed() true, which is what deactivates the player on activation end;
        Rust only marked the Player. New two-phase target-wait contract
        (AgentPrompt::BlastinTarget, no Java dialog): initial pick = STANDING opponents within
        3 of the star; roll-2 replacement = either team within 3 of the ORIGINAL target, star
        excluded, OPPOSING coach picks; coordinate-sorted single actionRng pick, EndTurn when
        empty. Harness lessons: RE_ROLL_PROPERTIES decline left a stale dialog spinning on a
        CONTINUE-after-fail step; the fix must clear ONLY IF SAME OBJECT — the injection can
        synchronously run the whole turn end and show the half-boundary ARGUE_THE_CALL dialog,
        which an unconditional clear wiped (END_TURN stuck 501 iters).
- Follow-up (found during Zzharg): `StepOutcome::clear_stack` is a DEAD FLAG — the driver
  never consumes it, so every `with_clear_stack()` site (LIME blitz-cancel, reset-to-move,
  EndThenIStartedBlastin) is inert. Parity holds today because those steps sit last in their
  sequences, but Java's `getStepStack().clear()` also wipes OUTER stale steps. Wire the flag
  through the driver 1:1 as its own item with its own full gate.
- Batch B — own step families:
  - [x] Cindy Piewhistle LIVE (2026-08-21): drafted @nr2 bb2025 halfling, 100/100 fresh Java
        logs, full gate 30/30 ×3, All You Can Eat fires in 89/100 games. SIX engine fixes:
        (1) PlayerAction DELEGATE resolution — ALL_YOU_CAN_EAT("allYouCanEat", 39, ..,
        THROW_BOMB) stores its delegate in changeActingPlayer, so everything downstream sees a
        plain bomb; (2) StepEndBomb's two-bomb chain was collapsed away in the PassState-stub
        era — ported 1:1 (markUsed + mustCompleteAction + second Pass sequence; then
        push_self + StepAllYouCanEat for the 4+ sent-off roll); (3) StepAllYouCanEat read the
        stub-era thrower_id proxy (cleared by EndBomb) instead of game.original_bombardier —
        the sent-off roll silently never rolled; (4) the MIXED Accurate pass modifier
        (bb2020/bb2025, -1 on QUICK/SHORT, !ttm) was missing — an old test even asserted
        "Accurate should not appear in BB2025"; (5) mixed Cannoneer (-1 LONG/LONG_BOMB)
        likewise missing; (6) the AYCE-failure eject pushed [EjectPlayer, Bribes] but Java's
        raw LIFO pushes run BRIBES (the argue-the-call d6) FIRST — Rust ejected without
        arguing, one die behind. Also widened StepInitPassing's BombRethrow prompt to ANY
        thrower==null park (ParityRunner's INIT_PASSING contract), covering the second AYCE
        bomb's fresh Pass sequence.
  - [x] Guffle Pusmaw ("Quick Bite") LIVE (2026-08-24): drafted @nr2 bb2025 nurgle (warrior
        2 → 13), 100/100 on fresh Java logs, full gate 30/30 ×3, Quick Bite fires in 5/100
        games (both teams' Guffles). Baseline after the draft was 96/100 — TWO engine fixes:
        (1) **the offer was dead.** Java shows a DialogSkillUseParameter (1 opponent) or a
        DialogPlayerChoiceParameter(QUICK_BITE) (2+) and CONTINUEs; Rust returned a bare
        `next()` behind a "client-only … headless falls through" comment — recurring bug
        shape #3, so nothing ever declared it (seed 32 i=4: Java spent 2 dice on the bite's
        armour roll that Rust never rolled). Ported both dialogs, added a
        `SelectPlayer{""}` decline arm (ParityRunner's PLAYER_CHOICE default sends an empty
        selection — without it a declined choice re-prompts forever) and a QuickBite arm in
        random_agent echoing the real skill (the HitAndRun lesson: the generic
        `SkillId::Block` placeholder fails the step's property check and refires the dialog).
        (2) **wrong Java constructor overload.** Java passes the 7-arg
        `DropPlayerContext(injury, false, false, null, catcherId, QUICK_BITE, true)` whose
        LAST argument is `requiresArmourBreak`; Rust called the `with_injury` shorthand whose
        4th argument is `eligibleForSafePairOfHands`, so it set the wrong flag and left
        `requiresArmourBreak` false — the drop step then knocked the catcher down on an
        UNBROKEN armour roll (seed 32 i=5: Java a00 Standing, Rust a00 Prone). Also ported
        the missing TOUCHBACK branch (`turnMode == KICKOFF && !bounds.isInBounds(ball)`),
        which Rust had dropped entirely. Data lesson: the canonical Java skill name is
        **"On The Ball"** (capital T), not the rulebook's "On the Ball".
  - [x] Swiftvine Glimmershard ("Furious Outburst") LIVE (2026-08-24): drafted @nr3 bb2025
        wood_elf (treeman 3 → 13, benched), 100/100 on fresh Java logs, full gate 30/30 ×3.
        FOUR dead ids switched on at once (InitFuriousOutburst, FirstMove/SecondMove,
        EndFuriousOutburst). Emphatically live: **241 executions in 95/100 games**, both engines
        agreeing exactly (241 JAVA_FO_PICK vs 241 Rust declarations, plus 30 matched stale-offer
        deselects). Progression 100/100 VACUOUS → 2 → 7 → 64 → 100 real.
        The mechanic was dead in THREE independent places, which is why no earlier sweep caught
        it: (a) neither engine OFFERED the action (it is a PlayerAction, FURIOUS_OUTPBURST — the
        typo is Java's — declared by the client as a plain sendActingPlayer and dispatched from
        StepEndSelecting; Rust legal_actions had the other seven bb2025 star specials but not
        this one, and ParityRunner's hand-maintained offer list did not either); (b)
        StepInitFuriousOutburst returned a bare `cont()` behind a "client-only … headless falls
        through" comment — the driver's STALL shape, so even a declared action would hang; (c)
        both move steps did the same (they publish MoveSquares and wait for
        CLIENT_FIELD_COORDINATE).
        THREE engine/agent bugs, each with a regression test:
        1. **Declared then instantly deselected.** `is_handled_acting_action` (Rust's mirror of
           ParityRunner.isHandledActingAction) omitted it, so Rust declared the action and threw
           it away while Java carried it out (seed 1 i=23). SAME SHAPE that kept Kick Team-Mate
           dead in every edition — the two lists are hand-maintained on both sides and nothing
           cross-checks them.
        2. **`blitzUsed` never consumed.** Furious Outburst counts as the team's Blitz Action;
           StepEndFuriousOutburst sets it, but gated on the BARE `has_acted` field where Java's
           `hasActed()` is COMPUTED (hasMoved||hasFouled||hasBlocked||…). The stab sets
           has_blocked, never the bare flag (seed 1 i=24: Java `f0000,1100` vs Rust `f0000,0100`),
           so the star could outburst every turn. Fixed at all 3 sites (bb2020+bb2025 end steps,
           FirstMove's wasted-skill report) — `acting_player.acted()` already existed for exactly
           this and its doc comment already warned the bare field is wrong here.
        3. **Stale turn-start offer → STOCK-JAVA NPE.** ParityRunner.computeEligiblePlayers
           snapshots eligibility at TURN START, but the blitz gets spent (and targets walk away)
           later in the turn. Declaring a stale outburst CRASHES the stock engine: every abort
           path in the bb2025 sequence jumps to the `END` label, which IS StepEndFuriousOutburst,
           and that step dereferences
           `fieldModel.getTargetSelectionState().getSelectedPlayerId()` unconditionally
           (NullPointerException at StepEndFuriousOutburst:71 — it killed the batched JVM at seed
           65 i=190, state `f1000,0000`, so seeds 65-100 reported `java=None` and looked like 36
           separate bugs). The real client never reaches it because SelectLogicModule
           re-evaluates isFuriousOutburstAvailable at CLICK time. Both agents now re-check at
           DECLARATION time and deselect — the same treatment sendFoulAction gives a foul whose
           victim has moved. The rule is factored into
           `legal_actions::is_furious_outburst_available` so the offer and the re-check cannot
           drift apart. **This is a latent crash in the shipped Java server, not just a harness
           quirk** — left unfixed here because ffb-common/ffb-server engine code is off-limits.
        Contract note: Java's `findEligiblePlayers` returns a **HashSet**, so the dialog's player
        order is identity-hash order and is NOT a contract (unlike BALEFUL_HEX/RAIDING_PARTY).
        Both sides COORDINATE-SORT before the single actionRng pick, following the
        pickBlockTarget/ANIMAL_SAVAGERY precedent. The move steps' abort is also NOT EndTurn:
        Java's step has no CLIENT_END_TURN handler, only a null-action CLIENT_ACTING_PLAYER, so
        an empty square list ends the PLAYER ACTION on both sides.
        Test lessons: the pre-existing `marks_blitz_used_when_has_acted` test set `has_acted`
        directly, so it passed while the real path was broken — a test pinning the wrong thing.
        And `PS_STANDING` does NOT imply the ACTIVE bit (a separate bit set by the turn-start
        refresh), which is a fixture trap for any rule checking isActive().
  - [x] Thorsson Stoutmead ("Beer Barrel Bash!") LIVE (2026-08-24): drafted @nr2 bb2025 dwarf
        (blitzer 2 → 12, benched), 100/100 on fresh Java logs, full gate 30/30 ×3.
        `ThrowKeg` + `EndThrowKeg` both live: **273 executions in 100/100 games**, both engines
        agreeing exactly (273 JAVA_KEG_PICK vs 273 Rust declarations, plus 59 matched no-target
        deselects). Progression 100/100 VACUOUS → 0 → 100 real.
        DATA TRAP: the canonical Java skill name is **"Beer Barrel Bash!" WITH the trailing
        exclamation mark** (skill/mixed/special/BeerBarrelBash), while rules/star_players spells
        it without one. Second name mismatch in three iterations after "On The Ball" (capital T)
        — the rules markdown is NOT a source for Java skill names, always read the Skill class.
        DECLARATION SHAPE (differs from every previous star): the client sends TWO commands —
        `sendActingPlayer(player, THROW_KEG)` to enter ClientStateId.THROW_KEG, then
        `sendThrowKeg(target)` once the coach clicks. BOTH land in StepInitSelecting, and the
        second publishes **TARGET_PLAYER_ID (not defenderId)**, which StepEndSelecting reads
        straight into ThrowKeg.SequenceParams. Rust folds the pair into ONE ActivatePlayer
        carrying the target and unfolds it back onto TARGET_PLAYER_ID at dispatch. Targets are
        ThrowKegLogicModule.isValidTarget (<=3 steps, STANDING, opposing team), coordinate-sorted,
        single actionRng draw on both sides; both agents DESELECT when no valid target exists
        (the client would have no square to click, so the declaration never completes).
        Offer rule is LogicModule.isThrowKegAvailable — REGULAR turn mode, base STANDING, unused
        canThrowKeg — and deliberately has NO target clause, matching Java.
        ONE engine bug, a REPEAT of a known shape:
        1. **StepEndThrowKeg was PUBLISH-ONLY.** Java calls
           `endPlayerActionGenerator.pushSequence(new EndPlayerAction.SequenceParams(gameState,
           false, true, endTurn))`; Rust merely published EndPlayerAction/EndTurn under a comment
           claiming "the driver owns the sequence stack". Nothing consumes those parameters and
           EndThrowKeg is the LAST step of the keg sequence, so the stack emptied with the
           activation still open and the driver STALLED (Continue with no prompt) — every dwarf
           bb2025 game ended at the first keg (seed 1 i=88; rust_total collapsed ~9s → 1.4s,
           0/100). This is the SAME publish-only shape as StepEndThenIStartedBlastin's Zzharg
           fix; it survived because the keg mechanic had never once executed. Unlike the Blastin
           step, Java's keg end-step does NOT clear the step stack — verified, not assumed.
        TEST LESSONS (second time in two iterations that a passing test pinned the wrong thing):
        two PRE-EXISTING step_end_throw_keg tests asserted on the PUBLISHED EndTurn parameter,
        i.e. they pinned the publish-only bug itself, and passed the whole time because the keg
        never ran. Rewritten to assert end_turn reaches the PUSHED sequence. And `Game::new`
        defaults to `TurnMode::StartGame`, not Regular — the second fixture trap after
        "PS_STANDING does not imply the ACTIVE bit". Default Game state does not resemble a live
        turn; any rule keyed on turn mode or state bits needs the fixture set explicitly.
  - [x] Grombrindal ("Wisdom of the White Dwarf") LIVE (2026-08-24): drafted @nr3 bb2025 dwarf
        (blitzer 3 → 13, benched, alongside Thorsson @nr2), 100/100 on fresh Java logs, full gate
        30/30 ×3. **245 Wisdom executions in 99/100 games** (and the keg rose to 283 in 98/100),
        both engines matching exactly. Progression 0 → 52 → 91 → 98 → 100. **BATCH B COMPLETE.**
        THE SHARED CHANNEL WAS THE REAL WORK. Java has ONE
        `DialogSelectSkillParameter(playerId, List<Skill>, SkillChoiceMode)` serving TWO mechanics
        — the Intensive Training prayer (INTENSIVE_TRAINING) and Wisdom
        (WISDOM_OF_THE_WHITE_DWARF) — answered by ClientCommandSkillSelection (Java reuses the
        CLIENT_PRAYER_SELECTION command id). Both call sites pass a FLAT list sorted by skill NAME.
        Rust modelled it as `available: Vec<(SkillCategory, Vec<u16>)>`, a category grouping with
        NO Java counterpart, and the consequences were visible in the code itself: step_prayer had
        to BUILD the grouping ("the prompt groups them by category because that is the shape
        AgentPrompt::SelectSkill takes" — the step bending to the prompt, not to Java); both
        agents immediately flattened and re-sorted it; uniform_agent gave up entirely, answering
        Acknowledge because "the prompt's ids are raw Java-side u16s with no lookup table back to
        SkillId anywhere in the codebase" (false — SkillFactory is exactly that, and random_agent
        already used it); and step_wisdom parked on a bare `cont()`, the stall shape. Reshaping
        the variant to Java's actual form (player_id, skill_ids, reason) fixed a stall, deleted an
        invented data structure and un-stubbed a second agent in one change.
        THREE ENGINE BUGS, each with a regression test:
        1. **The WISDOM PlayerChoice had a harness arm but no Rust agent arm.** Rust answered the
           MANDATORY dialog (minSelects=1) with an empty selection, skipped the grant, then took a
           different move square because Java had spent an actionRng draw it had not (seed 2 i=5).
           LESSON: every dialog needs TWO arms and the failure is quiet — the mechanic still
           appears to run on both sides and surfaces only as a downstream position difference.
        2. **StepThrowKeg marked the PLAYER's used-skill set, not the ACTING PLAYER's.** Java's
           `actingPlayer.markSkillUsed(skill)` is what makes hasActed() true, which is what
           deactivates the player at activation end — Thorsson stayed ACTIVE after his keg and
           could be activated again (seed 14 i=80: Java `a01:...,0` vs Rust `a01:...,1`). This is a
           bug in the code THIS CAMPAIGN shipped one iteration earlier, and iteration 3 gated
           30/30/30 with it present: a green gate bounds the seeds tested, not the code. Same
           Zzharg lesson recurring; `Game::mark_skill_used` sounds complete but is half of Java's.
        3. **GRANTED SKILLS NEVER EXPIRED — the most consequential bug of the campaign so far.**
           Java's two enhancement families key their removal DIFFERENTLY and the difference is
           load-bearing: `FieldModel.addWisdomSkill` tags the grant with
           `wisdomSkill.enhancementSourceName()` ("Granted by Wisdom of the White Dwarf") while
           `addSkillEnhancements` (Baleful Hex) tags with `skill.getName()`. Accordingly
           `GameMechanic.enhancementsToRemoveAtEndOfTurn` maps via **Skill::enhancementSourceName**,
           NOT getName(). Rust's set (bb2020 AND bb2025) held the plain skill name — its own
           comment said "via SkillFactory.forClass(..).getName()", so the original author read the
           wrong mapper. The key matched nothing, so a granted Break Tackle / Dauntless /
           Mighty Blow / Sure Feet survived EVERY later turn instead of expiring at end of turn: a
           permanent silent buff. Seed 14 showed it as a dodge Java failed on a 5 and Rust passed,
           three home turns after the grant (i=151 grant, i=184 dodge). Pinned with tests in both
           editions INCLUDING a negative assertion that the plain name is not used, plus a
           companion test pinning Baleful Hex's opposite convention so the two are never "unified".
        HARNESS: `SELECT_SKILL` had NO arm in ParityRunner, so it fell through to the
        **non-seeded RandomStrategy** — the default's own comment calls that "silent
        nondeterminism for parity". That affected the Intensive Training prayer too, which is
        already reachable today; it now has a deterministic arm (lowest skill name, zero rng)
        matching both Rust agents. Also added the WISDOM PlayerChoice arm (coordinate-sorted
        single actionRng pick — findStandingOrPronePlayers' order is not a documented contract),
        the isWisdomAvailable offer after THROW_KEG, and the ActingPlayer(MOVE) +
        ClientCommandUseTeamMatesWisdom pair (Java sets only the DISPATCH action here and never
        calls changeActingPlayer, so the declared action stays MOVE).
        PROCESS: a data edit must refresh BOTH halves — rebuild the Rust binary (data is
        `include_str!`-compiled) AND rerun gen_java_parity_data.py. I drafted Grombrindal, rebuilt
        Rust but forgot the XML, and got 0/100 at step 0 on every seed: Rust offered 4 actions
        where Java offered 3, so the shared draw hit x%4 vs x%3. Iteration 2 hit the same trap in
        the opposite direction. Either way it looks exactly like a mass engine divergence.
        GATE NOTE: the combined `--edition all` run died silently twice (15 and 43 matchups, all
        green, no GATE line, no error). Neither counts as a measurement. Running the three
        editions as SEPARATE invocations completed cleanly every time — prefer that.
- Batch C — already in data, just dormant:
  - [x] Zolcath the Zoat ("Excuse Me, Are You a Zoat?" → AutoGazeZoat) LIVE (2026-08-24):
        **182 executions in 100/100 games**, dark_elf bb2025 100/100 on the FIRST run, full gate
        30/30 ×3, **ZERO engine bugs** — the step was fully ported and only the offer and
        declaration were missing, exactly like Rodney Roachbait in Batch A. **§11 COMPLETE.**
        **CORRECTION to this entry as written:** it said "bb2020 star … → a bb2020 team", but
        `AutoGazeZoat` — step AND generator — exists ONLY in bb2025 in both engines
        (`step/bb2025/StepAutoGazeZoat.java`, `generator/bb2025/AutoGazeZoat.java`). A bb2020 host
        would never have reached the step. Drafted into **bb2025 dark_elf** @nr2 instead (witchelf
        2 → 12; he is available_for Dark Elf, and nr2 is an LOS jersey so opponents are within 3
        every drive). The skill IS registered for both editions, but in bb2020
        `canGazeAutomatically` is consumed by Black Ink / SelectGazeTarget instead.
        THE NEAR-MISS TO KNOW: the Zoat keys on **canGazeAutomaticallyThreeSquaresAway** while
        BLACK INK keys on **canGazeAutomatically**, and both sit on the SAME CLIENT_USE_SKILL
        dispatch chain in StepInitSelecting (Java :399 Black Ink, :407 the Zoat). Confusing them
        silently routes one mechanic into the other; pinned with a test asserting the Zoat is
        offered as AutoGazeZoat and NOT as BlackInk.
        Declared like Black Ink: ActingPlayer(MOVE) + ClientCommandUseSkill(zoat), Java setting
        only the DISPATCH action with forceGotoOnDispatch and never calling changeActingPlayer.
        Target dialog is a PlayerChoice(AUTO_GAZE_ZOAT) which had NO ParityRunner arm (→ the
        non-seeded RandomStrategy); added coordinate-sorted single-actionRng arms on BOTH sides.
        Firing evidence: 182 Rust declarations vs 181 JAVA_ZOAT_PICK dialogs — the step
        auto-picks when only one target is eligible, so the two counts agree.
        SIGNAL: the first four iterations each exposed engine bugs because they touched NEW
        plumbing (dialogs, sequences, enhancements); this one reused a route Black Ink had already
        proven and found none. **The bugs cluster where the plumbing is novel, not where the star
        is exotic.**
        FOLLOW-UP (not this iteration): Rust's `SkillId::ExcuseMeAreYouAZoat.properties()`
        registers BOTH edition properties unconditionally (`canGainGaze` for bb2020 AND
        `canGazeAutomaticallyThreeSquaresAway` for bb2025) where Java splits them across two
        edition-specific Skill classes. Inert today because bb2020 has no AUTO_GAZE_ZOAT dispatch,
        but it is the same shape as the bb2016-only `canGazeDuringMove` trap already recorded in
        legal_actions (an ungated property regressed vampire bb2025 from 0 to 100 fails).

Availability is league-rule based and the harness does not validate it — host wherever
convenient (precedent: Farblast into bb2016 dwarf). Stars ride outside the 1.1M budget.

## 12. Blitz/gaze SELECT sub-chain — the last substantial engine-fidelity gap (SCOPED 2026-08-24)

`SelectBlitzTarget SelectBlitzTargetEnd SelectGazeTarget SelectGazeTargetEnd`. Everything below is
measured or read from source, not assumed.

**The gap.** Java runs a two-phase blitz declaration on EVERY blitz; Rust folds the target into the
declaration and skips the chain entirely. Both spend exactly one `actionRng` target pick and reach
the same board state, so the matrices are 100/100 and the state hash cannot see the difference —
the same blind-spot class as the ACTIVE bit and `ttm_used`/`ktm_used`, but on the most frequent
action in the game. Measured: ~750 `JAVA_BLITZ_TARGET` selections per 100 games (dark_elf and dwarf
bb2025), versus a Rust drive trace of the same matchup showing only `InitSelecting` /
`EndSelecting` / `RemoveTargetSelectionState`.

**Java's flow** (bb2025; bb2020 is the twin):
1. Client sends `ClientCommandActingPlayer(pid, BLITZ_MOVE)`.
2. `StepInitSelecting` :114 — `if (playerAction == BLITZ_MOVE && targetSelectionState == null)`
   → `fDispatchPlayerAction = BLITZ_SELECT`, `changeActingPlayer(pid, BLITZ_MOVE, jumping)`
   (the ACTING action stays BLITZ_MOVE), `forceGotoOnDispatch = true`.
3. `StepEndSelecting` case `BLITZ_SELECT` → pushes the SelectBlitzTarget sequence:
   `SELECT_BLITZ_TARGET [SELECT]` → ActivationSequenceBuilder (the negatrait rolls) → `JUMP_UP` →
   `STAND_UP` → `SELECT_BLITZ_TARGET_END [END_BLITZING]`.
4. `StepSelectBlitzTarget` CONTINUEs waiting for the target; on selection it sets
   `TurnMode.SELECT_BLITZ_TARGET`, adds the blitz-target bit to the victim's PlayerState, builds a
   `TargetSelectionState(...).select()` (committing it when the acting player `hasActed()`), emits
   `ReportSelectBlitzTarget`, and applies any used skill's enhancements (Frenzy/Claws for blitz).
   With no standing opponents it instead sets `TargetSelectionState().skip()` and NEXT_STEPs.
5. Second pass: `targetSelectionState != null`, so BLITZ_MOVE dispatches normally to BlitzMove.
6. `StepSelectBlitzTargetEnd` is where Java **consumes the team blitz** (`setBlitzUsed`).

**What Rust already has — MOST OF IT.** An earlier estimate in this campaign called this
"high risk, expect the whole matrix to red, multi-iteration" on the assumption that steps needed
porting. That was wrong and is corrected here: `PlayerAction::BlitzSelect` exists;
`StepEndSelecting` already has a fully-ported, UNIT-TESTED `BlitzSelect` arm that pushes
`SelectBlitzTarget::build_sequence`; both generators, both step files, the dialog parameter and the
report are all present.

**What is actually missing — three things:**
1. **The routing branch.** `StepInitSelecting` has ZERO references to `BlitzSelect`. Add Java's
   :114 branch (untargeted BLITZ_MOVE → dispatch BLITZ_SELECT, force goto).
2. **The step body is a stub.** `bb2025/step_select_blitz_target.rs::execute_step` returns
   `StepOutcome::next()` where Java returns CONTINUE and waits. It needs the real body from (4)
   above: the wait+prompt, the skip path, the cancel path, the TargetSelectionState, the
   blitz-target state bit, the report, and the used-skill enhancements.
3. **The agents.** They must declare Blitz with NO folded target (so `targetSelectionState` is
   null on arrival) and answer the new target prompt.

**Why the RNG stream should survive — the key insight.** In Java the target is picked by
`SELECT_BLITZ_TARGET`, which sits BEFORE the ActivationSequenceBuilder negatrait rolls in the
sequence. In Rust today the agent picks the blitz target at ACTIVATION time, also before the
negatraits. Same relative position, one `actionRng` draw either way — which is exactly why parity
currently holds despite the different code paths. Keep the draw where it is in stream order and the
matrices should not move. That makes this a bounded fidelity change rather than a re-green
campaign, though it still touches every roster's most common action, so gate all three editions.

**Answer contract:** `ParityRunner.sendBlitzTargetSelection` → `pickBlockTarget` — adjacent
opponents whose state base is STANDING or MOVING (NOT hasTackleZones, no confused check),
coordinate-sorted, single `actionRng` pick; it already has handlers for `SELECT_BLITZ_TARGET` as
BOTH a step (:805) and a dialog (:859), so the Java side needs no work.

**Gaze twins:** `SelectGazeTarget`/`SelectGazeTargetEnd` are the same shape for bb2020 hypnotic
gaze — do the blitz pair first, then mirror.

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


### §12 progress log (branch `wip/blitz-select-chain`)

- `a05fe892` — chain WORKS: lineman bb2025 **0/20 → 6/20**. Full Java flow now runs
  (SelectBlitzTarget → negatraits → JumpUp → StandUp → SelectBlitzTargetEnd → Select →
  InitSelecting → EndSelecting → InitMoving → EndMoving → InitBlocking → EndBlocking).
  Five fixes, each traced not guessed: the InitSelecting BLITZ_SELECT routing branch; the real
  `step_select_blitz_target` body (was a stub returning `next()`); the post-BLITZ_SELECT
  continuation arm; `SelectBlitzTargetEnd`'s Select loop-back; the agent issuing the block at the
  Move prompt; `StepEndMoving::start` honouring a published `DispatchPlayerAction`; and threading
  `game.defender_id` into `BlitzBlockParams`.
- **Recurring shape in this area:** the capability existed and was CORRECT but was unreachable
  from the entry point the path actually uses (`EndMoving::start` vs `handle_command`;
  `BlitzBlock` defender via step parameter vs `..Default::default()`). Probe, don't read.
- **NEXT LEAD (14 seeds still failing, e.g. seed 3 at step 6):** the failing blitzes take a
  DIFFERENT sequence. On the passing seed 1, `EndSelecting` pushes a 27-step sequence and
  `InitMoving` prompts for a Move; on seed 3 it pushes **43** steps and `InitMoving` falls straight
  through into `InitActivation` with no prompt, so the blitz ends before blocking. Java blocks
  normally there (`currentMove=3 MA=6`). Suspect the prone / stand-up variant: compare what
  `EndSelecting` pushes for BLITZ_MOVE when the blitzer must stand up first, and check whether
  `standing_up` is still set from the pre-select pass.

**§12 divergence CONFIRMED as the PRONE blitzer (2026-08-24).** Java's own trace settles it:
`currentMove=3 MA=6`, and 3 is `MINIMUM_MOVE_TO_STAND_UP` — seed 3's blitzer stood up. The Rust
step ORDER shows where the difference lives: on the failing seed the activation steps
(`InitActivation`, `AnimalSavagery`, `SteadyFooting`, …) run AFTER `InitMoving`, whereas Java's
sequence is `SELECT_BLITZ_TARGET → ActivationSequenceBuilder → JUMP_UP → STAND_UP →
SELECT_BLITZ_TARGET_END` — the stand-up belongs INSIDE the select sequence, which is exactly the
order the PASSING seed shows (BoneHead/ReallyStupid/… → JumpUp → StandUp → SelectBlitzTargetEnd).

So for a prone blitzer Rust stands the player up in the MOVE sequence instead of the SELECT
sequence. Two things to check: (a) whether the new BLITZ_SELECT routing branch in
`StepInitSelecting` must preserve `standing_up` — Java calls
`changeActingPlayer(.., playerAction, isJumping())` and its pre-stand block is gated on
`playerAction.isMoving() || isStandingUp()`, and BLITZ_MOVE *is* moving; and (b) whether the
`JumpUp`/`StandUp` steps inside Rust's SelectBlitzTarget sequence actually stand him up.

**CORRECTION (2026-08-24): it is NOT the prone blitzer.** The previous note inferred that from
Java's `currentMove=3 MA=6`, reading 3 as `MINIMUM_MOVE_TO_STAND_UP`. A probe disproves it:

    BZPROBE EndSelecting BlitzMove seq_len=28 first=Some(InitMoving) standing_up=false prone=Some(false)

The blitzer is standing, not standing up, and `EndSelecting` pushes a perfectly ordinary 28-step
BlitzMove sequence starting with `InitMoving` — identical to the passing seed. `currentMove=3` was
just movement already spent, not a stand-up cost. Another reminder that an inference from a single
number is not a diagnosis.

**The REAL finding: stack RESIDUE.** `InitMoving` reports `stack_len=43` while the pushed sequence
is only 28, so ~15 steps were ALREADY on the stack when the loop-back pushed BlitzMove. The
passing seed shows 27 (i.e. 28 minus the running step) with no residue. Those leftover steps are
what run as `InitActivation`/`AnimalSavagery`/… after `InitMoving`, and they are why the blitz
never reaches its block.

NEXT: find who leaves ~15 steps behind. Most likely the SelectBlitzTarget sequence is not fully
consumed before `SelectBlitzTargetEnd` pushes Select — Java's `StepSelectBlitzTarget` CLEARS the
step stack on its end-turn/end-action path (`getGameState().getStepStack().clear()`), and Rust's
`with_clear_stack()` is only applied on that same path. Check whether the SELECTED path also needs
to clear, and note that `StepOutcome::clear_stack` was itself recorded as a DEAD FLAG in the Zzharg
work (BACKLOG §11) — verify the driver actually consumes it before relying on it.

**§12 status after the residue hunt (2026-08-24).** The blitz path is STRUCTURALLY IDENTICAL
between a passing and a failing seed: both lineman bb2025 seed 1 and seed 3 enter `InitMoving`
with `pa=Some(BlitzMove)` exactly **13** times. So the two-phase declaration, the select sequence,
the loop-back and the block dispatch all behave the same; what remains is SITUATIONAL, not a
missing piece of the chain.

Two measurement cautions learned here, both of which cost a wrong turn:
- `stack_len` in `FFB_DRIVE_TRACE` is not a simple depth you can subtract. The 27-vs-43 reading
  led to a "residue" theory that the push probe contradicted (`seq_len=28`, ordinary `InitMoving`
  first, identical to the passing seed). Do not diagnose from that counter alone.
- Sampling the FIRST few probe lines is meaningless here: a single seed produces ~1,023
  `InitMoving` entries. Filter to the case of interest (`pa=Some(BlitzMove)`) and COUNT before
  concluding anything.

NEXT: stop sampling and diff. Take failing seed 3, find the FIRST diverging activation by
`rng_calls` (it is i=6, where Java spends 3 dice — block + 2 armour — and Rust spends 0), then dump
only that activation's probe lines from both engines. The blitz at i=6 selects
`def=teamLinemanParityAway1` in Java; check whether Rust's `SelectBlitzTarget` offers the same
single candidate (`JAVA_BLOCK_PICK N=1`) and whether its block then finds a defender.

**§12 NARROWED to an exact discrepancy (2026-08-24): 8 → 5.** Probes on failing lineman bb2025
seed 3:

    BZPROBE SBTEnd ... tss=Some(SELECTED) defender=Some(...)   x8   (all SELECTED, all with a defender)
    BZPROBE InitSelecting enter pa=Some(BlitzMove)             x5
    BZPROBE InitSelecting continuation HIT pa=BlitzMove        x5
    BZPROBE EndSelecting BlitzMove seq_len=28 ...              x5

So **eight** blitzes select a target and reach `StepSelectBlitzTargetEnd` on the SELECTED branch,
but only **five** arrive at `StepInitSelecting` carrying `BlitzMove` — three lose the action
between `SelectBlitzTargetEnd` setting it and the pushed Select sequence reading it. Those three
are the failing blitzes: they fall through to a fresh activation and never block.

The prone theory is dead for good: all five EndSelecting pushes in this seed are byte-identical
(`standing_up=false prone=Some(false)`), and there are only five lines total, so this is not the
earlier sampling error.

NEXT: instrument `StepSelectBlitzTargetEnd`'s SELECTED branch itself — print
`game.acting_player.player_id` and the action IMMEDIATELY BEFORE and AFTER
`change_player_action(.., BlitzMove, ..)`, and again after `push_seq`. Candidates for the loss:
(a) `player_id` is None for those three so the change is skipped (the `if let Some(pid)` guard);
(b) something between the push and `InitSelecting` resets the action (`changeActingPlayer` clears
state — see the `reset_blocked_and_moving_players` precedent); (c) those three are the OTHER
team's blitzes and the acting player is swapped before the Select sequence runs.

**§12 handoff instrumented (2026-08-24): the write is CORRECT; the loss is after the push.**

    BZPROBE SBTEnd SELECTED pid_before=Some("away_02") pid_after=Some("away_02") pa_after=Some(BlitzMove)   x4
    BZPROBE SBTEnd SELECTED pid_before=Some("home_03") pid_after=Some("home_03") pa_after=Some(BlitzMove)   x2
    BZPROBE SBTEnd SELECTED pid_before=Some("home_01") ... pa_after=Some(BlitzMove)                          x1
    BZPROBE SBTEnd SELECTED pid_before=Some("home_02") ... pa_after=Some(BlitzMove)                          x1

All eight set `BlitzMove` on a valid acting player, so candidate (a) from the previous note — the
`if let Some(pid)` guard being skipped — is RULED OUT. The action is lost between
`StepOutcome::next().push_seq(select_seq)` here and `StepInitSelecting` reading it.

**New signal: `away_02` appears FOUR times.** A player should blitz once per turn and this step
sets `blitz_used`, so repeats mean the failed blitzes are being RE-DECLARED — the activation never
completes, the player stays available, and the agent offers Blitz again. That reframes the 8-vs-5
gap: it is likely 5 real blitzes plus 3 retries of ones that died, not 8 distinct blitzes of which
3 fail. Confirm by logging the turn number alongside each SBTEnd.

NEXT: find out whether the pushed Select sequence RUNS at all for the lost three. Log a counter in
`StepInitSelecting::start` unconditionally (not just the BlitzMove arm) and compare its count
against the 8 SBTEnd pushes; if Select ran but `pa` was not BlitzMove, something between the push
and the read resets it (`changeActingPlayer` clears state — cf. `reset_blocked_and_moving_players`);
if Select did NOT run, the push itself is being dropped, which would point at the driver's handling
of `push_seq` from a step that is the last in its own sequence — exactly the position
`SelectBlitzTargetEnd` occupies.
