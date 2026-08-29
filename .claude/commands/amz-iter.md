---
description: Run one iteration of the FFB amazon heuristic-agent parity campaign (all three rulesets)
---

Run ONE iteration of the **amazon-vs-amazon** heuristic-agent parity campaign, then stop and report.

## Goal

`HeuristicAgent` drives **both** engines on the same seed, with identical per-step state hashes, at
**100/100 on amazon-vs-amazon in bb2016, bb2020 AND bb2025**, at every scale.

```bash
cd C:/Users/Admin/niels/ffb-rust/ffb-rust
for E in bb2016 bb2020 bb2025; do
  for S in 0 1.0 1e6; do
    ./target/release/ffb-parity --home amazon --away amazon --edition $E --tier 3 \
        --seeds 1-100 --no-abort --agent heuristic --heur-scale $S --heur-classes all
  done
done
```

Nine x `PARITY: 100/100 games match` = the parity half of the goal.

`100/100 games match, but required coverage items are MISSING` is a **PASS** for parity — that
trailer is the tier-3 coverage checklist, not parity. But on THIS campaign the checklist is also a
deliverable (see "The three halves" below), so do not ignore it either.

Ledger (**read its TAIL first, every iteration**): `ffb-rust/docs/PARITY_AMAZON_CAMPAIGN.md`.
Agent spec: `ffb-rust/AGENT_CONTRACT_HEURISTIC.md`. Process: `ffb-rust/docs/PARITY_PROCESS.md`.
The completed lineman campaign — read it once, it is the playbook:
`ffb-rust/docs/PARITY_HEURISTIC_CAMPAIGN.md` and `.claude/commands/heur-iter.md`.

## Why amazon, and what is new about it

Lineman rosters have **no skills**. Amazons have one on every player, in all three editions, and
they are exactly the skills that reach engine code the campaign has never run:

| | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Linewoman | Dodge | Dodge | Dodge |
| Thrower | Dodge, **Pass** | Dodge, **Pass**, **Safe Pass**, **On the Ball** | same as bb2020 |
| Blitzer | Dodge, **Block** | Dodge, **Hit and Run**, **Jump Up** | same as bb2020 |
| 4th slot | Catcher: Dodge, **Catch** | Jaguar Warrior: Dodge, **Defensive** | Catcher: Dodge, **Defensive** |

Consequences to expect, and to check for BY NAME rather than waiting for them to surface as seeds:

- **Dodge on all 12** changes the dodge roll (re-roll on failure) and changes BLOCK results — a
  `DEFENDER_STUMBLES` resolves differently. Every path the agent prices as "cost of leaving a tackle
  zone" is mispriced if `Reach` does not know about it.
- **Block** changes block-die selection and the follow-up decision.
- **Pass / Safe Pass / Catch** put the ball-action chain under real skills for the first time. The
  lineman campaign measured `sendPassAction` / `sendHandOverAction` **unreachable** only because
  lineman ball actions route through `sendMoveAction`; both still re-pick their own target instead of
  honouring `heuristicTarget`. **Expect this to bite in the first few iterations.**
- **On the Ball** is a reactive move on an opponent's pass declaration — a whole prompt path.
- **Hit and Run** and **Jump Up** are bb2020/bb2025 only, so the editions diverge structurally, not
  just by generator.
