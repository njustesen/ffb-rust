# PARITY_PROCESS.md — How the state-hash parity campaign works

This is the methodology for driving **per-step state-hash parity** between the Java
(`ffb-server`, ground-truth) engine and the Rust engine. It captures the loop, the tracing
toolkit, and the discipline. The per-campaign *logs* live in `docs/PARITY_TTM.md` (current /
human + ogre tiers) and `docs/PARITY_TIER1.md` (lineman tier); the RNG *contract* the two
agents obey is `AGENT_CONTRACT.md`; step-port invariants are `docs/step_port/INVARIANTS.md`.

## What parity means

`ffb-parity` runs the SAME game (same seed, roster, edition, tier) through both engines
headless, driving each with a deterministic random agent, and compares a **state hash after
every step**. The Java `ParityRunner` (in `ffb-ai`) and the Rust `random_agent.rs` are two
implementations of the same agent contract; the engines must then produce byte-identical
state hashes step for step. The first differing step is the "frontier".

- **Ground truth is the STOCK Java engine.** Never modify `ffb-common` / `ffb-server` engine
  code. Only these are co-editable: the Rust engine (`crates/*`), `random_agent.rs`, and the
  harness `ParityRunner.java`.
- Editing `ParityRunner.java` means rebuilding the Java jar — do this deliberately: commit the
  Rust side first, rebuild, then re-verify the lineman tier is still 100/100 (a prior incident
  lost uncommitted work to a careless rebuild).

## Run commands

From `ffb-rust/ffb-rust`, after `cargo build --release -p ffb-parity`:

```bash
# lineman tier (default, no args) — the regression gate, must stay 100/100
./target/release/ffb-parity

# a specific tier / seed range
./target/release/ffb-parity --home human --away human --edition bb2025 --tier 3 --seeds 1-100
./target/release/ffb-parity --home human --away human --edition bb2025 --tier 3 --seeds 16-16
```

The harness stops at the first failing seed and prints `PARITY FAIL seed=N ... step S:` with
the Java and Rust `state_hash`/`post_hash` for the diverging step. `state_hash(step N)` is the
state entering step N; `post_hash(step N)` is the state after it resolves (== `state_hash(N+1)`).

## The iteration loop

1. **Recompile** the release binary.
2. **Smallest failing seed** — run the range, take the first `PARITY FAIL`.
3. **First divergence** — the reported step is the first hash mismatch. If the *chosen action*
   already differs there, the divergence is that step; if only the hash differs (same action),
   the real divergence is in the *previous* step's resolution (its `post_hash`). When the chosen
   actions match for many steps but a hash differs, the cause is usually an RNG desync a few
   steps earlier — locate its onset by diffing per-step `rng_calls` (it jumps by the extra/missing
   dice at the offending step).
4. **Root-cause ONE divergence** — read the Java step/behaviour that produces the ground-truth
   value, read the Rust translation, and find the single point they differ. No guessing, no hacks.
5. **Rust test** — add a colocated `#[cfg(test)]` test that pins the corrected behaviour.
6. **Verify** — `cargo test -p ffb-engine --release` green; the failing seed advances past the
   frontier; **no regression** (lineman tier 100/100, and the already-green seeds still pass).
   **REVERT immediately if anything regresses.**
7. **Commit** with an explicit path list (never `git add -A`, which would sweep regenerated
   `parity/*.jsonl` and worktrees), message `Parity(<tier>): …`, and append the diagnosis +
   result to `docs/PARITY_TTM.md`. Delete any temp trace files first so the tree stays clean.
8. **Chain** to the next frontier.

## Tracing toolkit (environment variables)

All are read by the `ffb-parity` binary; combine as needed. `strings` is unavailable on this
box — pipe binary-ish output through `tr -cd '[:print:]\n'` first.

| Var | Effect |
|-----|--------|
| `FFB_TRACE=1` | Rust `RUST_STEP i=N` (one per logged step, `i` == the comparator step index), `RUST_ACT_PICK` (action choice), `RUST_SMA`/`RUST_PICK` (move-target prompt + pick), `DRC_DRAW` (decision-rng draws) **and** Java `JSTEP i=N` / `JAVA_P2` / `JAVA_SMA` / `JAVA_PICK` / `JAVA_PASS` (via `-Dffb.parityDebug=true`). |
| `FFB_DICE_TRACE=1` | Per-die `DICE_TRACE pos=N sides=S result=R`; the Java lines carry a full `caller=` stack (which step/behaviour rolled it) — the ground truth for "what is this die". |
| `FFB_DRIVE_TRACE=1` | `DRIVE step=<StepId> stack_len=…` — one line per Rust step the driver runs; the primary tool for silent stalls / wrong step ordering. |

