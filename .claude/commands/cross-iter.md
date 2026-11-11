---
description: Run one iteration of the FFB cross-matchup parity campaign — drive the non-mirror reds green
---

Run ONE iteration of the cross-matchup campaign, then stop and report.

## Goal

**Every NON-mirror matchup green: all 4012 ordered pairs of drafted squads, 3 seeds each, all
three editions, Rust against the stock Java engine under the heuristic agent at `--heur-scale 1.0`.**

The 330-gate mirror matrix (a squad against ITSELF) is certified green. A rule whose two halves sit
on OPPOSITE teams — Tackle vs Dodge, Claws vs Thick Skull, Stab vs Regeneration, Diving Tackle vs
a dodger, Foul Appearance vs a blitzer, ... — is reachable only in a cross matchup, and until this
campaign none had been compared on more than 1 random-agent seed. GUI-client interop is explicitly
DEFERRED until logic parity is done (user decision 2026-11-11).

Done = a full cross sweep printing `PARITY: 3/3 games match` on all 4012 gates (0 FAILED, 0
panics, 0 `NO VERDICT`), THEN one full 330-gate mirror re-certification with no regression, THEN
`origin/main` pushed. `games match, but required coverage items are MISSING` is a PASS.

## Where it stands

- Baseline sweep: `docs/CROSS_SWEEP_<date>.txt` (+ `.json` with every red's first-fail seed/step).
- The campaign ledger is **`docs/BACKLOG.md` §I** — read its TAIL first: the previous iteration's
  stated next step and the refuted hypotheses. Do not re-run a refuted hypothesis.
- Sweep logs live on the data volume: `D:/ffb_cross/sweep/<worker>/<edition>/<home>_vs_<away>/`
  (`seed_N_{java,rust}.jsonl`, `seed_N_{java,rust}_reports.jsonl`). Scratch + `sweep.log` under
  `crossjobs/scratch/`.

## Non-negotiable rules

- **Java is the truth and is never edited.** Only Rust `crates/*` and the `ffb-ai` harness
  (`ffb-java/ffb/ffb-ai/.../parity/ParityRunner.java`) are co-editable. An env-gated probe in
  `ffb-server` is a LOCAL, uncommitted trace only.
- **Every Rust engine fix is a 1:1 port** of the corresponding Java method. Read the Java first. No
  hacks, no parity-only special cases, no constant tuned to make a seed pass. Where Java diverges
  from the printed rulebook, Java wins.
- **Every fix lands with a colocated `#[cfg(test)]` regression test** shown to FAIL without the
  fix. Assert the published SHAPE, not a die count.
- **A gate counts only if** the process exits without panicking AND prints `PARITY: N/M games
  match` with the denominator asked for.
- **`--reuse-java` is an iteration-speed tool only.** No gate verdict without a fresh JVM.
- Never run two parity runs of the SAME edition+matchup (same ordered pair) concurrently.
- **Never rebuild `target/release` or the jar while a sweep is running.** Develop and gate single
  pairs from a second target dir: `CARGO_TARGET_DIR=../target-fix cargo build --release -p
  ffb-parity` and run `../target-fix/release/ffb-parity.exe`.
- Never `git checkout --` a probed file; remove probes with targeted edits and read every `-` line.
- Commit with an explicit path list — never `git add -A` (it sweeps `parity*/`, `crossjobs/`,
  `D:`-mirrored census dirs and agent worktrees).

## Per-pair gate

```bash
cd C:/Users/Admin/niels/ffb-rust/ffb-rust
FFB_PARITY_ROOT="D:/ffb_cross/iter_${HOME}_${AWAY}_${ED}" PARITY_JVM_CORES=2 \
  ./target/release/ffb-parity.exe --home $HOME_SQUAD --away $AWAY_SQUAD --edition $ED --tier 3 \
  --seeds 1-30 --no-abort --agent heuristic --heur-scale 1.0 --heur-classes all
```

Gate ladder for a fix: (1) the target pair at seeds 1-30, (2) every other red pair in the same
group at seeds 1-3, (3) two MIRROR controls at 100 seeds — one per edition touched, picking a
squad that fields the mechanic — (4) `cargo test --workspace --release`. For a shared-code change
(`injury.rs`, `util_server_injury.rs`, the state hash, `driver.rs`) add a third mirror control.

## Iteration procedure

1. **Orient.** Tail of BACKLOG §I; the current `docs/CROSS_SWEEP_*.json` red list.
2. **Group the reds by fingerprint**: (first-divergence step pattern, the skill/mechanic both
   squads of the row/column share). One root cause usually reddens a whole ROW (`dwarf vs *`) or
   COLUMN (`* vs wood_elf`) — count rows/columns before picking.
3. **Pick** the largest group; within it the pair with the FEWEST failing seeds.
4. **Localise before theorising.** Re-run the seed with `FFB_TRACE=1` to a log file, then
   `python scripts/statediff_step.py <trace.log> --step <index+1>` — it names the field that
   differs (header, ball, or a player slot) before any dice are looked at.
   - The step number in a `PARITY FAIL` line is the 0-based comparison index; diff `index+1`.
   - Read the ACTIVE bits (trailing digit of each player row); a team's used players flipping
     0 -> 1 is a NEW-TURN RESET — a turnover whose cause is the task.
   - `FFB_DRIVE_TRACE=1` gives `DRIVE step=<Name> ... rng=..` — which step consumed which die.
   - `FFB_DICE_TRACE=1`: BOTH engines print `DICE_TRACE` (Java carries `caller=`). Compare by
     `sides=`/`result=`, never per-step `rng_calls` (Java counts per CALL, Rust per DIE).
   - `JAVA_DIE rng=<n> d6=<v> from=<class.method:line>` in the Java block names the exact Java
     roll site; grep the Rust twin for the same method.
   - `python scripts/report_diff.py <java_reports> <rust_reports>` aligns the report streams.
   - Find the LIVE path before theorising — many `step/bb20xx/*.rs` files are dead. Grep
     `step/driver.rs` for the arm that dispatches the StepId, and `make_step_for` for the
     edition routing.
5. **Root-cause ONE divergence**, fix it as a 1:1 port, add the regression test.
6. **Gate** per the ladder. The target's failing seeds must strictly drop, the controls must not
   move, the workspace tests must be clean. Else REVERT — but diff the failing SEED SETS first: a
   correct fix can measure neutral, and a half-fix has measured worse than none.
7. **Commit** (explicit paths) and append **§I.<n>** to `docs/BACKLOG.md`: what diverged, the root
   cause, the fix, the gate numbers, refuted hypotheses, and the next concrete step.
8. **Report** briefly and stop. One divergence per iteration.

When the red list is empty: re-run the full cross sweep (`scripts/cross_jobs.py --workers 5`,
then 5x `scripts/cross_worker.ps1` sharded, then `scripts/sweep_verdicts.py <sweep.log>
--per-cell 1 --expect 4012 --out docs/CROSS_SWEEP_<date>.txt --json docs/cross_sweep_<date>.json`),
then the mirror re-certification (`sweepjobs/w*.txt` + `scripts/sweep_worker.ps1`, then
`scripts/post_sweep.sh`), then push.

## Fault patterns that have each bitten more than once

- **One edition's twin fixed, the other missed.** Grep all three editions for the same call.
- **Self-delivery of a published parameter.** `apply_effects` delivers every published
  `StepParameter` to the publishing step too (Java `publishParameter` does `setParameter(self)`
  first). A step that uses a field reset as a "do it fresh" flag gets the old value written back
  (§I.1: the dodge die).
- **The harness decline is `SelectPlayer { "" }`.** Every `AgentPrompt::PlayerChoice` is answered
  by the generic arm of `random_agent.rs` with an EMPTY `SelectPlayer`, mirroring Java
  ParityRunner's `new Player[0]`. A step that only matches `Action::PlayerChoice { mode }`
  re-prompts forever and the Rust log ends early (§I.1).
- **Nesting, not logic.** Compare CONTROL FLOW, not statements; zero dice may differ.
- **Hash-invisible state.** The hash does not carry `passing`, the rooted flag, casualty
  severity, prayer-granted skills, or players numbered above 11; a divergence there stays silent
  until a downstream mechanic reads it.
- **Green is not exercised.** Confirm the mechanic FIRED in the gate (grep the Java report stream
  for its name and compare the Rust count) before calling a cross pair "covered".

## Traps

- Do not infer from step adjacency; when both traces show the same die and state and then
  diverge, the trace is exhausted — add a probe (on stderr, with the step index).
- Never census a running sweep; `agg.py` caches in-flight gates as complete.
- Stale artifacts lie: read a live `FFB_TRACE` run, not a leftover jsonl.
- A binary can be stale: check `target/release/ffb-parity.exe`'s timestamp against the last
  engine edit before gating, and never rebuild it under a running sweep.
- Tests can encode the bug: if a test fails on a fix you believe in, read the test against the
  Java before touching the fix.

## Stopping

Do **not** stop the loop on your own judgement — not at a stall, not on a hard divergence. Switch
tactics instead: another pair in the group, another group, a fixture test to isolate engine vs
harness. The loop ends only when the goal is met, or when the user says stop.

An iteration that root-causes but does not fix is a legitimate outcome **provided it is labelled
one**: commit the investigation, report the gate honestly as unchanged, and name the next concrete
step. Do not dress a partial as a win.
