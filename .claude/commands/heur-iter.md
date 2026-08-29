---
description: Run one iteration of the FFB heuristic-agent parity campaign (all three rulesets)
---

Run ONE iteration of the heuristic-agent parity campaign, then stop and report.

## Status — 🏁 GOAL MET (ITER70, 2026-08-29, commit `3b3746b12`)

The objective below is **achieved**. There is no active goal; the loop is stopped.
**Picking a follow-on tier is the user's decision** — see "Candidate next tiers" at the bottom.

| `--heur-classes all`, lineman v lineman, seeds 1-100 | argmax (0) | sampled (1.0) | uniform (1e6) |
|---|---|---|---|
| bb2016 | **100/100** | **100/100** | **100/100** |
| bb2020 | **100/100** | **100/100** | **100/100** |
| bb2025 | **100/100** | **100/100** | **100/100** |

Supporting gates, all green: fourteen-class rung 100/100 x 3 editions; `--agent random` lineman
tier-3 100/100 x 3 (never regressed once across 70 iterations); `cargo test --workspace --release`
14,664/0; `mvn -o -pl ffb-ai test` 35/0; the two Java trees agree.

Coverage moved as the campaign predicted: **GFI rolls 0 -> 5,663** and **touchdowns 0 -> 6** per 100
games (`T3_COVERAGE.md`). See `docs/DEAD_STEP_INVENTORY.md` for what that does and does not
reclassify.

## Goal (the objective — kept for reference and for re-running the gate)

`HeuristicAgent` drives **both** engines on the same seed, with identical per-step state hashes,
at **100/100 on lineman-vs-lineman in bb2016, bb2020 AND bb2025**.

```bash
cd C:/Users/Admin/niels/ffb-rust/ffb-rust
for E in bb2016 bb2020 bb2025; do
  ./target/release/ffb-parity --home lineman --away lineman --edition $E --tier 3 \
      --seeds 1-100 --no-abort --agent heuristic --heur-scale 1.0 --heur-classes all
done
```

Nine x `PARITY: 100/100 games match` (three editions x `--heur-scale` 0, 1.0, 1e6) = done.

`100/100 games match, but required coverage items are MISSING` **is a PASS** — that trailer is the
tier-3 coverage checklist, not parity. Two of its items, `action Pass` and `action HandOver`, count
only the IMMEDIATE declarations and so read 0 while the heuristic declares the MOVE variants; the
same run records 128 pass rolls. Logged in `docs/BACKLOG.md`.

Ledger (**read its TAIL first, every iteration**): `ffb-rust/docs/PARITY_HEURISTIC_CAMPAIGN.md`.
Agent spec: `ffb-rust/AGENT_CONTRACT_HEURISTIC.md`. Process: `ffb-rust/docs/PARITY_PROCESS.md`.

## Non-negotiable rules

- **Java is the truth.** Never edit `ffb-common` / `ffb-server`. Co-editable: Rust `crates/*` and the
  `ffb-ai` harness (which is where the Java agent lives).
- **Every Rust engine fix is a 1:1 port** of the corresponding Java method. Read the Java first, port
  the Java. No hacks, no parity-only special cases, no constant tuned to make a seed pass.
- **Every fix lands with a colocated `#[cfg(test)]` regression test** (and a JUnit test when the Java
  agent changed).
- **A sweep counts only if** the process exits without panicking AND prints
  `PARITY: N/M games match` with the denominator you asked for AND `TIMING ... rust_total=`.
  Counting the absence of `PARITY FAIL` lines is not a measurement. (Exit code 1 with
  `100/100 games match, but required coverage items are MISSING` is a PASS — that is the tier-3
  coverage checklist, not parity.)
- **`--reuse-java` is an iteration-speed tool only.** It has reported a stale cache as valid and
  turned a 100/100 gate into 30/100. No gate, and no "it went red" conclusion, is valid without a
  fresh JVM.
- **Harness edits land in BOTH Java trees** (`C:/Users/Admin/niels/ffb/ffb` builds the jar;
  `ffb-rust/ffb-java/ffb` is the tracked copy). Run `python scripts/check_java_trees.py` (`--fix`)
  and rebuild the jar before gating. Maven: `C:/Users/Admin/bin/maven/bin/mvn`, not on PATH, use `-o`.
- Commit the Rust side before rebuilding the jar. Never `git checkout --` a probed file; remove
  probes with targeted edits and read every `-` line of the diff.
- Never run two parity runs of the SAME matchup concurrently — they clobber
  `parity/<edition>/<home>_vs_<away>/seed_N_*.jsonl`.

## Iteration procedure

1. **Orient.** Read the TAIL of `docs/PARITY_HEURISTIC_CAMPAIGN.md` for the current frontier and the
   previous iteration's stated "Next".
