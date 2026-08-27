# Parity campaign — the HEURISTIC agent, both engines

Goal: `HeuristicAgent` drives **both** Rust and Java on the same seed, with every decision and every
per-step state hash identical, at **100/100 on lineman-vs-lineman bb2025**.

Process: `docs/PARITY_PROCESS.md` (unchanged — Java engine is ground truth, fix Rust only,
`ffb-ai` harness is co-editable, a regression test per fix, revert on regression).
Agent spec: `AGENT_CONTRACT.md` §10 (to be written).
Design + measurement history of the agent itself: `docs/HEURISTIC_AGENT.md`.

---

## ITER0 (2026-08-27) — baseline measured, premise confirmed, two open questions closed

`docs/HEURISTIC_AGENT.md` claims the heuristic agent reaches game states random play never did, but
**no heuristic event dumps were checked in** (`parity/heuristic/` did not exist; `git ls-files |
grep -i heuristic` returned only the source, the doc and the HTML viewer). So the premise of this
whole campaign was unverified. Measured it:

```
./target/release/ffb-parity --heuristic 1.0 --mode wide --home lineman --away lineman \
    --edition bb2025 --seeds 1-100 --out parity/heur_baseline      # 32.3s
```

against the checked-in random-agent baseline (`parity/bb2025/lineman_vs_lineman/*_rust_events.jsonl`,
same roster / edition / seed range).

| event | RandomAgent | HeuristicAgent | |
|---|---|---|---|
| `playerAction` (activations) | 27,696 | 14,469 | fewer activations… |
| `playerMoved` | 26,278 | **61,819** | …and 2.4× the movement |
| **squares per activation** | **0.95** | **4.27** | the one-square ceiling is gone |
| **`goForItRoll`** | **0** | **4,400** | GFI was structurally unreachable |
| **`touchdown`** | **0** | **40** | scoring was structurally unreachable |
| **re-rolls consumed** (`"rerolled":true`) | **0** | **501** | the random contract always declines |
| `pushback` | 689 | 1,157 | |
| `pickupRoll` | 93 | 336 | |
| `catchRoll` | 107 | 288 | |
| `passRoll` | 86 | 231 | |
| `handOver` | 35 | 132 | |
| `throwIn` | 2 | 13 | |
| `argueTheCall` | 44 | 59 | |
| `blockRoll` | 1,048 | 1,681 | |
| `dodgeRoll` | 780 | 866 | |

Premise **confirmed**: touchdowns, GFI and re-roll consumption go from exactly zero to substantial.
Those three are the reason this campaign exists — they are the code the parity matrix has never
executed, and they are where the engine bugs will be.

### Declared actions (heuristic, 100 seeds)

```
12198 Move   933 BlitzMove   748 Block   232 Foul   223 PassMove   135 HandOverMove
```

`PassMove` / `HandOverMove` are the move-variant declarations from commits `fb4ee0052`/`1f5e10604`,
so the give chain is live and will be under a state hash for the first time.

### Two reachability questions the plan left open — both CLOSED, both negative

- **`PuntTarget` is NOT reachable.** No `Punt` appears in the declared-action histogram across 100
  seeds. The concern was that `build_plans`' default arm (`heuristic_agent.rs:2637`, flat 0.40)
  declares bb2025 `PlayerAction::Punt`; it never gets offered. Stub it, loudly.
- **`SkillUse` is NOT reachable.** `grep -c skillUse` = **0**. The lineman fixture carries
  `skill_categories_normal: [General]` so the Intensive Training prayer *can* grant a skill
  (`runner.rs:808-816`), but that comment cites a **bb2020** seed and no skill is granted here at
  equal TV in bb2025. Stub it, loudly.

Both were cheap to port and are now out of scope for this tier — which is the point of measuring
before writing 5,000 lines of Java.

**Next:** ITER1 — the Rust hygiene commits (§ "Order of work" step 1): freeze `FFB_HOPELESS_DAMP`,
`canon_key` replacing the seven id sorts, canonical iteration fixing the `build_threat` `threat_str`
tie-break, then `det_math`.

---

## ITER1 (2026-08-27) — Rust hygiene: the ordering key, and a real non-determinism bug

