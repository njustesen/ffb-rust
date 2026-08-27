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

---

## ITER5 (2026-08-27) — Java begins: `DetMath` agrees with Rust to the last bit

The biggest technical risk in the plan, retired first.

### The result

`DetMath.java` is a line-for-line transcription of `det_math.rs`, and `DetMathTest` reads **the
same golden file** the Rust test reads — not a copy, so the two cannot drift.

```
mvn -o -pl ffb-ai test -Dtest=DetMathTest
Tests run: 3, Failures: 0, Errors: 0, Skipped: 0
```

**All 348 pinned `(input bits, output bits)` vectors match exactly.** So the central premise of
ITER2 — that f32 arithmetic composed only of correctly-rounded primitives is bit-identical across
Rust and Java — is now demonstrated rather than argued, on real values from the ranges the agent
uses. Every scored weight flows through `expF32` and every Dijkstra key through `lnF32`, so this was
the one thing that had to work before the remaining ~5,000 lines were worth writing.

### One subtlety that would have been a silent one-ulp bug

Rust's `f32::round` is **ties-away-from-zero**. Java's `Math.round` rounds halves toward positive
infinity and `Math.rint` rounds them to even — *neither* matches. The range reduction
`k = round(x * LOG2E)` hits an exact .5 only occasionally, so this would not have shown up in a
smoke test; it would have surfaced as a rare, seed-dependent divergence deep in a game. Written out
explicitly as `roundTiesAway` and pinned by its own test.

This is the shape of bug the golden table exists to catch, and it was caught on the first run.

### Both Java trees, and a check that they agree

There are two copies of the Java source: `C:/Users/Admin/niels/ffb/ffb` (the Maven build tree, whose
jar `ffb-parity` actually loads) and `<repo>/ffb-java/ffb` (the git-**tracked** reference copy).
Only the second is under version control, so an edit applied to one and not the other is invisible
to `git status` and silently makes the reviewed source differ from the measured source.

`scripts/check_java_trees.py` diffs the co-editable harness packages between them, `--fix` syncs
build-tree -> tracked-tree (never the reverse: the build tree is what the jar was compiled from, so
it is the one that describes the measurement). Verified in both directions — it reports agreement,
it detects a one-line probe, and `--fix` restores agreement.

**Run it before any gate that follows a harness edit.**

### Gates

- `mvn -o -pl ffb-ai test -Dtest=DetMathTest` — 3/0, 348 vectors bit-identical.
- `python scripts/check_java_trees.py` — trees agree.
- Maven lives at `C:/Users/Admin/bin/maven/bin/mvn` and is **not on PATH**; use the absolute path,
  with `-o` (offline) since the dependencies are already in `~/.m2`.

**Next:** ITER6 — the `heuristic/` package skeleton (`AgentPrompt`/`AgentAction` mirrors, the
`Xoshiro` draw helpers, `ClassMask`), the `HeuristicDriver` adapter inside `ParityRunner`, and
rung 1 (`coin,receive`) green.

---

## ITER6 (2026-08-27) — rung 1 green: the two agents agree, end to end

The Java agent now answers real prompts, and the cross-language machinery works.

### What landed

`ffb-ai/.../parity/heuristic/`: `PromptClass` + `ClassMask` (bit positions are contract — a
`--heur-classes` spelling must select the same class on both sides), `Sampler` (the `unit`/`argmax`/
`pick`/`softmaxPick` mirror), and `HeuristicDriver`, which gets first refusal on every dialog and
returns false for any class outside its mask so `ParityRunner`'s random policy answers it untouched.
`ParityRunner` gains `--agent`, `--heur-scale`, `--heur-classes`, and the Rust harness forwards all
three to the JVM using `ClassMask::to_spec()` — the canonical spelling, so a differently-ordered or
duplicated list cannot mean two different things on the two sides.