2. **Pick the target**: the item named as Next, or — if a gate is red — the LOWEST failing seed of
   the edition with the FEWEST failures.
3. **Root-cause ONE divergence.** Ordered by how often each actually paid off over 70 iterations:

   - **Census before picking a seed.** Count how many reds are STALLS (`rust=None` in the FAIL line
     — Rust ran out of steps) versus real divergences, and compare the three editions' red lists for
     a shared seed AND step. Two commands turned "87 bb2016 reds" into "one dead command" (ITER59),
     and the shared-seed check found a cause worth two editions at once (ITER54/62).
   - **A shorter Rust log with no state mismatch is a STALL, not a subtle divergence.**
     `FFB_TRACE=1` → `LOOP applied=<action> prompt_after=None finished=false` names the dead action
     outright (ITER58-61).
   - **Dump BOTH agents' full candidate lists at the diverging decision and diff them.** This is the
     single highest-yield tool for an agent disagreement: it localised a foul weight to 6 rows out
     of 1,378 (ITER45), and twice proved the lists were IDENTICAL — so the fault was in the
     DELIVERY, not the scoring (ITER48, ITER63). "Picked differently" vs "picked identically and
     declared differently" is the distinction no golden fixture can see.
   - **`FFB_DIE_AT=<n>`** prints a Rust backtrace at an exact die position. Use it the moment the
     two dice streams diverge at a known index; it found ITER69 in one command.
   - `FFB_DICE_TRACE=1` → Java's line carries a `caller=` stack naming the step that rolled. Fastest
     way to identify a missing or extra roll (ITER65: the extra die was `handleStaller`).
   - `FFB_TRACE=1` → `RUST_STEP` / `JSTEP` state strings for a state-only divergence. Diff the dice
     STREAM (`sides=/result=`), not per-step `rng_calls` — a count gap is often step-boundary
     attribution (ITER7).
   - **Diff the two sides' float CONSTANTS** (strip comments and the `#[cfg(test)]` module from
     `heuristic_agent.rs`, do the same to the Java `heuristic/` package, compare the multisets). Two
     texts, not two executions — it found the coverage terms in minutes, a defect no execution diff
     could surface at the scale being gated (ITER56).
   - `FFB_DRIVE_TRACE=1` for stalls / wrong step ordering (`DRIVE step=<Name> stack_len=...`).
   - For an agent-side disagreement, extend the cross-language goldens
     (`agent/testdata/det_math_golden.txt`, `sampler_golden.txt`) rather than reasoning about it.
   - Find the LIVE code path before theorising: several per-edition `step/bb20xx/*.rs` files are dead.
     Grep `driver.rs` for the arm that dispatches the StepId.

   **Fault patterns to check for by name** — every one of these bit more than once:
   - **Ported-but-unreached code.** The arithmetic is right and nothing calls it (`MoveReplay`,
     `foulWeight`, `novelty`/`floor`, `PathFinderWithPassBlockSupport`). A golden proves the
     arithmetic, never that production calls it.
   - **Contract rules that live in the harness LOOP, not the scorer** (`SKIP_INACTIVE`, `turn < 1`,
     one-activation-per-non-REGULAR-window, `justDeselected`). The heuristic replaced
     `RandomAgent`'s pick loop wholesale and inherited none of them — read the two loops side by
     side rather than waiting for each rule to surface as a seed.
   - **Two copies of one rule that drifted.** Diff the edition twin — but confirm against Java: the
     twin is sometimes the CORRECT one (ITER53) and sometimes legitimately different (ITER61).
   - **Vocabulary and coordinate-FRAME mismatches at a seam.** `HandOff` vs `HandOver`,
     `"HAND_OVER_MOVE"` vs `"HandOverMove"`, and — the sharpest — porting a Java command handler
     without porting its INPUT convention: Java's client sends the away coach's frame, Rust agents
     send canonical, so a faithful `coord.transform()` un-mirrored a coordinate that was never
     mirrored (ITER69).
4. **Fix it** — Rust engine (1:1 port) or the Java agent/harness — and add the regression test.
5. **Gate before committing.** All must hold, else REVERT:
   - the target's failure count strictly drops;
   - `--agent random` lineman tier-3 is still 100/100 in **bb2016, bb2020 and bb2025**;
   - the heuristic rungs already green stay green;
   - `cargo test --workspace --release` clean; `mvn -o -pl ffb-ai test` clean if Java changed.
6. **Commit** with an explicit path list (never `git add -A` — it sweeps `parity/*.jsonl` and the
   agent worktrees), then append a `## ITER<n>` section to the ledger: what diverged, the root
   cause, the fix, the gate numbers, and the next frontier.
7. **Report** briefly and stop. One divergence per iteration.

## Traps that cost real time (all of these actually happened)

