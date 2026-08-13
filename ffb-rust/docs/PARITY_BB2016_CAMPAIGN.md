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
| undead | **0** (76→2→0) | 🟢 100/100 GREEN (ITER64 prone-Blitz GFI + ITER65 Blizzard GFI modifier) | GREEN |
| dwarf | **0** (79→35→30→0) | 🟢 100/100 GREEN (ITER59 Stand Firm, ITER61 casualty-SW argue, ITER63 KO-vs-argue order) | GREEN |
| elf | **0** (was 84) | 🟢 100/100 GREEN (ITER66 Side Step auto-use) | GREEN |
| ogre | **0** (was 98) | 🟢 100/100 GREEN (ITER69 bb2016 TTM spends the PASS; needed a jar rebuild) | GREEN |
| wood_elf | **0** (98→81→19→0) | 🟢 100/100 GREEN (ITER71 startedStanding, ITER73 rooted pre-draw, ITER77 declined-re-roll edition gate) | GREEN |
| goblin | **99** (was 100) | ITER78 PETTY_CASH + casualty dice, ITER79 Ball & Chain drop, ITER80 unhandled-action deselect; seed 1 now fails at i=123 (half 2) | in progress |
| halfling | 100 | same PETTY_CASH block as goblin (treasury 180k) — unblocked by ITER78, needs a re-scout | queued |
| vampire | 100 | systematic — Bloodlust bb2016 | queued |

Counts above re-scouted 2026-08-13 AFTER ITER56-58 with FULL 1-100 `--no-abort` runs (no timeout).
The older `undead 44` / `necromantic 44` figures were unreliable (truncated triage) — the true
counts are 76 / 58. Green rosters re-verified 0 fails in the same sweep.

Green (27): wood_elf, ogre, elf, undead, dwarf, necromantic, renegades, underworld, lineman, amazon, chaos, chaos_dwarf, chaos_pact, dark_elf, dark_elf_league_fumbbl,
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

### ITER62 (2026-08-13) — dwarf seed 5 halftime: DIAGNOSIS ONLY, no fix landed

dwarf seed 5 (lowest of the remaining 30). First STATE mismatch i=129, first `rng_calls`
divergence i=131. Step 128 is the last turn of half 1, so this is again the halftime transition —
and **the dice match exactly through pos 81** (both engines roll the same 14 dice in step 128).
It is a pure state divergence, then step 130 rolls 5 dice in Java and 0 in Rust.

`FFB_DICE_TRACE` callers for the halftime dice (Java): pos 71 =
`rollKnockoutRecovery / StepEndTurn.recoverKnockout:496`, pos 72 = `rollArgueTheCall` (**6** →
success), pos 73 = `rollArgueTheCall` (**3** → fail). `FFB_DRIVE_TRACE` shows Rust rolling all
three inside its `EndTurn` step too — so Rust IS rolling both argue dice now (ITER61 landed) and
the KO-recovery die, in the same order.

Order-independent state facts at i=129 (counting off-field entries, NOT trusting the label→player
mapping):
- Java: 1 off-field player total — one `-1,-1,Reserve` (a banned Deathroller) on ONE team; no `Ko`.
- Rust: 2 off-field — one `-1,-1,Reserve` AND one `-1,-1,Ko`, both in the same team's window.

So there are (at least) TWO layered divergences at this halftime:
1. the argue success/failure appears to land on a different team's Deathroller, and
2. Rust still has a KO'd player where Java has none — i.e. the single KO-recovery die produced a
   different outcome, or the engines disagree on how many players were KO'd going into halftime.

**METHOD WARNING (cost most of this iteration):** the `state=` string's `aN`/`hN` labels are
**positional, not id-based**, and the two engines order the list differently once players are
off-field — Java's window was `[banned, 10 on-pitch]` while Rust's was `[banned, Ko, 9 on-pitch]`
with a different 11th player. Do NOT conclude "engine X banned team Y's player" from these labels.
Next iteration should get the identity directly instead: read `ReportSecretWeaponBan` /
`ReportArgueTheCallRoll` player ids from BOTH sides (the Rust `parity/*_rust_events.jsonl` argue
entry for this seed is the FOUL argue after `refereeSpotsFoul` — a different mechanic — so a
secret-weapon-specific trace is needed), or add a gated JAVA_ARGUE/JAVA_SWBAN eprintln pair.

Also still open from ITER61 (probably the same root cause as (1) above): bb2016 `argueTheCall`
rolls one d6 **per id in the client command** and ParityRunner sends only the FIRST eligible player
once per team, so bb2016 should roll AT MOST ONE argue die per team; the shared bb2025 step loops
over every flagged player.

No code change; dwarf stays at 30. 22 🟢 / 8 🔴 unchanged.

### ITER63 (2026-08-13) — KO-recovery vs Secret-Weapon-argue ORDER is edition-specific → **dwarf 100/100 GREEN**