Both engines share one `GameRng` (game dice) seeded per game; the agent's two streams
(`decision_rng`, `action_rng`) are separate and must be consumed identically by both agents.

### Two diagnostic patterns that recur

- **RNG desync (dice differ / hashes differ but the earlier dice matched):** diff the die
  *positions* — `DICE_TRACE pos=` for the same roll is off by one between engines. Walk back the
  per-step `rng_calls` to find where the count first diverges (0→1); that step rolled one extra
  (or one fewer) die. Then find which roll is spurious/missing and why (a skill re-roll not
  used, a negatrait rolled in the wrong branch, a step that shouldn't run, etc.).
- **State-only divergence (dice match, `post_hash` differs):** compare the state STRINGS at the
  post-step. `FFB_TRACE` emits the Rust `state=` field and the Java `JSTEP state=` line; match on
  the turn marker and diff player by player. Format:
  `h<half>t<turnH><turnA>a<active> b<x>,<y>,<inplay> pa00:x,y,State|a01:…|h00:…`, where
  `away_N = a(N-1)`, `home_N = h(N-1)`.

## Discipline (non-negotiable)

- Fix ONLY the Rust engine / agent / co-editable harness; the Java engine stays stock.
- Root-cause every divergence to a specific Java-vs-Rust difference; add a Rust test; verify
  advance **and** no regression before committing; REVERT if regressed.
- Single-threaded. Never let a subagent mutate git in a shared working dir (a prior parallel
  agent ran `git stash` and wiped work).
- Own the *how*; surface only genuine requirement/goal questions.

## Tiers

The campaign runs progressively richer rosters so each tier exercises new mechanics:
- **lineman v lineman** (`PARITY_TIER1.md`) — movement, dodge, GFI, blocks, pickup, kickoff.
- **full human v full human** (`PARITY_TTM.md`) — adds Block, Dodge/Catch (catchers), Pass, and
  an **Ogre** (Big Guy: Bone Head, Throw Team-Mate) → negatrait rolls, pass/catch re-rolls,
  blitz/foul edge cases.
- **ogre v ogre** — exercises Throw Team-Mate (no human roster has a Right-Stuff/throwable
  player).

## Coverage

`docs/COVERAGE_REPORT.md` (+ `coverage_report.html`) show which mechanics a parity run actually
exercised — actions, dice, injuries, kickoff events, the `GameEvent` catalog. Coverage is a
*breadth* check (did we touch mechanic X), separate from the per-step hash *correctness* check.

## A sweep is only VALID if the run exits 0 and prints `rust_total`

`grep -c "^PARITY FAIL"` counts the ABSENCE of failure lines, which is also what you get when the
harness dies before comparing anything. On 2026-08-14 a whole bb2020 matrix was reported green this
way: the Rust engine panicked on the first game of every roster
(`bb2020/stand_firm_behaviour.rs:37`), the process exited 101, and every roster read "0 fails".

**Use the POSITIVE signal the harness already prints — do not count the absence of failures.**

    PARITY: 100/100 games match[, but required coverage items are MISSING]

That line states how many games were actually COMPARED. Grep for it, and check the denominator is
the seed count you asked for:

    out=$(./target/release/ffb-parity --home R --away R --edition E --tier 3 --seeds 1-100 --no-abort 2>&1)
    echo "$out" | grep -oE "PARITY: [0-9]+/[0-9]+ games match"   # empty => NOTHING RAN

Supporting guards:
- **exit code**: a panic is **101**. Note a clean parity run still exits **1** when the tier-3
  coverage checklist has missing items, so exit != 0 does NOT mean parity failed — but 101 does mean
  the process died.
- **the combined timing line** `TIMING java_total=… rust_total=… (N seeds; batched JVM)`. A run that
  prints only the java-only `TIMING java_total=… (batched JVM, N seeds)` never finished its Rust
  loop.