- **`JSTEP` is printed on `System.err`.** A probe on `System.out` does NOT interleave with it, and
  through a pipe the whole probe block can appear after the whole step log. That ordering artifact
  produced two confident, wrong conclusions in a row. Probe on **stderr**, and carry `stepIndex`
  anyway so the two can be aligned however they buffer.
- **A correct fix can measure WORSE**, and a correct fix can measure *nothing*. ITER47 declared
  `STAND_UP` for prone players — right about the name, wrong about the semantics (`STAND_UP` is
  stand-up-and-END) — and dropped a seed from 185 to 22. ITER65-66 fixed two of one rule's three
  causes and moved no gate at all. Diff the failing SEED SETS before concluding a fix is wrong.
- **Tests can encode the bug.** Twice a test asserted the Rust behaviour rather than Java's
  (ITER65's inverted `checkForStaller`, ITER69's mirrored pass target) and failed the moment the
  code was corrected — which is the useful half. If a test fails on a fix you believe in, read the
  test against the Java before touching the fix.
- **A gate cannot catch what it never executes.** The random agent never throws a pass in the bb2016
  gate — measured, zero occurrences over seeds 1-12 — which is how ITER69's bug sat under a 100/100
  random gate for 69 iterations. "Green" is not "exercised".
- **Counting your own edits does not tell you what you missed.** ITER56 replaced six call sites, and
  six was also the number that existed afterwards; there were seven. Grep for the pattern that must
  no longer exist ANYWHERE, and confirm it returns nothing.
- **Regex probe-removal can eat live code.** It did once; the build caught it. After removing any
  probe, `git diff` the file and read every `-` line.
- **Stale artifacts lie.** `parity/*_events.jsonl` is left over from an earlier run — read a live
  `FFB_TRACE` run, not the file.

## Ordered work queue — ✅ ALL COMPLETE

Every item below is done; kept because each records what it cost and what it taught.

1. ✅ **Movement** (`--multimove`, then the real agent). Exposed the `is_valid_move` / `Action::Move`
   guard gap and the `has_acted` vs computed `acted()` rush bug.
2. ✅ **Rung `reroll`** — and every other cheap class.
3. ✅ **The main agent port**: `Features` → `Reach` → value model → `build_plans` → WIDE
   `ActivatePlayer` + `Move`, each stage gated by a cross-language fixture first.
4. ✅ **The remaining classes**, then `--heur-classes all`.
5. ✅ **Final gate**: three editions x three scales, plus the full regression set. **9/9 at 100/100.**

Structurally unreachable for this tier, stubbed to fail loudly (`UNPORTED_PROMPT`): `SkillUse`,
`PuntTarget`.

## Candidate next tiers — the user picks, not the loop

None of these is started. Listed with what is already known about each:

- **Richer rosters than lineman.** The obvious next axis, and the one the campaign's own findings
  point at: `sendPassAction` / `sendHandOverAction` still re-pick their target instead of honouring
  `heuristicTarget` (ITER63 measured them unreachable *only because* lineman ball actions route
  through `sendMoveAction`), and `sendThrowTeamMateAction` needs a TTM roster to be reachable at all.
  Expect those to bite immediately.
- **`Mode::Deep`** and the three A/B control modes — explicitly out of scope for this tier.
- **The tier-3 coverage checklist.** Fix `action Pass` / `action HandOver` to count the MOVE
  variants so the run stops printing `REQUIRED ITEMS MISSING`, and decide whether the remaining
  `absent (optional)` rows are worth driving.
- **Two known-wrong things deliberately left alone** (both in `docs/BACKLOG.md`): bb2020's
  `rollXCoordinate` uses `die(24)` where Java is `rollDice(26) - 1`, and
  `legal_activate_player_actions` never checks the ACTIVE bit — made unreachable by the agent fix
  rather than fixed, because fixing it would shorten the eligible list and break the random
  contract's `idx % N`.

## Stopping

Do **not** stop the loop on your own judgement — not at a stall, not on a hard divergence. Switch
tactics instead (different seed, different edition, a fixture test to isolate agent-vs-engine). The
loop ends only when the goal above is met in all three rulesets, or when the user says stop.

Switching tactics is a real move and it worked: after two iterations failed to close one seed
(ITER66-67), the right call was to record the trace precisely and move to the edition holding 44 of
the 47 remaining reds — not to keep grinding the same seed.

An iteration that root-causes but does not fix is a legitimate outcome **provided it is labelled
one**: commit the investigation with the gate honestly reported as unchanged, and name the next
concrete step (ITER54, 66, 67, 68 did this). Do not dress a partial as a win.

**This goal IS met.** The loop is stopped. Run the final gate above to re-verify, and **ask** before
starting any follow-on tier — the next goal is the user's decision, not yours.
