# PARITY_BB2016_CAMPAIGN.md — drive all 30 bb2016 mirror matchups to 100/100

GOAL: `docs/TEAM_MATRIX_BB2016.md` all 30 rosters 🟢 100/100 (mirror, `--edition bb2016 --tier 3 --seeds 1-100`).

Ground rules (see `docs/PARITY_PROCESS.md`, non-negotiable):
- **Java is the truth.** Never edit `ffb-java/ffb-common` or `ffb-java/ffb-server` engine code.
  Co-editable: Rust `crates/*`, `random_agent.rs`, harness `ParityRunner.java` (needs a jar rebuild).
- Every Rust change is a **1:1 port** of the corresponding Java class/method. No hacks, no
  parity-only special-cases. Read the Java, port the Java.
- Every fix lands with a colocated `#[cfg(test)]` regression test.
- Verify advance **and** no regression (lineman bb2016+bb2025 100/100, `cargo test -p ffb-engine`)
  before committing. REVERT if regressed.

## Run commands

From `ffb-rust/ffb-rust`:

```bash
cargo build --release -p ffb-parity
# full 1-100 for one roster, counting every failing seed (do NOT trust parallel/timeout triage)
./target/release/ffb-parity --home R --away R --edition bb2016 --tier 3 --seeds 1-100 --no-abort
# single seed, first divergence only
./target/release/ffb-parity --home R --away R --edition bb2016 --tier 3 --seeds N-N
```

**NEVER run two parity runs of the SAME matchup concurrently** (even different editions): both
write `parity/<home>_vs_<away>/seed_N_{java,rust}.jsonl`, so they clobber each other and produce
bogus mass failures (observed: lineman bb2016 reported 95/100 fails, actually 0). Different
matchups run in parallel safely — they use different directories.

Tracing: `FFB_TRACE=1` (RUST_STEP/JSTEP state strings), `FFB_DICE_TRACE=1` (per-die + Java
`caller=` stack), `FFB_DRIVE_TRACE=1` (Rust step order / stalls). Redirect to a file — the
volume is large. Reliable signal = per-step `rng_calls` deltas + state-string diffs, NOT
`DICE_TRACE pos=` (Java counts per call, Rust per die).

## Status (update every iteration)

Baseline entering this run (post-commit `5e86d749`): **19 🟢 / 11 🔴**.

| Roster | fails /100 | Diagnosis | State |
|---|---:|---|---|
| renegades | **0** (38→8→1→0) | 🟢 100/100 GREEN (ITER55 TTM routing, ITER56 declined-re-roll, ITER57 RightStuff dropPlayer) | GREEN |
| underworld | **0** (44→8→1→0) | 🟢 100/100 GREEN (ITER55-56 TTM + ITER58 bb2016 InitPassing routing) | GREEN |
| necromantic | **0** (58→1→0) | 🟢 100/100 GREEN (ITER59 + ITER60 Stand Firm) | GREEN |
| undead | 76 | stand-up-blitz-GFI (ITER51 diagnosis) | queued |
| dwarf | **30** (79→35→30) | ITER59 Stand Firm + ITER61 bb2016 casualty-SW argue — **still NEXT TARGET** | queued |
| elf | 84 | untraced (suspect AG / pass) | queued |
| ogre | 98 | earlier non-TTM blocker | queued |
| wood_elf | 98 | untraced | queued |
| goblin | 100 | earlier non-TTM blocker masks the TTM win — retrace seed 1 | queued |
| halfling | 100 | systematic (every seed) — likely a roster/skill-load or first-step gap | queued |
| vampire | 100 | systematic — Bloodlust bb2016 | queued |

Counts above re-scouted 2026-08-13 AFTER ITER56-58 with FULL 1-100 `--no-abort` runs (no timeout).
The older `undead 44` / `necromantic 44` figures were unreliable (truncated triage) — the true
counts are 76 / 58. Green rosters re-verified 0 fails in the same sweep.

Green (22): necromantic, renegades, underworld, lineman, amazon, chaos, chaos_dwarf, chaos_pact, dark_elf, dark_elf_league_fumbbl,
high_elf, human, khemri, khemri_fumbbl, lizardman, nippon, norse, nurgle, orc, skaven, slann,
slann_fumbbl.