Three changes to `heuristic_agent.rs`, each verified separately, plus one pre-existing breakage
repaired. No Java yet.

### 1a. `FFB_HOPELESS_DAMP` frozen to `const HOPELESS_DAMP: f32 = 0.25`

Env-dependent **policy** cannot be mirrored by a Java agent, and this one scales the value of
advancing a carrier who can no longer reach the endzone in time. It was also a per-call
`std::env::var` (allocation + lock) inside `value_at`, the hottest function in the agent.

### 1b. `canon_key` — player ids no longer enter any ordering

`AGENT_CONTRACT.md` §6 already forbids id ordering ("Rust ids `home_01..home_11` and Java ids
`teamLinemanParityHome1..11` sort differently; ids must never enter an ordering"), but the
heuristic agent sorted by id in **seven** places. Replaced all seven with `(side, jersey nr)` —
the key both state-hash implementations already use — and replaced `pid_key`'s FNV hash of the id
(the `block_memo` key) with a packed `(side<<8)|nr`.

**Verified a byte-exact no-op, not argued to be one:** re-ran 100 lineman seeds and `diff -r`'d the
whole event-dump tree against the ITER0 baseline — identical. All seven sites are single-sided
(`c1`/`ps` = acting team, `foes`/`ep` = opponents, `mates` = team-mates, `Touchback` = receiving
team), and *within a side* `(side, nr)` reproduces lexicographic `home_NN` order exactly. Across
sides the two orders differ (canonical puts home first, lexicographic puts `away_*` first) — which
is harmless precisely because no site mixes sides, and is what the regression test asserts.

### 1c. `build_threat` iterated a randomly-seeded `HashMap` under a tie-break — a live bug

```rust
if (d == 1 || !opp_blitz_spent) && reach > self.threat_reach[s][i] {
    self.threat_reach[s][i] = reach;
    self.threat_str[s][i] = ostr as i8;   // strict `>` — FIRST writer wins the tie
}
```

`field_model.player_coordinates` is a `std::collections::HashMap` with the default **randomly
seeded** hasher. Two opponents standing adjacent to the same square both score `reach == 1.0`, so
they tie, and whichever the hasher yielded first recorded **its strength**. `threat_str` feeds
`strength_factor` -> `exposure` -> every arrival weight.

**Measured, not inferred.** Temporarily reverted just that loop, rebuilt, and ran human-vs-human
seeds 1-20 twice with the same binary: **all 20 seeds produced different event streams between the
two runs.** With the fix, three separate processes produce byte-identical output. So the heuristic
agent was non-deterministic run-to-run on any roster with mixed ST — which also means every
`human`/`orc`/`ogre` A/B number in `docs/HEURISTIC_AGENT.md` carried this as unmodelled noise.
Inert for the all-ST3 lineman fixture, which is why it never surfaced.

Audited the other three `player_coordinates` iterations and left them alone, with the reasoning
recorded on `canon_players`: `positions_stamp` is a commutative `wrapping_add` (and is only a cache
key), `Features::build` writes one distinct square per player plus integer increments plus a
constant-addend f32 sum, and `build_support` uses integer counters and `max`. Only `build_threat`
needed it.

Tests added: `threat_str_tie_is_broken_canonically_not_by_hash_order`,
`canon_players_is_sorted_by_side_then_nr`, `canon_key_reproduces_id_order_within_a_side`.

### 1d. Pre-existing: `ffb-server` did not compile

`crates/ffb-server/src/net/wire_prompt.rs` had a non-exhaustive match — `AgentPrompt::BlitzTarget`
was added by an earlier heuristic-agent commit without a corresponding arm, so `ffb-server` and its
tests failed to build and took `cargo test --workspace` down with them (exit 101). Added the arm as
`None`, matching the established convention for prompts with no client dialog (`BlockTarget`,
`MultiBlockTargets`, `ThrowTeamMateTarget`, `PuntTarget`): the blitz victim reaches the server as a
`ClientCommandTargetSelected`, never as a rendered dialog.

### Gates

- `cargo test -p ffb-engine --release` — **7,309 / 0**.
- `cargo test --workspace --release` — **14,630 / 0, exit 0** (was exit 101 before 1d).
- `ffb-parity --home lineman --away lineman --edition bb2025 --tier 3 --seeds 1-100 --no-abort`
  — **`PARITY: 100/100 games match.`**, exit 0, `TIMING java_total=33.545s rust_total=7.682s`.
- Lineman and human heuristic event dumps byte-identical across the 1b/1c changes.

### TRAP FOUND: `--reuse-java` reported a stale cache as valid

The same gate run **with `--reuse-java` returned `PARITY: 30/100 passed, 70 FAILED`** while
printing `REUSE java logs for lineman vs lineman (bb2025) — cached batch matches`. Re-running the
identical command with a fresh JVM, minutes later and with no code change in between, returned
**100/100**. The cached logs were stale local leftovers (`parity/` is gitignored) and the failures
began at seed 31 with seeds 1-30 clean — the signature of an interrupted earlier batch whose
manifest was written anyway. `java_logs_reusable` checks that every seed log is *present*, not that
it is *current*.

Every failure looked exactly like a real divergence: a different active team and a different
activated player at step 1. Chasing even one of them would have burned an iteration on a phantom.

**Operational rule for this campaign: `--reuse-java` is an iteration-speed tool only. No gate,
and no "it went red" conclusion, is valid without a fresh JVM run.** `docs/PARITY_PROCESS.md` says
reuse "is never silent"; this shows it can be silently wrong, which is worse. Folding the agent
kind / scale / mode / class-mask into the fingerprint (planned anyway for the heuristic arm) does
not fix this, because the staleness was not in the fingerprint's inputs.

**Next:** ITER2 — `det_math.rs` (bit-exact `exp_f32`/`ln_f32`), the 7 call sites, and the
`as u32` -> explicit clamp at `reach_with`. This one deliberately re-baselines the A/B corpus.

---

## ITER2 (2026-08-27) — `det_math`: bit-reproducible `exp`/`ln`, and it cost nothing

`crates/ffb-engine/src/agent/det_math.rs` (new) plus the seven call sites.

### The problem, and why it is only seven lines

f32 `+ - * /` are **correctly rounded** by IEEE-754 in both languages, Java 17+ is unconditionally
strict-FP (JEP 306), and Rust does not contract to FMA — so essentially all of the agent's
arithmetic is already bit-portable for free. Grepping the whole agent confirms there is no `sqrt`,
`powf`, `powi`, `log2` or `log10` anywhere, and the two `.ceil()` calls are exact in both. That
leaves exactly **seven** transcendental calls, where Rust's libm and Java's `Math`/`StrictMath` are
three different implementations with no bit-agreement guarantee.

`docs/HEURISTIC_AGENT.md` §11 prescribes converting the whole agent to i32 milli-weights for this.
That was written before the code existed and assumed transcendentals were pervasive. Replacing
seven call sites is a far smaller blast radius than rewriting 3,657 lines of value model, and it
leaves every weight in the §22-§29 A/B corpus untouched.

### The approach

**The functions do not need to match libm's accuracy. They need to be identical to each other.**
So `exp_f32`/`ln_f32` are built exclusively from correctly-rounded f32 primitives — `+ - * /`,
comparisons, and exponent surgery via `to_bits`/`from_bits` — with a fixed-degree polynomial in a
fixed evaluation order. Every step is bit-determined by the IEEE spec, so `DetMath.java`
(`Float.floatToRawIntBits`/`intBitsToFloat`) will be identical **by construction rather than by
luck**. `exp` uses Cody-Waite reduction plus a degree-7 Horner polynomial and a `2^k` scale; `ln`
splits off the exponent and uses the odd series in `(m-1)/(m+1)`.

348 `(input bits, output bits)` vectors are pinned in
`crates/ffb-engine/src/agent/testdata/det_math_golden.txt`, regenerated only by the deliberately
`#[ignore]`d `emit_golden_table`. The Java twin will assert on the same file, so a drift on either
side fails a unit test instead of a 100-seed sweep.

### The re-baseline that turned out not to be needed

The plan budgeted for a one-time policy shift here, on the reasoning that last-ulp differences
would occasionally flip which option a draw selects. Measured instead of assumed:

| arm | seeds | streams changed |
|---|---|---|
| lineman, sampled (`--heuristic 1.0 --mode wide`) | 100 | **0** |
| human, sampled | 20 | **0** |

**Byte-identical.** `det_math`'s output agrees with the platform libm to within a few ulp over the
ranges the agent actually uses (a unit test holds it to 1e-6 relative for `exp` over non-positive
arguments and 4e-6 for `ln` over `p_step` in `(0, 1]`), and across ~120k decisions the cumulative
softmax walk never landed close enough to a boundary for that to matter. So the §22-§29 corpus
stands unchanged and no re-measurement is owed.

### Also: an explicit clamp on the Dijkstra key

`(-ln(p_step) * KEY_SCALE) as u32` saturates in Rust but is undefined-ish in Java, which converts
out-of-range floats differently. Replaced with `.clamp(0.0, 1.0e9) as i64 as u32` and pinned by
`dijkstra_key_increment_stays_in_the_clamped_range`, which walks the worst case the search can
reach (16 consecutive `p_roll(6)` steps) and asserts the clamp is a no-op there. Verified
byte-identical over 100 lineman seeds. The Java twin can now use `long` throughout with no
reasoning about unsigned casts.

### Gates

- `cargo test -p ffb-engine --lib agent::` — 57/0.
- `cargo test --workspace --release` — **14,636 / 0**, exit 0.
- Lineman tier-3, fresh JVM, seeds 1-100: **bb2025 100/100, bb2020 100/100, bb2016 100/100**.
- Heuristic event dumps byte-identical to the ITER0 baseline for lineman (100 seeds) and human (20).

**Next:** ITER3 — write `AGENT_CONTRACT.md` §10 from the code (RNG channel, the exact per-prompt
draw-count table, the canonical ordering rule, the frozen constants, the `det_math` requirement),
then the `ClassMask` ladder and the `--agent heuristic` plumbing on both sides.

---

## ITER3 (2026-08-27) — the contract, written before the Java

`AGENT_CONTRACT_HEURISTIC.md` (new), the companion to `AGENT_CONTRACT.md`. Written **first**, on
purpose: a 5,000-line port checked against a spec is reviewable, one checked against a 3,657-line
moving target is not.

Nine sections, every one derived by reading the code rather than the design doc: the two RNG
channels and the exact `unit()` formula with its Java transcription; the **draw-count table**; the
per-call-site temperatures; the arithmetic rules (f32 is portable, `det_math` for the
transcendentals, no other transcendentals, no out-of-range casts, explicit max loops); the canonical
`(side, nr)` ordering rule with all nine ordering sites and the container-iteration audit; the frozen
constants; the shared-vs-two-agent and game-construction differences between the parity arm and the
experiment arm; the five modes; and the tier scope with its stub list.

Two places where the code and `docs/HEURISTIC_AGENT.md` disagree are called out explicitly, with the
code winning: there is no `TempTable` type (§8's table is intent; the temperatures are literals at
17 call sites), and §11's "convert everything to i32 milli-weights" is superseded by ITER2.

### The draw-count table is now executable, not prose

That table is the single easiest thing to get subtly wrong in the port — a decision that costs one
draw where the other side spends two desynchronises the stream, and every later decision is then
unrelated noise. So it is pinned by two tests (`pick_draw_counts_match_the_contract`,
`softmax_pick_draw_counts_match_the_contract`) that observe the agent's private RNG directly and
count consumption by replaying a clone:

| condition | `pick` draws | verified |
|---|---|---|
| `n <= 1`, any temperature | 0 | yes, across `temp_scale` in {0, 0.05, 1, 1e6} |
| `temp_scale <= 0` (argmax) | 0 | yes |
| `0 < temp_scale < 0.1` | 1 | yes — `eps` is 0, so `eps > 0.0 &&` short-circuits and the probe draw is never taken |
| `temp_scale >= 0.1` | exactly 2 | yes, on both the escape and the cumulative branch |

`softmax_pick`: 1 when it decides, 0 for `n <= 1` or argmax. Confirms the plan's table, and confirms
that **argmax consumes nothing at all** — which is what makes `--heuristic 0` the right first rung
of the ladder: no sampler for the two sides to disagree about.

### Gates

- `cargo test -p ffb-engine --lib agent::heuristic_agent` — 18/0.
- `cargo test --workspace --release` — **14,638 / 0**, exit 0.

**Next:** ITER4 — the `ClassMask`/`PlanMask` ladder plus `--agent heuristic` and `--tier 4`
per-decision logging on the Rust side, with rung 0 (empty mask, everything delegated to
`RandomAgent::new_parity`) required to be 100/100 by construction.

---

## ITER4 (2026-08-27) — the class ladder, and rung 0 is green against real Java

The mechanism that lets a 5,000-line Java port be gated continuously instead of at the end.

### `ClassMask`

`PromptClass` (16 variants, one bit each) + `ClassMask` + `prompt_class_of`. `HeuristicAgent` gains
a `classes: ClassMask` and an embedded `parity: Box<RandomAgent>`; the first thing `act` does after
reading the prompt is

```rust
if !self.classes.has(prompt_class_of(&prompt)) {
    return self.parity.act(gs);
}
```

so every class not switched on is answered by the byte-matched `AGENT_CONTRACT.md` contract. The
default is `ClassMask::ALL`, so the experiment arm and the A/B corpus are untouched — verified by
re-running 100 lineman seeds and diffing the dumps against ITER0: identical.

### Harness

`runner::AgentSpec { Random, Heuristic { temp_scale, mode, classes } }`, threaded through
`run_rust_headless`, with `--agent random|heuristic`, `--heur-scale <f32>` and
`--heur-classes <all|none|csv>` on the CLI. Deliberately **orthogonal** to the existing
`--heuristic <scale>`, which is Rust-only self-play that never starts a JVM; overloading it would
conflate an experiment with a gate.

The driver is a small `enum Driver` rather than `Box<dyn Agent>`, because tier 2 needs
`RandomAgent::pick_t2_activation`, which is not on the `Agent` trait and has no heuristic meaning —
pairing `--agent heuristic` with `--tier 2` is rejected at startup rather than papered over.

Also documented at the construction site, per `AGENT_CONTRACT_HEURISTIC.md` §7: the parity arm uses
**one shared agent** for both coaches (mirroring `ParityRunner`'s single object), never the
two-agent home/away split `run_heuristic_game` uses for head-to-head A/B.

### Rung 0 is green — against the real Java engine

```
ffb-parity --home lineman --away lineman --edition bb2025 --tier 3 --seeds 1-100 \
    --no-abort --agent heuristic --heur-scale 0 --heur-classes none
Rust driver: HeuristicAgent (scale=0, mode=Wide, classes=none)
PARITY: 100/100 games match.          exit 0
```

The heuristic agent now drives the Rust side of a real parity run, end to end, at 100/100. Pinned
in-process as well by `rung_zero_is_indistinguishable_from_the_parity_agent`, which walks a whole
game asserting the two agents return the *same action* and leave the *same state hash* at every
step — not merely the same final result.

### And rung 1 fails, which is the point

```
--heur-classes coin,receive  ->  PARITY: 0/10 passed, 10 FAILED.
```

Expected, and checked deliberately: Java still answers `CoinChoice`/`ReceiveChoice` with the random
contract while Rust now scores them. Without this negative control, rung 0's green would be
consistent with the mask doing nothing at all — the vacuous-green trap recorded in
`parity_tier_multiblock`. The mask demonstrably changes behaviour, so rung 0's green is real.

`coin,receive` is therefore the first thing the Java agent has to implement.

### Gates

- `cargo test -p ffb-engine --lib agent::heuristic_agent` — 20/0.
- Lineman bb2025 tier-3, fresh JVM, `--agent random`: 100/100 (unchanged).
- Lineman bb2025 tier-3, fresh JVM, `--agent heuristic --heur-classes none`: **100/100**.
- Heuristic experiment dumps byte-identical to ITER0 (100 lineman seeds).

**Next:** ITER5 — the Java side begins: `DetMath.java` against the shared golden file, the
`heuristic/` package skeleton, the prompt adapter, and rung 1.