Continuing ITER62's dwarf seed 5 halftime (step 128, dice identical, state divergent). The three
halftime dice are **4, 6, 3**. Java's `FFB_DICE_TRACE` callers: pos 71 = `rollKnockoutRecovery`
(4 → recovers), pos 72 = `rollArgueTheCall` (6 → away keeps its Deathroller), pos 73 =
`rollArgueTheCall` (3 → home's Deathroller banned).

Rust's observed halftime state was exactly what you get by spending those same dice in the OTHER
order: 4 and 6 on the two argues (away fails → banned, home succeeds → keeps) and 3 on the KO
recovery (< 4 → stays down). That accounted for BOTH of ITER62's "layered" symptoms at once — the
ban on the wrong team AND the leftover `Ko` — so it was one bug, not two.

ROOT CAUSE (grep of both Java sources):
- bb2016 `StepEndTurn`: `recoverKnockout` (line **281**) runs BEFORE `reportSecretWeaponsUsed`
  (364) → `askForSecretWeaponBribes` (373/380) → `askForArgueTheCall` (387/395) →
  `removeUsedSecretWeapons` (404).
- bb2025 `StepEndTurn`: `reportSecretWeaponsUsed` (**415**) → argue (427/442) → bribes (454/462) →
  `removeUsedSecretWeapons` (471) → **then** KO recovery/fainting via `getFaintingCount` (478 → 597).

The two editions run these two rng-consuming blocks in **opposite order**, and bb2016 games use the
shared bb2025 step. (The bribes/argue order also swaps, but bribes roll no dice without an AVOID_BAN
inducement, so they do not shift the stream in parity.)

FIX: extracted the KO-recovery/fainting block from `step/bb2025/step_end_turn.rs` into
`recover_knockouts_and_fainting(...)` plus a `ko_recovery_done` guard, and call it BEFORE the Secret
Weapon send-off when `game.rules == Rules::Bb2016`, leaving the bb2025 call site where it was.

Test: `bb2016_recovers_knockouts_before_the_secret_weapon_argue`.
Verified: dwarf 30 fails → **0 (100/100 GREEN)**; no regression across all 28 swept bb2016 rosters;
lineman bb2016 100/100; lineman/goblin/dwarf bb2025 all 100/100; ffb-engine 7093/0. **23 🟢 / 7 🔴.**

LESSON: when the dice VALUES match but the state diverges at a drive end, suspect the ORDER of two
rng-consuming blocks rather than either block's logic — and check the Java line numbers in both
editions, because this step's phase order is not shared.

Remaining reds: undead 76 · elf 84 · ogre 98 · wood_elf 98 · goblin 100 · halfling 100 ·
vampire 100. **undead is the next target.**

### ITER64 (2026-08-13) — a prone Blitz must set going-for-it (stand-up eats the move) → undead 76 → 2

undead seed 1, step 187: `Activate(away_01, Blitz)` where away_01 is **PRONE** (`pa00:12,6,Prone`) —
a stand-up Blitz by a Mummy (MA 3). Java spends 7 dice, Rust 4:

| | Java | Rust |
|---|---|---|
| pos 93 | `rollGoingForIt / StepGoForIt.goForIt:163` | *(missing)* — Rust's 93 is already block die 1 |
| 94-95 | block dice (`nDice=2`) | 94 = block die 2, 95 = armour die 1 |
| 96-97 | armour 2d6 = 4+6 = 10 → BROKEN | 95+96 = 1+4 = 5 → held, so no injury at all |
| 98-99 | injury 2d6 | — |

ROOT CAUSE: Java's `StepInitSelecting.prepareStandingUp()` gates its stand-up branch on
`actingPlayer.getPlayerAction().isMoving()`, and **both the GUI client and ParityRunner declare a
Blitz as `BLITZ_MOVE`** (`declared = (action == BLITZ) ? BLITZ_MOVE : action`) — and `BLITZ_MOVE`
IS moving while plain `BLITZ` is not (`PlayerAction.isMoving()`, ffb-common). That branch is what
sets `currentMove = min(MINIMUM_MOVE_TO_STAND_UP, MA)` and
`goingForIt = UtilPlayer.isNextMoveGoingForIt(game)`; for a standing-up player
`isNextMoveGoingForIt` returns `3 >= MA`, so a Mummy (MA 3) must Rush to make its blitz block.
Rust stores the declared action as `PlayerAction::Blitz`, whose `is_moving()` is false, so the whole
branch was skipped: `goes_for_it` stayed false and `StepGoForIt` rolled nothing.

FIX: accept `PlayerAction::Blitz` at that gate in `bb2016/move_/step_init_selecting.rs`
(`if action.is_moving() || action == PlayerAction::Blitz`), documenting that Rust's `Blitz` IS
Java's declared `BLITZ_MOVE`.

**FAILED APPROACH, REVERTED:** renaming the mapping instead —
`PlayerActionChoice::Blitz => PlayerAction::BlitzMove` in `pac_to_player_action` — is the more
literal port but took lineman bb2016 from 0 → **99** fails: the rest of the bb2016 blitz path
(dispatch arms, `is_blitzing` checks, StepEndSelecting routing) keys on `PlayerAction::Blitz`.
Narrow the gate, don't rename the variant.

Test: `prone_blitz_sets_going_for_it_when_standing_up_eats_the_move` (MA 3 → Rush; MA 6 → no Rush).
Verified: undead 76 → **2**; no regression across all 28 swept bb2016 rosters (the 23 green ones at
0 fails); lineman bb2016 100/100; lineman bb2025 100/100; ffb-engine 7094/0.

### ITER65 (2026-08-13) — bb2016/bb2020 had NO Going-For-It modifiers at all → **undead 100/100 GREEN**

undead seed 21, step 5: away_01 (a PRONE Mummy at (13,7)) does a stand-up Move to (12,6). Java
spends 3 dice, Rust 6:
- Java: pos 21 `rollGoingForIt` = **2** → **FAILS** → pos 22-23 `InjuryTypeDropGFI` armour 2d6
  (1+5=6, held) → the step ends (the fall is a turnover).
- Rust: pos 21 GFI = 2 → treated as SUCCESS → carried on into the move's dodge (pos 22) and a block,
  spending 3 dice Java spends in later steps.

ROOT CAUSE: `DiceInterpreter.minimumRollGoingForIt = max(2, 2 + modifierTotal)`, and Java's
`modifiers/mixed/GoForItModifierCollection` — annotated `@RulesCollection(BB2016)` **and**
`@RulesCollection(BB2020)` — registers **Blizzard +1** (plus two Moles-under-the-Pitch entries). The
weather here is a Blizzard, so the minimum roll is 3 and a Rush of 2 fails.

Rust's `GoForItModifierFactory::for_rules` routed `Rules::Bb2025 | Common` to the bb2025 collection
and **everything else to the BASE `GoForItModifierCollection`, which registers nothing** — so bb2016
and bb2020 games applied NO GFI modifier ever and always rushed on 2+. A correct 1:1
`modifiers/mixed/go_for_it_modifier_collection.rs` already existed in the tree (Blizzard +1 + both
Moles variants, matching Java's per-team-id predicates) — it was simply never wired up.

FIX: one arm of `for_rules` — route non-bb2025 to `MixedCollection` (plus its `GfiCollection` impl).

Test: `every_edition_applies_the_blizzard_gfi_modifier` (asserts Blizzard is registered at +1 and
yields a minimum roll of 3 for bb2016, bb2020 AND bb2025).
Verified: undead 2 fails → **0 (100/100 GREEN)**; no regression across all 28 swept bb2016 rosters;
lineman bb2016 100/100; lineman/human/wood_elf bb2025 100/100; ffb-engine 7094/0, ffb-mechanics
1156/0, ffb-model 2780/0. **24 🟢 / 6 🔴.**

NOTE: this also silently affected **bb2020**, which has no parity matrix — worth a look if bb2020 is
ever exercised.

Remaining reds: elf 84 · ogre 98 · wood_elf 98 · goblin 100 · halfling 100 · vampire 100.
**elf is the next target.**

### ITER66 (2026-08-13) — bb2016 Side Step must auto-USE (harness policy) → **elf 100/100 GREEN**

elf seed 1, step 58. A textbook single-player state divergence with matching dice — the diff at i=59
is exactly one line:

    a00: J=13,7,Prone   R=15,7,Prone

home_03 at (13,8) blitzes away_01 at (14,8); block die 6 = Pow. Java: `JAVA_PUSHBACK
pushed=Away1 to=(13,7) homeChoice=false`. Rust offered the standard behind-the-defender squares
`[(15,7),(15,8),(15,9)]` and pushed to (15,7). away_01 is an **Elf Blitzer — Block + Side Step**, and
`homeChoice=false` is the giveaway: the DEFENDER's team chose the square, i.e. Side Step fired.

ROOT CAUSE: Rust's `skill_behaviour/bb2016/side_step_behaviour.rs` auto-**DECLINED** the undecided
Side Step (`side_stepping.insert(id, false); return true;`). Java shows a `DialogSkillUseParameter`
and the parity harness (ParityRunner `SKILL_USE`) auto-**USES** every offered skill except
`DumpOff` / `PrimalSavagery` / `SafePairOfHands` — Side Step is not excluded, so Java side-steps.
Accepting switches the push to `PushbackMode::SIDE_STEP`, whose candidate squares are taken around
the DEFENDER with `home_choice` set to the defender's own team.

This is the **exact same trap as ITER59's bb2016 Stand Firm auto-use** — the pattern is now 2/2, so
treat every "headless: auto-decline" comment in a bb2016 skill behaviour as suspect until checked
against ParityRunner's SKILL_USE policy.

FIX: auto-ACCEPT when undecided (one line + the early `return` removed). The pre-existing test
`side_step_headless_auto_declines` pinned the wrong behaviour and was rewritten as
`side_step_headless_auto_uses` (also asserting SIDE_STEP mode and `home_choice == false` for an
away-team side-stepper).
Verified: elf 84 fails → **0 (100/100 GREEN)**; no regression across all 28 swept bb2016 rosters;
lineman bb2016 100/100; lineman + dark_elf bb2025 100/100; ffb-engine 7094/0. **25 🟢 / 5 🔴.**

KNOWN GAP (latent, no bb2020 parity matrix): `skill_behaviour/bb2020/side_step_behaviour.rs` still
auto-declines — its `side_step_headless_auto_declines` test is still green. Same for the bb2025
Stand Firm `pushbackStack.clear()` noted in ITER60.

Remaining reds: ogre 98 · wood_elf 98 · goblin 100 · halfling 100 · vampire 100.
**ogre/wood_elf are the next targets.**

### ITER67 (2026-08-13) — ogre: root-caused to a ParityRunner gap needing a JAR REBUILD (no fix landed)

ogre seed 1 has **no rng divergence at all** and Java's log simply STOPS at i=6:
`STUCK_STEP: INIT_SELECTING unadvanced for 501 iters — ending game`, then
`END_REASON: finished iter=523 half=1 turnHome=1 turnAway=0`. Java never leaves home's first turn, so
for this seed **there is no ground truth to match**.

The Java tail shows the harness re-declaring the same action forever:

    JAVA_TTM pid=…Home6 N=2 idx=1 thrown=…Home8
    JAVA_P2 pid=…Home6 action=THROW_TEAM_MATE si=7      (repeated ~500×, actionRng consumed each time)

ROOT CAUSE: Java's bb2016 `StepInitSelecting.handleCommand` gates `CLIENT_THROW_TEAM_MATE` on
`checkCommandWithActingPlayer(...) && **!game.getTurnData().isPassUsed()**`, and a bb2016 TTM consumes
the team's PASS (`ThrowTeamMateBehaviour` → `turnData.setPassUsed(true)`), NOT a separate ttm flag.
Ogre seed 1 declares **two** TTMs in one turn (i=2 `Activate(Home5, THROW_TEAM_MATE)` and i=6
`Activate(Home6, THROW_TEAM_MATE)`). The second is illegal: the engine rejects the command, the step
stays UNHANDLED, and `ParityRunner.filterStaleActions` — which filters
`case THROW_TEAM_MATE: keep = !td.isTtmUsed();` — does not know bb2016 spends `passUsed`, so it keeps
re-offering TTM and spins.

Rust meanwhile RESOLVED the second TTM. Its `Action::ThrowTeamMate` command arm does gate on
`game.turn_data().pass_used` (step_init_selecting.rs:161), but the **folded-target dispatch arm**
(`PlayerAction::ThrowTeamMate =>` ~line 240, the path the agent actually takes) does not.

FIX REQUIRED (three parts, and the agent halves MUST land together or the action-pick `N` diverges):
1. Rust `bb2016/move_/step_init_selecting.rs`: the folded TTM dispatch arm must honour `!pass_used`,
   1:1 with Java's gate.
2. Rust `random_agent.rs` `filter_stale_actions` (line ~271): under bb2016, filter
   `PlayerAction::ThrowTeamMate` by `!td.pass_used` rather than `!td.ttm_used`.
3. Java `ParityRunner.filterStaleActions`: same change — **this needs a JAR REBUILD.**

NOT ATTEMPTED THIS ITERATION: a jar rebuild is a deliberate operation (per `docs/PARITY_PROCESS.md`:
commit the Rust side first, rebuild, then re-verify the lineman tier before trusting any result), and
starting it at the tail of a long iteration risks losing work. ogre stays at 98. Next iteration should
run that sequence for parts 1-3 together.

Also checked and RULED OUT this iteration: the "headless: auto-decline" sweep of
`skill_behaviour/bb2016/` found a third instance — `grab_behaviour.rs:103` auto-declines a Grab that
Java offers via `DialogSkillUseParameter` (so the harness would USE it). **No bb2016 roster has Grab**
(verified across all 29 `data/rosters/bb2016/roster_*.json`), so it cannot explain any remaining red.
Worth fixing for correctness, but it is not on the critical path.

### ITER68 (2026-08-13) — bb2016 Take Root marked its skill on the PLAYER, not the ACTING PLAYER

wood_elf seed 1, step 46. Java rolls 2 dice, Rust 1. `FFB_DICE_TRACE` callers + `FFB_DRIVE_TRACE`:
- Java: pos 32 = `TakeRootBehaviour$1.handleExecuteStepHook:56`, pos 33 = `StepStandUp.executeStep:103`.
- Rust: its single die at pos 32 comes from **StandUp** (the DRIVE trace shows `TakeRoot` running and
  rolling nothing). So the missing die is **Take Root**, not the stand-up.
  (A prone Treeman has MA 2 < `MINIMUM_MOVE_TO_STAND_UP`, so the stand-up roll is correct in both.)

Java `skillbehaviour/bb2016/TakeRootBehaviour`: `doRoll = UtilCards.hasUnusedSkill(**actingPlayer**,
skill)` and `actingPlayer.markSkillUsed(skill)` — the used-skill set is the ACTING PLAYER's, i.e.
per-ACTIVATION (cleared by `UtilActingPlayer.changeActingPlayer`). Rust's `bb2016/step_take_root.rs`
read and wrote **`Player.used_skills`**, which persists for the whole game, so a Treeman rolled Take
Root only ONCE per game and every later activation silently skipped its d6.

FIXED (1:1, with tests `take_root_marks_the_acting_player_not_the_player` and the corrected
`marks_skill_used_on_roll`). This is the same class as the bb2025 Bone Head per-activation fix, whose
note already warned "bb2020/bb2016 BoneHeadBehaviour still use Player.used_skills" — Take Root was
another instance.

**HONEST RESULT: this did NOT move the parity needle — wood_elf is still 98.** The Treeman at this
frontier is evidently ALREADY `rooted` in Rust, so `TakeRootBehaviour`'s outer guard
(`if (!playerState.isRooted())`) short-circuits before `doRoll` is even consulted, leaving the fix
untested by this seed. **`rooted` is a PlayerState FLAG that the parity state hash does not record**
(the hash carries only `base`), so a rooted-flag divergence is HASH-INVISIBLE — exactly like the
active-bit bug from the earlier bb2016 amazon work.

NEXT (wood_elf): find where the two engines disagree on the `rooted` flag. Java clears it via
`playerState.changeRooted(false)` (e.g. `UtilServerInjury.dropPlayer`) and bb2016 `StepTakeRoot`
also does `actingPlayer.setGoingForIt(false)`; the bb2025 wood_elf fix was "Take Root
(old_player_state + dodging-clear)", so compare the bb2016 rooted set/clear sites against bb2025's.
A gated trace on `set_player_state` for the Treeman (like the old `FFB_H06DBG`) is the fastest probe,
since the hash cannot see this.

Verified no regression: all 28 swept bb2016 rosters unchanged (the 25 green ones at 0 fails);
lineman bb2016 100/100; ffb-engine 7095/0.

### ITER69 (2026-08-13) — the bb2016 TTM/passUsed fix landed WITH a jar rebuild → **ogre 100/100 GREEN**

Executed ITER67's three-part plan. All three parts had to ship together, because two of them are the
two agent halves and a mismatch shifts the action-pick `N`.

1. Rust `bb2016/move_/step_init_selecting.rs` — the folded-target TTM dispatch arm now honours
   `!turn_data().pass_used`, matching Java's `CLIENT_THROW_TEAM_MATE` gate.
2. Rust `random_agent.rs` `filter_stale_actions` — under bb2016, `ThrowTeamMate` requires
   `!ttm_used && !pass_used`.
3. Java `ParityRunner.filterStaleActions` — `keep = isBb2016(game) ? (!isTtmUsed() && !isPassUsed())
   : !isTtmUsed()`, using the file's existing `isBb2016(game)` helper.

**IMPORTANT TOOLING DISCOVERY — the live Java tree is NOT `ffb-rust/ffb-java/`.**
`crates/ffb-parity/src/runner.rs` resolves `PARITY_CP` to
`C:\Users\Admin\niels\ffb\ffb\ffb-ai\target\ffb-ai-jar-with-dependencies.jar` (its first candidate).
That jar was built from **`C:\Users\Admin\niels\ffb\ffb\`**, whose jar was current (Aug 13) while
`ffb-rust/ffb-java/`'s was from Jun 21. Verified impact: `ParityRunner.java` DIFFERS between the two
trees, but the ENGINE trees agree except for 6 `ffb-server` files + 1 `ffb-common` file — the local
trace-instrumented copies (`DiceRoller`, `Fortuna`, bb2025 `StepGoForIt`/`StepPickUp`/
`StepCatchScatterThrowIn`, `mixed StepPassBlock`). **None of the files cited by any campaign fix is in
that differing set, so every earlier diagnosis stands.** Edit the harness at `niels/ffb/`, not
`ffb-rust/ffb-java/`.

**Jar rebuild method (no maven on PATH; a full `mvn` build is unnecessary):**
```bash
J=/c/Users/Admin/niels/ffb/ffb/ffb-ai/target/ffb-ai-jar-with-dependencies.jar
cp "$J" "$J.bak-preTTM"                     # ALWAYS back up first
cd /c/Users/Admin/niels/ffb/ffb/ffb-ai
javac -nowarn -cp "$J" -d /tmp/pr_build src/main/java/com/fumbbl/ffb/ai/parity/ParityRunner.java
cd /tmp/pr_build && jar uf "$J" com/fumbbl/ffb/ai/parity/ParityRunner*.class
```
JDK 17 (Eclipse Adoptium) is on PATH. Compiling the single class against the fat jar and `jar uf`-ing
the result back in takes seconds and avoids a full module build. Remember the inner classes
(`ParityRunner$1`, `ParityRunner$PendingStep`).

Verified in the process-mandated order: lineman bb2016 **100/100** and lineman bb2025 **100/100**
FIRST (the rebuilt harness is safe), then ogre 98 → **0 (100/100 GREEN)**. Full 28-roster bb2016
sweep: no regression, 24 swept green + ogre + lineman = **26 🟢 / 4 🔴**. ffb-engine 7096/0.
Test: `folded_throw_team_mate_is_rejected_once_the_pass_is_used`.

Remaining reds: wood_elf 98 (the hash-invisible `rooted` flag — see ITER68) · goblin 100 ·
halfling 100 · vampire 100.

### ITER70 (2026-08-13) — wood_elf: ITER68's `rooted` theory DISPROVEN; frontier re-characterised (no fix)

Probed wood_elf seed 1 step 46 with a temporary gated `FFB_ROOTDBG` trace on every `rooted`
transition in `FieldModel::set_player_state` (added, used, then reverted — the tree is clean).

**ITER68's stated next step was wrong and is retracted.** The `rooted` flag is NOT stuck: it toggles
correctly (`away_01 false→true`, `true→false`, twice over the game). More importantly, the player at
the frontier is **home_01, and its rooted flag never changes at all** — only `away_01`'s does. So a
rooted-flag disagreement is not the cause.

What IS established for wood_elf seed 1 step 46:
- The activating player is **home_01 = `woodelf.treeman`** (team slot 1), and it is **PRONE**
  (`RUST_PICK … prone_predraw`), doing a MOVE.
- Java rolls TWO dice: pos 32 `TakeRootBehaviour$1.handleExecuteStepHook:56`, then pos 33
  `StepStandUp.executeStep:103`. Rust rolls ONE — its pos 32 is the StandUp roll (the prompt right
  after is `ReRollOffer{action: "STAND_UP"}`), so the **Take Root d6 is the missing die**.
- Rust's SELECT-sequence `TakeRoot` step DOES run (`FFB_DRIVE_TRACE`: `TakeRoot` → … → `JumpUp` →
  `StandUp`, die at StandUp) but rolls nothing.
- home_01 is NOT rooted, so the step's `if is_rooted → next` early-out is not firing.
- Both `Select` and `Move` sequences contain TAKE_ROOT (Java and Rust agree), and Java's pos-32 roll
  is the SELECT one (it immediately precedes StandUp).

By elimination the remaining candidate is the `do_roll` gate:
`has_skill(TakeRoot) && !acting_player.used_skills.contains(TakeRoot)`. A Treeman has the skill, so
`acting_player.used_skills` must still contain `TakeRoot` at activation start — i.e. the
per-activation clear is not happening on the bb2016 prone-Move activation path.
**This is a hypothesis, NOT verified** — `step_init_selecting.rs:193` does call
`change_player_action`, whose `set_player` clears `used_skills` on a genuine player change, and the
previous actor was home_06, so it *should* clear. Next iteration should instrument the gate directly
(print `do_roll`, `has_skill`, and the `used_skills` set at entry) rather than reason about it.

Note ITER68's Take Root change (per-activation `used_skills`, commit `0fea88a5`) is KEPT: it is a
verified 1:1 correction, and reverting it would only restore an equally-wrong persistent
`Player.used_skills` read. It remains parity-neutral.

No code change this iteration. wood_elf stays 98; **26 🟢 / 4 🔴** unchanged.

### ITER71 (2026-08-13) — `startedStanding` is BB2025-ONLY → wood_elf 98 → 81

Instrumented the Take Root `do_roll` gate as ITER70 planned. The trace printed **ZERO** lines, which
was the finding: **`crates/ffb-engine/src/step/bb2016/step_take_root.rs` is DEAD CODE.**
`driver.rs:211` maps `StepId::TakeRoot` to `step_take_root::StepTakeRoot` resolved through
`use crate::step::bb2025::shared::*`, and nothing anywhere references the bb2016 file.

**This retracts ITER68's claim to have fixed anything.** That change edited the dead bb2016 file, which
is exactly why it was parity-neutral — not the `rooted` short-circuit I hypothesised. (ITER70 already
retracted the `rooted` theory; this retracts the file too.)

The live gate is `step/bb2025/shared/step_take_root.rs`, and it applied bb2025's condition to every
edition. Java's three behaviours differ:
- bb2025 `TakeRootBehaviour`: `if (startedStanding && !playerState.isRooted())`
- bb2020 `TakeRootBehaviour`: `if (!playerState.isRooted())`
- bb2016 `TakeRootBehaviour`: `if (!playerState.isRooted())`

`startedStanding` (`actingPlayer.getOldPlayerState().getBase() == STANDING`) is **BB2025-only**, so a
PRONE player standing up STILL rolls Take Root in bb2016/bb2020. wood_elf seed 1 step 46: home_01 is
the prone Treeman; Java rolls Take Root (pos 32) then the stand-up d6 (pos 33), Rust rolled only the
stand-up.

FIX (both halves are required together):
1. Compute `started_standing` only when `rules == Bb2025`, else `true`.
2. Add Java's `doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill)` +
   `actingPlayer.markSkillUsed(skill)`, which the shared step never had. TAKE_ROOT is in BOTH the
   Select and Move sequences, so without that pairing one bb2016 activation would now roll it TWICE —
   the bb2025 `startedStanding` short-circuit had been masking the omission.

Tests: `prone_take_root_rolls_in_bb2016_but_not_bb2025` (1 die for bb2016 and bb2020, 0 for bb2025)
and `take_root_rolls_once_per_activation` (Select rolls, Move does not).
Verified: wood_elf 98 → **81**; no regression — 25 swept bb2016 rosters green + lineman = 26 🟢 / 4 🔴;
lineman bb2016 100/100; lineman, **wood_elf** and halfling bb2025 all 100/100; ffb-engine 7098/0.

**METHOD LESSON (cost ITER68 + ITER70): confirm the code you are about to "fix" is actually EXECUTED
before diagnosing it.** A one-line gated eprintln that prints nothing is the cheapest possible proof
of dead code, and would have saved two iterations of reasoning about the wrong file. Check
`driver.rs`'s `make_step_for` / glob imports first — `bb2016/step_take_root.rs` is dead, and other
`step/bb2016/*` files may be too.

### ITER72 (2026-08-13) — wood_elf seed 2: narrowed to an **actionRng** (agent-stream) divergence; no fix

New frontier after ITER71. wood_elf seed 2: first state mismatch i=66, and the diff is a single line:

    h09 (home_10): J=5,7,Standing   R=3,7,Standing

Both engines activate home_10 for a MOVE at i=65 and both offer the **same number of targets**:

    JAVA_SMA pid=…Home10 coord=4,7 targets=6 → JAVA_PICK N=6 idx=4 t=(5,7)
    RUST_SMA pid=home_10 N=6            → RUST_PICK N=6 idx=0 t=(3,7)

Same `N`, different `idx` ⇒ the **actionRng draw differs**, while the shared GAME dice still match
(first `rng_calls` divergence is later, i=72) and the state hash matched through i=65. So this is an
**agent-stream (actionRng) misalignment**, not an engine-dice bug — a different class from every fix
so far this campaign.

**METHOD WARNING — do not repeat my mistake here.** I tried to locate the first divergence by
diffing the ordered list of `JAVA_PICK` vs `RUST_PICK` lines and got two different, both-wrong
answers. The lists are NOT 1:1 comparable: Rust emits `RUST_MOVE_PRE` for the prone pre-draw path
(which reuses an already-drawn target) as well as `RUST_PICK`, and it also draws for
`RUST_ACT_PICK` / `RUST_BLOCK_PICK`, so index alignment silently shifts. Any conclusion of the form
"engine X picked twice for player P" from those lists is unreliable.

CORRECT NEXT TOOL: compare actionRng **call counts per comparator step**, not pick lists.
- Rust already prints its actionRng counter as `arc=` on `RUST_ACT_START` / `RUST_ACT_PICK` /
  `RUST_BLOCK_PICK` / `RUST_ACT_END`.
- Java has no equivalent counter in the trace — add a gated one (e.g. print an `actionRng` call count
  alongside `JAVA_P2`) to `ParityRunner` in the LIVE tree `C:\Users\Admin\niels\ffb\ffb\`, rebuild the
  jar with the ITER69 recipe, then diff the per-step deltas to find the first step where the two
  agents consume a different NUMBER of actionRng draws.
Every actionRng consumer in the harness is a candidate: `sendMoveAction`, `sendBlockAction` /
`sendBlitzTargetSelection`, `sendPassAction`, `sendHandOverAction`, `sendThrowTeamMateAction`,
`sendFoulAction`, and the phase-1 action pick.

No code change this iteration. wood_elf stays 81; **26 🟢 / 4 🔴** unchanged.

### ITER73 (2026-08-13) — a ROOTED player's pre-drawn move square was discarded → wood_elf 98→81→**19**

Built the tool ITER72 called for: a temporary counted wrapper around every
`actionRng.nextLong()` in the LIVE `ParityRunner` plus `arc=` on the `JSTEP` line, jar rebuilt with the
ITER69 recipe. Diffing **per-step actionRng counts** (not pick lists) found the first PERSISTENT
divergence immediately:

    i=61..64  J 120,122,124,126   R 120,122,124,126   (aligned, +2 per step)
    i=65      J 128               R 129               (-1, never recovers)

So Rust makes ONE EXTRA actionRng draw during step 64 — `Activate(home_01, Move)`, the **rooted**
Treeman. Rust's own trace spells it out:

    RUST_ACT_PICK pid=home_01 ... arc=126          (action pick)
    RUST_SMA/RUST_PICK ... idx=4 t=(13,6) rooted_predraw   (arc 127 — the square JAVA uses)
    RUST_SMA pid=home_01 N=7 / RUST_PICK ... idx=1 t=(11,7)  (arc 128 — a SECOND draw)

ROOT CAUSE: `random_agent.rs` pre-draws a move target at activation for players that cannot use the
normal `StepInitMoving` prompt path — PRONE (its Select-sequence negatrait may fail first) and ROOTED
(its `StepMove` is a no-op) — but stored it in `pending_move` **only `if is_prone`**. For a rooted
player the draw happened and was then thrown away, so the Move prompt drew again: 3 actionRng calls
where `ParityRunner.sendMoveAction` makes 2. From that point the agent streams were permanently
offset — first visible as home_10 moving to (3,7) instead of (5,7) at i=65.

FIX: store `pending_move` for both cases (drop the `if is_prone`). One line.

Test: `predrawn_move_square_is_reused_with_no_extra_draw` — pins the reuse contract (the prompt list
deliberately excludes the pre-drawn square, so a re-draw could not return it, and the actionRng count
must not move). The *storing* half is covered end-to-end by the parity result.
Verified: wood_elf 81 → **19**; full 30-roster bb2016 sweep shows **26 green**, no regression;
lineman bb2016 100/100; lineman + wood_elf bb2025 100/100; ffb-engine 7099/0.
The temporary Java `arc=` instrumentation was reverted and the jar rebuilt clean.

**TOOL LESSON: per-step actionRng call-count deltas are the right instrument for agent-stream
divergences** — it found in one shot what two iterations of pick-list diffing got wrong. Rust already
prints `arc=`; the Java counter is ~4 lines (`arcCount`/`arcNext()` wrapper + `JSTEP` field) and
rebuilds in seconds.

### ITER74 (2026-08-13) — wood_elf seed 14: `blitzUsed` after a FAILED stand-up blitz (diagnosis only)

wood_elf seed 14 (lowest of the remaining 19). First state mismatch i=40, but the cause is at **i=39**,
where the two engines pick a different ACTION for the same player:

    J i=39: Activate(…Away2, BLITZ)   →  JAVA_P2 action=BLITZ_MOVE, JAVA_BLOCK_PICK, JAVA_BLOCKROLL nDice=1
    R i=39: Activate(away_02, Move)   →  RUST_ACT_PICK pid=away_02 **N=1** idx=0 action=Move

Rust offers only ONE action (Move) — Blitz was filtered out of its turn-start snapshot by
`filter_stale_actions`, i.e. **Rust has `blitz_used = true` and Java has `blitzUsed == false`.**

Origin is i=34: `Activate(away_01, Blitz)` — away_01 is the PRONE Treeman, so this is a stand-up
blitz. Both engines roll the SAME two dice and `rng_calls` matches exactly (27 → 29 in both):
pos 28 = `TakeRootBehaviour` d6 **5** (success), pos 29 = `StepStandUp` d6 **1** (FAIL). Both then
offer a STAND_UP team re-roll, which both harnesses DECLINE. After that:
- Rust's eligible list becomes `[("away_01",[Move]), ("away_02",[Move]), …]` — every player Move-only,
  so `blitz_used` was set.
- Java still offers BLITZ_MOVE to away_02 five steps later, so its `blitzUsed` is still false.

**The observable is certain; the Java mechanism is NOT yet pinned, and the source reads the other way.**
Java's bb2016 `StepStandUp` failure branch explicitly does
`case BLITZ: case BLITZ_MOVE: … game.getTurnData().setBlitzUsed(true);` — and its bb2016
`StepEndSelecting` `case STAND_UP_BLITZ:` also sets it (Rust mirrors that arm at
`bb2016/move_/step_end_selecting.rs:349`), while `case BLITZ_MOVE:` does NOT. So on a plain reading
Java should also end up with `blitzUsed == true`. It demonstrably does not.

Most likely explanation to test next: the failure branch is guarded by
`if ((getReRolledAction() == STAND_UP) || !UtilServerReRoll.askForReRollIfAvailable(...))`, so when a
re-roll IS offered Java defers, and the ParityRunner's decline may never re-enter `StepStandUp`
(it deselects instead) — leaving `blitzUsed` unset. NEXT STEP: add a gated print of
`turnData.isBlitzUsed()` (both teams) next to `JAVA_P2` in the live `ParityRunner`, rebuild with the
ITER69 recipe, and confirm exactly when Java sets it; only then port the guard. Do NOT "fix" Rust by
deleting its `StandUpBlitz` blitz_used assignment — that arm is a correct 1:1 port of
`StepEndSelecting`, so the wrong-path is elsewhere (most likely Rust reaching the StandUpBlitz arm
where Java's action is still BLITZ_MOVE).

No code change. wood_elf stays 19; **26 🟢 / 4 🔴** unchanged.

### ITER75 (2026-08-13) — Java's `blitzUsed` mechanism PINNED; declined-re-roll path split (parity-neutral)

Ran ITER74's probe: gated `blitzUsedH`/`blitzUsedA` prints next to `JAVA_P2`, jar rebuilt. Java is
unambiguous — the flag stays FALSE right through the failed stand-up blitz and only flips after the
NEXT player's blitz reaches target selection:

    si=35 Away1 action=BLITZ_MOVE  blitzUsedA=false     <- the prone Treeman's stand-up blitz (FAILS)
    si=36..39 (away MOVEs)         blitzUsedA=false     <- still false
    si=40 Away2 action=BLITZ_MOVE  blitzUsedA=false     <- away_02 blitzes
    si=41 Away11 action=MOVE       blitzUsedA=true      <- set only now

MECHANISM (Java bb2016 `StepStandUp`): the per-action used-flag switch
(`case BLITZ: case BLITZ_MOVE: … setBlitzUsed(true)`) lives INSIDE the roll's failure branch. On the
DECLINED-re-roll pass `rollStandUp` is set false BEFORE the roll
(`if (STAND_UP == getReRolledAction()) { if (source == null || !useReRoll(...)) rollStandUp = false; }`),
so control reaches only the trailing
`if (!rollStandUp) { setPlayerState(PRONE, active=false); publish END_PLAYER_ACTION; GOTO failure; }`
block — the flags are never touched. ITER74's hypothesis was right.

FIXED that faithfully: the live `bb2025/move_/step_stand_up.rs` called `fail_stand_up` (which runs
`handle_failed_stand_up`, the flags switch) on its `already_rerolled` path. Split out
`end_stand_up_without_flags` — Java's trailing block verbatim — and call it there.

**HONEST RESULT: parity-neutral. wood_elf is still 19.** Instrumenting every plausible
`blitz_used = true` site showed the flag is set by `handle_failed_stand_up`'s Blitz arm reached via the
OTHER call — the `ask_for_reroll_if_available` returned-None path (no team re-roll available), which
fires 4× in seed 14. So for THIS seed the offer is not actually reaching the declined-re-roll branch,
and the remaining question is why Rust takes the no-re-roll-available path (setting the flag, which
Java's equivalent branch also does) while Java still ends with `blitzUsedA == false`.

NEXT: instrument the two `fail_stand_up` / `end_stand_up_without_flags` call sites plus
`ask_for_reroll_if_available`'s decision for away_01 at i=34, and compare against a Java-side gated
print inside `StepStandUp`'s failure branch. The Java-side observable (flag stays false) is certain;
the Rust-side path that sets it is now narrowed to one call.

Landing the split anyway: it is a verified 1:1 correction of Java's control flow with a real behavioural
difference (a declined stand-up re-roll must not consume the team's Blitz/Pass/Foul), even though this
seed does not exercise it.
Verified: no regression — 27-roster bb2016 sweep 26 green, wood_elf unchanged at 19; lineman bb2016
100/100; ffb-engine 7099/0. All temporary instrumentation (Rust `FFB_BUDBG`, Java `blitzUsed` prints)
reverted and the jar rebuilt clean.

### ITER76 (2026-08-13) — wood_elf: narrowed to `ask_for_reroll_if_available("STAND_UP")`, with an open contradiction

Instrumented the stand-up failure path with the acting player id and the re-roll decision (all
temporary, all reverted — tree is clean at `3c6b0684`).

ESTABLISHED:
1. `already_rerolled` is **false on every entry**, for every failed stand-up in the seed, including
   away_01's Blitz at i=34. So Rust NEVER reaches the declined-re-roll branch that ITER75 fixed —
   which is exactly why that fix was parity-neutral here.
2. `handle_failed_stand_up` (the used-flag switch) is therefore reached via the
   `ask_for_reroll_if_available` returned-None path, and it is what sets `blitz_used`.
3. **But every condition inside that function passes**:
   `SUDBG rr-check pid=away_01 min=4 rr_left=2 rr_used=false tm=Regular allowed=true home_playing=false`
   — i.e. `td.rerolls > 0 && !td.reroll_used && team_re_roll_allowed` is all true, and there is only ONE
   definition of `ask_for_reroll_if_available` (`step/util_server_re_roll.rs:30`), which the step does
   import. On that reading it must return `Some`, yet the very next trace line is
   `handle_failed_stand_up` — i.e. the `if let Some(prompt)` did not take.

**That contradiction is unresolved and I am not going to guess past it.** Two candidate explanations,
both cheap to settle and neither yet tested:
- The observed `fail-path → rr-check → handle_failed` triple is actually spanning TWO step entries and
  my probe placement made it look like one. Fix: print a per-entry counter / the step's address, and
  print immediately INSIDE both arms of the `if let`.
- `find_skill_reroll_source` returns a Some that is then rejected downstream, or the prompt is emitted
  and the agent's decline path re-enters a FRESH `StepStandUp` instance so `re_roll_state` is lost
  (which would also explain `already_rerolled` never being true). Fix: log the step instance identity
  across entries; if the driver rebuilds the step, that is the real bug and it is NOT specific to
  stand-up.

The second hypothesis is the more interesting one: if a re-entered step loses its `re_roll_state`, every
`AbstractStepWithReRoll` translation is affected, and the ITER56 TTM fix (declined re-roll must keep
`re_rolled_action`) only worked because it is checked within a single entry.

COST NOTE: this is the 4th consecutive iteration on wood_elf, with ITER73 (98→19) the only numeric
gain; ITER74/75/76 produced one parity-neutral fix plus this narrowing. wood_elf 19 is still the
fewest-fails roster, so the protocol keeps pointing here, but if the next iteration does not resolve the
contradiction it is worth deliberately switching to goblin/halfling/vampire (100 each, all untraced —
likely a single systematic cause each) rather than continuing to pay down this one.

No code change. wood_elf stays 19; **26 🟢 / 4 🔴** unchanged; ffb-engine 7099/0; tree clean.

### ITER77 (2026-08-13) — the OTHER `already_rerolled` site; edition-gated → **wood_elf 100/100 GREEN (27/30)**

**FIRST, A CORRECTION: commit `3c6b0684` (ITER75) contained NO code change.** Its message describes an
`end_stand_up_without_flags` split that is not in the diff — a mid-iteration
`git checkout -- crates/ffb-engine/src` (reverting blunt instrumentation) also reverted that edit, and
the commit went in without re-checking the diff. The verification quoted there (sweep 26 green,
wood_elf 19) was accurate, but it was measuring an unchanged tree. **Always `git show --stat` / grep the
committed diff for the intended symbol before writing the message.**

Resolving ITER76's contradiction: prints inside BOTH arms of the offer showed `some=true` every time —
the re-roll IS always offered, so the post-roll `already_rerolled` branch I had been staring at is never
the one that fires. bb2016 `StepStandUp` has **TWO** `reRolledAction == STAND_UP` sites and they behave
differently:

| site | Java bb2016 | Java bb2020/bb2025 |
|---|---|---|
| **PRE-roll** `if (STAND_UP == getReRolledAction()) { if (source == null \|\| !useReRoll) …` | `rollStandUp = false;` **only** → trailing block, NO flags | `rollStandUp = false;` **+ `handleFailedStandUp(...)`** |
| **POST-roll** (rolled and failed again) | `rollStandUp = false;` + the inline used-flag switch | `handleFailedStandUp(...)` |

The harness always declines team re-rolls, so the PRE-roll site is the hot path. Rust called
`fail_stand_up` (which runs the flags switch) there for every edition, so under bb2016 a declined
stand-up re-roll consumed the team's Blitz — leaving away_02 with only Move at i=39 where Java blitzes.

FIX: `end_stand_up_without_flags` (Java's trailing block verbatim) at the PRE-roll site **gated on
`rules == Bb2016`**; bb2020/bb2025 keep `fail_stand_up`. The gate is not cosmetic — applying the bb2016
behaviour to bb2025 regressed **wood_elf bb2025 from 0 to 17 fails**, which is how the edition
difference was caught.

Test: `declined_stand_up_reroll_does_not_consume_the_action` — asserts bb2025 DOES consume the Blitz at
that site and bb2016 does NOT, plus that a plain failed stand-up with no re-roll available still
consumes it.
Verified: wood_elf bb2016 19 → **0 (100/100 GREEN)**; wood_elf bb2025 back to 0; 30-roster bb2016 sweep
**27 green**, no regression; lineman bb2016 100/100; lineman + dwarf bb2025 100/100; ffb-engine 7100/0.

Remaining reds: goblin 100 · halfling 100 · vampire 100 — all untraced, each likely ONE systematic cause.

---

## ITER78 — goblin: the Java game never started (PETTY_CASH), and BB2016 casualties rolled the BB2020 dice

**Two independent blockers, both root-caused against the Java source.**

### (a) The Java side never played a single turn — `STUCK_STEP: PETTY_CASH`
`ffb-parity --home goblin --away goblin --edition bb2016 --seeds 1-1` ended with
`END_REASON: finished iter=1002 half=0 turnHome=0 turnAway=0` — Java produced **zero** steps, so all
100 seeds "failed" at step 0 with nothing to compare.

`bb2016/start/StepPettyCash` (the BB2016 step; BB2020/BB2025 use `mixed/start/StepPettyCash`, which
never blocks) waits for a `ClientCommandPettyCash` whenever a team's treasury is **>= 50,000** and
`FORCE_TREASURY_TO_PETTY_CASH` is off. A diagnostic print in the harness gave:
`home tv=1020000 tr=80000 away tv=1020000 tr=80000 pettyCash=true force=false`.
Only **two** bb2016 parity teams have that much left over — `team_goblin.json` (80k) and
`team_halfling.json` (180k). Every other roster has < 50k and auto-selects, which is exactly why
this never surfaced before and why it hits precisely two of the three remaining reds.

`RandomStrategy.respondToDialog(PETTY_CASH)` already answers deterministically with
`sendPettyCash(0)`, so the response was fine — the **routing** was not: `ParityRunner.getDialogTeamId`
had no `DialogPettyCashParameter` arm, so the command was injected with `MatchRunner.inject` (home)
no matter which team the dialog was raised for. `UtilServerSteps.checkCommandIsFromHomePlayer` then
kept crediting the home team, the away team's dialog re-fired forever, the 500-iteration
`DIALOG_LOOP_CLEARED` guard nulled it, and the step sat unadvanced until `STUCK_STEP`.

FIX (harness only, `ParityRunner.java` + jar rebuild): a `DialogPettyCashParameter` arm in
`getDialogTeamId`, plus an explicit `case PETTY_CASH:` that sends `ClientCommandPettyCash(0)` via
`injectForTeam(..., teamId.equals(home))`. Java now plays the full match
(`END_REASON: finished iter=901 half=2 turnHome=8 turnAway=8`).

### (b) BB2016 casualties rolled ONE d16 where Java rolls a d6 AND a d8
With Java running, seed 1 diverged at **i=1** — home_03 blitzes the away Fanatic (`a02`), both-down,
armour broken, injury `6,6 = 12` → Casualty. Then:

| | rng 19 | rng 20 |
|---|---|---|
| Java | `d6=5` (`DiceRoller.rollCasualtyRenamed:212`) | `d8=8` |
| Rust | `d16=1` | *(d6 consumed as rng 20)* |

`InjuryTypeServer.setInjury()` takes **both** the casualty roll and its interpretation from the
game's `RollMechanic`:
- `bb2016/RollMechanic.rollCasualty` → `rollCasualtyRenamed()` = `[d6, d8]`, and
  `interpretCasualtyRollAndAddModifiers` switches on **`roll[0]` alone**: `6` → RIP, `4-5` → Serious
  Injury, `1-3` → Badly Hurt — **no casualty modifiers at all**.
- `bb2020`/`bb2025` roll `[d16, d6]` and map `d16 + casualty modifiers`.

Rust's `injury.rs::outcome_to_player_state` hardcoded the BB2020 shape (`rng.die(16)`, `rng.d6()`,
`CasualtyModifierFactory`, `casualty_tier_to_player_state`) for **every** edition, even though the
correctly-ported per-edition `RollMechanic`s were sitting right there unused. So every BB2016
casualty desynced the shared dice stream by one call/one die at the first casualty of the game.
The Decay second-casualty block had the same hardcoding.

FIX: `outcome_to_player_state`'s `Casualty` arm and the Decay block now call
`roll_mechanic_for(game.rules).roll_casualty(rng)` and
`.interpret_casualty_roll_and_add_modifiers(...)` — a literal transcription of
`InjuryTypeServer.setInjury()`. The dead `casualty_tier_to_player_state` helper is gone (the tier
tables live in the mechanics, where Java keeps them). BB2020/BB2025 behaviour is **bit-identical**:
their mechanics already do exactly what the inline code did (`map_casualty_roll_bb2025` ==
`casualty_tier_to_player_state`, and both apply `CasualtyModifierFactory`).

Test `casualty_roll_uses_the_editions_roll_mechanic` (bb2016 draws d6+d8 and reads roll[0] → Serious
Injury; bb2025 still draws d16+d6 and maps the d16).

**Verified:** goblin seed 1 dice now match Java exactly through rng 22
(`6,6 | d6=5, d8=8 | 4, 2`). Gates: lineman bb2016 **0/100**, lineman bb2025 **0/100**,
dwarf bb2025 **0/100**, `cargo test -p ffb-engine` **7101/0**.

**Fail count unchanged at 100** — honest reporting: seed 1 now survives to a *later*, different
divergence instead of dying at step 0, but the seed still fails, so no seed flipped green yet.

### FRONTIER for ITER79 — the apothecary
seed 1 i=1 now diverges only on the aftermath of that casualty: `a02` is **Ko** in Java and
**Injured** in Rust, and Java draws **4 more dice** (rng 23-26: `4,3,5,1`) that Rust does not.
Rust's agent answers the `UseApothecary { player_id: away_03, apothecary_type: "team" }` prompt with
a bare `Acknowledge` (= decline, 0 dice) where the Java side runs `bb2016/StepApothecary`. Next
iteration: read `bb2016/StepApothecary` + `ParityRunner`'s `APOTHECARY_CHOICE` handling and port it.

---

## ITER79 — goblin: `dropPlayer` never rolled the Ball & Chain injury

seed 1 i=1 (home_03 Fanatic blitzes the away Fanatic, both down). Rust drew **6** dice where Java
drew **10**. Widening the Java dice trace to a full `com.fumbbl` stack (new `FFB_DICE_DEEP=1` →
`-Dffb.parityDebugDeep`, gated logging in `DiceRoller`, no behaviour change) named every caller:

| rng | Java caller | |
|---|---|---|
| 17,18 | `InjuryTypeBallAndChain.handleInjury` ← `UtilServerInjury.dropPlayer:341` ← `PilingOnBehaviour:104` | defender chain injury |
| 19,20 | `rollCasualtyRenamed` ← that injury | its casualty |
| 21,22 | `InjuryTypeBlock.injuryRoll` ← `PilingOnBehaviour:115` | defender block injury |
| 23,24 | `InjuryTypeBallAndChain.handleInjury` ← `dropPlayer:341` ← `PilingOnBehaviour:164` | **attacker** chain injury |
| 25,26 | `InjuryTypeBlock.injuryRoll` ← `PilingOnBehaviour:165` | **attacker** block injury |

`UtilServerInjury.dropPlayer(step, player, PRONE, mode, spoh)` branches on
`placedProneCausesInjuryRoll`: a Ball & Chain player is **not** placed prone — it takes a full
`InjuryTypeBallAndChain` roll (2d6, plus a casualty roll on 10+) whose result is published for the
apothecary step. Rust's `drop_player` had that branch documented as *"treated the same as regular
drops here — the full injury roll is a TODO"*, so each Ball & Chain player dropped cost **2 missing
d6**. Only the Goblin Fanatic carries the skill in the bb2016 matrix, which is why 27 rosters never
noticed. (`stun_player_rng` already had the branch — same Java method with `pPlayerBase = STUNNED`,
added earlier for the Pitch-Invasion path.)

FIX: `drop_player_rng` — the full Java `dropPlayer` including the B&C branch — plus two call-site
changes in `StepDropFallingPlayers`:
- **attacker**: `drop_player` → `drop_player_rng`; it was already at Java's line-164 position.
- **defender**: Java drops at line 104 and rolls the block injury at line 115, but Rust dropped the
  defender only at publish time, i.e. AFTER the block injury. For a B&C defender that reverses the
  two rolls. The bb2016 branch now drops (and stashes the parameters) before the injury roll and
  publishes the stashed parameters at the original site. Reorder is bb2016-gated — bb2025 goes
  through `SteadyFootingContext` and is untouched.

Test `drop_player_rng_rolls_the_chain_injury_for_a_ball_and_chain_player`.

**Verified:** goblin seed 1 advances i=1 → **i=2**. Gates: lineman bb2016 **0/100**, lineman bb2025
**0/100**, goblin **bb2025 0/100** (the roster with the same Fanatic — proves the bb2016 gating
held), `cargo test -p ffb-engine` **7102/0**.

**Fail count still 100** — seed 1 now dies one step later, at the next distinct cause.

### FRONTIER for ITER80 — `THROW_BOMB`
seed 1 i=2: the away Bombardier activates `THROW_BOMB`. `ParityRunner` has **no** handler
(`UNHANDLED_ACTING_ACTION: THROW_BOMB → deselecting`), so Java burns a no-op step and re-activates;
Rust actually throws the bomb and then ends the game (`game_end` at i=3). Needs the bomb action
ported into `ParityRunner` (deterministic target, jar rebuild) — the same shape as the earlier
`sendThrowTeamMateTarget` work.

---

## ITER80 — goblin: an activation the harness abandons must not become a step

seed 1 i=2: the away Bombardier is activated with `THROW_BOMB`.
`ParityRunner.sendConcreteAction`'s switch handles only
`MOVE, STAND_UP, BLOCK, BLITZ, BLITZ_MOVE, BLITZ_SELECT, STAND_UP_BLITZ, FOUL(_MOVE),
PASS(_MOVE), HAND_OVER(_MOVE), THROW_TEAM_MATE(_MOVE)`; anything else reaches
`default: UNHANDLED_ACTING_ACTION … MatchRunner.inject(new ClientCommandActingPlayer(null, null,
false))` — a deselect that changes nothing and leaves the turn running.

Rust **carried the action out**: it threw the bomb and then ended the game outright
(`game_end` at i=3, where Java plays to i=901).

**Two halves, one cause — the harness abandons the activation and Rust did not.**

1. **Rust (`random_agent`)**: new `is_handled_acting_action`, a mirror of the Java switch, placed
   next to the existing no-target-FOUL deselect. An unhandled action now `continue 'reselect`s:
   the player-pick decisionRng and action-pick actionRng are already spent and the player is
   already in `used_this_turn`, so the team's turn continues with a different player — exactly
   what ParityRunner's deselect does. Test
   `unhandled_acting_actions_mirror_the_parity_runner_deselect`.
2. **Harness (`ParityRunner`)**: that alone left Rust one step SHORT, because Java calls
   `recordStep` at phase 1 — *before* `sendConcreteAction` decides to abandon the action — so the
   abandoned activation was logged as a phantom no-op step (identical pre/post hash) that Rust,
   which re-picks inside its own loop, never produces. Every later step index was shifted by one.
   Phase 1 now checks `isHandledActingAction` and `continue`s **without** recording, the same way
   the existing inactive-player check does (`continue; // rejected pick — decisionRng call
   consumed, no step logged`). The RNG draws still happen and the player is still marked used, so
   only the *log* changes, not the game.

**Verified:** goblin seed 1 advances i=2 → **i=123** (half 2), and the roster's fail count finally
drops: **100 → 99**. Because the harness half is global, the regression sweep was widened: lineman
bb2016 **0/100**, lineman bb2025 **0/100**, and **16** green bb2016 rosters re-verified at 0 —
dark_elf, nurgle, slann, lizardman, ogre, underworld, elf, undead, dwarf, necromantic, renegades,
wood_elf, amazon, human, orc, norse. `cargo test -p ffb-engine` **7103/0**.

Note this closes the same latent hole for `Stab`, `KickTeamMate`, `HypnoticGaze`, `Swoop`, `Punt`,
`BreatheFire`, `ProjectileVomit` and `SecureTheBall` — all of which ParityRunner also deselects.

### FRONTIER for ITER81
goblin seed 1 i=123, half 2, away_08 MOVE: state hashes already differ on entry
(java `12db8b48…` vs rust `5d82c251…`), so the divergence is earlier in half 2 — bisect from the
first mismatching step.