## Iteration protocol

1. Pick the roster with the FEWEST fails (cheapest green), unless a shared root cause makes a
   higher-fail roster higher leverage (a fix hitting 4 rosters beats one hitting 1).
2. Run its lowest failing seed alone; find the first divergent step.
3. Root-cause to ONE Java-vs-Rust difference. Read the Java class named in the trace.
4. Port it 1:1 + regression test. `cargo test --release -p ffb-engine`.
5. Re-run the roster 1-100 `--no-abort`; confirm the fail count DROPS and lineman stays 100/100.
6. Commit with an explicit path list (never `git add -A` — it would sweep `parity/*.jsonl` and
   `.claude/worktrees`). Append the diagnosis here and update the table + matrix doc.
7. Chain to the next frontier.

## Iteration log

(append newest at the bottom)

### ITER56 (2026-08-13) — bb2016 TTM declined re-roll must keep `re_rolled_action`

Java `skillbehaviour/bb2016/ThrowTeamMateBehaviour.handleExecuteStepHook` gates on
`ReRolledActions.THROW_TEAM_MATE == step.getReRolledAction()` and, when the source is null or the
re-roll is unusable, `GOTO_LABEL(goToLabelOnFailure)` — it NEVER clears `reRolledAction`. Rust's
`bb2016/ttm/step_throw_team_mate.rs::handle_command` cleared BOTH `re_rolled_action` and
`re_roll_source` on `UseSkill{false}` / `UseReRoll{false}`, so `execute_step` re-entered as a FRESH
throw and rolled the accuracy d6 a second time — an extra die that desynced the shared stream (and
often flipped a fumble into a spurious success + 3× scatter). The parity harness always declines
team re-rolls (`ParityRunner.sendUseReRoll(action, null)`), so a fumbled bb2016 TTM must stay
fumbled. FIX: clear only `re_roll_source`. Test
`declined_reroll_keeps_action_clears_source_and_gotos_failure`.

IMPACT (measured, much larger than expected): **renegades 8 fails → 1** (only seed 80 left),
**underworld 8 → 1** (only seed 72). The spurious second accuracy die was the dominant residual
TTM desync for both rosters.

### ITER57 (2026-08-13) — bb2016 StepRightStuff failed landing must `dropPlayer` → **renegades 100/100 GREEN**

renegades seed 80, first state mismatch i=62; the real divergence is step 61's resolution
(home_04 Troll THROW_TEAM_MATE of the ball-carrying Goblin h10 at (4,6)). State-only + Rust one
die SHORT. `FFB_DICE_TRACE` caller stacks pinned it exactly:

| pos | Java caller | Rust |
|---|---|---|
| 57 | `ThrowTeamMateBehaviour$1.handleExecuteStepHook:78` accuracy d6=1 → FUMBLE | same |
| 58 | `StepRightStuff.executeStep:135` landing d6=2 → fail, re-roll declined | same |
| 59-60 | `InjuryTypeTTMLanding.handleInjury:34` armour 2d6 | same |
| **61** | **`StepCatchScatterThrowIn.scatterBall:446` d8=1 — the ball bounce** | *(missing)* — Rust's pos 61 is already the next activation's Bone Head d6 |

ROOT CAUSE: Java's `StepRightStuff.executeStep()` `if (!doRoll)` block publishes the injury result
AND calls `UtilServerInjury.dropPlayer(this, thrownPlayer, THROWN_PLAYER)`, publishing its returned
parameters. `dropPlayer` sets `fieldModel.ballMoving = true` and returns
`CATCH_SCATTER_THROW_IN_MODE = SCATTER_BALL` whenever the dropped player's square equals the ball
square — that parameter is what makes the sequence's following CATCH_SCATTER_THROW_IN step bounce
the ball. It also returns `END_TURN` (turnover) for the acting team's own carrier, which Java keeps
when `fThrownPlayerHasBall` and removes otherwise. Rust's `land_injury` published only
`ThrownPlayerCoordinate(None)`, so the ball stayed under the prone Goblin and the bounce d8 was
never rolled.

FIX: port the block 1:1, mirroring the already-correct bb2020 translation. Also collapsed the
duplicated `drop_thrown_player` injury branch into the same `land_injury` path — Java has ONE
injury site (`doRoll = !fDropThrownPlayer` falls through to `if (!doRoll)`), not two.