The reuse fingerprint now includes the agent, scale and class mask. Without that, a heuristic-arm
Java log could be served to a random-arm gate as if current — the ITER1 trap in a new costume.

### Rung 1: 100/100 at every arm

| arm | seeds | result |
|---|---|---|
| `--heur-scale 0` (argmax, **zero** agent RNG) | 100 | **100/100** |
| `--heur-scale 1.0` (sampled) | 100 | **100/100** |
| `--heur-scale 1e6` (uniform over the same options) | 100 | **100/100** |

(Exit code 1 on the heuristic arms is the tier-3 *coverage checklist*, not parity: the agent now
always receives, so some kickoff events stop occurring. `PARITY: 100/100 games match` is the verdict
that matters — `docs/PARITY_PROCESS.md` documents this exit-code subtlety.)

### The bug it found, and how the ladder found it

Argmax was 100/100 while sampled was **91/100** — which immediately localised the fault to the
sampler or its inputs, because argmax consumes no RNG at all. That is precisely why the plan puts
argmax first.

The sampler itself was then exonerated by a new cross-language golden file
(`agent/testdata/sampler_golden.txt`, 120 `unit()` vectors + 1,200 whole decisions covering every
temperature band): Java reproduced **every index and every draw count**. The draw count is checked,
not just the answer — two implementations can agree on the pick while disagreeing on its cost, and
the extra draw then poisons every later decision.

So the inputs differed, and a probe on both sides said so in one line:

```
JPROBE receive half=0 w=0.85
RPROBE receive half=1 w=0.65
```

**Java's `Game.getHalf()` still returns 0 at the pre-kickoff receive choice, where Rust's `g.half`
is already 1.** Two different distributions that happen to agree ~82% of the time — hence 9 seeds
in 100, and hence invisible to any smoke test. Fixed in the Java agent with
`Math.max(1, game.getHalf())`, the same normalisation `ParityRunner.stateString` already applies to
this exact field for this exact reason.

A harness/agent mismatch, not an engine divergence — but exactly the class of thing that would have
been blamed on the engine if it had surfaced later, under a fully-ported agent, as a mystery
9%-of-seeds failure.

### Gates

- Rung 1, all three arms, 100 seeds: **100/100**.
- Rung 0 (`--heur-classes none`): 100/100. `--agent random`: 100/100. Neither moved.
- `cargo test --workspace --release`: **14,640 / 0**. `mvn -o -pl ffb-ai test`: **16 / 0**.
- `python scripts/check_java_trees.py`: trees agree.

**Next:** ITER7 — rung 2 (`followup,reroll,blockchoice`, plus `skill`/`intercept` if reachable).
`ReRollOffer` is the interesting one: the random contract always declines, so accepting exercises
every re-roll path in the engine for the first time (501 re-rolls per 100 seeds under the full
agent, against 0 under random play).

---

## ITER7 (2026-08-27) — the multimove spike, and the first real engine bug

Reordered the plan: the movement gap goes **before** rungs 2-4, because the coverage payoff (GFI
0→4,400, touchdowns 0→40, the five scoring-gated dead steps) and the 1:1-port risk both sit there,
while `blockchoice`/`pushback`/`followup` add almost nothing random play does not already produce.

### The spike

`--multimove N` on **both** harnesses (`RandomAgent::multimove` /
`ParityRunner --multimove N`, default off). The random agent plans up to N one-step squares ahead
and submits them as one `Action::Move` / one `CLIENT_MOVE`. Both sides walk by the *same* candidate
rule at every step — the eight neighbours of the square being planned from, on pitch, unoccupied,
coordinate-sorted, not already on the path — one `actionRng` draw per square, capped at MA + 2. To
guarantee the rule really is the same one, Java's inline neighbour loop was factored out into
`freeNeighbours`, the byte-mirror of Rust's `free_neighbours`.

This answers *"do the two engines consume a multi-square move stack identically?"* using the
already-byte-matched random agent — no `Features`/`Reach`/value-model port in the way — and it drags
GFI, mid-path dodges and mid-path turnovers into the gate for the first time.

