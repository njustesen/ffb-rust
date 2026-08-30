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
Agent spec: `ffb-rust/AGENT_CONTRACT_HEURISTIC.md` (§10 is the turn-loop contract). Plan and
diagnosis of the 2026-08-30 stall: `C:/Users/Admin/.claude/plans/implement-the-new-agent-zazzy-balloon.md`.
The completed lineman campaign — read it once, it is the playbook:
`ffb-rust/docs/PARITY_HEURISTIC_CAMPAIGN.md` and `.claude/commands/heur-iter.md`.

## Status (2026-08-30, ITER30 — `695cb5936`)

**Nine parity gates at 100/100** — bb2016 / bb2020 / bb2025 × scales 0 / 1.0 / 1e6 — plus the nine
lineman gates and the six random controls. Event coverage harvested and analysed in
`docs/EVENT_COVERAGE.md` (findings F1–F5: the event stream is mostly blind to skill use; parity is
the proof; BACKLOG §E6/E7). The parity frontier is EMPTY. Open: the agent-quality half (skill-aware
scoring for Dodge/Block/Catch/Pass/Safe Pass in the value model and `Reach`, kept simple, goldens
updated deliberately) and the BACKLOG §E hygiene units. Any new red: `frontier.sh` first.

## Why amazon, and what is new about it

Lineman rosters have **no skills**. Amazons have one on every player, in all three editions, and
they are exactly the skills that reach engine code the campaign has never run:

| | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Linewoman | Dodge | Dodge | Dodge |
| Thrower | Dodge, **Pass** | Dodge, **Pass**, **Safe Pass**, **On the Ball** | same as bb2020 |
| Blitzer | Dodge, **Block** | Dodge, **Hit and Run**, **Jump Up** | same as bb2020 |
| 4th slot | Catcher: Dodge, **Catch** | Jaguar Warrior: Dodge, **Defensive** | Catcher: Dodge, **Defensive** |

bb2025 also fields the star **Estelle la Veneaux** (jersey 2: Baleful Hex, Sidestep, Guard, Loner 4,
Disturbing Presence). **On the Ball** is what opens the pass-block and kickoff-return windows —
bb2016 has neither, which is why it went green first and why every late red was a window.

## The three halves of the goal

1. **Parity** — the nine gates above. Java is the truth; when Rust and Java disagree, **Rust is
   wrong until proven otherwise** and the fix is a 1:1 port of the Java method.
2. **The agent plays amazons well** — "utilize the new race best possible while keeping it fast and
   somewhat simple". The value model and `Reach` should be aware of the skills that change the
   *costs and risks they already model* (Dodge on a dodge roll, Block on a block, Catch on a catch,
   Pass/Safe Pass on a throw). **Simplicity is a stated requirement.** Every scoring change is a
   change to BOTH agents and must keep the cross-language goldens bit-identical or update them
   deliberately; record `TIMING ... rust_total=` every iteration.
3. **Event coverage** — `docs/EVENT_COVERAGE.md`: which skill events fired, and for each skill on
   the pitch whose events are absent, whether the agent never creates the situation (fix the agent)
   or the engine never fires it (bug, or a genuine dead path — say which).

## Non-negotiable rules

- **Java is the truth.** Never edit `ffb-common` / `ffb-server`. Co-editable: Rust `crates/*` and the
  `ffb-ai` harness (where the Java agent lives).
- **Every Rust engine fix is a 1:1 port** of the corresponding Java method. Read the Java first, port
  the Java. No hacks, no parity-only special cases, no constant tuned to make a seed pass.
- **Every fix lands with a colocated `#[cfg(test)]` regression test written FROM THE JAVA** — "what
  must this do?", never "what does the Rust do?". Seven tests in this campaign pinned a bug because
  they were written from the Rust (ITER6, 10 x2, 12, 23, 21). A test that fails on a fix you believe
  in: read the test against the Java before touching the fix.
- **A sweep counts only if** the process exits without panicking AND prints `PARITY: N/M games match`
  with the denominator you asked for AND `TIMING ... rust_total=`.