Tests: `failed_landing_of_ball_carrier_drops_player_and_requests_ball_scatter`,
`failed_landing_without_ball_scatters_but_does_not_end_turn`.
Verified: renegades 1 fail → **0 (100/100)**; lineman bb2016 100/100; lineman bb2025 100/100;
`cargo test -p ffb-engine` green. Commit `a1db7893`. **20 🟢 / 10 🔴.**

NOTE (latent, not hit yet): Rust `util_server_injury::drop_player_with_base` omits Java's
`&& game.getTurnMode() != TurnMode.BLITZ` guard on the ball-scatter branch, and adds a
`FieldCoordinateBounds::FIELD.is_in_bounds` early return Java does not have. Worth checking when a
BLITZ-turn-mode drop shows up as a frontier.

### ITER58 (2026-08-13) — the bb2016 PASS step-set was routed to the bb2025 steps → **underworld 100/100 GREEN**

underworld seed 72. First state mismatch i=78, but the first `rng_calls` divergence is i=75
(Java 52 / Rust 53): step 74 (`Activate(away_03, PASS)`) rolled one die in Rust and none in Java.

Java trace: `JAVA_PASS pid=…Away3 coord=(25,7)` then `UNHANDLED_STEP: INIT_PASSING`. The thrower is
at (12,9) — 13 squares away — so Java's `StepInitPassing.executeStep()` finds
`findPassingDistance(...) == null`, no branch matches, and it returns WITHOUT calling
`setNextAction`. The step stays current; ParityRunner has no `INIT_PASSING` case, so the stuck step
falls to its `default:` branch and injects `ClientCommandEndTurn` → turnover, ball unmoved, zero
dice. **This is not a harness gap** — it is stock Java's behaviour for an out-of-range throw, and
the runner's EndTurn is just how the contract resolves it.

Rust trace: `RUST_STEPPASS thrower=away_03 … pass_coord=(25,7) dist=None` — emitted from
`step/bb2025/pass/step_pass.rs`. ROOT CAUSE: `bb2016::move_::step_end_selecting` already pushes the
**bb2016 Pass sequence**, but `make_step_for(id, Rules::Bb2016)` had no pass entries, so the driver
instantiated the **bb2025** step classes for every StepId in it. The shared
`mixed::pass::step_init_passing` range-checks with `ffb_model::util::passing::passing_distance`
(the bb2020+ table), which accepts (dx 13, dy 2); the bb2016 `PassMechanic::find_passing_distance`
throwing-range table rejects it. So Rust accepted the throw, rolled the accuracy d6 and offered an
interception that stock Java never rolls.

FIX (two parts):
1. `bb2016/pass/step_init_passing.rs`: the no-branch-matched fall-through now produces the same
   observable result as Java-plus-runner — `GOTO_LABEL(gotoLabelOnEnd)` + `END_TURN(true)` — instead
   of `Continue`, which the headless driver cannot resolve (identical to how the already-verified
   shared `mixed` step handles it).
2. `driver.rs` `make_step_for`: route `StepId::InitPassing` to the bb2016 impl for bb2016 games.

**Scope lesson — routing the WHOLE bb2016 pass step-set REGRESSED badly and was reverted.** Adding
`Pass`/`Intercept`/`HailMaryPass`/`EndPassing`/`PassBlock` alongside `InitPassing` took lineman
bb2016 from 0 → 38 fails and underworld from 1 → 33: those five bb2016 step files are less complete
than the bb2025 ones they were shadowing. Only `InitPassing` is routed. The other five remain a
known gap — port them individually, each verified on its own, not as a block.

Tests: `out_of_range_pass_ends_the_turn_without_rolling` (bb2016 InitPassing).
Verified: underworld 1 fail → **0 (100/100)**; lineman bb2016 100/100. **21 🟢 / 9 🔴.**

### ITER59 (2026-08-13) — bb2016 Stand Firm must publish `FOLLOWUP_CHOICE = false` → necromantic 58 → 1, dwarf 79 → 35