**Answer: no.** First run, `--multimove 4`, lineman bb2025: **0/100**.

### Bug 1 (mine, in the harness): the pre-drawn path

Rust pre-draws a prone/rooted player's move target at *activation* (mirroring Java's phase-2
`sendMoveAction`) and reuses it at the Move prompt. I had extended only the standing branch, so a
stood-up player walked 1 square in Rust and 4 in Java. Fixed by extending the path at the pre-draw
site — the same point in the stream where Java draws it — with `spent = 0`, because at pre-draw time
`acting_player` is still the *previous* activator (the same staleness the no-cap candidate list
already works around). Frontier moved 29 → 54.

### Bug 2 (real, in the Rust engine): a stood-up player could never rush

`crates/ffb-model/src/util/util_player.rs`, `is_next_move_going_for_it`:

```rust
if ap.standing_up && !ap.has_acted && !player.has_skill_property(CAN_STAND_UP_FOR_FREE) {
    3 >= player.movement_with_modifiers()
```

Java calls `hasActed()`, which is **computed** — `hasMoved || hasFouled || hasBlocked || hasPassed
|| hasTriggeredEffect || !usedSkills.isEmpty() || isForgone`. Rust has exactly that as `acted()`,
whose doc comment already warns *"Callers mirroring Java's `hasActed()` MUST use this, not the bare
stored field"* — and this caller used the bare stored field.