- The three editions have **different rosters** (bb2016 has a Catcher and no Jaguar Warrior;
  bb2020's 4th slot is the Jaguar Warrior Blocker). A fix for one edition is not automatically a fix
  for another — but always diff the twin before assuming it isn't.

## The three halves of the goal

The user asked for three things. All three must hold at the end, and each iteration should say which
one it moved.

1. **Parity** — the nine gates above. Java is the truth; when Rust and Java disagree, **Rust is
   wrong until proven otherwise** and the fix is a 1:1 port of the Java method.
2. **The agent plays amazons well** — "utilize the new race best possible while keeping it fast and
   somewhat simple". Concretely: the value model and `Reach` should be aware of the skills that
   change the *costs and risks they already model* (Dodge on a dodge roll, Block on a block, Catch on
   a catch, Pass/Safe Pass on a throw). That is a small, principled set of terms.
   **Simplicity is a stated requirement, not a preference.** Do not add a term the agent cannot
   justify from a quantity it already computes; do not add a per-skill special case where a modifier
   on an existing probability will do; do not regress run time (record `TIMING ... rust_total=` every
   iteration and treat a large rise as a failure to be explained). Every scoring change is a change
   to BOTH agents (Rust and the Java port) and must keep the cross-language goldens bit-identical or
   update them deliberately.
3. **Event coverage** — run the tier-3 coverage checklist and read it. The point of a skilled roster
   is that it reaches engine code lineman play cannot. If a skill above is on the pitch and its
   events are absent from the run, that is a finding: either the agent never creates the situation
   (fix the agent) or the engine never fires it (a bug, or a genuine dead path — say which).
   `docs/EVENT_COVERAGE.md` and `T3_COVERAGE.md` are where this is recorded.

## Non-negotiable rules

- **Java is the truth.** Never edit `ffb-common` / `ffb-server`. Co-editable: Rust `crates/*` and the
  `ffb-ai` harness (which is where the Java agent lives).
- **Every Rust engine fix is a 1:1 port** of the corresponding Java method. Read the Java first, port
  the Java. No hacks, no parity-only special cases, no constant tuned to make a seed pass.
- **Every fix lands with a colocated `#[cfg(test)]` regression test** (and a JUnit test when the Java
  agent changed).
- **A sweep counts only if** the process exits without panicking AND prints
  `PARITY: N/M games match` with the denominator you asked for AND `TIMING ... rust_total=`.
  Counting the absence of `PARITY FAIL` lines is not a measurement.
- **`--reuse-java` is an iteration-speed tool only.** It has reported a stale cache as valid and
  turned a 100/100 gate into 30/100. No gate is valid without a fresh JVM.
- **Harness edits land in BOTH Java trees** (`C:/Users/Admin/niels/ffb/ffb` builds the jar;
  `ffb-rust/ffb-java/ffb` is the tracked copy). Run `python scripts/check_java_trees.py` (`--fix`)
  and rebuild the jar before gating. Maven: `C:/Users/Admin/bin/maven/bin/mvn`, not on PATH, use `-o`.
- Commit the Rust side before rebuilding the jar. Never `git checkout --` a probed file; remove
  probes with targeted edits and read every `-` line of the diff.
- Never run two parity runs of the SAME matchup concurrently — they clobber
  `parity/<edition>/<home>_vs_<away>/seed_N_*.jsonl`. Amazon and lineman runs CAN overlap.

## No regressions — the standing gate

Every commit must keep ALL of these green. If one goes red, REVERT rather than chase it:

- **The lineman heuristic goal**: nine gates, three editions x scales 0 / 1.0 / 1e6, still 100/100.
  Re-run the cheapest meaningful subset each iteration and the full nine before any push.
- **`--agent random`** lineman tier-3 AND amazon tier-3, 100/100 in all three editions.
- `cargo test --workspace --release` clean; `mvn -o -pl ffb-ai test` clean if Java changed.
- The two Java trees agree (`python scripts/check_java_trees.py`).
- Run time does not blow up: compare `rust_total=` against the previous iteration.

## Iteration procedure

1. **Orient.** Read the TAIL of `docs/PARITY_AMAZON_CAMPAIGN.md` for the frontier and the previous
   iteration's stated "Next".
2. **Pick the target**: the item named as Next, or — if a gate is red — the LOWEST failing seed of
   the edition with the FEWEST failures.
3. **Root-cause ONE divergence.** Ordered by what actually paid off over the 70 lineman iterations:
   - **Census before picking a seed.** How many reds are STALLS (`rust=None` — Rust ran out of
     steps) vs real divergences? Do the three editions share a seed AND step?
   - A shorter Rust log with no state mismatch is a **stall**: `FFB_TRACE=1` →
     `LOOP applied=<action> prompt_after=None finished=false` names the dead action outright.
   - **Dump BOTH agents' full candidate lists at the diverging decision and diff them.** Highest
     yield of any tool. It separates "picked differently" (scoring) from "picked identically and
     declared differently" (delivery) — a distinction no golden fixture can see.
   - `FFB_DIE_AT=<n>` prints a Rust backtrace at an exact die position; use it the moment the two
     dice streams diverge at a known index.
   - `FFB_DICE_TRACE=1` → Java's line carries a `caller=` stack naming the step that rolled. The
     fastest way to find a missing or extra roll — **and on this roster, expect skill re-rolls
     (Dodge, Catch, Pass) to be exactly that.**
   - `FFB_TRACE=1` → `RUST_STEP` / `JSTEP` state strings. Diff the dice STREAM (`sides=/result=`),
     not per-step `rng_calls`.
   - **Diff the two sides' float CONSTANTS** (strip comments and `#[cfg(test)]` from
     `heuristic_agent.rs`; same for the Java `heuristic/` package; compare the multisets).
   - `FFB_DRIVE_TRACE=1` for stalls and step ordering.
   - Find the LIVE code path before theorising: several per-edition `step/bb20xx/*.rs` files are
     dead. Grep `driver.rs` for the arm that dispatches the StepId.

   **Fault patterns to check for by name** — each bit more than once in the lineman campaign:
   - **Ported-but-unreached code.** Right arithmetic, no caller. A golden proves the arithmetic,
     never that production calls it.
   - **Contract rules that live in the harness LOOP, not the scorer.** The heuristic replaced
     `RandomAgent`'s pick loop wholesale; read the two loops side by side.
   - **Two copies of one rule that drifted.** Diff the edition twin — but confirm against Java: the
     twin is sometimes the correct one, and sometimes legitimately different.
   - **Vocabulary and coordinate-FRAME mismatches at a seam** (`HandOff` vs `HandOver`; Java's
     client sends the away coach's frame, Rust agents send canonical).
   - **New for amazon: a skill honoured on one side only.** If Rust consults a skill the Java agent
     does not (or vice versa), the candidate lists diverge with no engine bug at all. Skill lookup is
     by NAME and the names differ between editions — `Bone Head` vs `Bone-Head` has already cost a
     session once.
4. **Fix it** — Rust engine (1:1 port) or the agent — and add the regression test.
5. **Gate before committing** (the standing gate above, plus: the target's failure count strictly
   drops). Else REVERT.
6. **Commit** with an explicit path list (never `git add -A` — it sweeps `parity/*.jsonl` and the
   agent worktrees), then append a `## ITER<n>` section to the ledger: what diverged, the root
   cause, the fix, the gate numbers, the timing, and the next frontier.
7. **Report** briefly and stop. One divergence per iteration.

## Traps that cost real time (all of these actually happened)

- **`JSTEP` is printed on `System.err`.** A probe on `System.out` does not interleave with it. Probe
  on stderr, and carry `stepIndex` so the two can be aligned however they buffer.
- **A correct fix can measure WORSE, and a correct fix can measure nothing.** Diff the failing SEED
  SETS before concluding a fix is wrong.
- **Tests can encode the bug.** If a test fails on a fix you believe in, read the test against the
  Java before touching the fix.
- **A gate cannot catch what it never executes.** "Green" is not "exercised" — which is the entire
  reason this campaign exists.
- **Counting your own edits does not tell you what you missed.** Grep for the pattern that must no
  longer exist ANYWHERE and confirm it returns nothing.
- **Regex probe-removal can eat live code.** `git diff` after every probe removal; read every `-`.
- **Stale artifacts lie.** `parity/*_events.jsonl` is left over from an earlier run.

## Stopping

Do **not** stop the loop on your own judgement — not at a stall, not on a hard divergence. Switch
tactics instead (different seed, different edition, a fixture test to isolate agent-vs-engine).

An iteration that root-causes but does not fix is a legitimate outcome **provided it is labelled
one**: commit the investigation with the gate honestly reported as unchanged, and name the next
concrete step. Do not dress a partial as a win.

The loop ends when all three halves hold — nine parity gates at 100/100, the agent's amazon-specific
scoring in and justified, the coverage analysis done and recorded — **or when the user says stop**.
When the goal is met: update the docs (this file's status block, the ledger,
`AGENT_CONTRACT_HEURISTIC.md` if the agent changed, `EVENT_COVERAGE.md` / `T3_COVERAGE.md`,
`BACKLOG.md` for anything deferred), commit, and **push**.