necromantic seed 3 step 3: home_01 (Werewolf — Claw/**Frenzy**/Regeneration) blocks away_03
(Flesh Golem — Regeneration/**Stand Firm**/Thick Skull). Java rolled 8 dice, Rust 2.

Java (`JAVA_BLOCKROLL nDice=-2` twice): block dice 4,5 → defender picks 4 = Pushback → Stand Firm
avoids the push → **Frenzy second block** (dice 2,6 → 2 = Both Down) → armour 2d6 for BOTH players
→ turnover. End state: `a02:13,8,Prone` and `h00:12,7,Prone` — nobody moved.

Rust: one block, no Frenzy, and end state `a02:13,8,Standing` **and `h00:13,8,Standing`** — two
players stacked on one square, the attacker having followed up onto the defender it never pushed.

ROOT CAUSE: Java's `skillbehaviour/bb2016/StandFirmBehaviour` publishes TWO parameters when the
push is avoided — `STARTING_PUSHBACK_SQUARE = null` **and `FOLLOWUP_CHOICE = false`**. Rust's
bb2016 hook set `do_push` / cleared the squares but never published `FollowupChoice(false)`, and
the bb2016 `StepPushback` never drained `hook_state.published` at all. So `StepFollowup` moved the
attacker onto the defender's square. The knock-on: bb2016 `StepEndBlocking`'s `forceSecondBlock`
(Frenzy) branch requires `attackerPosition.isAdjacent(defenderPosition)`, and a co-located attacker
is not *adjacent* — so Frenzy silently stopped firing. One missing publish, two visible symptoms.

The shared bb2025 `StandFirmBehaviour` already did `state.published.push(FollowupChoice(false))`
and the bb2025 `StepPushback` already drained `hook_state.published`; the bb2016 pair was the gap.

FIX: `skill_behaviour/bb2016/stand_firm_behaviour.rs` pushes `StepParameter::FollowupChoice(false)`;
`step/bb2016/step_pushback.rs` drains `hook_state.published` into its outcome (mirroring bb2025).

Test: `stand_firm_suppresses_the_attackers_followup`.
Verified: necromantic 58 → **1**; **dwarf 79 → 35** (shared win — the Deathroller has Stand Firm);
no regression across all 28 swept bb2016 rosters; lineman bb2016 100/100; ffb-engine 7090/0.

### ITER60 (2026-08-13) — bb2016 Stand Firm must also clear the pending pushback STACK → **necromantic 100/100 GREEN**

necromantic seed 70 step 27 (the last necromantic failure after ITER59). home_02 (Werewolf,
Frenzy) blitzes away_02 at (12,7). Java rolled 4 dice (two `JAVA_BLOCKROLL nDice=2`), Rust 2.
Final states: **Java — nothing moved at all** (`a01:12,7`, `h01:11,6`, byte-identical hash);
**Rust — `a01:12,8` AND `h02:12,8`**, two players stacked on one square, and no Frenzy re-block.

The push square the harness chose, (12,8), was **already occupied by home_03 — a Flesh Golem,
which has Stand Firm**. So this is a CHAIN push: Java's
`doPush = (fieldModel.getPlayer(lastPushback.getCoordinate()) == null)` is false, the step
re-enters to push the occupant first, the occupant stands firm, and Java's `StandFirmBehaviour`
calls **`state.pushbackStack.clear()`** — discarding the original defender's already-chosen move.
Nothing moves; the attacker (FOLLOWUP_CHOICE=false from ITER59) stays at (11,6), still adjacent to
the un-pushed defender at (12,7), so `forceSecondBlock` (Frenzy) fires for the second block roll.

Rust's hook cleared `pushback_squares` (the CANDIDATES) but the pushback STACK lives on the step,
not the hook state, and was never cleared — so the `do_push` branch applied the pending push anyway.

FIX: new `clear_pushback_stack` flag on the shared `StepPushbackHookState` (default false); the
bb2016 `StandFirmBehaviour` sets it, and the bb2016 `StepPushback` clears `self.pushback_stack`
when it is set. Scoped to bb2016 — the bb2025 behaviour is untouched.

Test: `stand_firm_clears_the_pending_pushback_stack`.
Verified: necromantic 1 → **0 (100/100)**; no regression across all 29 swept bb2016 rosters;
lineman bb2016 100/100; lineman bb2025 100/100; ffb-engine 7091/0. **22 🟢 / 8 🔴.**