A player who stands up **for free** (MA >= 4, no roll) gets `has_moved = true` while `standing_up`
stays set — Java's `StepStandUp` only clears `standingUp` on the *rolled* path, and Rust faithfully
mirrors that. So Java falls through to `current_move >= MA` and rushes normally, while Rust stayed in
the stand-up branch for the whole activation evaluating `3 >= 6` — false. `goes_for_it` never became
true, `StepGoForIt` returned early, and **Java rolled a Rush die that Rust did not** (seed 1, i=29:
Java `rng_calls=24` against Rust's `23`).

Structurally invisible until an agent moved more than one square per activation, which is exactly
what the spike was built to expose. One-line fix, `acted()` for `has_acted`, plus
`a_freely_stood_up_player_can_still_rush`.

**`--multimove 4`: 0/100 → 99/100.**

### The remaining seed

Seed 58, first divergence at dice index ~92 of ~104 (both streams identical up to there), inside a
pass chain: Java's roll 82 is `StepIntercept.intercept` returning a natural 6. A **distinct, later
bug** — next iteration. Note the per-step `rng_calls` gap at i=172 (java 82 / rust 92) was
misleading: the dice *values* agreed there, so it was step-boundary attribution, not a divergence.
Diff the dice STREAM, not the per-step counts.

### Gates

- `--multimove 4` lineman bb2025: **99/100** (was 0/100).
- `--multimove` off: **100/100** in bb2025, bb2020 **and** bb2016 — the existing gate is untouched.
- Heuristic rung 0 and rung 1 (sampled): 100/100 each.
- `cargo test --workspace --release`: **14,641 / 0**.

**Next:** seed 58's pass/interception divergence, then raise `--multimove` until it is green at the
full MA+2, then rung 2 (`reroll`).

---

## ITER8 (2026-08-27) — seed 58 root-caused: a stale catch/scatter mode survives a successful interception

Diagnosis only; no code change. The fix needs one more fact (below) and lands next iteration.

### What happens

`--multimove 4`, lineman bb2025, seed 58, away_03's PASS at i=171. Both engines agree through the
interception: **both roll a 6 and both succeed** (Rust probe: `roll=6 min=5 easy=false ok=true`;
Java's `caller=` stack shows `StepIntercept.intercept` at the same stream position). Java's pass
activation then **ends** — its next die is the following activation's dodge. Rust instead spends
**ten more dice** inside `CatchScatterThrowIn`.

`FFB_RNG_STEPS` gives the attribution Java's `caller=` gives for free:

```
RNGSTEP 44 step=MissedPass          78->81  pid=away_03 pa=Pass
RNGSTEP 45 step=CatchScatterThrowIn 82->85  pid=away_03 pa=Pass
RNGSTEP 46 step=CatchScatterThrowIn 85->88  pid=away_03 pa=Pass
RNGSTEP 47 step=CatchScatterThrowIn 88->91  pid=away_03 pa=Pass
RNGSTEP 48 step=CatchScatterThrowIn 91->92  pid=away_03 pa=Pass
RNGSTEP 49 step=MoveDodge           92->93  pid=home_09 pa=Move      <- Java is here at 83
```

A probe at the step's mode switch shows why:

```
RCS mode=Some(ThrowIn) ball=Some(17,2) inplay=true
```

**`StepCatchScatterThrowIn` wakes up with a stale `ThrowIn` mode and runs a whole throw-in /
bounce / catch chain.** With no mode its `None => {}` arm is inert, which is what Java's equivalent
effectively is here: `bb2025/StepResolvePass:43` publishes **no** `CATCH_SCATTER_THROW_IN_MODE` on
the `isInterceptionSuccessful()` branch.

Ruled out along the way, each with evidence rather than reasoning:

- **Not the natural-6 rule.** `roll == 6 || (roll != 1 && roll >= minimum_roll)` is present and fired.
- **Not the agent.** Both harnesses attempt the interception (Rust `SelectPlayer` on a coord-sorted
  list; Java `sendInterceptorChoice`).
- **Not `StepResolvePass`'s branch structure.** Rust's interception branch early-`return`s, so it is
  exclusive exactly like Java's `if / else if / else`, and it publishes no mode.

So the stale mode is published **earlier in the same pass sequence** — `StepMissedPass`'s three
scatters put the ball out of bounds, which is the natural source of a `ThrowIn`. This is the same
family as the two hazards `StepResolvePass` already documents in its own comments: Rust threads step
parameters that **outlive** the point Java re-creates its per-pass `PassState`.

### The one fact still needed

Whether Java also publishes `THROW_IN` there and then *skips* the step, or never publishes it at
all. Read `bb2025/StepMissedPass.java` and the Java `Pass` sequence's post-`Intercept` routing, then
port whichever it is 1:1. Note `StepParameter::CatchScatterThrowInMode` carries a bare mode with no
"clear" form, so a fix that needs to *unset* it will have to add one — mirroring the
`InterceptorId(None)` clear already used on the failure branch.

### Correction to ITER7

ITER7 said the two dice streams "agreed up to index 91". That was the wrong inference: both engines
draw from the **same** stream, so equal values only mean equal draw *counts*, not equal behaviour —
Rust was already spending its draws on `CatchScatterThrowIn` while Java spent them on a dodge and an
armour/injury roll. **Compare dice by their step ATTRIBUTION (`FFB_RNG_STEPS` vs Java's `caller=`),
not by value.** Matching values across a divergence are the null result, not evidence.

### Gates

Unchanged from ITER7 (no code landed): `--multimove 4` 99/100; `--multimove` off 100/100 in all
three editions; heuristic rungs 0 and 1 green; workspace 14,641/0. Probes removed, tree clean.

**Next:** finish this fix, then raise `--multimove` toward MA+2, then rung 2 (`reroll`).

---

## ITER9 (2026-08-27) — the stale mode fixed 1:1; `--multimove 4` is 100/100

### The fix

`bb2025/StepMissedPass.executeStep` in Java publishes **nothing** — it sets `passCoordinate`,
`outOfBounds` and the ball coordinate, then returns `NEXT_STEP`. The out-of-bounds routing lives one
step later in `StepResolvePass`, which publishes `ThrowIn` + `THROW_IN_COORDINATE` for a ball and
`BOMB_OUT_OF_BOUNDS` for a bomb.

Rust's `step_missed_pass.rs` published `ThrowIn` **there as well**, and its own comment said so:
*"Java's bb2020/bb2025 MissedPass publishes NOTHING … This Rust-side publish stays for the ball path
(measured green across all editions)."* A deliberate deviation, kept because it measured green.

It measured green because it was **redundant**: with no interception, `StepResolvePass` publishes the
identical pair a moment later — and identical in value too, since `MissedPass` has just set
`pass_coordinate = last_valid_coordinate`, which is exactly what `ResolvePass` publishes as the
throw-in coordinate.

It stops being redundant the moment an interception **succeeds**. Java's `ResolvePass` then takes its
`isInterceptionSuccessful()` branch, which publishes no mode at all, leaving
`StepCatchScatterThrowIn` inert. Rust's early `ThrowIn` was already in flight, so that step woke with
a stale mode and ran a throw-in/bounce/catch chain — ten dice Java never rolls.

Removed the publish; `StepResolvePass` alone owns the routing, as in Java. The test that asserted the
old behaviour now asserts the Java contract and is renamed
`publishes_nothing_even_when_the_ball_goes_out_of_bounds`.

**A "measured green" deviation is a latent divergence waiting for a path that reaches it.** This one
waited for an agent that moves far enough to stand in a pass corridor.

### Gates

- **`--multimove 4` lineman bb2025: 100/100** (was 99/100).
- `--agent random` lineman tier-3: **100/100 in bb2016, bb2020 and bb2025**.
- Heuristic rung 0 and rung 1: 100/100 each.
- `cargo test --workspace --release`: **14,641 / 0**; pass-family suite 130/0.

### New frontier: `--multimove 6` is 0/100, and it is the queue's item 1

Raising the spike to 6 and 8 squares fails immediately. Seed 1, first dice divergence at i=28:

```
JAVA  pos=29,30  StepGoForIt.rush      <- TWO rushes
RUST  RNGSTEP 18,19,20  step=GoForIt    <- THREE rushes
```

**A player may rush at most twice (MA + 2). Java refuses the ninth square; Rust executes it.** The
agent plans up to `ma + 2 - spent`, but at pre-draw time `spent` is 0 for a prone player who will go
on to spend 3 standing up — and Java's `sendMoveAction` reads 0 there too, so both harnesses propose
the same over-long path. Java's ENGINE declines it and Rust's does not.

That is exactly the gap this campaign's queue names first: `Action::Move` carries neither a player id
nor a `from`, so two of Java's three `CLIENT_MOVE` guards cannot be evaluated, `is_valid_move` has
**zero** production call sites in Rust against ten in the Java server, and
`step_init_moving.rs:70` says *"UtilServerPlayerMove.isValidMove + fetchMoveStack not ported; trust
agent path"*. The move stack is trusted to exhaustion instead of being bounded by what the player can
actually do.

**Next:** port that gap 1:1 — add `player_id` + `from` to `Action::Move`, wire `is_valid_move` and the
acting-player check into all three `StepInitMoving`/`StepInitSelecting` pairs, publish `MOVE_START`,
and bound the move-stack walk by `UtilPlayer.isNextMovePossible` the way Java does. Then re-raise
`--multimove` to MA+2 in all three editions.

---

## ITER10 (2026-08-27) — the movement rung is de-risked: full MA+2 walks, 100/100 in all three editions

### The divergence

`--multimove 6` and `8` were 0/100. Seed 1, i=28: **Java rolls two rushes, Rust rolls three.**

Chasing it through the engine ruled out three plausible culprits, each with evidence:

- `StepGoForIt`'s guard is `isGoingForIt() && currentMove > MA` on **both** sides — no MA+2 cap in
  either, so Java is not refusing the third rush there.
- `updateMoveSquares` gates on `isNextMovePossible` on **both** sides.
- The move-square map is stale between squares on **both** sides (it is only refreshed when the move
  stack empties), so `StepInitMoving`'s `getMoveSquare(coordinateTo)` returns null for the 2nd square
  onward in Java too, and neither engine is bounding the walk with it.

Java's own `JAVA_GFI` debug line (already in `ParityRunner` under `-Dffb.parityDebug`) then made it
exact — a matching `RUST_GFI` was added under `FFB_TRACE`, and the two read:

```
JAVA_GFI  away1  currentMove=7,8      goingForIt=true    <- stops at 8
RUST_GFI  away_01 currentMove=7,8,9   goingForIt=true    <- goes to 9
```

### Root cause — the spike harness, not the engine

A path-length probe on both sides settled it in one line:

```
JAVA_PATH pid=…Away1 len=5 currentMove=3 multimove=6
RUST_PATH pid=away_01 len=6 (predrawn)
```

**Java's `sendMoveAction` reads `ap.getCurrentMove()` at the moment it draws, and for a prone player
that moment is AFTER the stand-up — `currentMove=3`, so it plans 5 squares.** Rust pre-draws the
prone/rooted path at *activation*, before the stand-up, and ITER7 passed `spent = 0` there, so it
planned 6. One square too many, cm reaching 9, and a third rush the rules do not allow.

Fixed by charging the same 3 movement Java has already spent (`MovementCalc::STAND_UP_COST`) when
the pre-drawn player is prone. A rooted player never moves, so nothing is spent there.

This is **spike scaffolding, not engine code** — but it had to be right before the spike could say
anything about the engine.

### What this settles

**The two engines consume a multi-square move stack identically**, at the full MA + 2, including
mid-path rushes, mid-path dodges and mid-path turnovers, in **bb2016, bb2020 and bb2025**. That was
the open risk the whole spike existed to answer, and the answer is yes.

Coverage under the passing gate, against the random-agent baseline:

| | baseline | `--multimove 8` |
|---|---|---|
| `playerMoved` | 26,278 | **56,763** |
| **`goForItRoll`** | **0** | **14,150** |

Touchdowns are still 0 and re-rolls still 0 — the random agent neither aims at an endzone nor ever
accepts a re-roll. Those need the heuristic agent, which is the rest of the campaign.

Note the `is_valid_move` gap is **not** closed and is no longer blocking: the random agent always
sends a `from` that matches the player's current square, so Java's guard never fires. It becomes
live again when the heuristic agent replays a *stale* plan across prompts, which is the one place
Java drops a command and Rust does not. Keep it on the queue for the `Move` rung.

### Diagnostics kept

`JAVA_PATH` (under `-Dffb.parityDebug`) and `RUST_PATH` / `RUST_GFI` (under `FFB_TRACE`), matching
the file's existing `JAVA_SMA` / `JAVA_PICK` / `JAVA_GFI` convention. Comparing planned path length
and per-rush `currentMove` across the two engines is exactly the diff that solved this one.

### Gates

- **`--multimove 4`, `6` and `8`: 100/100 in bb2025; `--multimove 8` 100/100 in bb2016 and bb2020.**
- `--agent random`: 100/100 in all three editions.
- Heuristic rung 0 and rung 1: 100/100.
- `cargo test --workspace --release`: **14,641 / 0**. Java trees in sync.

**Next:** rung 2 — `reroll`. The random contract always declines, so accepting runs every re-roll
path in the engine for the first time (0 → 501 per 100 seeds under the full agent). Needs a Java
`ReRolledAction` → Rust action-string mapping and the ball carrier.

---

## ITER11 (2026-08-27) — rung 2 (`reroll`): the Java arm lands at 91/100

### What landed

`HeuristicDriver.useReRoll` mirrors Rust's `AgentPrompt::ReRollOffer` arm: two options in Rust's
order (index 0 = use, index 1 = decline), weight
`clamp(consequence * 0.833 * scarcity, 0, 1)`, `T = 0.20`.

Two mirroring decisions worth recording:

- **Compare `ReRolledActions` CONSTANTS, not names.** Rust spells them SCREAMING_SNAKE (`"GFI"`,
  `"PICKUP"`); Java's are display strings (`"Go For It"`, `"Pick Up"`, `"standUp"`). Matching on
  identity is what both engines actually mean and cannot drift on a spelling. Rust's `"GFI"` is what
  its `StepGoForIt` passes; Java's rush passes `ReRolledActions.RUSH`, and `GO_FOR_IT` is folded in
  beside it since both sit in the same bucket.
- **`scarcity` is 0 when the team's bank is empty**, so `w_use` is 0 and the agent declines — even a
  SKILL re-roll offered with no team re-rolls left. Mirrored rather than "fixed".

The decision lives in the driver; the *sending* stays in `ParityRunner.reRollSourceFor`, which
reuses the existing capture/inject/dialog-clear plumbing and returns `null` (decline) whenever the
driver is absent or does not own the class — so tier 2, tier 3 and every lower rung keep their
byte-matched streams untouched.

### Result

**91/100 at argmax, and 113 re-rolls consumed where the baseline had 0.** Every re-roll path in the
engine is now executing for the first time. Rung 0 and rung 1 unaffected.

### The remaining 9 seeds: one state-only divergence, precisely located

Seed 1. **No dice divergence at all** — both engines roll exactly 94 dice, and every logged step's
`state_hash` matches, including step 303's. Only the *resolution* of the final activation differs.
End-of-game state strings (`JAVA_END` / the new `RUST_END`) differ in exactly one field:

```
JAVA_END … r3,3 …
RUST_END … r2,3 …
```

The **home team's re-roll count**: Java 3, Rust 2. The `r` field agrees at all 303 logged steps and
diverges only afterwards.

Rust's three recorded `use_reroll` consumptions for that seed are:

```
half=1 turn=3 home_playing=false  3->2  away_01
half=1 turn=8 home_playing=true   3->2  home_02
half=2 turn=1 home_playing=true   4->3  home_02
```

Away: 2 at the end of half 1, reset to 3 at half 2, never spent again → 3. Correct.
Home: 2 at the end of half 1, reset to 3, **+1 from a kickoff Extra Re-roll = 4**, spent once → 3.
That arithmetic gives 3, which is what Java reports — so Rust loses **one more** somewhere in the
end-of-game resolution, and not through `use_reroll`.

**This is why the class had to be switched on to find it:** with the random contract declining every
offer, `r` is `3,3` on both sides for the whole game and any accounting error is invisible.

Candidate sites for the extra loss (only three places decrement outside `use_reroll`):
`mechanic/bb2025/state_mechanic.rs:61` and `mechanic/mixed/state_mechanic.rs:66` — both the **Leader**
skill's grant/revoke pair, which linemen should never trigger — and `start_half`'s reset. Instrument
every write to `turn_data_*.rerolls` through the end-of-turn → end-of-half → end-of-game cascade and
compare against Java's `TurnData.setReRolls` call sites.

### Diagnostics kept

`RUST_END state=` in `run_rust_headless` under `FFB_TRACE`, the counterpart to `ParityRunner`'s
existing `JAVA_END`. A divergence in the resolution of the LAST logged step has no following step to
diff, so without it the end-of-game state can only be compared as an opaque hash. That one line is
what turned this from "seed 1 fails" into "the home re-roll count is one low".

### Gates

- Rung 2 (`coin,receive,reroll`) argmax: **91/100**, up from every-reroll-seed-diverging.
- `--agent random`: 100/100 in bb2016, bb2020 and bb2025.
- Rung 0 and rung 1: 100/100.
- `cargo test --workspace --release`: **14,641 / 0**.

**Next:** the home re-roll accounting above. Then rung 2 sampled + uniform, then rung 3.

---

## ITER12 (2026-08-27) — the one-drive re-roll was clawed back twice (91 → 95/100)

### Root cause

Java's `RollMechanic.updateTurnDataAfterReRollUsage` (`bb2025:464-488`) spends the **one-drive
re-rolls first**. Consuming a team re-roll decrements the pool AND the first non-zero one-drive
counter, in the order Brilliant Coaching → Pump Up The Crowd → Show Star:

```java
if (rrActuallyUsed) turnData.setReRolls(turnData.getReRolls() - 1);
if (turnData.getReRollsBrilliantCoachingOneDrive() > 0) {
    if (rrActuallyUsed) turnData.setReRollsBrilliantCoachingOneDrive(... - 1);
    return ReRollSources.BRILLIANT_COACHING;
}
```

Rust's `use_reroll` decremented only the pool. The counter therefore still read 1 at the end of the
drive, where `StepEndTurn::remove_rerolls_lasting_for_drive` subtracts the sum back out again — so a
re-roll that had **already been spent** was clawed back a second time, costing the team a
**permanent** re-roll.

Measured on lineman bb2025 seed 1: home was granted Brilliant Coaching at the half-2 kickoff
(3 → 4), spent it (4 → 3), then lost another at the final whistle (3 → 2) where Java ends on 3.

Everything around it was already a faithful port and was checked first: the drive-end body is
identical, the `fNewHalf || fTouchdown` call condition is identical (and `checkEndOfHalf` is true at
turn 8/8, so Java *does* run it), and the kickoff grant sets the counter on both sides.

**Structurally invisible until an agent accepts a re-roll.** Under the random parity contract every
offer is declined, so the counters never move and the double-subtraction has nothing to bite.

### How it was found

`FFB_RR_STEPS` (new, same shape as `FFB_RNG_STEPS`): print which STEP changed a team's re-roll bank.
The `r` field is in the state hash, so an accounting error is a divergence, but nothing else says who
moved it. Four lines named the culprit immediately:

```
RRSTEP step=InitKickoff          home 0->3 away 0->3 (half=1)
RRSTEP step=EndTurn              home 2->3 away 2->3 (half=2)   <- half reset
RRSTEP step=ApplyKickoffResult   home 3->4 away 3->3 (half=2)   <- Brilliant Coaching
RRSTEP step=EndTurn              home 3->2 away 3->3 (half=2)   <- the bug
```

### Gates

- Rung 2 (`coin,receive,reroll`) argmax: **95/100**, up from 91/100.
- `--agent random`: 100/100 in bb2016, bb2020 and bb2025.
- Rung 0 and rung 1: 100/100.
- `cargo test --workspace --release`: **14,642 / 0**, including
  `spending_a_team_reroll_also_spends_a_one_drive_reroll`.

### Process note — a rule I broke and reverted

While diagnosing this I added a temporary `System.err.println` probe to
`ffb-server/.../bb2025/StepEndTurn.java`. **That is stock engine code and off-limits** — the whole
campaign rests on Java being unmodified ground truth. Reverted, stock jar rebuilt, file verified
probe-free. The answer came from *reading* the Java and instrumenting only the Rust side, which is
what should have happened first. Instrument Rust; read Java.

### The remaining 5 seeds are a DIFFERENT divergence

Seed 16 is not the same shape — the end state differs in ball position, player states and re-rolls
at once:

```
JAVA_END … b19,3,true … r3,2 … pa00:-1,-1,Bh,…
RUST_END … b20,4,true … r2,3 … pa00:-1,-1,Reserve,…
```

Note `r3,2` against `r2,3` — home and away transposed, which is worth a look on its own. A
substantive divergence much earlier in the game rather than an end-of-drive accounting slip. Failing
seeds: 16, 45, 62, 89, 99.

**Next:** seed 16 — find its FIRST diverging step (dice attribution via `FFB_RNG_STEPS` vs Java's
`caller=`), then continue rung 2 to green and on to sampled + uniform.
