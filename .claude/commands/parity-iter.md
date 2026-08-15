---
description: Run one iteration of the FFB mirror-parity campaign (bb2020 frontier)
---

Run ONE iteration of the cross-ruleset mirror-parity campaign, then stop and report.

## Goal

Every team mirror matchup at 100/100 in ALL THREE rulesets.
- bb2025: 30/30 GREEN (done — regression-guard only)
- bb2016: 30/30 GREEN (done — regression-guard only)
- **bb2020: ACTIVE FRONTIER.** Ledger + full history: `ffb-rust/docs/PARITY_BB2020_CAMPAIGN.md`.
  Process rules: `ffb-rust/docs/PARITY_PROCESS.md`.

## Non-negotiable rules

- **Java is the truth.** Never edit `ffb-java/ffb-common` or `ffb-java/ffb-server` engine code.
  Co-editable: Rust `crates/*`, `random_agent.rs`, and the harness `ParityRunner.java`
  (harness edits need a jar rebuild).
- **Every Rust change is a 1:1 port** of the corresponding Java class/method. Read the Java first,
  port the Java. NO hacks, NO parity-only special cases, NO tuning constants to make a seed pass.
- **Every fix lands with a colocated `#[cfg(test)]` regression test** (both sides where the Java
  harness changed too).
- **A sweep is valid ONLY if the process exits 0 AND prints `TIMING ... rust_total=`.** Counting the
  absence of `PARITY FAIL` lines is NOT a measurement — a Rust panic aborts before any comparison.
- **Never run two parity runs of the SAME matchup concurrently** (even different editions) — they
  clobber `parity/<home>_vs_<away>/seed_N_*.jsonl`. Different matchups in parallel are fine.
- **Never `git checkout --` a probed file** — it destroys uncommitted harness work. Remove probes
  with targeted edits only.
- Parallel subagents sharing this working dir get **read-only git** (no stash/checkout/reset).

## Iteration procedure

1. **Orient**: read the TAIL of `ffb-rust/docs/PARITY_BB2020_CAMPAIGN.md` for the current frontier
   roster, its fail-seed list, and the last iteration's "next step".
2. **Target** the roster with the FEWEST failing seeds; within it, the LOWEST failing seed.
3. **Root-cause** the first diverging step. Tools that actually work:
   - `rng_calls` + `FFB_DRIVE_TRACE=1` + `JIDSTATE` — NOT `DICE_TRACE` global position
     (Java logs per-call, Rust per-die, so positions sit offset).
   - Compare dice by SIDES sequence, not index.
   - State-only divergence (dice match) → diff Rust vs Java state STRINGS at the post-step
     (`FFB_TRACE=1` → RUST_STEP / JSTEP).
   - Find the LIVE code path with a gated `Backtrace::force_capture()`, never by reading — stale
     duplicate impls have cost whole iterations.
   - A stall = a step returning `Continue` with `prompt.is_none()`.
4. **Port the Java fix**, add the regression test.
5. **Gate before committing** (all must pass, else REVERT):
   - the target roster's fail count strictly DROPS
   - `lineman` 100/100 in bb2016 AND bb2020 AND bb2025
   - at least two previously-green bb2020 rosters still 100/100
   - `cargo test --workspace` clean
6. **Commit AND push** when the gate is green. If no progress, commit the documented findings only.
7. **Append an `## ITER<n>` section** to `docs/PARITY_BB2020_CAMPAIGN.md`: what was measured, the
   root cause, the fix, the gate numbers, and the concrete next step. State partial results as
   partial — never claim a green that was not measured.

## Run commands

From `ffb-rust/ffb-rust`:

```bash
cargo build --release -p ffb-parity
./target/release/ffb-parity --home <roster> --away <roster> --edition bb2020 --tier 3 --seeds 1-100 --no-abort
./target/release/ffb-parity --home <roster> --away <roster> --edition bb2020 --tier 3 --seeds N-N
```

## Reporting

End the iteration with a short status line: target roster, before → after count, gate results,
commit hash (or "no commit — findings only"), and the next step. Do not stop the campaign on your
own judgement; the loop continues until the user says stop.
