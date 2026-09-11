---
description: Run one iteration of the FFB matrix-green campaign — drive the re-draft's red cells green
---

Run ONE iteration of the matrix-green campaign, then stop and report.

## Goal

**Every cell of the parity matrix green at all three sampling scales: 330/330.**

110 cells (29 bb2016 / 40 bb2020 / 41 bb2025) x scales `1.0`, `0`, `1e6`, 100 seeds each, Rust
against the stock Java engine under the heuristic agent.

User decision 2026-09-11: **fix reds until the matrix is green before any coverage work.** This
supersedes §H.14 phase 2 (Brawler/Hatred) — which cannot be gated cleanly anyway, because two of its
seven Brawler cells are red. It also matches `docs/BACKLOG.md`'s own standing rule: commit only at a
green matrix.

Done = a full sweep printing `PARITY: 100/100 games match` on all 330 gates, 0 failures, 0 panics,
0 missing verdicts. `100/100 games match, but required coverage items are MISSING` **is a PASS** —
that trailer is the mechanic-coverage checklist, not parity.

## Where it stands — 16 gates / 13 cells red

Baseline: `docs/SWEEP_2026-09-10_REDRAFT.txt` (311/330). **Its three `goblin` bb2016 reds are
STALE** — that sweep ran a binary built before §H.16 landed; re-gated on the committed engine the
cell is 100/100 at all three scales. Do not re-investigate it.

Ordered queue, grouped so one investigation closes several gates. Work top down; each group's
evidence is in `docs/BACKLOG.md` §H.16-§H.21.

| # | group | gates | cells |
|---|---|---:|---|
| A | turn-boundary / ACTIVE-bit family | 3 | `gnome` bb2025 @1.0, `old_world_alliance_treeman` bb2020 @1e6, `renegades_37733` bb2025 @1.0 |
| B | "Java knocks down or injures, Rust does not" | 3 | `nurgle` bb2020 @0, `human` bb2020 @0, `goblin` bb2025 @1.0 |
| C | Cloud Burster re-asks the interception dialog (§H.17) | 1 | `high_elf` bb2020 @1e6 |
| D | **Java SETUP hang — HARNESS fix** (§H.19) | 1 | `khorne` bb2020 @0 |
| E | die-SIZE divergence (§H.18) | 3 | `goblin` bb2020 @1.0 / @0 / @1e6 |
| F | untriaged singles | 5 | `khemri` bb2025 @1.0, `renegades` bb2020 @1.0, `slann` bb2020 @1e6, `dark_elf` bb2025 @1.0, `renegades_37733` bb2025 @1e6 |

Pull **C** forward if A stalls: it is the only thing standing between the matrix and *"an
interception never succeeds in 330 gates across 41,514 pass rolls"*, so it buys a structural coverage
hole as well as a gate. Its root cause is already known — Java keeps `interceptorChosen` in the PASS
STATE, Rust keeps it on the STEP, so the re-pushed step shows the dialog again and takes a whole
second decision. The fix is a model change: move the flag onto the game's pass state, set it when the
choice is answered, clear it when a new pass begins. Do NOT make `set_parameter(InterceptorId)` set
it — `execute_step` publishes `InterceptorId(None)` on the no-interceptors path, so that would skip
the FIRST dialog too.

**D is not an engine bug.** `SPIN: step=SETUP dialog=SETUP_ERROR mode=SETUP`, force-ended at
`END_REASON: max_iterations iter=2000000`, with a KO'd home player — the team was setting up with
fewer than 11 available. Fix it in `ParityRunner.java` (co-editable). Prior art: the reserves fix
(`canonical_setup_action` + `ParityRunner.placeReserves`) — filter available FIRST, then cap at 11
fielded; placing the first 11 jerseys and filtering afterwards under-fields a short squad and Java
loops.

## Non-negotiable rules

- **Java is the truth.** Never edit `ffb-common` / `ffb-server` as a fix. Co-editable: Rust
  `crates/*` and the `ffb-ai` harness. An env-gated probe in `ffb-server` is fine as a LOCAL,
  uncommitted trace — it needs `mvn -o -pl ffb-server,ffb-ai install` and invalidates
  `--reuse-java`.
