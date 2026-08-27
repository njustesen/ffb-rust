---
description: Run one iteration of the FFB heuristic-agent parity campaign (all three rulesets)
---

Run ONE iteration of the heuristic-agent parity campaign, then stop and report.

## Goal

`HeuristicAgent` drives **both** engines on the same seed, with identical per-step state hashes,
at **100/100 on lineman-vs-lineman in bb2016, bb2020 AND bb2025**.

```bash
cd C:/Users/Admin/niels/ffb-rust/ffb-rust
for E in bb2016 bb2020 bb2025; do
  ./target/release/ffb-parity --home lineman --away lineman --edition $E --tier 3 \
      --seeds 1-100 --no-abort --agent heuristic --heur-scale 1.0 --heur-classes all
done
```

Three × `PARITY: 100/100 games match` at `--heur-scale` 1.0 **and** 0 **and** 1e6 = done.

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
3. **Root-cause ONE divergence.** Tools that actually work here:
   - Diff the **dice STREAM** (`sides=/result=` sequence), not per-step `rng_calls`. A per-step count
     gap is often step-boundary attribution, not divergence (ITER7).
   - `FFB_TRACE=1` → `RUST_STEP` / `JSTEP` state strings for a state-only divergence.
   - `FFB_DICE_TRACE=1` → the **Java** line carries a `caller=` stack naming the step that rolled it.
     That is the fastest way to identify a missing or extra roll.
   - `FFB_DRIVE_TRACE=1` for stalls / wrong step ordering.
   - For an agent-side disagreement, extend the cross-language golden files
     (`agent/testdata/det_math_golden.txt`, `sampler_golden.txt`) rather than reasoning about it.
   - Find the LIVE code path before theorising: several per-edition `step/bb20xx/*.rs` files are dead.
     Grep `driver.rs` for the arm that dispatches the StepId.
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

## Ordered work queue

Work top to bottom; the ledger's TAIL is the authority on where we are.

1. **Movement first** (`--multimove`). Finish greening `--multimove 4`, then raise it to the full
   MA+2, in all three editions. This is where the coverage is (GFI, touchdowns, the five
   scoring-gated dead steps) and where the 1:1-port gap lives: `is_valid_move` has **zero**
   production call sites in Rust against ten in the Java server, and `Action::Move` carries neither a
   player id nor a `from`, so two of Java's three `CLIENT_MOVE` guards cannot even be evaluated
   (`step_init_moving.rs:69` — "not ported; trust agent path"). Port that gap 1:1.
2. **Rung `reroll`.** The one cheap prompt class with real coverage payoff: the random contract
   always declines, so accepting runs every re-roll path for the first time (0 → 501 per 100 seeds).
   Needs a Java `ReRolledAction` → Rust action-string mapping and the ball carrier.
3. **The main agent port**: `Features` → `Reach` → the value model → `build_plans` → WIDE
   `ActivatePlayer` + `Move`. Gate each stage with a cross-language FIXTURE test (dump Rust's rasters
   and reach keys for fixed boards; assert Java reproduces them) before wiring it to a game.
4. **The remaining cheap classes**: `blockchoice`, `pushback`, `followup`, `blocktarget`,
   `blitztarget`, `touchback`, `kick`. Then `--heur-classes all`.
5. **Final gate**: all three editions × all three scales, plus the regression set above.

Stub any structurally-unreachable prompt so it fails LOUDLY (`UNPORTED_PROMPT`) rather than silently
falling back. Measured unreachable for this tier: `SkillUse`, `PuntTarget`.

## Stopping

Do **not** stop the loop on your own judgement — not at a stall, not on a hard divergence. Switch
tactics instead (different seed, different edition, narrower `--multimove`, a fixture test to isolate
agent-vs-engine). The loop ends only when the goal above is met in all three rulesets, or when the
user says stop.

When it IS met: run the full final gate, update the ledger, `docs/DEAD_STEP_INVENTORY.md` and memory,
report — and then **ask** before starting any follow-on tier (Deep mode, richer rosters). The next
goal is the user's decision, not yours.