- **`--reuse-java` is an iteration-speed tool only.** No gate is valid without a fresh JVM.
- **Harness edits land in BOTH Java trees** (`C:/Users/Admin/niels/ffb/ffb` builds the jar;
  `ffb-rust/ffb-java/ffb` is the tracked copy). `python scripts/check_java_trees.py`, then rebuild:
  `cd C:/Users/Admin/niels/ffb/ffb && C:/Users/Admin/bin/maven/bin/mvn -o -q -pl ffb-ai -am -DskipTests package`.
- Commit the Rust side before rebuilding the jar. Never `git checkout --` a probed file; remove
  probes with targeted edits and read every `-` line of the diff.
- **Never run two parity processes on the same edition+matchup concurrently — not even on disjoint
  seeds.** It produced two false reds in ITER29. Different editions may overlap; the `--agent random`
  control runs under `FFB_PARITY_ROOT=parity_random` and may overlap with anything.
- **Never rebuild `target/release` while a gate is running** (phantom reds). `cargo test` (debug) is
  fine.

## The three loops — how an iteration is shaped

Do not run an outer loop to answer an inner question.

**Inner — one target seed; seconds to minutes; no commit, no gate.**
Target = the LOWEST failing seed of the edition with the FEWEST reds.
1. `sh scripts/first_state_divergence.sh <edition> <seed>` — the exact activation whose RESOLUTION
   diverged, whether the declaration differs, and whether it followed a side/turn flip.
2. Trace both engines at that activation and **diff the traces**:
   - `FFB_STEPTRACE=1` → `RSTATE` (Rust, every prompt) / `JSTATE` (Java, every harness iteration):
     turn mode, acting side, both turn counters, acting player with derived `acted()` next to stored
     `has_acted`, every on-pitch player's `base:active`. **This is the state the per-step hash cannot
     see** and where every late amazon red lived.
   - `FFB_ENDTURN=1` → `RET`/`JET`: which branch ended each turn.
   - `FFB_MOVEP=1` → `RMOVEP`/`JMOVEP`: what each move prompt offered and answered.
   - `FFB_CANDSUM=1` / `FFB_CAND=<k>` → candidate lists and weights at activation k.
   - `FFB_KR=1`, `FFB_DRIVE_TRACE=1`, `FFB_DICE_TRACE=1`, `FFB_DIE_AT=<n>`, `FFB_TRACE=1` as before.
3. Fix (1:1), `cargo test -p ffb-engine`, `cargo build --release -p ffb-parity`, re-run the seed.
   Loop until its first divergence moves past the mechanism or the seed passes.

**Middle — the frontier; ~5–10 min.**
`sh scripts/frontier.sh <edition> [seeds]` (reds default to the latest gate log) → one table; then the
20-seed probe x3 editions. Success = the table shrinks or changes family AND the probe is
≥ baseline. Run frontier and probe for the SAME edition sequentially, never together.

**Outer — the full standing gate; ~40 min; then commit + ledger.**
`gate.sh <tag> <edition>` x3 in parallel (amazon then lineman each) + `rand.sh <tag> <edition>` x3
(isolated root, may overlap) + `cargo test -p ffb-engine` + `mvn -o -pl ffb-ai test` if Java changed
+ `check_java_trees.py`. Compare `rust_total=` against the previous iteration.

## The unit-port rule

A **mechanism** — everything Java does in one step or generator, e.g. a window's open/re-open/close,
or one `ActingPlayer` method and all its callers — is ONE change set. Port all of its parts together.
**Components are never reverted for measuring worse alone**; keep-or-revert is decided on the outer
gate of the whole unit. ITER28's `acted()` was correct and was reverted for measuring worse alone;
ITER25 narrowed a correct rule on a probe that only made sense together with the `push_self` fix in
the same gate. Both cost iterations.

A unit may legitimately produce no gate movement and still be worth committing, if it leaves the path
measurably closer and the ledger says what remains.

## The tool rule

Every analysis script is run first on a GREEN seed (must report no divergence) and on a KNOWN red
(must reproduce the known index) before its output is believed. Three wrong readings in ITER27 and
one in the ITER29 diagnosis were parser bugs, not engine findings. A saved `.err` file does not carry
the binary that produced it — re-measure before building on one.

## Fault patterns to check for by name (each bit more than once)