- **Every Rust engine fix is a 1:1 port** of the corresponding Java method. Read the Java first. No
  hacks, no parity-only special cases, no constant tuned to make a seed pass. Where Java diverges
  from the printed rulebook, **Java wins** — it is what we are porting.
- **Every fix lands with a colocated `#[cfg(test)]` regression test**, and the test must be shown to
  FAIL without the fix. Assert the published SHAPE, not a die count: a count passes vacuously if the
  roll is made and discarded.
- **A sweep counts only if** the process exits without panicking AND prints `PARITY: N/M games match`
  with the denominator asked for. Counting the absence of `PARITY FAIL` lines is not a measurement.
- **`--reuse-java` is an iteration-speed tool only.** It once reported a stale cache as valid and
  turned a 100/100 gate into 30/100. No gate verdict is valid without a fresh JVM.
- **Harness edits land in BOTH Java trees** (`C:/Users/Admin/niels/ffb/ffb` builds the jar;
  `ffb-rust/ffb-java/ffb` is the tracked copy). `python scripts/check_java_trees.py --fix`, then
  rebuild. Maven is `C:/Users/Admin/bin/maven/bin/mvn`, not on PATH, use `-o`.
- Never run two parity runs of the SAME edition+matchup concurrently, even on disjoint seeds.
- Never `git checkout --` a probed file; remove probes with targeted edits and read every `-` line of
  the diff.
- Commit with an explicit path list — never `git add -A`; it sweeps `parity*/**.jsonl`, the
  `out/out2/out3` census dirs and the agent worktrees.

## Per-cell gate

```bash
cd C:/Users/Admin/niels/ffb-rust/ffb-rust
for SC in 1.0 0 1e6; do
  FFB_PARITY_ROOT="parity_g_${CELL}_${ED}_${SC}" ./target/release/ffb-parity.exe \
    --home $CELL --away $CELL --edition $ED --tier 3 --seeds 1-100 --no-abort \
    --agent heuristic --heur-scale $SC --heur-classes all
done
```

Plus a control: one green cell that exercises the same code, re-gated to show no regression. For a
shared-code change (`injury.rs`, `util_server_injury.rs`, the state hash) run `cargo test --workspace
--release` and at least two green cells in different editions.

## Iteration procedure

1. **Orient.** Read the TAIL of `docs/BACKLOG.md` — the previous iteration's stated "next", and the
   refuted hypotheses for the group you are on. Do not re-run a refuted hypothesis.
2. **Pick the target**: the group at the top of the queue that is not closed, and within it the cell
   with the FEWEST failing seeds.
3. **Localise before theorising.** `scripts/statediff_step.py <trace.log> --step <i+1>` first, every
   time: it names the field that differs — header, ball, or a specific player slot — before any dice
   are looked at. It distinguishes "the ball moved" from "a player's state differs" from "the dice
   diverged", and it found a bug with no dice signature at all.
   - The step number in a `PARITY FAIL` line is the **0-based comparison index**; the trace and the
     JSONL call the same step `i = index + 1`. Diff `--step index+1`.
   - **Read the active bits.** The trailing digit of each player row is the ACTIVE bit and it decides
     whether a player can be activated. A team's previously-used players flipping 0 -> 1 is a
     NEW-TURN RESET, which tells you a turn ended; if it ended with players still available, that is
     a **turnover**, and finding its cause is the task.
   - **Decode the `f` field**: `blitz_used, foul_used, hand_over_used, pass_used`, home then away.
     Flags and active bits differing are usually CONSEQUENCES of a turn boundary, not causes.
   - `FFB_DRIVE_TRACE=1` gives `DRIVE step=<Name> stack_len=.. rng=..` — the step sequence and which
     step consumed which die. Best single tool for "which code ran".
   - `FFB_DICE_TRACE=1`: **both** engines print `DICE_TRACE`, told apart by the `caller=` stack,
     which only Java carries. Compare the two streams by `sides=`/`result=`, not per-step
     `rng_calls`. A **die-SIZE** difference at the same position means the engines are in different
     code, not that a roll is missing.
   - `FFB_DIE_AT=<n>` prints a Rust backtrace at an exact die position.
   - Find the LIVE path before theorising — many per-edition `step/bb20xx/*.rs` files are dead. Grep
     `step/driver.rs` for the arm that dispatches the StepId.
