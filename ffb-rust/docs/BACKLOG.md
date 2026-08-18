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

## 2. BB2020 deflection fidelity, then re-enable interception

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
- [ ] Green the remaining TWO seeds, then gate and commit. After the narrowed CatcherId fix:
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

- [ ] Compare Rust's bb2016 catch modifier set against Java's `modifiers/bb2016/CatchModifierCollection`
      and `bb2016/AgilityMechanic.minimumRollCatch`, exactly as was done for the interception minimum
      (fix 5). Print the individual modifiers, not just the total. Then re-check `elf bb2016 83`,
      which may share the cause.

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

## 3. Audit the bare skill-roll comparisons

`crates/ffb-engine/src/dice_interpreter.rs` already has the correct rule, yet ~25 sites still compare
`roll >= min_roll` directly (bone head, really stupid, wild animal, blood lust, take root, dauntless,
jump up, chainsaw, GFI, shadowing, …). One of them was a real bug this session (fix 6 above).

- [ ] For each site, check whether the Java counterpart calls `isSkillRollSuccessful`. Fix only those.

**Do not blanket-change them.** Only some Java call sites use that rule, and a sweeping edit is exactly
the "measured worse" trap this project has hit twice. Fix a site when a red seed points at it and the
Java call site is verified — not before.

---

## 4. Remaining dead-step targets

Work in this order; each needs the trigger verified as reachable *before* any harness plumbing.

- [ ] `SelectGazeTarget` / `SelectGazeTargetEnd` — all editions; `HypnoticGaze` itself already runs.
- [ ] `DauntlessMultiple` — `Dauntless` itself runs.
- [ ] The bomb chain — `InitBomb`, `EndBomb`, `ResolveBomb`, `Bombardier2`.
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