- **Ported-but-unreached code.** Right arithmetic, no caller. Grep `driver.rs` for the arm that
  dispatches the StepId; several `step/bb20xx/*.rs` twins are dead. `make_step` (bb2025 table) and
  `make_step_for(id, rules)` are different tables — fix the twin the live table imports FIRST.
- **A stub standing in for a generator.** `select_sequence()` was `InitSelecting` + 18 `NoOp`s for
  the whole campaign, and the kickoff-return window was its only caller.
- **Stored flag where Java derives.** `has_acted` (use `acted()`); a Player-only `used_skills` insert
  where Java is `actingPlayer.markSkillUsed` (use `util_server_steps::mark_skill_used`).
- **Raw `player_id = None` where Java is `changeActingPlayer(null)`** (use
  `change_player_action_to_none`).
- **Level-held vs edge-triggered parameters.** A Rust field filled by `consumes_parameter` stays set;
  Java's is re-assigned on every publish.
- **`repeat()` where Java is `pushCurrentStepOnStack()`** — that is `push_self()`.
- **Two callers of one Java helper are not one contract.** The random path freezes
  `eligibleThisTurn`; the heuristic path recomputes. Read the caller you are porting.
- **Contract rules that live in the harness LOOP, not the scorer** — see the contract's §10.
- **Two copies of one rule that drifted** — five `mark_skill_used`, one fixed in the star campaign.
- **Vocabulary and coordinate-frame mismatches at a seam** (`HandOff`/`HandOver`, `home_06`/`Home6`,
  `Some(n)`/`n`) — normalise before diffing.
- **A skill honoured on one side only**; skill lookup is by NAME and names differ between editions.

## Traps that cost real time (all of these actually happened)

- **`JSTEP` is printed on `System.err`.** Probe on stderr, and carry an index so the two sides align.
- **The candidate-count classifier is a LAGGING indicator** (16 activations late on seed 33) and
  mislabels whose-turn divergences as LIST on a mirror matchup. `first_state_divergence.sh` first.
- **The shared jsonl directory.** Before `FFB_PARITY_ROOT`, the random control destroyed 14 of 19
  bb2020 logs. Any post-hoc analysis: re-run the seed with the right agent first.
- **A correct fix can measure WORSE, and a correct fix can measure nothing.** Diff the failing SEED
  SETS, and ask whether it is a component of a unit before concluding.
- **A gate cannot catch what it never executes.** "Green" is not "exercised".
- **Regex probe-removal can eat live code.** `git diff` after every probe removal; read every `-`.
- **Stale artifacts lie.** `T3_COVERAGE.md` is rewritten by EVERY ffb-parity run, including the
  random control and the lineman regression; harvest coverage from a dedicated run.

## Iteration procedure

1. **Orient.** Ledger TAIL for the frontier and the previous "Next".
2. **Middle loop first** if the frontier is stale: `frontier.sh` per red edition.
3. **Inner loop** on the lowest red of the edition with the fewest reds until it moves.
4. **Name the unit** the fix belongs to; port the rest of that unit before measuring.
5. **Middle loop** → **outer gate** → commit with an explicit path list (never `git add -A`) →
   `## ITER<n>` in the ledger: what diverged, the trace lines that named it, the unit, the gate
   numbers, the timing, the next frontier.
6. **Report** briefly and stop.

## Stopping

Do **not** stop the loop on your own judgement — not at a stall, not on a hard divergence. Switch
tactics instead (a different seed, a different edition, a fixture test to isolate agent-vs-engine,
a trace you have not yet diffed).

An iteration that root-causes but does not fix is a legitimate outcome **provided it is labelled
one**: commit the investigation with the gate honestly reported as unchanged, and name the next
concrete step. Do not dress a partial as a win.

The loop ends when all three halves hold — nine parity gates at 100/100, the agent's amazon-specific
scoring in and justified, the coverage analysis done and recorded — **or when the user says stop**.
When the goal is met: update the docs (this file's status block, the ledger,
`AGENT_CONTRACT_HEURISTIC.md` if the agent changed, `EVENT_COVERAGE.md`, `BACKLOG.md` for anything
deferred), commit, and **push**.