4. **Root-cause ONE divergence**, fix it as a 1:1 port, add the regression test.
5. **Gate.** The target's failure count must strictly drop, the controls must not move, the
   workspace tests must be clean. Else REVERT — but diff the failing SEED SETS first: a correct fix
   can measure neutral or worse, and a half-fix has measured worse than none.
6. **Commit** (explicit paths) and append to `docs/BACKLOG.md`: what diverged, the root cause, the
   fix, the gate numbers, and the next concrete step. **Record refuted hypotheses too** — they are
   what stops the next pass repeating the work.
7. **Report** briefly and stop. One divergence per iteration.

When every group is closed, run the full sweep to certify:
`scripts/sweep_worker.ps1` x 4 shards (see §H.12 / the 2026-09-10 sweep for the invocation), then
`python scripts/sweep_verdicts.py <sweep.log> --out docs/SWEEP_<date>.txt --json docs/sweep_<date>.json`.

## Fault patterns that have each bitten more than once

- **One edition's twin fixed, the other missed.** The recurring shape of this whole campaign. §H.16.2
  was already correct in bb2020/bb2025 *and already had a test*; only the bb2016 copy was wrong.
  Before fixing, grep all three editions for the same call and check which are right.
- **The rng-less variant used where it cannot be.** `drop_player` / `stun_player` say in their own
  doc comments that they are only for call sites "where no Ball & Chain player can occur". ~20 call
  sites still use them and the assumption is unaudited. **Do not fix them speculatively** — wait for
  a red and fix the site it names.
- **Nesting, not logic.** §H.16.3 was Java nesting `putPlayerIntoBox` inside the precedence guard
  while Rust had it outside. Zero dice differed. Compare the CONTROL FLOW, not just the statements.
- **Hash-invisible state.** The hash carries half/turn/active/score/ball/weather/turn-mode, per-team
  flags and re-rolls, the acting player, and for each of the first 11 players by number: coordinate,
  state label, effective MA/ST/AG/AV and the ACTIVE bit. It does **not** carry `passing`, the rooted
  flag, casualty severity, prayer-granted skills, or anything about players numbered above 11. A
  divergence in those stays silent until a downstream mechanic reads it.
- **Green is not exercised.** A mechanic no drafted roster carries, or that the agent never chooses,
  is untested however green the gate.

## Traps

- **Do not infer from step adjacency.** It produced three confident wrong hypotheses in a row on
  group A. When both traces show the same die and the same state and then diverge, the trace is
  exhausted — add a probe.
- **Never census a running sweep.** `scripts/sweep_census/agg.py` caches one JSON per gate and skips
  a gate whose file exists, so it will cache in-flight gates and report itself complete. Three gates
  came back `games=0` while the sweep called two of them 100/100 — indistinguishable from a vacuous
  green. If it has happened, delete `out/`, `out2/`, `out3/` wholesale.
- **Event names are not the mechanic names.** There is no `brilliantCoaching` event; the re-roll
  grant is `kickoffExtraReRoll`, shared with Cheering Fans. Grepping the obvious name returns 0 and
  looks like a dead mechanic.
- **`JSTEP` is on stderr.** A probe on stdout does not interleave with it and through a pipe can
  appear as one block after another. Probe on stderr and carry the step index anyway.
- **Tests can encode the bug.** If a test fails on a fix you believe in, read the test against the
  Java before touching the fix.
- **Stale artifacts lie.** Read a live `FFB_TRACE` run, not a leftover `parity*/**.jsonl`.
- **A binary can be stale.** The 2026-09-10 sweep's `goblin` bb2016 reds are stale for exactly this
  reason. Rebuild before gating, and never rebuild `target/release` while a gate or sweep is running.

## Stopping

Do **not** stop the loop on your own judgement — not at a stall, not on a hard divergence. Switch
tactics instead: a different cell in the same group, a different group, or a fixture test to isolate
engine-vs-harness. The loop ends only when all 330 gates are green, or when the user says stop.

An iteration that root-causes but does not fix is a legitimate outcome **provided it is labelled
one**: commit the investigation, report the gate honestly as unchanged, and name the next concrete
step. Do not dress a partial as a win.