KNOWN GAP (not fixed, bb2025 is green so it is latent there): the bb2025 `StandFirmBehaviour` has
the same missing `pushbackStack.clear()`. Worth porting if a bb2025 chain-push-onto-Stand-Firm
frontier ever appears.

Remaining reds after ITER60: dwarf 35 · undead 76 · elf 84 · ogre 98 · wood_elf 98 ·
goblin 100 · halfling 100 · vampire 100. **dwarf is the next target.**

### ITER61 (2026-08-13) — Secret Weapon eligibility is edition-specific → dwarf 35 → 30

dwarf seed 3: first `rng_calls` divergence at i=156 (Java 83 / Rust 85) — step 155 is the LAST turn
of half 1, so this is the halftime Secret Weapon send-off. `FFB_DICE_TRACE` callers:
Java pos 78 AND 79 are both `DiceRoller.rollArgueTheCall:141 StepEndTurn.argueTheCall:540`;
Rust rolled only ONE d6 there, so its half-2 kickoff scatter d8 landed at pos 79 instead of 80 and
read the wrong shared-stream position.

State diff at the frontier: `pa00:-1,-1,Injured` — the AWAY Deathroller is a **casualty** at
halftime. Java i=156 has it as `-1,-1,Reserve` (PS_BANNED renders as Reserve); Rust left it
`Injured`. So Java argued for the casualty and banned it; Rust skipped it entirely.

ROOT CAUSE: Java's `StepEndTurn.getPlayerIds` eligibility filter differs by edition —
- bb2025: `!PlayerState.REMOVED_FROM_PLAY.contains(playerState.getBase())`, plus the IllBeBack
  (`ignoreFirstSecretWeaponSentOff`) opt-out → a casualty secret weapon is neither argued nor banned.
- bb2016: `playerResult.hasUsedSecretWeapon() && playerState.getBase() != PlayerState.BANNED` **only**
  → a casualty secret weapon IS argued for (one d6) and IS set BANNED; bb2016 has no IllBeBack clause.

bb2016 games run the SHARED (bb2025) `StepEndTurn`, which applied the bb2025 filter in both the argue
phase and `removeUsedSecretWeapons`. FIX: edition-gate that filter (`game.rules == Rules::Bb2016`) in
both phases of `step/bb2025/step_end_turn.rs`.

Test: `casualty_secret_weapon_is_argued_and_banned_only_in_bb2016` (asserts exactly 1 argue d6 +
BANNED under bb2016, and 0 dice + still-a-casualty under bb2025).
Verified: dwarf 35 → **30**; no regression across all 28 swept bb2016 rosters; lineman bb2016
100/100; lineman bb2025 100/100; **goblin bb2025 100/100** (the bb2025 secret-weapon roster);
ffb-engine 7092/0.

**FAILED APPROACH, REVERTED (record so it is not retried):** routing `StepId::EndTurn` to
`step/bb2016/step_end_turn.rs` took lineman bb2016 from 0 → **100** fails. That file is an early
translation whose own doc-comment lists ArgueTheCall / secret weapons / prayers / per-drive reroll
removal / fainting as untranslated stubs — it is far LESS complete than the bb2025 step it would
shadow. Porting bb2016's `reportSecretWeaponsUsed` / `argueTheCall` / `removeUsedSecretWeapons` into
it first did not help, because the step is not routed. **The bb2016 end-turn/pass/block divergences
must be fixed by edition-gating the SHARED bb2025 steps, not by routing the bb2016 files** — same
lesson as ITER58's pass step-set. This now applies to: `StepEndTurn`, and the five bb2016 pass steps
(`Pass`/`Intercept`/`HailMaryPass`/`EndPassing`/`PassBlock`).

STILL OPEN for bb2016 secret weapons (not needed by this frontier, likely needed for goblin 100):
Java bb2016 `argueTheCall` rolls one d6 **per player id in the client command**, and ParityRunner
sends only the FIRST eligible player, once per team — so bb2016 should roll AT MOST ONE argue die per
team. The shared step loops over every flagged player. Dwarf has one Deathroller per team so both
readings agree; goblin bb2016 (Looney + Bombardier + Fanatic) will not.
