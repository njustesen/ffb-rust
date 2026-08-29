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

---

## ITER13 (2026-08-27) — the one-team-re-roll-per-turn latch is BB2016-ONLY (95 → 97/100)

### Root cause

`setReRollUsed(true)` appears in **exactly one place in all of Java**:
`bb2016/RollMechanic.updateTurnDataAfterReRollUsage`, whose entire body is

```java
turnData.setReRollUsed(true);
turnData.setReRolls(turnData.getReRolls() - 1);
```

BB2020 and BB2025 **override that method** (`bb2025:464-488`, the one-drive version from ITER12) and
never touch the flag. So in those editions the latch stays false and a team may spend **more than one
team re-roll in a turn**, bounded only by the bank.

The availability CHECK is shared — `RollMechanic.isTeamReRollAvailable` tests `!isReRollUsed()` for
every edition — so it is only the **SET** that is edition-specific. Rust's shared `use_reroll` set it
unconditionally, so BB2020/BB2025 refused the second re-roll of a turn.

Seed 16, dice 14-18, from Java's `caller=` stacks against Rust's `FFB_RNG_STEPS`:

```
JAVA  14 dodge(2) fail   15 dodge(6) RE-ROLL ok   16 dodge(2) fail   17 dodge(6) RE-ROLL ok   18 dodge(6)
RUST  14 dodge(2) fail   15 re-roll ok            16 dodge(2) fail   -> FallDown, armour+injury (6 dice)
```

Rust re-rolled the first failed dodge and refused the second, so home_01 fell over where Java stayed
up — and from there the games are unrelated.

**Invisible under the random parity contract**, which declines every offer: the latch is never set,
so whether it *would* have blocked a second re-roll never comes up.

### Five stale tests, all pinning the deviation

`use_reroll_trr_decrements_count`, both `step_hail_mary_pass` variants, `step_catch_of_the_day`,
`step_getting_even` and `step_animal_savagery` all asserted `reroll_used` after a BB2020/BB2025
re-roll. Two of them already asserted the bank hit 0, making the latch line redundant; the others
now assert the **bank**, which is what a team re-roll actually spends. A test that pins the wrong
behaviour is worse than no test — it makes the fix look like the regression.

### Gates

- Rung 2 (`coin,receive,reroll`) argmax: **97/100**, from 95/100 (and 91/100 at ITER11).
- `--agent random`: 100/100 in bb2016, bb2020 and bb2025 — bb2016 is unaffected because it still
  latches.
- Rung 0 and rung 1: 100/100.
- `cargo test --workspace --release`: **14,643 / 0**, including `reroll_used_latch_is_bb2016_only`,
  which checks the latch AND the consequence (a second offer) across all three editions.

**Next:** the last 3 rung-2 seeds, then rung 2 at sampled + uniform, then rung 3.

---

## ITER14 (2026-08-27) — a real port gap closed, and it did NOT move the number (still 97/100)

Stating that plainly first: the fix below is a genuine 1:1 gap with a regression test, but rung 2 is
**still 97/100** and seeds 45, 62 and 99 still fail. This iteration's value is mostly the *negative*
result, which narrowed the frontier a lot.

### What was fixed

Java's `askForReRollIfAvailable` takes the **player who made the roll** and reaches
`RollMechanic.isTeamReRollAvailable`, whose first condition is `actingTeam.hasPlayer(pPlayer)` — a
team re-roll is only ever offered for a roll made by the team whose turn it is. Rust's
`ask_for_reroll_if_available` took **no player at all** and skipped the condition, so a roll by the
opponent — a catch after a scattered ball is the common one — was offered a team re-roll out of the
acting team's bank.

Added `ask_for_reroll_if_available_for(game, player_id, …)` carrying Java's gate. The 4-arg form now
delegates with `acting_player.player_id`, which is a no-op for the ~105 callers that roll for the
acting player, and `bb2025/shared/StepCatchScatterThrowIn` passes the real **catcher**, as Java does
at `:590-592`. Test: `a_team_reroll_is_only_offered_for_the_acting_teams_own_roll`.

Three `step_stand_up` fixtures then failed because they set `acting_player.player_id` **without ever
putting that player on a team** — impossible in a real game, and now caught by the gate. Fixed the
fixtures rather than weakening the gate.

### What the negative result rules out

The two re-roll **arms agree exactly** where both fire. Probing both sides on seeds 1-3 gave 14
paired decisions with identical inputs and identical weights, e.g.

```
JARM action=Dodge weCarry=false rerolls=3 turn=1 cons=0.55 scar=1.0 w=0.45815
RARM action=DODGE weCarry=false rerolls=3 turn=1 cons=0.55 scar=1   w=0.45815
```

So the agent port is not the problem. On seed 45 the Java arm is **never called at all** while Rust's
fires five times — Rust's engine OFFERS re-rolls Java's engine does not. The remaining divergence is
re-roll **availability**, engine-side.

Also checked and eliminated: Java gates the whole dialog on `minimumRoll >= 0`, and Rust ignores
`minimum_roll` entirely (`_minimum_roll`) — but no Rust caller passes a negative value, so that
specific gate is not it.

### Where to look next

Rust's `ask_for_reroll_if_available` is ~30 lines. Java's real implementation is
`RollMechanic.askForReRollIfAvailable` (`bb2025:247-292`), which assembles a `ReRollProperty` set —
MASCOT, TRR, an additional-source lookup, LONER, PRO — and shows the dialog only when

```java
dialogShown = properties.stream().anyMatch(ReRollProperty::isActualReRoll)
    || reRollSkill != null || modificationSkill != null;
```

Rust models none of that structure. The next candidates are `findAdditionalReRollProperty`,
`isMascotAvailable`, and `find_skill_reroll_source` versus Java's
`UtilCards.getUnusedRerollSource(actingPlayer, reRolledAction, ignoreSkills)` — note Java's takes the
ACTION and an ignore-set, and Rust's takes neither.

### Gates

- Rung 2 argmax: 97/100 — **unchanged**. Seeds 45, 62, 99.
- `--agent random`: 100/100 in bb2016, bb2020 and bb2025. Rung 0: 100/100.
- `cargo test --workspace --release`: **14,644 / 0**.

**Next:** compare Rust's re-roll availability against `RollMechanic.askForReRollIfAvailable` as a
whole, rather than one condition at a time.

---

## ITER15 (2026-08-27) — RUNG 2 GREEN: 100/100 in all three editions, all three scales

### Correction to ITER14 first

ITER14 concluded "Java's arm is never called on seed 45". **That was wrong, and the cause was my own
tooling.** Java runs first in the batched JVM, so every `JARM` line precedes every `RARM` line, and
the `| tail -8` I used to compare them showed only the Rust half. Re-run without the window: the two
arms fire **9 times each** on seed 45 and agree on every value (the only textual difference is
`scar=1.0` against `scar=1`).

This is the trap already recorded in memory as *trace window truncation* — never conclude from a
`head`/`tail`-bounded view — and it cost an iteration. When two engines are compared in one stream,
**one runs to completion before the other**, so any window is a window on one engine.

### Root cause

`RRCMD step=CatchScatterThrowIn home 2->0` — the bank dropped by **two in a single command**.

Java has exactly **one** `useReRoll` in that step (`bb2025/shared/StepCatchScatterThrowIn:517`,
inside the catch path); its `handleCommand` for the re-roll answer only records the source via
`AbstractStepWithReRoll`. Rust consumed it in `handle_command` **as well as** in `catch_ball`, so one
accepted catch re-roll spent two from the bank.

Removed the `handle_command` consumption; `catch_ball` is the single consumer, as in Java. The
`self.roll = 0` fresh-die reset stays.

### The diagnostic had to be fixed first

`FFB_RR_STEPS` (added ITER12) showed only two lines for seed 45 and none of the decrements, because
the driver's per-step wrapper brackets step **execution** only — `use_reroll` runs while the agent's
answer is applied. Taught the trace to bracket `handle_command` too (`RRCMD`), and the double
decrement was immediately visible.

**Same attribution gap as the Intercept die in ITER8.** A diagnostic that wraps one of two entry
points makes half the mutations invisible, and an empty trace then reads as "nothing happened".

### Rung 2 is green everywhere

| | argmax | sampled | uniform |
|---|---|---|---|
| bb2025 | **100/100** | **100/100** | **100/100** |
| bb2020 | **100/100** | — | — |
| bb2016 | **100/100** | — | — |

Re-rolls are now genuinely exercised in a passing parity gate — 113 consumed per 100 seeds against
**0** for the entire history of this matchup.

### Gates

- Rung 2 (`coin,receive,reroll`): 100/100 in bb2025 at scales 0 / 1.0 / 1e6, and 100/100 in bb2016
  and bb2020 at argmax.
- `--agent random`: 100/100 in all three editions. Rung 0: 100/100.
- `cargo test --workspace --release`: **14,645 / 0**. All probes removed, both Java trees in sync,
  jar rebuilt clean.

### Score so far

Three classes ported (`coin`, `receive`, `reroll`) and **six real engine bugs** found, every one of
them a 1:1 port gap that the random-agent gate structurally could not reach:

1. `build_threat`'s `HashMap` tie-break — run-to-run non-determinism on mixed-ST rosters.
2. `has_acted` vs computed `acted()` — a stood-up player could never rush.
3. `StepMissedPass` publishing a mode Java does not — ten phantom dice after an interception.
4. The one-drive re-roll clawed back after being spent.
5. The one-per-turn re-roll latch applied outside bb2016.
6. A catch re-roll consumed twice.

**Next:** rung 3 — `blockchoice`, `pushback`, `followup`, `blocktarget`, `blitztarget`. These need
the `Features` core tier (occ / tz / row_prefix) on the Java side, which is the first real slice of
the scorer port.

---

## ITER16 (2026-08-27) — rung 3a: `pushback` + `blocktarget`, 100/100 first try

### Scoping the rung before writing it

Rung 3 is five classes (`blockchoice`, `pushback`, `followup`, `blocktarget`, `blitztarget`) and
`act_with_features` routes all of them through the feature block — but reading the arms shows they do
**not** all read the rasters:

| class | needs |
|---|---|
| `blocktarget` | nothing — the arm is `Action::EndPlayerAction` |
| `pushback` | the BALL only (`f.ball_carried`, `f.ball`) plus coordinates |
| `followup` | **`f.tz_against`** — the tackle-zone raster |
| `blockchoice`, `blitztarget` | `block_weight`, hence the strength/threat machinery |

So `pushback` + `blocktarget` land with no raster port at all. Split the rung there rather than
paying for `Features` before anything needs it.

### What landed

`HeuristicDriver.pushbackChoice` mirrors Rust's arm: options are the UNLOCKED pushback squares sorted
by `(x, y)` — the same list and order Rust's prompt carries, since `step_pushback.rs` filters
`!sq.locked` — weighted off-pitch 1.0-with-ball / 0.95, sideline 0.55, interior 0.20, times 1.3 for
any square further from the **defender's** own endzone. `T = 0.15`.

`ParityRunner.sendPushback` consults it and falls back to `AGENT_CONTRACT.md` §7's deterministic
min-`(x, y)` pick whenever the driver is absent or does not own the class, so the lower rungs keep
their streams. Rust's prompt carries the step-local defender (the occupant being pushed); Java derives
the same player via `pushOrigin` on any candidate square, which agrees because every candidate pushes
the same player.

`blocktarget` needed no work: both sides already deselect on the mid-sequence block-target ask.

### Checked for vacuity

A green rung proves nothing if the arm never changes a decision. The deterministic contract picks
min-`(x, y)`; the heuristic picks max weight. Comparing end-of-game hashes across the two masks:

```
seed 1  rung2 280b01223e03355e  ->  rung3a 44b9941ba9568636   (changed)
seed 2  9ef2c36108fc7b09        ->  9ef2c36108fc7b09          (same)
seed 3  59cf8e0006d26b4f        ->  59cf8e0006d26b4f          (same)
```

Seed 1's game genuinely diverges under the new arm and still matches Java, so the green is real.

### Gates

- Rung 3a (`coin,receive,reroll,pushback,blocktarget`): **100/100** in bb2025 at scales 0 / 1.0 / 1e6,
  and **100/100** in bb2016 and bb2020 at argmax.
- Rung 2 and rung 0: 100/100. `--agent random`: 100/100 in all three editions.
- `cargo test --workspace --release`: **14,646 / 0**, including `pushback_weight_table`, which pins
  the shared weights and the geometry both sides mirror.

One invariant worth having found while writing that test: the 1.3 endzone multiplier **never crosses
a tier** (0.20·1.3 = 0.26 < 0.55, and 0.55·1.3 = 0.715 < 0.95). Ordering is tier-first,
multiplier-second — so a disagreement about the multiplier can only reorder squares *within* a tier,
which bounds how badly that part can go wrong. My first draft of the test asserted the opposite and
failed, which is how the property surfaced.

**Next:** rung 3b — `followup`, which needs the `Features` CORE tier (`tz_against`) in Java. That is
the first genuine slice of the raster port, and the point to introduce the cross-language FIXTURE
test the plan calls for: dump Rust's `tz` raster for fixed boards and assert Java reproduces it
before wiring it to a game.

## ITER17 — rung 3b: `blockchoice`, and BB2020's block-roll dialog is a third class

`followup` was the stated next step and I dropped it after reading the two sides. Rust's `FollowUp`
arm needs `f.tz_against` (a raster) *and* `target_coord`, which is the `DEFENDER_POSITION` step
parameter published by `StepInitBlocking`. Java's `DialogFollowupChoiceParameter` carries only
`getId()` — the harness has no way to read that square, and deriving it from the board is ambiguous
once a chain push has moved people. So `followup` waits for the raster tier and a parameter it can
actually see.

`blockchoice` needs the ball, the four block skills and `can_surf`. **No rasters, no new state
capture** — a strictly better target, and it went in.

### The divergence

Ported the `AgentPrompt::BlockChoice` table (§6.3, `T = 0.12`) to `HeuristicDriver.blockChoice`,
along with `pushSquares`/`canSurf`, and wired it into `BLOCK_ROLL` and `BLOCK_ROLL_PROPERTIES`.
bb2025 and bb2016 went green at every scale; **bb2020 collapsed to 39/100 at argmax and 15/100
sampled**, from 100/100 the iteration before.

Root cause: **each edition shows a DIFFERENT block-roll dialog**, and I had wired two of the three.

| edition | Java `StepBlockRoll` shows | `DialogId` |
|---|---|---|
| bb2016 | `DialogBlockRollParameter` | `BLOCK_ROLL` |
| bb2020 | `DialogBlockRollPartialReRollParameter` | `BLOCK_ROLL_PARTIAL_RE_ROLL` |
| bb2025 | `DialogBlockRollPropertiesParameter` | `BLOCK_ROLL_PROPERTIES` |

All three map to the **same** Rust prompt: the driver has no `Rules::Bb2020` arm for `BlockRoll`, so
bb2020 runs the shared (bb2025) `StepBlockRoll` and raises the shared `AgentPrompt::BlockChoice`.
I had reasoned that `BLOCK_ROLL_PARTIAL_RE_ROLL` was unreachable — a grep for `PartialReRoll` in
`crates/ffb-engine/src/step/` returns nothing — and left it on the index-0 answer. The grep was
right and the conclusion was wrong: **Rust has no step named for that dialog because it does not
model the dialog, only the choice.** A Java dialog with no same-named Rust step is not evidence of an
unreachable path.

The probe that settled it took one run: `FFB_BC_PROBE` printing `dice/idx/own/nd` from both sides on
bb2020 seed 1-4 produced **20 `RUST_BC` lines and zero `JAVA_BC` lines** — the Java arm was never
entered at all, which is a much louder signal than a hash diff.

### The fix

`ParityRunner`: split the `BLOCK_ROLL` / `BLOCK_ROLL_PARTIAL_RE_ROLL` fall-through into three cases,
each casting its own dialog type and routing through one shared `heuristicBlockChoice(game, nrOfDice,
blockRoll)` helper. `nrOfDice < 0` → `own_choice = false` → every weight flipped to `1 - w`, exactly
as Rust does; the helper returns 0 whenever the class is off, so the random gate is untouched.

`HeuristicDriver.hasBall` mirrors `Features::ball_carried`: in play, **on the pitch**, and not loose.
Skills are matched by NAME (`"Block"`, `"Wrestle"`, `"Tackle"`, `"Dodge"`) because Rust checks
`has_skill(SkillId::Block)` — the literal skill. Keying off `NamedProperties` would have been wrong
as well as uncompilable: several skills register the same property, so a Wrestle-only player would
read as having Block.

### Non-vacuity

`FFB_BC_PROBE`, bb2025 seeds 1-10 at argmax: 117 block-die choices, **85 single-die (index forced),
32 two-die, of which 16 chose index 1**. bb2016 seeds 1-5: 4 of 12 multi-die choices took index 1.
The arm genuinely disagrees with the index-0 default in both, so both greens are real.

### Gates

- Rung 3b (`coin,receive,reroll,pushback,blocktarget,blockchoice`): **100/100 in all three editions
  at all three scales** — nine sweeps, fresh JVM, no `--reuse-java`.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,647 / 0**, including the new `block_choice_weight_table`,
  which pins the face ordering, the five Both-Down skill combinations, and the `push_squares` /
  `can_surf` geometry Java re-implements. `mvn -o -pl ffb-ai test`: clean.

**Next:** rung 4 — `blitztarget` (same shape as `blocktarget`, already green) and then `kick` /
`touchback`, which are pure geometry. `followup` stays parked until the `Features` raster tier lands
in Java, and that raster tier is the last thing standing between here and `--heur-classes all`.

## ITER18 — rung 4a: `blitztarget`, and an empty candidate list ends the TURN

`AgentPrompt::BlitzTarget` looked like it needed the raster tier and does not. Its arm consults the
activation PLAN first, but a plan only exists once `activateplayer` is ported, so with that class off
the arm falls straight through to a scored enumeration over `block_weight` — and `block_weight`'s
only heavy dependency is `Features::block_strength`, which is a memo around
`ServerUtilPlayer::find_block_strength`. **That method exists in Java**, and `ffb-ai` already depends
on `ffb-server`, so the port calls the original rather than re-deriving the assist arithmetic:

```java
int aStr = ServerUtilPlayer.findBlockStrength(game, att, attStr, def, false);
int dStr = ServerUtilPlayer.findBlockStrength(game, def, def.getStrengthWithModifiers(), att, false);
```

The rest is the die-count → weight table, the ball multiplier (1.35), the crowd-surf multiplier
(1.5, or 1.9 on the carrier) and the Block-without-Block/Wrestle penalty (0.70).

### The divergence

First gate: **89/100** in bb2025 at argmax — usefully red, so the arm is not vacuous. Lowest failing
seed 8, and the failure shape was `rust=None`: Rust's log simply *stopped* 54 steps before Java's,
with the score 0-0 mid-drive. `FFB_DRIVE_TRACE` named it in one line:

```
NO_PROGRESS seed=8 half=2 turn=5 active=away: 50 iterations with unchanged hash
  prompt=Some(BlitzTarget { attacker_id: "away_04", eligible_players: [] }) — aborting game
```

An **empty** candidate list. The engine raises `BlitzTarget` whenever *any* in-bounds opponent can be
blocked, but the candidates are only the **adjacent** ones — so a blitzer with no neighbour is
prompted with nothing to pick. `ParityRunner.sendBlitzTargetSelection` answers that case with
`ClientCommandEndTurn`, and `RandomAgent` mirrors it with a comment saying explicitly that
`EndPlayerAction` is wrong there. The heuristic arm returned `EndPlayerAction`, and
`StepSelectBlitzTarget` then waited for a target that never came.

Fix: `Action::EndTurn`, with `an_empty_blitz_target_list_ends_the_turn` asserting both agents give
the same answer. The test sets `DriverGameState::pending_prompt` directly — that field is
`pub(crate)`, so an agent arm can be exercised on a board the harness would need hundreds of steps to
reach.

Worth noting what the bug was *not*: nothing about scoring, weights or the sampler. Two iterations
running, the failure has been in the **plumbing around** the ported arm rather than in the arm —
ITER17's was a dialog id, this one a fall-through action. The scorer keeps porting cleanly; the
contract around it is where the port leaks.

`ParityRunner`: `pickBlockTarget`'s candidate computation is now `blockTargetCandidates`, which
draws no RNG, so the heuristic can score the same list without spending the `actionRng` draw the
Rust side does not spend in this configuration.

### Gates

- Rung 4a (`coin,receive,reroll,pushback,blocktarget,blockchoice,blitztarget`): **100/100 in all
  three editions at all three scales** — nine sweeps, fresh JVM.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025 (the
  `blockTargetCandidates` extraction is on the random path too).
- `cargo test --workspace --release`: **14,648 / 0**. `mvn -o -pl ffb-ai test`: clean.

Note `blitztarget` is inert in bb2016: that edition has no `SELECT_BLITZ_TARGET` step at all — the
harness drives a 3-command blitz instead — so bb2016's green there is carried by the other classes.

**Next:** rung 4b — `kick` and `touchback`, both pure geometry with no raster dependency. After that
the only classes left are `followup`, `intercept`, `setup`, `activateplayer` and `move`, and all five
want the `Features` raster tier in Java. That tier is now unambiguously the last thing between here
and `--heur-classes all`.

## ITER19 — rung 4b: `touchback`

The receiver table is small: `0.3 + 0.4·min(MA/9, 1)`, `+0.3` for Sure Hands, `−0.5` for standing
within one square of the line of scrimmage. No rasters, no plan, no `Features` beyond the board.

The one thing that needed care is the **eligible set**, which is not the set the random contract
uses. `StepTouchback` builds the receiving team's on-pitch players **with tackle zones**;
`ParityRunner`'s existing arm scans `isStanding()` and takes the one nearest to the kick-from square.
Those are different predicates — a player without tackle zones is a legal choice for the old rule and
not for the new one — so the heuristic path builds its own list rather than sharing that block's
loop, then re-sorts by `(side, nr)` and sends the chosen player's coordinate
(`transform()`ed when the away team receives, as the existing arm does).

Green first try, in all three editions at all three scales. The interesting part was the
**non-vacuity check**, because a first-try green is exactly the shape a vacuous rung has:
`FFB_TB_PROBE` over bb2025 seeds 1-20 found **13 touchbacks, and the arm picked index 3 twelve times
and index 1 once — never index 0**. The class fires often and never agrees with the old rule by
accident, so the green is real.

### A weight-table property worth keeping

My first draft of `touchback_weight_table` asserted that the LOS penalty always outweighs Sure Hands,
and it failed: `w(MA 9, Sure Hands, on LOS) = 0.5` against `w(MA 3, plain, off LOS) = 0.433`. The
penalty (0.5) is larger than the Sure Hands bonus (0.3) but smaller than the MA span (0.4 across
MA 1-9), so **the LOS term reorders the list, it does not partition it**. An implementation that
treated it as a veto would disagree exactly there. Same lesson as `pushback_weight_table` in ITER16:
writing the assertion is what finds the property, and the draft that fails is the useful one.

### Gates

- Rung 4b (`coin,receive,reroll,pushback,blocktarget,blockchoice,blitztarget,touchback`):
  **100/100 in all three editions at scales 0, 1.0 and 1e6** — nine sweeps, fresh JVM.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,649 / 0**. `mvn -o -pl ffb-ai test`: clean.

**Next:** `kick` — the last class before the raster tier, and much the largest of the cheap ones. Its
arm models the scatter risk of every candidate square *plus* the 25/216 Weather-Change → Nice gust
that takes three further d8 steps, computed by iterating a per-cell survival array rather than
enumerating paths. It needs no `Features`, but it is real arithmetic, so expect the port to be a
transcription job with a fixture test rather than a twenty-line table.

## ITER20 — rung 4c: `kick`, and a divergence with byte-identical picks

The largest of the cheap classes. The arm scores all 195 squares of the receiving half by
`p_safe² · exp(−d²)`, where `p_safe` folds two ways to lose the ball: the kick scatter (8 directions ×
1..`dmax`, `dmax` = 3 with Kick and 6 without) and the **gust** — the 25/216 Weather-Change→Nice
result that sends the ball three further single-square d8 scatters, each re-tested against the half
bounds. The three-step survival array `q` is built by ITERATING a per-cell array, not by enumerating
8³ paths per candidate.

Every one of those sums is f32, and f32 addition is not associative, so the port had to keep the
direction order, the loop nesting and the division points exactly as Rust has them. It did: the first
probe showed the two sides agreeing to the last bit.

### The divergence

`--heur-classes kick` alone: **0/10**, diverging at the very FIRST activation. And the probe said the
two agents agreed completely:

```
JAVA_KICK n=195 idx=97 home=false dmax=6 act=(6,7)
RUST_KICK n=195 idx=97 home=false dmax=6 act=KickBall { coord: (6,7) }
```

Same option count, same index, same weights to the last bit, same square — and the game still
diverged. The scoring was never the problem: **the two coordinates were in different frames.**

`StepKickoff` mirrors an away coach's coordinate back into the server frame, because a client command
from the away team arrives in the away-relative frame. `RandomAgent`'s KickBall arm pre-transforms
for exactly that reason and carries a comment saying what happens if it does not — "without this
pre-transform, StepKickoff mirrored (6,8)→(19,8), landing the ball in the kicking half → spurious
touchback". The heuristic arm scored in the server frame and emitted the server frame, so every away
kick in the game landed in the KICKING half.

Fix: score in the server frame, emit `if home_kicking { target } else { target.transform() }` — the
Java harness already did the matching `home ? hk : hk.transform()`. Test:
`an_away_kick_is_emitted_in_the_client_frame`.

**The lesson generalises past this arm.** Agreement on the *decision* is not agreement on the
*command*. A probe that compares what the two agents CHOSE will happily print two identical lines
while the engines do different things with them, and every instinct says to go looking at the scoring
model. Three iterations, three bugs in the plumbing around a correctly-ported arm — a dialog id
(ITER17), a fall-through action (ITER18), and now a coordinate frame.

### Gates

- Rung 4c (`coin,receive,reroll,pushback,blocktarget,blockchoice,blitztarget,touchback,kick`):
  **100/100 in all three editions at scales 0, 1.0 and 1e6** — nine sweeps, fresh JVM, and this is
  the first rung where all nine came back with no coverage warning either.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,651 / 0**. `mvn -o -pl ffb-ai test`: clean.

**Next:** the cheap classes are DONE. Everything left — `followup`, `intercept`, `setup`,
`activateplayer`, `move` — wants the `Features` raster tier in Java. Start it the way the plan says:
a cross-language FIXTURE test that dumps Rust's `occ` / `tz` / `row_prefix` rasters for a set of fixed
boards and asserts Java reproduces them, BEFORE wiring any of it to a game. If those cannot agree,
nothing downstream can, and a fixture is a far cheaper place to find that out than a 100-seed sweep.

## ITER21 — `intercept`, and the Loner roll that was never ported

`intercept` was on the "needs the raster tier" list and should not have been: the arm reads nothing
but `p_roll(target_number)`. Two options — attempt at `p_roll(0) * 1.5 = 1.25`, decline at a flat
`0.20`, `T = 0.20`. Accepting takes `find_interceptors().first()`, the ENGINE's own first candidate,
deliberately *not* the coordinate-sorted one the random pick draws from, so the Java side sends
`UtilPassing.findInterceptors(...)[0]` unsorted.

Non-vacuity: end-of-game hashes over bb2025 seeds 1-20 differ from the `none` mask on **2 seeds**.

### The divergence

bb2025 and bb2016 green at every scale; **bb2020 at `--heur-scale 1.0` came back 99/100**, seed 54.
Adding a class shifts the sampler stream, and a different stream walked into a bug that had been
sitting there the whole time.

The route to it is worth recording, because two diagnostics lied on the way:

1. The state hashes first differ at step 149, inside a blitz. Both sides' block-choice weights are
   `[0.9, ...]` — but Java's second die is `0.4` and Rust's is `0.9`. Java's dialog says the dice are
   `[6, 3]`; Rust's prompt says `[6, 6]`. Same attacker, same defender, same board.
2. The **dice-stream diff said the first difference was at position 83** — a d8 in Java against a d6
   in Rust, forty-odd rolls later. That is the trap `feedback_parity_dice_comparison` warns about:
   Rust was drawing the same values one position EARLIER, and positions 70-72 happened to be
   `6, 6, 6`, so an off-by-one inside a run of equal values is invisible to a diff.
3. What actually localised it was per-step `rng_calls`: identical up to step 136, where Java spends
   **three** and Rust **two** — and the state hashes still matched afterwards, because the extra roll
   changed nothing that step. `FFB_DICE_TRACE`'s Java `caller=` named it outright:
   `RollMechanic.useReRoll:306`.

That line is the **Loner** roll:

```java
if (pPlayer.hasSkillProperty(NamedProperties.hasToRollToUseTeamReroll)) {
    int roll = gameState.getDiceRoller().rollSkill();
    successful = DiceInterpreter.getInstance().isSkillRollSuccessful(roll, minimumLonerRoll(pPlayer));
    stepResult.addReport(new ReportReRoll(pPlayer.getId(), ReRollSources.LONER, successful, roll));
} else { successful = true; }
```

All three editions do it. **Rust did it in none** — `util_server_re_roll.rs` had no mention of the
property, and `GameEvent::LonerRoll` existed in the model with no producer anywhere in the engine.

### Why a lineman had Loner

Worth stating, because "there is no Loner in lineman vs lineman" is the obvious objection and it is
almost right. Seed 54's kickoff rolled **Cheering Fans**, which in BB2020 grants a Prayer to Nuffle,
which rolled **BAD_HABITS** for the home team — and Bad Habits gives Loner to the *opposing* team.
`away_01` re-rolled a dodge two turns later. Three coincidences had to line up before this roll could
ever be observed: the `reroll` class has to ACCEPT an offer (the random contract always declines),
the kickoff has to produce a prayer, and the prayer has to be that one.

### The fix

`use_reroll` now takes `&mut GameRng` and performs the Loner roll after spending, returning whether
it succeeded — **the re-roll is spent either way**, which is the half a naive "return false" gets
wrong. 131 call sites updated. Test:
`a_loner_rolls_for_the_team_reroll_and_spends_it_either_way`, which pins both halves and asserts a
non-Loner re-roll consumes NO die — that last one is what kept the bug invisible.

### Gates

- `coin,receive,reroll,pushback,blocktarget,blockchoice,blitztarget,touchback,kick,intercept`:
  **100/100 in all three editions at scales 0, 1.0 and 1e6**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,652 / 0**. `mvn -o -pl ffb-ai test`: clean.

**Seven engine bugs now**, all 1:1 port gaps the random gate could not structurally reach.

**Next:** the `Features` raster tier in Java — the cross-language FIXTURE test first (`occ`, `tz`,
`row_prefix` for fixed boards), before wiring any of it to a game. Only `followup`, `setup`,
`activateplayer` and `move` are left, and all four want it.

## ITER22 — `followup`, and `getSkills()` is not `has_skill`

`followup` had been parked twice for needing "the raster tier and a parameter Java cannot see".
Both halves of that turned out to be wrong:

- `f.tz_against(c, home)` is *how many standing opponents of `home` stand next to `c`*. Rust reads it
  out of a whole-pitch raster because it queries it thousands of times; the follow-up arm asks for
  exactly **two** squares, so the Java side counts them pointwise. Building a raster there would be
  a way of spending 390 cells to answer two questions.
- `target_coord` is the DEFENDER_POSITION step parameter, and `DialogFollowupChoiceParameter` really
  does carry nothing. But the harness can reconstruct it: snapshot the defender's square at
  **block-dice time**, which is after the defender is fixed and before the push moves him, and which
  fires exactly once per block in every edition. Verified against the live games — the snapshotted
  defender id matches `getDefenderId()` at follow-up time on every single follow-up.

Non-vacuity is emphatic: the random contract always declines, so **18 of 20** bb2025 seeds end on a
different board with the class on.

### The divergence

bb2025 and bb2016 green everywhere; **bb2020 at argmax 99/100**, seed 93. But `--heur-classes
followup` ALONE was 100/100 on bb2020 — so, exactly like ITER21, the new class did not cause the bug,
it walked the game onto a path that reached one.

Chasing it backwards through three probes: the state first differs after a blitz, where two AWAY
players end up in different squares (a chain push) → the pushback probe shows the same attacker
pushing a DIFFERENT defender → the blitz-target probe shows the same attacker, the same two
candidates, and different weights:

```
JAVA_BT att=(12,6) cand=[(13,7), (13,6)] idx=0 w=[0.1,  0.1]
RUST_BT att=(12,6) cand=[(13,7), (13,6)] idx=1 w=[0.07, 0.1]
```

`0.07 = 0.1 × 0.70` is `block_weight`'s "defender has Block and I do not" penalty. Rust saw Block on
the player at (13,7); Java did not.

Root cause: **`Player.getSkills()` excludes temporary skills.** Rust's `has_skill` iterates
`starting + extra + TEMPORARY`; the Java mirror has to use `getSkillsIncludingTemporaryOnes()`.
Seed 93 rolled the **INTENSIVE_TRAINING** prayer for the away team, which grants a skill — and the
lineman it landed on was a Blocker to one engine and a plain lineman to the other.

This is a bug in the Java driver, not the engine, and it silently affected three already-green
classes: `blockchoice` (Block/Wrestle/Tackle/Dodge), `blitztarget` (via `block_weight`) and
`touchback` (Sure Hands). All three were green only because no prayer had yet handed out a skill that
mattered on a seed those classes could reach.

Two prayer-granted-skill bugs in two iterations (Loner in ITER21, Block here) is not a coincidence:
prayers are the only way a lineman game gets skills at all, so they are where every
skill-reading code path gets its first real test.

### Gates

- `coin,receive,reroll,pushback,blocktarget,blockchoice,blitztarget,touchback,kick,intercept,followup`:
  **100/100 in all three editions at scales 0, 1.0 and 1e6**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,653 / 0**, with `follow_up_weight_table`.
  `mvn -o -pl ffb-ai test`: clean.

**Next:** `setup` is a structural no-op — the heuristic's `TeamSetup` arm calls the very
`canonical_setup_action` the random agent calls, with no sampler draw, so switching it on cannot
change anything and its green would prove nothing. That leaves `activateplayer` and `move`, which
ARE the raster tier, plus `other` (which routes to `UniformAgent`, so it is not a no-op and needs
checking before `--heur-classes all` is claimed).

## ITER23 — `setup`, `skill`, `other`: everything that is not the raster tier

Three classes at once, because two of them are no-ops and the third is one line.

**`setup`** is a STRUCTURAL no-op: the heuristic's `TeamSetup` arm calls the very
`canonical_setup_action` the random agent calls, with no sampler draw. On or off, byte-identical.
**`skill`** is measured unreachable in this tier (ITER0). Both confirmed empirically: 0 of 20 bb2025
seeds change their end-of-game hash with either switched on. A green that proves nothing is still
worth *recording* as proving nothing.

**`other`** was not a no-op, and should have been. `PromptClass::Other` is the tail the agent does
not model at all — and it fell through to `UniformAgent`, whose `PlayerChoice` arm sorts candidates
**by player id**. The two engines generate different ids (`home_06` vs `teamLinemanParityHome6`), so
that arm cannot agree with the Java side by construction; it also breaks the campaign's own rule
that ids never enter an ordering. `RandomAgent`, the byte-matched contract, carries a dozen
reason-specific `PlayerChoice` arms — RAIDING_PARTY, AUTO_GAZE_ZOAT, WISDOM, FURIOUS_OUTBURST,
ANIMAL_SAVAGERY — every one coordinate-sorted for exactly that reason.

bb2020 seed 26 is the whole story in one game: **one** `Other` prompt in 300 steps, a `PlayerChoice`
raised by a prayer, and the game diverged at step 0.

Fix: the `_` arm delegates to `self.parity` instead of `self.fallback`, which makes `Other` mean what
it says — the agent does not model this prompt, so it uses the reference contract. Test:
`the_unmodelled_tail_answers_with_the_parity_contract` asserts the class is a no-op in both
directions AND that both agree with `RandomAgent` itself.

This is a change to the AGENT, not the engine, and it is worth being explicit that it is not a
parity hack: a fallback that orders players by a generated id is wrong on its own terms, and the
`--heuristic` experiment reaches the same prompts.

### Gates

- `coin,receive,reroll,pushback,blocktarget,blockchoice,blitztarget,touchback,kick,intercept,followup,setup,other,skill`
  — **fourteen classes, everything except `activate` and `move` — 100/100 in all three editions at
  scales 0, 1.0 and 1e6.**
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,654 / 0**. `mvn -o -pl ffb-ai test`: clean (no Java
  change this iteration).

**Next:** `activate` and `move` — the `Features` raster tier, and the last thing between here and
`--heur-classes all`. Start with the cross-language FIXTURE test the plan calls for: dump Rust's
`occ` / `tz` / `row_prefix` for a set of fixed boards and assert Java reproduces them, before wiring
any of it to a game.

## ITER24 — the `Features` CORE raster tier, pinned by a cross-language fixture

The last two classes, `activate` and `move`, stand on `Features`, and `Features` stands on three
whole-pitch arrays. The plan says to pin those with a FIXTURE before wiring anything to a game, and
that is exactly what this iteration does — no parity sweep involved, and none needed.

- Rust: `emit_features_golden` (ignored) writes `agent/testdata/features_golden.txt` — five boards,
  each with its `occ`, both `tz` sides, both `rowPrefix` sides and both `unactivated` values.
- Java: `Features.java` (CORE tier) + `FeaturesTest.coreRastersMatchRust`, which reads that file.

The boards are chosen to break a careless reimplementation: pitch corners (neighbour clipping), a
full line of scrimmage with players adjacent across it (both `tz` sides at once), prone players
(they occupy a square but mark nothing), and a mixture of activated and unactivated players.

**`Features.build` takes a snapshot, not a `Game`.** The raster arithmetic and the model plumbing are
separate concerns and only the arithmetic has to agree bit for bit, so the fixture pins the
arithmetic and `snapshot(Game)` stays a thin adapter the parity sweep exercises. Without that split
the fixture would have to construct a whole Java `Game` per board.

### What the first run found

`occ`, `tz` and `rowPrefix` matched on all five boards immediately — including the `rowPrefix`
off-by-one, which is an exclusive prefix over `W + 1` columns per row and the single most likely
thing to get wrong. The one disagreement was `unactivated`, and it was the **fixture** that was
wrong, not either implementation: `PlayerState::new(PS_STANDING)` does NOT set the ACTIVE bit — that
is a bit of its own — so Rust saw zero active players where the Java test had assumed all of them
were. Fixed by emitting the flag the Rust engine actually computed as a column in the golden file,
so neither side gets to assume it.

That is the same lesson as the state hash not being able to see the ACTIVE bit
(`parity_tier_ttm`), arriving from a different direction.

### Is the fixture worth anything?

Checked, rather than assumed: changing the Java `rowPrefix` loop from `p.x + 1` to `p.x` — the
exclusive/inclusive off-by-one — fails it immediately and precisely:

```
FeaturesTest.coreRastersMatchRust:121 rowPrefix[1] entry 202 (row 7, col 13)
  on board single_centre ==> expected: <0> but was: <1>
```

Row, column and board, not "a sweep went red". Restored afterwards and re-verified green.

### Gates

- `mvn -o -pl ffb-ai test`: **17 tests, 0 failures**, including the new fixture.
- `cargo test --workspace --release`: **14,654 / 0**.
- The fourteen-class rung is still **100/100 in all three editions** at argmax, and `--agent random`
  is still 100/100 in all three. No behaviour changed this iteration — only the emitter and the new
  Java class, neither of which is on a live path yet.

**Next:** `build_threat`, `build_lane` and `build_support` (the HEAVY tier) extended into the same
fixture, then `Reach` — whose quantised Dijkstra key is the next thing that cannot be checked any
other way. `build_threat` is the one with a KNOWN ordering hazard (ITER1: it writes under a strict
`>` and ties were resolved by hash order), so the fixture needs a board with mixed ST to pin it.

## ITER25 — the HEAVY tier: threat, lane, support

`build_threat`, `build_lane` and `build_support` ported to Java and added to the same fixture.
Six boards now, and the golden file carries the board state the heavy tier reads that the core tier
does not: where the ball is, whether it is loose or carried, and whether each team has spent its
blitz.

Three of those inputs were chosen because each switches a whole term on or off:

- **a LOOSE ball** (`prone_marks_nothing`) — `build_support`'s screen term falls back to the ball's
  square when nobody carries it;
- **a CARRIED ball with the away blitz already spent** (`line_of_scrimmage`) — cage and mark get a
  target, and `build_threat`'s block term switches off for every non-adjacent away player. That
  `(d == 1 || !opp_blitz_spent)` guard is easy to drop;
- **mixed strength** (`mixed_strength_ties`, ST 3/4/5) — see below.

### The mixed-ST board earns its place

`build_threat` writes `threat_str` under a strict `>` against `threat_reach`, so two opponents that
reach a square equally TIE, and whichever is visited first records ITS strength. That is the ITER1
bug — Rust resolved the tie by `HashMap` order and was non-deterministic run to run on any roster
with mixed ST. An all-ST3 fixture cannot see it: every tie writes the same 3.

Checked that the board actually catches it, by reversing the Java sort within a side:

```
FeaturesTest.coreRastersMatchRust threatStr[0] at (0,0) on board mixed_strength_ties
  ==> expected: <5> but was: <4>
```

So a class of bug that previously surfaced as run-to-run non-determinism is now a unit-test failure
naming the square. Restored and re-verified green.

### The one real disagreement

`threat_str` is seeded to **3**, not 0 — an unthreatened square reports the baseline ST rather than
none — and `lane` to 1.0 and `support` to 0.10. Java's `new float[]` is zero, so the first run failed
at `(0,0)` on the EMPTY board. `lane` and `support` are overwritten cell by cell so their defaults
never show, but `threat_str` is written only where some opponent can reach, which leaves the default
visible over most of the pitch. Worth noting that the empty board — the one that looks like it tests
nothing — is what caught it.

Float arrays are compared **bit for bit**, not within a tolerance. A tolerance would be the wrong
test: these values are compared with `>` and fed to a softmax, so a last-bit difference can reorder
two options and change the answer. f32 arithmetic is bit-portable between the two languages
(ITER2), so exact equality is achievable, and anything less would let a real divergence through.

### Gates

- `mvn -o -pl ffb-ai test`: **17 tests, 0 failures**, the fixture now checking 108 arrays across
  6 boards.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax; `--agent random` 100/100 in
  all three. Nothing on a live path changed.

**Next:** `Reach` — the quantised-key Dijkstra. It is the last piece before the value model, it has
its own determinism hazard (a binary heap ordered by an integer key, where ties must break the same
way in both languages), and it is the natural next fixture: dump the reach key and path for a fixed
board and mover, and assert Java reproduces both.

## ITER26 — `Reach`: the quantised-key Dijkstra

The last piece before the value model, and the one where a reimplementation is most likely to agree
approximately and disagree exactly. `Reach.java` + `ReachTest`, pinned by `reach_golden.txt`: five
cases, and for each one the key, cost, GFI count and back-pointer of every one of the 390 squares,
the visit set, and twelve explicit back-pointer WALKS.

Cases chosen so each turns on a different branch: an open pitch (pure GFI, no dodges), a gauntlet
(the mover starts marked, so every step is a dodge and the target depends on the destination's
tackle zones), prone with a team re-roll (stand-up eats MA, `gate` becomes `p_roll(4)`, and the
re-roll applies to the first roll only), Blizzard with Sure Feet (GFI target 3, and the re-roll
branch rather than the bare roll), and BB2016 with Dodge (a different dodge-target formula
entirely).

### Why `prev` and the paths are in the fixture

Two routes to the same square can carry the **identical key** — a dodge past one tackle zone costs
the same wherever it happens — so an implementation whose heap breaks ties differently produces the
same arrival PROBABILITIES by a different ROUTE. Every key would match and the agent would still
walk somewhere else. The back-pointer array catches that; the keys alone cannot.

### What dropping the tie-break actually does

Ordering the Java queue on `key` alone, as a `PriorityQueue<Item>` naturally invites, fails
immediately:

```
ReachTest.reachMatchesRust key at (11,0) in case open ==> expected: <746> but was: <1492>
```

Worth noting that it changed the **key**, not just the route. Rust orders on `(key, cost, idx)`, and
without the `cost` term a higher-cost cell with an equal key can be popped first, be marked `seen`,
and propagate a worse key onward — so the tie-break is load-bearing for correctness, not only for
determinism. I had expected this to show up as a differing `prev` with matching keys. Restored and
re-verified green.

### Honest limits of this fixture

The explicit clamp on the key increment (`clamp(0.0, 1.0e9) as i64 as u32`, there because `as u32`
saturates in Rust and the Java cast does not) is **not** exercised by these boards: `p_step` never
gets near `1e-6`, so the conversion is in range on every step. The clamp is ported and commented on
both sides, but this fixture is not evidence that it agrees — only that nothing here needs it.

### Gates

- `mvn -o -pl ffb-ai test`: **18 tests, 0 failures** — the new `ReachTest` checks 5 cases × 4 cell
  arrays plus 60 path walks.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax; `--agent random` 100/100 in
  all three. Nothing on a live path changed — `Reach.java` has no caller yet.

**Next:** the value model — `value_at` / `exposure` / `strength_factor` and the arrival weights that
read `Reach`. That is the last thing between here and `build_plans`, and the same fixture treatment
applies: dump Rust's per-square value for a fixed board and mover before anything is wired up.

## ITER27 — `value_at`, and a bite-check that lied twice

The value model: what a square is WORTH to a particular mover. `ValueModel.java` + `ValueModelTest`,
pinned by `value_golden.txt` — four boards, sixteen movers, and for each the value of all 390 squares
plus **the rule that produced it**.

The rule column matters as much as the number. Three branches (carrier / pickup / support-or-receiver)
are genuinely different formulas, and two implementations can agree on a value while disagreeing
about which branch produced it — invisible until the board changes and the branches diverge. All four
rule characters appear in the golden, and the test asserts each one actually fires.

### The bite-check was wrong twice before it was right

This is the part worth recording, because the failure mode was *silent* both times.

1. **First attempt: the perturbation never landed.** The script that flipped `strengthFactor`'s 0.5
   to 0.55 had no assert. It printed its success message unconditionally, replaced nothing, and the
   test passed — which I read as "that branch is unreachable". It was a broken experiment reporting
   a real-sounding result. Every perturbation script now asserts its target matched.
2. **Second attempt: the branch really was unreachable, for a reason the first attempt hid.**
   `strength_factor(att, def)` takes the THREAT as `att` and the MOVER as `def`, so `2 * att < def`
   needs a mover more than twice as strong as the threat. Adding an ST 7 mover was still not enough:
   the board's opponents were ST 5 and ST 4, and `threat_str` only holds 3 where NOBODY reaches —
   and where nobody reaches, `threat_reach` is 0 and the factor is multiplied away.

The fix is an isolated **ST 3** opponent standing well clear of the other two, so the squares around
him carry `threat_str = 3` with a positive reach. Then the perturbation fails immediately:

```
ValueModelTest.valueAtMatchesRust value at (0,0) for strength_factor/st7_outmuscles
  ==> expected: <0.014880952> but was: <0.014450868>
```

Generalisable: **a raster's default value is not observable wherever the raster's own gate is zero.**
Covering a branch of `strength_factor` needed a board where the threat exists AND is weak, which is
not what "add a strong mover" produces.

### Gates

- `mvn -o -pl ffb-ai test`: **19 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax. Nothing live changed —
  `ValueModel.java` has no caller yet.

**Next:** the arrival weights (`Arrival`, the signed weight of arriving at a square, which combines
`Reach::p_arrive` with `value_at`) and then `build_plans`. After that the WIDE `ActivatePlayer` and
`Move` arms can be wired up, and the live gate begins — which is where the remaining engine
divergences are, since multi-square movement, GFI chains and scoring paths all go live at once.

## ITER28 — `arrival_parts`: where reach meets value

The composition step: `w = p·V − (1−p)·c_turnover − rush_penalty`, with a short-circuit for a
carrier arriving IN the endzone — a touchdown ends the drive, so there is no "after" to lose and
only the rush is priced. `Arrival.java` + `ArrivalTest`, pinned by `arrival_golden.txt`.

Both halves were already pinned separately. What is new is that they are combined the same way, that
the GFI count carried out of the reach search is the one the penalties see, and that the touchdown
branch fires on exactly the right squares.

**All four parts are compared, not just `w`.** A sum of three terms reaching the same total by
different routes is precisely the disagreement a single number hides: a value model that is too
generous and a turnover cost that is too harsh cancel on one board and diverge on the next.

Three cases, each with a term that would otherwise be dead:

- `carrier_can_score` — the carrier stands **6** squares from the endzone with MA 6, so the touchdown
  short-circuit is genuinely reachable. Put him out of range and that branch is never taken, which
  is the easy version of this fixture to write and the useless one. A variant with `unactivated = 0`
  isolates the `c_turnover` scaling, and one with `turns_left = 1` the hopeless damp.
- `gauntlet_rushes` — marked on all four sides, so distant squares cost GFI: `rush_penalty` and the
  `gfi` factor both bite, and four times harder for a non-carrier. The golden carries GFI counts of
  0, 1 **and** 2.
- `loose_ball_in_reach` — the pickup value and the arrival probability multiply, which is the whole
  point of the composition.

The test asserts the coverage rather than assuming it: it fails if no square needed a rush, and
fails if the endzone short-circuit never fired. After ITER27 — where a bite-check reported a real
result from an experiment that had silently done nothing — the fixtures now say out loud which
branches they reached.

Bite-check: `rushPenalty`'s non-carrier constant 0.40 → 0.35 fails at
`w at (2,0) for gauntlet_rushes/noncarrier ==> expected: <-1.5448468> but was: <-1.4448467>`.

### Gates

- `mvn -o -pl ffb-ai test`: **20 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax. Nothing live changed.

**Next:** `build_plans` and then the WIDE `ActivatePlayer` and `Move` arms. Every ingredient they
read is now pinned bit for bit — rasters, reach, value, arrival — so what remains is the plan
selection itself and the wiring. After that the live gate begins, and with it multi-square movement,
GFI chains and scoring paths that the random gate has never executed.

## ITER29 — the plan orderings: `top_moves`, `run_up_squares`, `risked`

`build_plans` is ~320 lines and enumerates from two ordered destination lists. The weights inside
them are already pinned; what this iteration pins is the **order**, because the agent samples an
INDEX into these lists — two implementations that agree on every weight and transpose one pair pick
different squares.

`Plans.java` + `PlansTest`, three boards: an open field (where most reachable squares score
identically, so ties do nearly all the ordering work), a carrier within reach of the endzone, and a
four-way gauntlet (where LIST MEMBERSHIP is as much of the contract as order — most of the pitch is
unreachable). The golden stores the full ordered lists, and a mismatch reports the position plus
both cell coordinates rather than "a sweep went red".

`run_up_squares` deserves its own mention: it contains **two different orderings**, and collapsing
them is the obvious port mistake. The mover's own square goes first unconditionally so "use none of
my move" is never lost, and the rest are ranked by arrival probability weighted by forward
progress — deliberately NOT by arrival weight, because a throwing platform is judged by whether he
gets there and how far up the pitch it is, not by what standing there is worth.

### One bite-check that discriminates, and one that cannot

Collapsing the run-up metric to bare `p_arrive` fails immediately:
`runUpSquares position 1 (open_field_ties/plain) ==> expected: <40> but was: <28>`.

Dropping the index tie-break from `top_moves` **does not fail**, and cannot: the list is built from
`Reach::order`, which is sorted ascending, and both languages' sorts are stable, so ties keep their
input order either way. The explicit fallback is kept — it is what Rust writes, and it stops the
ordering depending on a property that lives in a different method — but the fixture does not test
it. Recorded in the Javadoc as a measured caveat rather than left as an implied guarantee; the
alternative is a comment claiming coverage the test does not have.

That is now the second fixture whose bite-check found a branch it could not reach (ITER27 was the
first). Running the check is cheap; assuming the fixture is strong is not.

### Gates

- `mvn -o -pl ffb-ai test`: **21 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax.

**Next:** the remaining `build_plans` weights that have no fixture yet — `handoff_weight`,
`pass_weight`, `foul_weight`, `proxy_value` — and then the enumeration itself plus the WIDE
`ActivatePlayer` arm. The enumeration is where Java needs the harness's legal-action list rather
than a fixture, so that is the point at which the live gate takes over from the goldens.

## ITER30 — `receiver_of`, `handoff_weight`, `foul_weight`

`receiver_of` is the biggest single function in the plan layer and the one with the most ways to be
quietly wrong — and every one of them is an **off-by-one-step**, not a bad constant:

| term | the mistake it invites |
|---|---|
| `active` | a receiver who has already gone has `reach_after = 0` and one turn fewer; that single flag separates a touchdown from a token credit |
| `effective_d` | where the BALL ends up, `d_rcv − reach_after`. Using `d_rcv` prices every give as though the receiver never moved afterwards |
| `scores_now` | false when the CARRIER could score by himself — without it the agent gives away a run it had already won |
| `p_run_in` | a five-way ladder on the receiver's MA, with a GFI factor per rush |

`BallMoves.java` + `BallMovesTest`, four boards: a give that scores, the SAME board with the receiver
already activated, a board where the carrier can score himself, and a foul board with prone victims
on, next to, and far from the ball (the three branches of the `victim` term) plus assists on both
sides.

All five parts of the receiver are compared — `p_catch`, `v`, `scores_now` and both turn counts —
because the flag and the counts are what the hand-off price branches on, and a value that matches
with the wrong flag behind it is a disagreement waiting for the next board. The test also asserts
`scores_now` is seen BOTH ways; a fixture where it is always false would pass while testing almost
nothing.

Foul assists are fed IN by the golden rather than recomputed, the same split as `Features::build`
taking a snapshot: the arithmetic is what needs pinning, and production calls the engine's own
`UtilPlayer` on both sides.

### Bite-checks

Both of the subtle terms discriminate, and the second fires exactly on the board built for it:

```
effectiveD = dRcv                         → v give_that_scores/home_2 from (14,7)  differs
scoresNow ignores carrierScoresNow        → v carrier_can_score_himself/home_2 from (20,7) differs
```

### Gates

- `mvn -o -pl ffb-ai test`: **22 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax.

**Next:** `pass_weight` — the last plan weight — which needs the engine's own pass mechanics
(`find_passing_distance`, `minimum_roll_simple`, `evaluate_pass_simple`); Java has all three, so the
port calls them rather than re-deriving a range table. Then `proxy_value`, the enumeration itself,
and the WIDE `ActivatePlayer` arm.

## ITER31 — `pass_weight`: the last plan price

A fumble is a turnover on the spot, so a pass has to be an EXPECTATION rather than a preference, and
the three outcomes are priced apart:

```
p_complete = p_accurate · p_catch
p_lost     = p_fumble + p_scatter · 0.45 + p_accurate · (1 − p_catch)
w          = p_complete · v − p_lost · risk
```

**The 0.45 is the whole argument.** A scattered ball is *not* a turnover — it lands three squares
away and either side may reach it — so pricing a scatter like a fumble makes the agent refuse every
pass it should be making. Perturbing it to 1.0 fails the fixture at
`short_to_scorer/home_2 from (12,7)`.

The six-face loop is the other load-bearing part: it asks the ENGINE'S OWN grader which faces are
ACCURATE and which FUMBLE rather than deriving them from the target number, because those are
different questions — a 1 fumbles whatever the target is, and the accurate band differs by edition.
Tackle zones on the thrower shift the effective ROLL, not the target, which is why the counts and
not a modifier are what cross the boundary.

Four cases: a short pass to a scorer, the same throw with the thrower MARKED (the accurate/fumble
split moves 2/2 → 0/4 → 1/3, so the shift is visibly applied to the right side), a **Blizzard** where
Long is not a legal throw at all (Rust returns `None`, and the option must not exist), and BB2016,
which grades on a different table.

The golden carries the graded face counts as INPUTS — the same split as `Features` taking a snapshot
and foul assists being fed in. Those are shared engine mechanics the parity matrix already covers,
so re-deriving them in the test would pin a second copy of the pass tables instead of the arithmetic
on top. `BallMoves.gradeFaces` is the production path that calls the real mechanic, and the live gate
is what will exercise it.

The test asserts its own coverage: an illegal throw must appear, and the accurate/fumble split must
take at least three distinct values — a fixture where every throw grades 2/2 would pass while
testing a three-outcome model at one point.

A column off-by-one in the test read `tz_on_thrower` as the expected weight on the first run. It
failed loudly (`expected: <0>`), which is the right way for that mistake to go.

### Gates

- `mvn -o -pl ffb-ai test`: **23 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax.

**Every weight `build_plans` reads is now ported and pinned.** What is left is `proxy_value` (the
search-free tier-1 estimate), the enumeration itself, and the WIDE `ActivatePlayer` arm — and the
enumeration needs the harness's legal-action list rather than a fixture, so that is where the live
gate takes over.

## ITER32 — `proxy_value`: the last weight

`proxy_value` is the §20.3 tier-1 stand-in — what scores every player the reach search did NOT run
for. No Dijkstra: the eight adjacent squares scored exactly, plus an admissible ceiling over
everything inside `MA + 2` read straight off the rasters, discounted to 55% because the ceiling is
optimistic by construction.

Worth being explicit about why it matters. A disagreement here **reorders the activation queue
without changing a single move** — it looks like a different decision when it is really a different
sort, and that is a much harder failure to read backwards from a state hash than a wrong square.

One trap in the port: the ceiling is **not** `value_at`. It drops the exposure and sideline terms
and keeps only advance × lane (carrier), a flat 0.9 (loose ball), or the support raster. Reusing
`value_at` there is the natural simplification and changes the number. Written out rather than
delegated, with a comment saying so.

Folded into the existing plans golden rather than a new file, since it needs the same board and
mover. Bite-check: dropping the 0.55 discount fails at `proxyValue (open_field_ties/carrier)`.

### Gates

- `mvn -o -pl ffb-ai test`: **23 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax.

### Where the port stands

Every numeric input to the last two prompt classes is now ported AND pinned by a cross-language
fixture:

| layer | Java | fixture |
|---|---|---|
| `exp`/`ln` | `DetMath` | 348 vectors |
| sampler | `Sampler` | 120 unit + 1200 pick vectors |
| rasters (core + heavy) | `Features` | 6 boards × 12 arrays |
| reach | `Reach` | 5 cases, every cell + 60 path walks |
| value | `ValueModel` | 4 boards × 16 movers × 390 cells + rule |
| arrival | `Arrival` | 3 boards × 7 movers, all four parts |
| orderings + proxy | `Plans` | 3 boards, full ordered lists |
| give / foul | `BallMoves` | 4 boards, all five receiver parts |
| pass | `BallMoves.passWeight` | 4 boards incl. Blizzard-illegal and bb2016 |

**Next:** the enumeration in `build_plans` and the WIDE `ActivatePlayer` arm. That needs the
harness's legal-action list, not a fixture — so this is the point where the goldens hand over to the
live gate, and where multi-square movement, GFI chains and scoring paths become reachable for the
first time.

## ITER33 — the tier-1 activation ranking

Before any search runs, every eligible player gets a search-free score, and only the top `TIER2`
(16) get a Dijkstra. So this ladder decides **who is even considered properly** — and a disagreement
here does not present as a wrong move. It presents as the right move made by the wrong player, or by
a player the other engine never scored at all.

`Activation.java` + `ActivationTest`, six boards covering all six rungs plus both post-ladder
effects (the negatrait ×0.55, and the `awaiting_run` override that OVERWRITES rather than scales).

### The bite-check failed to bite, twice, and both were real

1. Swapping `can_fetch` (0.92) and `is_carrier` (0.88) — my comment had called this "the rung most
   easily reordered" — changed **nothing**. The two are **mutually exclusive by construction**: a
   `can_fetch` needs a LOOSE ball and `is_carrier` needs a CARRIED one. Their relative order is
   unobservable, and no board can pin it. The comment was wrong and is now corrected in place; a
   Javadoc asserting a property no test covers is worse than no Javadoc.
2. Swapping `prone && marked` (0.70) and `proxy > 0.25` (0.45) — the ONE adjacent pair that can both
   hold — also changed nothing, because a prone marked player is normally hemmed in and his proxy
   never cleared 0.25 on any existing board.

Fixed by adding `prone_marked_with_support`: the prone marked player stands next to our own carrier,
so the cage term lifts the support raster around him and his discounted ceiling reaches **0.3025**.
The swap now fails at `wPlayer prone_marked_with_support/home_2`.

Three iterations in a row where the bite-check found something the fixture could not reach
(ITER27 `strength_factor`, ITER29 the sort tie-break, and both of these). The pattern is consistent
enough to state plainly: **a fixture that passes tells you nothing about the branches it never
entered**, and the only cheap way to find those is to break the code on purpose and watch.

### Gates

- `mvn -o -pl ffb-ai test`: **24 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax.

**Next:** the rest of `handle_activate` — the per-declaration grouping (contiguous runs of the
candidate list, NOT a keyed lookup) and the two-level softmax draw that groups by declaration and
scores each group by its best child. Then the enumeration wiring in `ParityRunner`, and the live
gate.

## ITER34 — the two-level activation draw

The agent does **not** sample flatly over candidates. It groups them by DECLARATION — the
`(player, action)` pair the engine actually receives — scores each group by its BEST child, samples
a group at `T = 0.18` and then a child within it at `T = 0.10`.

The reason is cardinality. A Move declaration can carry two thousand destinations and a Block nine,
so a flat draw lets the Move branch drown the Block one purely by how many squares exist. Scoring a
group by its max keeps argmax identical to a flat draw while fixing the sampled case.

Three things have to agree, and only the first is obvious:

1. **contiguous runs, not a keyed lookup.** `build_plans` emits one action at a time, so a
   declaration's options are adjacent. A keyed lookup merges two *non-adjacent* runs of the same
   declaration into one group and changes the sampling tree.
2. `EndTurn` is its own group, appended last, weight exactly 0.0 — so it beats every negative branch
   and loses to every positive one.
3. **the draw count.** Two `softmax_pick` calls spend one draw each, *unless* a level has a single
   entry, where it spends none. A grouping that produces one group too many or too few costs a draw,
   and every decision after it reads a different number.

Extracted `group_declarations` in Rust so the emitter and the live path cannot drift — a fixture
that reimplements the grouping is pinning its own copy of it.

Five synthetic candidate lists chosen for shape rather than realism: twelve Move destinations
against two Block targets (the cardinality case), one player with two adjacent actions, **interleaved
runs** of the same declaration, all-negative weights (so EndTurn's 0.0 wins), and an exact tie. The
test asserts it sees draw counts of 0, 1 **and** 2 — a set of cases that always spends two would
leave the singleton path untested.

Bite-check: replacing the contiguous rule with a keyed lookup fails at
`group count (interleaved_runs) ==> expected: <4> but was: <3>` — the board that exists for it.

### Gates

- `mvn -o -pl ffb-ai test`: **25 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung **100/100 in all three editions** at argmax; `--agent random` 100/100 in all
  three.

**Next:** the wiring. `ParityRunner` has to enumerate the same candidate list — which needs the
harness's legal-action list per eligible player, the thing no fixture can supply — and then
`--heur-classes activate` can go live. That is the first sweep where a mistake shows up as a parity
failure rather than a red unit test.

## ITER35 — `build_plans`: the enumeration shape

Every weight is pinned; what this iteration pins is the SHAPE of the candidate list — how many
entries each declared action contributes, in what order, with which kind and target. That list is
the input to the two-level draw, so a list differing by one entry picks a different action even when
every weight agrees.

Four decisions a port loses easily, all now observable:

- **Move offers EVERY reachable square** (254 on an open pitch), weight-ordered — not a top-K.
  Pruning by arrival probability was measured once and was catastrophic: 1.76 touchdowns per game
  down to 0.19. `p_arrive` is an admissible BOUND but not an admissible RANKING — a one-square
  shuffle arrives with p = 1.0 and a six-square scoring run with p ≈ 0.3, so a top-K is almost
  exactly the set of moves that go nowhere.
- **The loose-ball square is `Pickup`, not `Move`** — picking it up changes the value model, so the
  activation may legitimately continue.
- **Blitz stops at adjacency**: non-adjacent victims are SKIPPED, not scored low, so the candidate
  COUNT is the observable difference (1, not 2, on the fixture board).
- **An empty reachable set still emits one candidate** at 0.02, so a player who cannot move does not
  vanish from the declaration list.

Bite-check: pruning Move to a top-12 fails at `candidate COUNT (move_open) ==> expected: <254> but
was: <12>`.

### `w_player` crosses the boundary as a parameter

The first run failed on a Move weight, and the cause was the fixture rather than the port: the
emitter passes `w_player = 1.0` to `build_plans` explicitly, while the test recomputed it from the
tier-1 ladder and got 0.30. `build_plans` takes it as a PARAMETER, so it now crosses as one —
recomputing it pins the ladder a second time (it already has its own fixture) and would silently
test a different call if the emitter ever passed something else.

The same reasoning that put the `Features` snapshot, the foul assists and the pass face-counts on
the golden rather than in the test: **whatever the function takes as an argument should arrive as an
argument.**

### Gates

- `mvn -o -pl ffb-ai test`: **26 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax.

**Next:** the HandOff and Pass enumerations (they need the harness's receiver eligibility, so the
live gate is the natural place), then wiring `handle_activate` into `ParityRunner`'s activation loop
and switching `--heur-classes activate` on.

## ITER36 — the HandOff and Pass enumerations, and a fixture that graded its own homework

The last two enumeration branches. Both have the same shape — the carrier moves FIRST and gives or
throws at the end — so every square he can reach next to a team-mate is a candidate, not just the
mates he already touches. HandOff keeps the best `GIVE_SPOTS` (2) per receiver; Pass enumerates
receivers × run-up squares.

Two things had to change in the GOLDEN before any of it was observable:

- HandOff and Pass candidates carry a **path**, not a `dest` — the run-up IS the plan. Without
  emitting the path, every give candidate looked identical: same receiver, no dest, and a weight
  that floors to 0.0 because the raw give price is negative and the coverage floor is 0.
- `w_player` and `novelty` are `build_plans` PARAMETERS and now cross as parameters (ITER35).

### The fixture graded its own homework

The first version of the give test built its `weightFrom` callback **out of the golden's own rows**:
it returned a weight only for squares the golden already listed. So the `GIVE_SPOTS` cap could never
bind, and perturbing it from 2 to 3 changed nothing — the callback simply had nothing else to offer.

Rewritten to price gives with the real `BallMoves.handoffWeight`, which has its own fixture from
ITER30. Now the cap binds and the perturbation fails:
`HandOff candidate COUNT (give_and_pass) ==> expected: <4> but was: <6>`.

Worth naming, because it is a distinct failure mode from the previous three: not a branch the
fixture could not reach, but a fixture **wired to its own expected output**. It passes for the same
reason a mirror agrees with you. The tell was that the bite-check did not bite; the fix was to feed
it an independent computation.

### Gates

- `mvn -o -pl ffb-ai test`: **26 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax.

**The whole agent is now ported and pinned.** Rasters, reach, value, arrival, orderings, proxy,
give/foul/pass prices, the tier-1 ladder, the enumeration and the two-level draw all have
cross-language fixtures, and every one of them has been perturbed on purpose to confirm it fails.

**Next:** wire `handle_activate` into `ParityRunner`'s activation loop and switch
`--heur-classes activate` on with `move` still OFF. Both sides then move by the byte-matched random
contract, so what that gate tests is exactly the activation choice — the ladder discipline applied
one last time.

## ITER37 — `handle_activate` end to end

The composition. Every layer already had its own golden; what none of them could catch is a bug in
the WIRING between two of them — each can be perfectly right while the order they run in, or what
one hands the next, is wrong. `ActivationChoice.choose` + `ActivationChoiceTest` pin the whole
decision: same entry point, same return value, no engine underneath. Five boards × three scales.

The `move_vs_block` board is the one that shows the draw doing work: at argmax it declares
`Block away_01`, at 1.0 and 1e6 it declares `Move`. The test asserts that at least one case CHANGES
its answer across scales — a set of boards where argmax and the sampled pick always agree would
exercise the two-level draw without ever testing it.

### The composition rule the bite-check exposed

Rust sorts the candidates CANONICALLY, ranks into a **separate index vector**, and then iterates the
**canonical** list — consulting the rank only to decide whether a reach search runs. The ranking
picks who gets a SEARCH; it does not reorder the candidate list. Iterating in ranked order instead
produces the same candidates in a different sequence, and the declaration grouping is positional.

The bite-check did not fail on the first four boards: their rankings happened to coincide with
canonical order, so the distinction was invisible. Added
`ranking_differs_from_canonical`, where home_02 carries the ball and therefore outranks home_01 —
and the perturbation now fails at
`player (ranking_differs_from_canonical @ scale 1.0) ==> expected: <home_01> but was: <home_02>`.

Note it fails at scale 1.0 and NOT at argmax: reordering the list cannot change a max, only which
option a sample lands on. A fixture that only ran at argmax would have missed it entirely.

### Gates

- `mvn -o -pl ffb-ai test`: **27 tests, 0 failures**.
- `cargo test --workspace --release`: **14,654 / 0**.
- Fourteen-class rung still **100/100 in all three editions** at argmax.

**Next:** `handle_move` — the plan replay, with the engine-guard conditions that gate each terminal
action — and then the harness wiring. `activate` and `move` have to go live together: the plan is
set by one and consumed by the other, and the random contract's per-activation state (its
`moved_this_activation`, its pre-drawn move target) is only refreshed when IT sees the activation
prompt, so splitting them would leave that state stale on one side.

## ITER38 — `handle_move`: the plan replay, extracted

The move handler is a state machine with **seven exits and four engine guards**, and in Rust it was
control flow tangled with the mutations it drives (`pl.fired = true; self.plan = Some(pl); return
...`). That shape cannot be pinned against a Java twin, because there is no way to call it with
made-up inputs.

Extracted as `replay_plan(kind, is_mine, path_empty, delivered, fired, &ReplayFacts) -> Replay`,
a pure function of the board facts the caller gathers. `handle_move` now gathers the facts, asks it,
and turns the verdict into an `Action`.

The rules it encodes, in order:

- an EMPTY square list means no MOVEMENT is left, **not** that there is nothing to do — a pending
  give, throw, blitz or foul still has to be sent. Bailing here threw away every give whose run-up
  spent the carrier's whole move, which is most of the good ones.
- a path is delivered only when the offered squares contain its next step; if the board moved under
  the plan, re-decide rather than insist.
- every terminal action is gated on the engine's OWN condition (`BlitzMove` + not having blocked +
  adjacent; `FoulMove` + not having fouled; the pass/give action set + the target still on the
  pitch) and latched with `fired`, because `StepInitMoving` re-emits this prompt when its guard
  fails and a resend would spin forever.
- a delivered plain move ends — moving twice reaches the same square.
- once fired, only a **blitz** has anything legitimate left (its post-block movement), and a
  **pickup** re-decides because it genuinely changed the value model.

One asymmetry is preserved deliberately and called out in the comment: `terminal_pending` is read
off the plan WITHOUT checking whose plan it is, so a pending give belonging to another player still
suppresses the early exit. Tightening that to "this player's plan" is the obvious cleanup and would
change behaviour.

### Verifying a refactor the gate cannot see

`handle_move` is not exercised by the current mask — `move` is the one class still off — so neither
the parity gate nor the workspace tests would have caught a mistake here. Verified behaviourally
instead: ten full games at `--heur-classes all --heur-scale 1.0`, capturing **Rust's own**
end-of-game hashes before and after the extraction. Identical on all ten. Plus
`plan_replay_state_machine`, which walks every exit and all four guards explicitly.

### Gates

- `cargo test --workspace --release`: **14,655 / 0**.
- Fourteen-class rung **100/100 in all three editions** at argmax; `--agent random` 100/100 in all
  three.

**Next:** port `replay_plan` to Java against a table-driven golden, then wire both `activate` and
`move` into `ParityRunner` together and run the first live gate.

## ITER39 — `replay_plan` in Java, pinned EXHAUSTIVELY

Every other golden in this campaign samples: a handful of boards chosen to reach the branches that
matter, with a deliberate perturbation to confirm they do. **Three times that check found the sample
was not adequate** — ITER27 (a `strength_factor` branch no board reached), ITER33 (two rung pairs,
one of them unobservable in principle), ITER36 (a fixture wired to its own expected output).

This one does not sample. `replay_plan`'s input space is ten booleans × seven plan kinds × seven
relevant player actions, so all **50,176** combinations fit in one 55 KB file — 49 rows of 1024
verdicts, one character each — and `MoveReplayTest` walks every one. There is no branch it can miss
and no board chosen badly.

Worth doing here specifically: the state machine has seven exits and nothing about its shape
suggests which combinations are interesting, so any sample would have been a guess. Where exhaustive
enumeration is affordable, it removes the question rather than answering it.

All 50,176 matched on the first run.

### The bite-check, on the case the comment warned about

Perturbing the preserved asymmetry — gating `terminalPending` on "is this MY plan", the obvious
cleanup that ITER38's comment explicitly flagged — fails immediately, and names the input in full:

```
Blitz/None: isMine=false pathEmpty=false delivered=false fired=false blocked=false
  fouled=false adjacent=false onPitch=false includesNext=false squaresEmpty=true
  ==> expected: <R> but was: <E>
```

A sampled fixture would have had to guess that `isMine=false` with a blitz plan and an empty square
list was worth a board.

### Gates

- `mvn -o -pl ffb-ai test`: **28 tests, 0 failures**.
- `cargo test --workspace --release`: **14,655 / 0**.
- Fourteen-class rung **100/100 in all three editions** at argmax.

**Next:** the wiring. `ParityRunner` gets the activation branch and the move-replay branch together
— they are one unit, since the plan is set by one and consumed by the other — and then
`--heur-classes activate,move` goes live for the first real gate.

## ITER40 — `ActivationDriver`: the live half, wired to a real Game

The state the decisions are threaded through, and the adapters that answer eligibility questions
from an actual `Game` rather than a fixture snapshot. Everything it DECIDES is computed by classes
that already have cross-language fixtures; what is new is the plumbing between them and the engine.

- `chooseActivation` builds `Features` from the game, assembles the eligible list, and delegates to
  the already-pinned `ActivationChoice.choose`. It then records `usedThisTurn` and, for a ball move,
  `awaitingRun` — the receiver who must be activated next, or the throw bought nothing.
- `recordPlan` re-runs the reach search to turn the chosen candidate's destination into a PATH,
  which is exactly what Rust's `handle_activate` tail does rather than carrying the path through the
  draw.
- `replayMove` gathers the seven board facts and asks the exhaustively-pinned `MoveReplay.decide`.
- `GameBoard` answers block/blitz/foul targets and receivers, calling the ENGINE's own
  `ServerUtilPlayer.findBlockStrength` and the edition's `PassMechanic` rather than re-deriving
  either.

### Three API mismatches, all caught by the compiler

`PlayerState` has no `isProne()` — only `isProneOrStunned()` and `isStunned()`, so the prone
predicate is the difference of the two. `os.isProne() || os.isStunned()` for foul targets collapses
to `isProneOrStunned()`. And the pass mechanic is reached through
`game.getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.PASS.name())`, the same lookup
the engine's own skill behaviours use, so the EDITION's table is what answers.

Worth noting these are the kind of mistake that is cheap precisely because it is a compile error.
The expensive ones in this campaign have all been the opposite: code that compiled, ran, and
quietly meant something else.

### Gates

- `mvn -o -pl ffb-ai test`: **28 tests, 0 failures**.
- `cargo test --workspace --release`: **14,655 / 0**.
- Fourteen-class rung **100/100 in all three editions** at argmax; `--agent random` 100/100 in all
  three. Nothing live changed — `ActivationDriver` has no caller yet.

**Next:** the two call sites in `ParityRunner` — the activation loop and `sendMoveAction` — and then
`--heur-classes ...,activate,move` for the first live gate. Both switch on together: the plan is
created by one and consumed by the other, and the harness's prone-move RNG choreography is split
across both, so separating them would desynchronise the stream by construction.

## ITER41 — the wiring, and the first live gate

Both call sites are in: the activation loop picks player AND action from the heuristic in one
decision (replacing both RNG picks wholesale, since the two-level draw groups by declaration), and
`sendMoveAction` replays the plan. A guard rejects `activate` without `move` at startup — the plan is
created by one and consumed by the other, so switching them on separately is a configuration error,
not a supported rung.

First live run: **0/20**, and seed 1 diverged at **step 0**. Four real defects, each found by asking
for data rather than reading the code again:

1. **`ActivationChoice.choose` hardcoded `home = true`.** Every AWAY decision was computed as though
   it attacked the wrong endzone — wrong raster side, wrong `endzone_distance`, wrong everything
   downstream.
2. **Movers were built with default 6/3/3** instead of the player's own MA/AG/ST and skills. The
   `Eligible` record simply did not carry them.
3. **`Features.build(Game)` delegated to the fixture overload** — empty `BoardState` and
   `heavy = false`, so the live agent scored every square on a board with **no ball and no threat,
   lane or support rasters**. This is the one worth remembering: the fixture overload exists because
   fixtures do not have a `Game`, and routing a live game through it fails silently and plausibly.
   The agent preferred a block over a run because no run had any value to compute.
4. **The bb2025 `findPassingDistance` override NPEs at activation time.** It dereferences
   `getActingPlayer().getPlayerAction()` to ask whether the throw is a bomb, and the agent scores
   passes BEFORE declaring an action. More importantly, **Rust does not implement that override at
   all** — its `find_passing_distance` is the base method — so the base is what the two agents must
   agree on. The Java agent now reads the engine's own `PASSING_DISTANCES_TABLE` reflectively rather
   than transcribing a copy, and the skipped refinement (PASS_TO_PARTNER, which needs a partnered
   pair) is recorded as a pre-existing Rust port gap rather than papered over.

After the four: seed 1 matches for **9 steps** before diverging, and the first activation of the
game agrees exactly (`away_04/Move` on both sides). That is the only form "the target improves" can
take for a class whose first measurement is 0.

### Gates

- `--heur-classes all` bb2025 argmax: **0/20**, seed 1 to step 9 (was step 0). The frontier.
- Fourteen-class rung: **100/100 in all three editions** at argmax — unchanged.
- `--agent random`: **100/100** in all three editions.
- `cargo test --workspace --release`: 14,655 / 0. `mvn -o -pl ffb-ai test`: 28 / 0.

**Next:** seed 1 step 9 — the first divergence that is not a wiring mistake.

## ITER42 — the plan was never recorded, and block targets sort by COORDINATE

Three defects, all on the Java side, all found by comparing what the two agents actually did rather
than by re-reading the port.

**1. The plan was never recorded.** `chooseActivation` returned a decision and threw away what the
decision was FOR, so `activation.plan()` was always null and `replayMove` re-planned from scratch on
every movement prompt. `ActivationChoice.Decision` now carries the `Kind` and the destination, and
the driver calls `recordPlan`.

**2. Phase 2 re-picked the block target at random.** The heuristic chose a victim when it declared
the block; `sendBlockAction` then ignored it, ran `pickBlockTarget`, and spent an `actionRng` draw
the Rust side does not spend. Same declaration on both sides, different defender.

**3. Block targets sort by COORDINATE; blitz foes sort CANONICALLY.** This is the instructive one.
Rust's `legal_block_targets` and `legal_foul_targets` both sort by `(x, y)` — the comment says
"Java `pickBlockTarget` sorts by (x, y) before picking — match that order" — while `build_plans`'
blitz branch sorts its foes with `canon_key`. The two orderings are deliberately different, and
sorting both canonically swapped the victims:

```
seed 1, identical board:  Java away_01 blocks home_01
                          Rust away_01 blocks home_02
```

A single sort comparator, in a helper that looked like it should obviously be canonical because
every OTHER ordering in this campaign is. The rule that has held all campaign — never order by
player id — made the wrong answer look like the right one here, because coordinate order is neither
id order nor canonical order.

### Where the frontier is

`--heur-classes all`, bb2025 argmax: still **0/20**, but seed 1 now diverges at **step 12** (was 9,
was 0 before ITER41). Six defects in two iterations, and each one moves the frontier a few steps
further. The failures are localised and the diagnosis loop is short — a probe on each side, one
comparison, one fix.

### Gates

- Fourteen-class rung: **100/100 in all three editions** at argmax.
- `--agent random`: **100/100** in all three editions.
- `cargo test --workspace --release`: 14,655 / 0. `mvn -o -pl ffb-ai test`: 28 / 0.

**Next:** seed 1 step 12.

## ITER43 — the blitz: a victim already chosen, and a move still owed

Two defects, both about a blitz being a TWO-part plan that the harness was treating as one.

**1. The blitz victim was re-scored instead of replayed.** Rust's `BlitzTarget` arm consults the
plan FIRST and returns its victim without sampling — the activation already decided who to hit.
Java went on calling the ITER18 scored path, which risks a different victim and, worse, spends a
sampler draw the Rust side does not spend. A draw-count difference desynchronises everything
downstream, not just the choice it belongs to. Now replayed from the plan, with Rust's
still-on-the-pitch check preserved so a vanished victim still falls through to the scored
enumeration.

**2. The blitzer was denied his remaining movement.** The harness blocked and then sent `CONFIRM`,
ending the activation. But a blitz is the ONE plan with something legitimate left after its terminal
action — `replay_plan` returns `Replan` for a fired blitz precisely so the blitzer spends the rest of
his move — and Rust's event stream shows exactly that: `block away_01`, pushback, then
`playerMoved home_01 → (11,6)`. Confirming there ends the activation a move early.

Both are the same shape of mistake: the harness's blitz path was written for an agent that had no
plan, so it re-derived what the plan already knew and stopped where a planless agent would.

### Where the frontier is

The first differing STATE moved from `i = 10` to **`i = 13`**. The blitzer now blocks the right
victim and does move afterwards — he just lands on (11,7) where Rust lands on (11,6), which is the
next thing to chase (a follow-up versus a post-block move). The reported failing STEP is still 12,
because a step is only logged at an activation boundary while the state diverges inside one.

Worth noting for the next iteration: the step number and the state index move at different
granularities, and the state index is the finer signal.

### Gates

- Fourteen-class rung: **100/100 in all three editions** at argmax.
- `--agent random`: **100/100** in all three editions.
- `mvn -o -pl ffb-ai test`: 28 / 0.

**Next:** home_01 landing on (11,7) rather than (11,6) after the blitz.

## ITER44 — `spent`, and why the blitzer still does not move

**The fix.** Rust's `budget_of` sets `spent` from the ACTING player's `current_move`; the Java
driver passed 0. So every re-plan handed the mover a full fresh allowance — most visibly a blitzer
who had already stood up and blocked, whose Java reach was longer than his Rust one. Both call sites
(`recordPlan` and `replan`) now use `spentBy(game, playerId)`, which returns 0 for anyone who is not
the acting player, since only he has spent anything.

**What it did not fix, and why that is the real finding.** The blitzer still ends on (11,7) where
Rust puts him on (11,6) — and the reason is not scoring at all. Instrumenting showed `replan` is
**never called**, because the harness's phase-2 blitz arm is entered exactly ONCE per blitz:

```
JAVA_BZARM pid=Home1 sent=false tss=Away1     <- block sent here
(no second entry)
```

After the block the harness never revisits phase 2 for that player, so `sendMoveAction` — and with
it the whole replay path — is unreachable. ITER43's change (call `sendMoveAction` instead of
`CONFIRM` on the second entry) is correct and is simply never reached.

The player in question is PRONE before the blitz, so this is a stand-up blitz: he stands, blocks,
and in Rust then spends what movement is left. The harness's loop was built for an agent that had
nothing left to do after a block, and it ends the activation structurally rather than by choice.

That makes the next step a control-flow change in the harness rather than a scoring one — a
different kind of fix from the last three, and worth naming as such rather than continuing to look
for a weight that disagrees.

### Gates

- Fourteen-class rung: **100/100 in all three editions** at argmax.
- `--agent random`: **100/100** in all three editions.
- `mvn -o -pl ffb-ai test`: 28 / 0. Frontier unchanged at state `i = 13`.

**Next:** make the harness revisit phase 2 after a blitz block, so the blitzer's remaining movement
is reachable at all.

## ITER45 — the blitzer's second look, and fouls that were worth nothing

Two harness defects, found in sequence: the first made a whole engine prompt unreachable, and the
second was only visible once it was.

### The `INIT_MOVING` deselect

ITER44 concluded that `sendMoveAction` — and with it `ActivationDriver.replan` and the entire
`MoveReplay` state machine — was never called, because the blitzer never came back through phase 2
of `INIT_SELECTING`. That was true, and it was the wrong entry point to be watching. A probe on
*every* `handleStep` shows the engine does offer the blitzer his post-block movement; it just
arrives as its own step:

```
INIT_SELECTING tm=REGULAR ap=Home1/BLITZ_MOVE/blocked=false/move=3   <- phase 2 sends the block
PUSHBACK       tm=REGULAR ap=Home1/BLITZ/blocked=true/move=4
INIT_MOVING    tm=REGULAR ap=Home1/BLITZ_MOVE/blocked=true/move=4    <- the second look
```

`case INIT_MOVING` injected an unconditional deselect. That is the correct contract for the random
agent — it is *why* random play has a hard ceiling of one square per activation — but it discards
every plan that outlives the engine's first move prompt: post-block blitz movement, a give whose
run-up has not finished, a re-plan after the board moved. Rust's `StepInitMoving` re-emits the move
prompt and lets `replay_plan` answer; the harness now does the same when the heuristic owns the
activation. `replayMove` still returns `END_PLAYER_ACTION` for a plain delivered move, so the random
behaviour is what a spent plan reduces to rather than a special case.

Seed 1's first divergence moved from **step 12 to step 78** on this change alone.

### Fouls scored at a hardcoded zero

Step 78 was then a clean, isolated signal: same pre-state, same player picked, different action —
Java declared `MOVE`, Rust declared `Foul`. Dumping both candidate lists at that decision and
diffing them gave **exactly 6 differences out of 1,378 candidates**, every one a foul against the
same victim, Java `0.000000` against Rust `0.072016`.

`ActivationDriver.foes` read:

```java
float w = fouls ? 0.0f : blockWeight(playerId, o.getId(), attStr);
```

`BallMoves.foulWeight` had been ported faithfully and pinned by golden vectors the whole time. Only
the *call* was missing, so no fixture could see it: the arithmetic was tested in isolation and never
reached in the game. Every foul candidate tied at `wPlayer * 0` and lost to any move, so the Java
agent never fouled while Rust weighed the armour break against the ejection risk and regularly did.
It is not a difference the state hash can show directly — it surfaces one step later, as a victim
who is Prone on one side and KO'd on the other.

Wired it to `BallMoves.foulWeight` with the four board facts it reads: the victim's armour, the
engine's own `UtilPlayer.findOffensiveFoulAssists`/`findDefensiveFoulAssists`, and the team's
remaining Bribe the Ref inducements (Rust keeps that as a plain count on the team; the engine keeps
it in the turn's inducement set, so it is read back out by type name).

**The lesson, and it is the same one three times now:** a golden fixture proves the arithmetic, not
that anything calls it. Both defects this iteration were *unreached correct code* — `MoveReplay` and
`foulWeight` were each fully ported, fully tested, and dead. The check that finds this class of bug
is a diff of the two agents' candidate lists at a decision they disagree on, which localised the
foul weight to six rows out of 1,378 in one run.

### Regression test

`BallMovesTest.foulWeightIsNeverZero` walks the plausible input range — every armour value, both
assist directions, all three ball-proximity tiers, with and without bribes — and asserts the weight
is never zero, pinning the fact that made the placeholder wrong: `0.0f` is not a value this model
can produce, so any caller reporting one is not calling it. It is an indirect guard; the direct one
would need a live `Game`, which the `ffb-ai` test tree has no scaffolding for. The parity sweep is
what actually proves the wiring.

### Gates

- `--heur-classes all` bb2025 argmax: still **0/20**, but seed 1 now diverges at **step 139**
  (was 12). Seeds 2 and 3 at steps 12 and 72.
- Fourteen-class rung (`coin,receive,reroll,pushback,blocktarget,blockchoice,blitztarget,touchback,
  kick,intercept,followup,setup,other,skill`): **100/100 in bb2016, bb2020 and bb2025** at argmax.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,655 / 0**. `mvn -o -pl ffb-ai test`: **29 / 0**.
- `python scripts/check_java_trees.py`: the two Java trees agree.

**Next:** seed 1 step 110 is already root-caused and is the following iteration's fix. Rust offers
a foul candidate (`home_03` on `away_01`) that Java does not offer at all — Java has *no* foul
candidate there. The eligible-action list the harness hands the agent is a **turn-start snapshot**,
so a victim who went down *during* the acting team's own turn never appears in it, while Rust
recomputes legality live. The snapshot is a harness artifact, not engine behaviour: the Java engine
would allow that foul. Refresh the action list live inside `eligibleFor` only — `filterStaleActions`
must keep its snapshot semantics on the random path, where `idx % N` alignment depends on it.

## ITER46 — the eligible list was a turn-start photograph

The divergence ITER45 left root-caused, fixed. At seed 1 step 110 Rust offered a foul
(`home_03` on `away_01`) that Java did not offer **at all** — Java had no foul candidate there, so
this was never a scoring difference.

The harness computes `eligibleThisTurn = computeEligiblePlayers(game)` **once, when the turn
starts**, and every activation that turn is picked from that list. Rust reads `eligible_players`
straight off the engine's `ActivatePlayer` prompt, which the engine recomputes for every activation.
The two agree until the board changes *during* the acting team's own turn — and knocking an opponent
down is precisely that. After a home block puts `away_01` on the ground, FOUL becomes legal for all
his neighbours; the engine says so, Rust sees it, and Java is still reading a photograph taken
before the block.

The snapshot is not wrong for the random path — it is load-bearing there. That agent picks with
`idx % N` against a shared stream, so the list must not change size mid-turn or the two sides
desynchronise. The heuristic consumes no RNG at this point, so it can afford the truth. The live
recompute is therefore scoped to the `activation != null` branch, and `filterStaleActions` keeps its
snapshot semantics everywhere else.

`computeEligiblePlayers` is a pure function of the current game state, so calling it again is
exactly what the engine would report.

### On the missing unit test

This fix has none, and it is worth saying why rather than inventing a vacuous one. The property
being fixed — *the activation loop reads the list live instead of reusing a field* — is control flow
in a 60-line loop, not the return value of any function. `computeEligiblePlayers` itself was already
correct and unchanged. Extracting the action menu into a pure, testable function is the right shape
(it is how `Reach`, `BallMoves` and `MoveReplay` became testable in this campaign), but that menu is
~120 lines of skill-property lookups feeding the byte-matched random contract, and refactoring it to
buy a test for a fix elsewhere is a poor trade inside one iteration. The `ffb-ai` test tree also has
no live-`Game` scaffolding; `HeadlessGameSetup.create` could provide it and is the obvious thing to
build when a fix actually needs it.

What proves this fix is the sweep: **seed 1's first divergence moved from step 139 to step 185**,
crossing into the second half.

### Gates

- `--heur-classes all` bb2025 argmax: **0/20**; seed 1 at **step 185** (was 139), seeds 2 and 3
  unchanged at steps 12 and 72.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,655 / 0** (no Rust change). `mvn -o -pl ffb-ai test`:
  **29 / 0**. The two Java trees agree.

**Next:** seed 2 step 12 is now the shallowest frontier and the cheapest to trace — Java declares
`Activate(Home9, MOVE)` against Rust's `Activate(home_09, Move)`, same player and same action, so
the pre-state hashes already differ and the real divergence is earlier in that game.

## ITER47 — the heuristic was activating players the engine refuses to move

Seed 2 step 12: both agents activate the same prone player with the same action; Java leaves him
prone at (11,7), Rust stands him up and moves three squares, and the dice counts are IDENTICAL. No
roll was involved, so nothing in the dice stream could point at it.

The state string's last per-player field is the **ACTIVE bit**, and that player's was `0`. Java's
`StepInitSelecting` guards its entire `CLIENT_ACTING_PLAYER` branch on `playerState.isActive()` —
for anyone else the command is ignored outright, the acting player stays null, and the activation is
a silent no-op. Rust's `legal_activate_player_actions` never looks at the bit and its engine
executes the activation.

Both random agents already honour this: the Java harness picks such a player, burns its
`decisionRng` call and re-picks (`SKIP_INACTIVE`, `AGENT_CONTRACT.md` §2.4), and Rust's
`RandomAgent` mirrors it exactly. That contract is what has been hiding the engine gap. The
heuristic replaced the whole pick loop and inherited none of it, so it activated players Java would
not move — a just-unstunned player, or a team-mate thrown this turn who lands STANDING but inactive.

Fixed on both sides at the same place, by giving the heuristic the contract the random agent has
always had: `handle_activate` and `handle_activate_deep` skip a player whose ACTIVE bit is clear,
and `ParityRunner.eligibleFor` drops him from the list it hands the scorer. Neither engine's
eligible list changes, which matters — the random path's `idx % N` alignment depends on inactive
players still being IN it and being rejected at pick time.

**The Rust engine gap is real and is now unreachable rather than fixed.** Adding the `isActive()`
guard to `legal_activate_player_actions` would shorten Rust's eligible list and break the random
contract that depends on its length. Logged for the backlog rather than done here.

### Two wrong turns worth recording

**A correct fix can measure worse.** Declaring `PlayerAction.STAND_UP` for prone players (Rust names
them `StandUp`) dropped seed 1 from 185 to 22 and seed 3 from 72 to 20. It was not wrong about the
naming — it was wrong about the semantics: `STAND_UP` is *stand up and end the activation*, so the
player rose and stopped. Reverted.

**stdout and stderr do not interleave.** `JSTEP` is printed on **`System.err`**
(`ParityRunner.java:1723`). A probe on `System.out` therefore lands in a different stream, and
through a pipe the whole probe output can appear after the whole step log. That ordering artifact
produced a confident, wrong conclusion — "`sendMoveAction` is never called for this player" — twice.
A probe that has to be read *in sequence with* the step log must print to `System.err`, and should
carry `stepIndex` anyway: with the index the two can be aligned no matter how they buffer, and
without it they cannot be aligned at all. (The candidate-list dumps that found the ITER45 foul bug
were on `System.err` and were correctly paired; it was the `System.out` probe that lied.)

### Gates

- `--heur-classes all` bb2025 argmax: **0/20**, but every seed deeper — seed 1 **185 → 216**,
  seed 2 **12 → 49**, seed 3 **72 → 188**.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,656 / 0** (+1: `heuristic_never_activates_an_inactive_player`,
  bite-checked — it fails when the guard is removed). `mvn -o -pl ffb-ai test`: **29 / 0**.
- The two Java trees agree.

**Next:** seed 2 step 49 is the shallowest frontier.

## ITER48 — the give was declared as a plain move, for the whole campaign

**First green seeds on `--heur-classes all`: 2/20.**

Seed 2 step 49: Java declared `Activate(Home5, MOVE)`, Rust `Activate(home_05, HandOffMove)`.
Dumping both candidate lists showed they were **identical** — 2,171 candidates, same order, same
weights, and both argmax landed on the *same* entry: index 849, `h04` give, `w=0.592533`.

So the agents agreed completely and the harness threw the answer away afterwards. Two vocabularies
disagree about the name of a give:

| | name |
|---|---|
| Rust's enumeration | `HandOff` |
| the harness's action names (`nameForAgent(PlayerAction.HAND_OVER)`) | `HandOver` |

`ActivationChoice.moveVariant` matched only `"HandOff"`, so `"HandOver"` — which is what actually
reaches it — passed through unchanged; `ParityRunner.actionFromName` has no case for it, and its
`default:` arm returns `PlayerAction.MOVE`. Every give the Java agent ever chose was declared as a
plain move. `Pass` was unaffected: both sides spell it the same, so `moveVariant` mapped it.

Fixed by making `moveVariant` accept both spellings. Guarded by
`ActivationChoiceTest.moveVariantMapsBothSpellingsOfEveryBallAction`, which also pins that the
no-movement-phase actions pass through unchanged.

**Why nothing earlier could find this.** The bug is downstream of every fixture in the campaign.
The goldens test scoring; the scoring was right. The state hash sees the consequence — a ball that
did not move — several steps later. Only a diff of the two agents' *chosen candidate* against their
*declared action* separates "picked differently" from "picked identically and declared differently",
and that distinction is what took ITER45–48 four iterations to learn to make.

### Gates

- `--heur-classes all` bb2025 argmax: **2/20** (was 0/20) — seeds 7 and 8 fully green. Seed 1 at
  216, seed 3 at 188, seed 6 **69 → 185**. Seed 2 still at 49: a second, independent divergence at
  the same step.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,656 / 0**. `mvn -o -pl ffb-ai test`: **30 / 0**.
- The two Java trees agree.

**Next:** seed 2 step 49's remaining divergence, then seed 10 at 42.

## ITER49 — the ball actions were dead on the Java side, in two places at once

**bb2025 argmax 87/100, bb2020 69/100** (from 2/20 and never measured at 100).

ITER48 fixed the give's *declaration*; this iteration found that even once declared correctly, a
give or pass could not complete. Two independent defects, both on the path between the plan and the
engine, and the second only became visible once the first was fixed.

### 1. The MOVE variant threw away the movement phase it exists to buy

`sendConcreteAction` routed `HAND_OVER_MOVE` to `sendHandOverAction` and `PASS_MOVE` to
`sendPassAction` — the immediate handlers. The whole point of declaring the MOVE variant is to get
a movement phase *before* the give or throw, which is what makes carrier-move-then-give possible in
one turn. The heuristic planned a run-up and the harness gave the ball from wherever the carrier
already stood.

Seed 2 step 49 made this unarguable: Java's recorded plan was byte-identical to Rust's — same
receiver, same destination index 195 = (13,7), same path `[(12,5), (13,6), (13,7)]` from (11,4) —
and Rust walked it while Java gave from (11,4) without moving.

The `*_MOVE` arms now go through `sendMoveAction` when the heuristic owns the activation; the
immediate forms `HAND_OVER` / `PASS`, which is what the random contract declares, still reach the
old handlers untouched.

### 2. `FIRE_TERMINAL` was unreachable for every terminal

With the run-up delivered, the give still never fired. `ActivationDriver` built its `MoveReplay`
facts with

```java
ap.getPlayerAction().name()      // "HAND_OVER_MOVE" -- the Java ENUM name
```

against a table written in Rust's `PlayerActionChoice` spelling (`"HandOverMove"`, `"PassMove"`,
`"BlitzMove"`, `"FoulMove"`). **No terminal has ever matched.** `dispatchable` was permanently
false, so `FIRE_TERMINAL` could not be returned and every give and pass the heuristic planned died
at the end of its run-up with the ball still in the carrier's hands. The blitz and the foul survived
only because they fire through their own arms rather than through the replay.

Added `ActivationDriver.pacName`, the engine's `PlayerAction` under the vocabulary `MoveReplay`
actually compares against, returning null for an action with no Rust spelling.

**Both defects are string mismatches between two vocabularies that no test compared.** The
`MoveReplayTest` golden walks all 50,176 input combinations of `decide` — and passes — because it
supplies the names itself. A fixture that generates its own inputs cannot discover that the
production caller supplies different ones. The new
`MoveReplayTest.everyTerminalGateNameReachesDecide` closes exactly that gap: it takes each real
`PlayerAction`, maps it the way production does, and drives `decide` with the result, asserting the
terminal can fire.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2025 87/100, bb2020 69/100, bb2016 2/100**.
  20-seed bb2025 went 2/20 → 18/20.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,656 / 0**. `mvn -o -pl ffb-ai test`: **31 / 0**.
- The two Java trees agree.

**Next:** bb2016 is now the outlier at 2/100 and is where the remaining structural work is — it
routes through different generators and step twins, and its blitz is the 3-command form. bb2025's
13 reds and bb2020's 31 are the other two fronts; seeds 14 and 20 are the shallowest bb2025 reds.

## ITER50 — bb2016's blitz re-picked its own victim

bb2016 seed 1 step 21: both sides activate `home_01` with a blitz, and one step later Java has ended
the whole team turn (t21 home → t22 away) while Rust is still in it with `blitz_used` set — and Rust
has spent one more die.

bb2016's blitz is the **3-command form** (`CLIENT_BLITZ_MOVE` with an empty path, then
`CLIENT_BLOCK`), so it goes through `sendConcreteAction`'s bb2016 arm rather than
`SELECT_BLITZ_TARGET`. That arm called `pickBlockTarget` — the *random* contract's picker, which
draws from `actionRng` — throwing away the victim the heuristic had already chosen when it declared
the blitz, and spending a die the Rust side does not spend.

`sendBlockAction` was given exactly this fix earlier in the campaign; bb2016's blitz was the one
remaining arm still re-picking. It now uses `heuristicTarget` the same way, and falls back to
`pickBlockTarget` only when the random agent is driving.

Like ITER46 this is a routing change with no pure-function seam to unit-test — the property is
"which of two branches the dispatcher takes", and `sendBlockAction`'s identical fix has no unit test
either for the same reason. The sweep is the evidence: seed 1 **21 → 185**, seed 5 **23 → 89**,
bb2016 overall **2/100 → 4/100**.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2016 4/100** (was 2), bb2020 69/100, bb2025 87/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,656 / 0** (no Rust change). `mvn -o -pl ffb-ai test`:
  **31 / 0**. The two Java trees agree.

**Next:** bb2016 seed 4 fails at **step 0**, which is a different and much earlier problem than the
rest of the bb2016 set — worth taking before the deeper ones.

## ITER51 — the heuristic never inherited the turn guards

bb2016 seed 4 diverged at **step 0** — its very first recorded step — with eight home players still
in the reserves box. The setup was not the problem: probing the prompt stream showed Rust's setup
placing exactly the formation Java placed. The problem was that the whole Rust game finished in
**5 ms**.

Bisecting the class mask pinned it precisely: the fourteen-class rung is green on this seed and
adding `activate` alone breaks it. The prompt stream then showed the run ending after 56 prompts
instead of 995, with the last few activations happening in `TurnMode::Blitz` at `turn_nr = 0` — the
**bb2016 Blitz! kickoff**.

`ParityRunner`'s INIT_SELECTING arm applies two rules ahead of the branch that reaches the agent at
all, so the Java side obeys them whichever agent is driving:

```java
if (turn < 1) { inject EndTurn; break; }                       // before the turn-key update
if (game.getTurnMode() != TurnMode.REGULAR && !usedThisTurn.isEmpty()) { ... EndTurn }
```

`RandomAgent` mirrors both, with the comments to prove it. The heuristic replaced the entire pick
loop and inherited neither, so during a Blitz! kickoff it kept activating through a turn Java had
already ended.

That is the same shape as ITER47's `SKIP_INACTIVE`: **a contract rule that lives in the harness
loop, not in the scorer, and that the heuristic bypassed by replacing the loop wholesale.** Three of
these have now been found. They are worth enumerating deliberately rather than one seed at a time —
anything `RandomAgent` does between "the engine asked" and "a player was picked" is a candidate.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2020 69 → 79/100**; bb2025 87/100, bb2016 4/100
  (seed 4 itself **step 0 → 97**).
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,657 / 0** (+1, both guards bite-checked separately).
  `mvn -o -pl ffb-ai test`: **31 / 0**. The two Java trees agree.

**Next:** enumerate the rest of `RandomAgent`'s between-prompt-and-pick contract and check the
heuristic against it as a set, rather than waiting for each rule to surface as a seed.

## ITER52 — the rest of the pick-loop contract, enumerated instead of discovered

ITER51 ended by saying these rules should be listed deliberately rather than found one seed at a
time. Doing that: reading `RandomAgent`'s activation loop against the heuristic's gives six contract
rules, and exactly one was still missing.

| # | rule | heuristic |
|---|---|---|
| 1 | `turn_nr < 1` → EndTurn, before the turn-key update | ITER51 |
| 2 | turn-key change clears the per-turn memory | had it (`refresh_turn`) |
| 3 | non-REGULAR mode allows ONE activation | ITER51 |
| 4 | `remaining.isEmpty() \|\| justDeselected` → clear and EndTurn | **missing** |
| 5 | `usedThisTurn.add` on every pick | had it |
| 6 | `filterStaleActions` | had it (`action_is_live`) |

Rule 4's `justDeselected` half was absent entirely. Java sets it when a non-REGULAR window ends a
turn, and the flag then ends the *following* turn as well — in Java that window's activation was the
original team's last processed one, so the turn it interrupted is over. `RandomAgent` carries the
flag with the same comment; the heuristic did not. The same exit also *clears* `usedThisTurn` rather
than waiting for the next turn key to do it, which the heuristic was also not doing.

**bb2020 79 → 87/100** on one flag.

The table is the point. Three of these rules were found by chasing individual seeds (ITER47, ITER51
twice); reading the loop once found the fourth in minutes. The general lesson for the rest of the
campaign: where the heuristic *replaced* a random-agent code path rather than extending it, diff the
two paths as a whole instead of waiting for divergences to surface one at a time.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2020 79 → 87/100**, **bb2016 4 → 5/100**,
  bb2025 87/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,657 / 0** (the guard test extended; the new assertion
  bite-checked). `mvn -o -pl ffb-ai test`: **31 / 0**. The two Java trees agree.

**Next:** bb2025 and bb2020 are both at 87/100 and bb2016 at 5/100. bb2016 is the outlier and its
remaining failures are likely structural (3-command blitz, different generators); the 13 bb2025
reds are the cleanest next target.

## ITER53 — a chain push named the wrong player, and the agent believed it

**The first Rust ENGINE bug of this stretch** — ITER45-52 were all harness and port defects.

bb2025 seed 25 step 35: both sides block with `away_04` and a chain-pushed player lands one square
apart, with identical dice. Probing both pushback choosers gave it immediately:

```
JAVA_PB att=away_04 def=away_10  sq=(21,12)(22,12)(23,12) pick=0   → (21,12)
RUST_PB att=away_04 def=home_07  sq=[(21,12),(22,12),(23,12)] pick=1 → (22,12)
```

The same three squares, and a different `defender_id`. Java names `away_10` — the team-mate
actually being shoved along the chain — and Rust names `home_07`, the block's *original* victim.
The agent's pushback weight multiplies by 1.3 when the square is further from the endzone the pushed
player is defending, so `def_home` flips the bonus: Rust's weights were `[0.2, 0.26, 0.26]` where
Java's made the first square the winner.

Java's `StepPushback` line 154:

```java
state.defender = fieldModel.getPlayer(defenderCoordinate);   // the OCCUPANT of the starting square
```

Rust read `game.defender_id`, which stays the original victim for the whole chain. The Rust comment
above the line already *said* `state.defender = fieldModel.getPlayer(defenderCoordinate)` — the
port had copied Java's comment and then not implemented it. bb2016's twin carries a step-local
`defender_id` with a comment describing this exact failure; bb2020 and bb2025 never got it.

The same id also reaches the StandFirm/Grab/SideStep hooks, which Java likewise passes
`state.defender`, so fixing it at the source rather than at the prompt is both simpler and more
faithful.

**Why the random gate could never catch this.** `AGENT_CONTRACT.md` §7 has the random agent take the
deterministic min-(x,y) pushback square — it never reads the defender at all. The prompt's
`defender_id` only becomes a decision input once an agent scores the squares, so this field could be
wrong indefinitely while 100/100 stayed green. It is exactly the class of bug the campaign exists to
find, and it took until the fifty-third iteration because everything upstream of it was broken first.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2025 87 → 89/100**, **bb2020 87 → 89/100**,
  bb2016 5/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025 — the gate that matters
  most here, since this is an engine change.
- `cargo test --workspace --release`: **14,658 / 0** (+1, bite-checked). `mvn -o -pl ffb-ai test`:
  **31 / 0**. The two Java trees agree.

**Next:** bb2025 and bb2020 have 11 reds each, bb2016 95. Worth checking whether the two editions'
reds are the same seeds — a shared cause would be worth more than either list.

## ITER54 — the two editions fail on the same seeds (investigation, no fix landed)

Answering ITER53's question, because it changes what to work on next. The red lists:

| | bb2020 | bb2025 |
|---|---|---|
| shared seed AND step | 14/178, 20/114, 30/177, 32/187, 35/114, 58/163, 72/72, 98/166 | same eight |
| edition-only | 62/65, 93/107, 97/140 | 62/83, 73/37, 87/134 |

**Eight of eleven reds are the same seed at the same step in both editions.** These are one small
set of shared causes, not twenty-two independent bugs, and each fix should be worth roughly double.

Started on the shallowest shared one, seed 72 step 72: a block whose crowd push badly hurts `h02` in
Java and `h05` in Rust, on identical dice. Diffing every pushback decision in the game shows all
sixteen *choices* agreeing (same squares, same pick index) while two of them name a different
defender — Java `h03` where Rust says `h02` — so ITER53's fix is right but incomplete: the
chain-aware defender is correct at the first chain level and wrong deeper in. The picks happen to
match there, so this is not yet proven to be the cause of the divergence; the next step is to trace
the push chain itself (defender, from-square, chosen square) rather than the agent's answers.

No code change; recorded so the next iteration starts from the measurement rather than repeating it.

## ITER55 — two copies of the block weighting, and only one knew about the crowd

bb2025 seed 72 step 72 (a shared red — the same seed and step fails in bb2020 too). Both sides
activate `away_03` with a block; a step later Java has `home_03` badly hurt and Rust has `home_06`,
on identical dice.

Walking it back through four probes: the injury applies to `game.defender_id`, which comes from the
block declaration, which comes from the agent's chosen target. Dumping the block candidates at that
decision:

```
JAVA_CAND away_03 Block tgt=home_03 w=0.092000    RUST_CAND away_03 Block tgt=home_03 w=0.092000
JAVA_CAND away_03 Block tgt=home_04 w=0.092000    RUST_CAND away_03 Block tgt=home_04 w=0.092000
JAVA_CAND away_03 Block tgt=home_06 w=0.092000    RUST_CAND away_03 Block tgt=home_06 w=0.138000
```

Same three victims, same order, one different weight — and 0.138 = 0.092 × 1.5. `home_06` stands on
the sideline at (11,14): it is the **crowd-surf bonus**, which Rust's `block_weight` has applied
since the agent was written and the Java port did not.

The reason it survived this long is that there are **two** copies of the block weighting.
`HeuristicDriver`'s scores the BLOCK-TARGET prompt and did apply the factor; `ActivationDriver`'s —
the copy that scores ACTIVATION candidates — did not. The two agents therefore agreed on every
ordinary block, and diverged only when a victim happened to stand where a push could send him off
the pitch. `canSurf` and `pushSquares` were already ported and correct; only this caller was
missing.

Both callers now go through one `HeuristicDriver.surfMultiplier`, because having two was the bug.
`ValueModelTest.surfMultiplierCoversEveryCase` pins its four cases.

**The generalisable point:** the campaign has been finding ported-but-unreached code (ITER45-49) and
contract rules bypassed by a replaced loop (ITER47, 51, 52). This is a third kind — *duplicated*
logic where one copy drifted. Worth grepping for other constants that appear in both driver classes.

### Also recorded: the seed-72 chain investigation from ITER54 was a dead end

All sixteen pushback decisions in that game agree on the square AND the pick; two of them name a
different defender, but both are at steps **146 and 154**, long after the divergence at step 72, so
they are consequences rather than causes. ITER53's fix is not incomplete after all — the two
mismatched entries are downstream of a board that had already diverged.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2025 89 → 90/100**, **bb2020 89 → 90/100**,
  bb2016 5/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,658 / 0** (no Rust change). `mvn -o -pl ffb-ai test`:
  **32 / 0**. The two Java trees agree.

**Next:** the remaining bb2025/bb2020 reds still share most of their seeds. Before chasing another
one individually, grep both driver classes for weighting constants that appear in only one of them —
this iteration's bug had exactly that shape and a second instance would be cheap to find.

## ITER56 — the two coverage terms, found by diffing constants instead of chasing a seed

ITER55 ended by suggesting a constant diff between the two sides rather than another seed hunt.
Doing it: strip comments and the test module from `heuristic_agent.rs`, do the same to the Java
`heuristic/` package, and compare the multisets of float literals. Most of the difference is noise
(doc section numbers, Rust-only deep mode), but `0.08` appears in Rust's live code and nowhere in
Java. It is the **novelty** bonus — and pulling that thread found a second term beside it.

Both are per-decision inputs that `PlanBuilder` already accepted and `ActivationChoice` fed a
hardcoded `0.0f` at all six of its candidate-building call sites:

| term | Rust | was |
|---|---|---|
| `novelty` | `0.08` the first time a board BUCKET is seen | `0.0f` |
| `floor(pac)` | `0.35 * (1 - min(seen/4, 1))` while an action is under-used | `0.0f` |

Both are **dead below `temp_scale` 0.1 and live above it**. That is the whole reason they survived
56 iterations: every gate so far has been argmax, where Rust computes zero for both and the port's
hardcoded zero is indistinguishable from the real thing. The activation golden cannot see them
either — it was emitted with both at zero.

So this iteration also measured the sampled gate for the first time. It was **0/100**, and the two
terms were a real part of that:

- **bb2025 `--heur-scale 1.0`: 0/100 → 6/100.** bb2020 likewise 0 → 6. bb2016 stays 0/100.

Ported `bucket` too (`ballz | carried<<6 | turn<<8 | weather<<12 | half<<16`), including the detail
that the weather index is the ordinal of **Rust's** `Weather` enum, whose order differs from the
Java enum's. `Decision` now also carries the BASE action name, because Rust's counter keys on its
`PlayerActionChoice` rather than on the declaration — a give counts as one `HandOver` however it is
declared.

**The lesson is about the method, not the bug.** Five of the last six iterations found the fault by
diffing two *executions*; this one found it by diffing two *texts*, in a few minutes, and it was a
defect no execution diff could have surfaced at the scale we were gating. Both techniques are worth
keeping — and a constant that appears on one side only is a cheap, high-yield thing to re-run
whenever the port grows.

### Gates

- `--heur-classes all`, 100 seeds, **argmax unchanged as expected**: bb2025 90/100, bb2020 90/100,
  bb2016 5/100 — both terms are zero at argmax, so this fix cannot move it.
- `--heur-classes all`, 100 seeds, **scale 1.0**: bb2025 **0 → 6/100**, bb2020 **0 → 6/100**,
  bb2016 0/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,658 / 0** (no Rust change). `mvn -o -pl ffb-ai test`:
  **33 / 0** (+1, novelty bite-checked). The two Java trees agree.

**Next:** the sampled gate is now the weakest at 6/100, and it is a different frontier from argmax —
everything the sampler touches (draw counts, temperatures, the RNG stream itself) is untested there.
Take its lowest failing seed next; the argmax reds will still be waiting.

## ITER57 — one of six call sites, and the sampled gate went 6/100 → 94/100

**bb2025 `--heur-scale 1.0`: 6 → 94/100. bb2020: 6 → 93/100. Uniform (1e6): bb2020 94/100,
bb2025 92/100.**

Took the sampled gate's shallowest failure, bb2025 seed 13 at step 2 — the same state on both sides,
the same player chosen, a different action. Dumping the candidates showed the two lists identical in
size and order with exactly **three** differing rows, all fouls:

```
J h00 foul tgt=a00 0.003657      R h00 foul tgt=a00 0.322000
```

0.322 = 0.92 × 0.35: Rust floored the foul at the coverage floor and Java used its raw weight.
ITER56 had wired the floor into the candidate builders and hit five of the six call sites — the FOUL
branch wraps its arguments across lines differently from the others, so the edit that replaced
`0.0f, 0.0f` never matched it:

```java
PlanBuilder.foulCandidates(e.id, pac, board.foulTargets(e.id), wPlayer,
    0.0f, 0.0f, out);          // <- the comma lands after wPlayer, not after the zeros
```

An eighty-fold difference on the one action still wrong, and it held the whole sampled gate at 6/100.

**A mechanical edit needs a mechanical check.** ITER56 counted its own replacements (it printed
"inline calls 4, multiline calls 1, immediate arm 1" = 6) and six was also the number of
`coverage.floor(pac)` occurrences afterwards — so the count *looked* right. It was: six replacements
across seven call sites. Counting what you changed does not tell you what you missed; the check that
would have caught it is grepping for the pattern that must no longer exist anywhere, which is now
one line and finds nothing.

### Regression test

`ActivationChoiceTest.everyActionBranchConsumesTheCoverageFloor` drives the real `choose` with a
stub board offering one foul and one move, and pre-seeds `seenAction` so the two branches get
DIFFERENT floors — a shared floor makes them tie and the test cannot see anything. Coverage off, the
move wins on raw weight; coverage on, the foul's untouched floor lifts it above. Bite-checked: it
fails on exactly the line ITER56 missed.

### Gates

- `--heur-classes all`, 100 seeds, **scale 1.0**: bb2025 **6 → 94/100**, bb2020 **6 → 93/100**,
  bb2016 **0 → 5/100**.
- **scale 1e6**: bb2020 94/100, bb2025 92/100 (first measurement).
- **argmax** unchanged at bb2025 90/100, bb2020 90/100, bb2016 5/100 — the coverage terms are zero
  there, so this cannot move it.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,658 / 0** (no Rust change). `mvn -o -pl ffb-ai test`:
  **34 / 0**. The two Java trees agree.

### Where the campaign now stands

| | argmax | scale 1.0 | scale 1e6 |
|---|---|---|---|
| bb2025 | 90 | 94 | 92 |
| bb2020 | 90 | 93 | 94 |
| bb2016 | 5 | 5 | — |

bb2016 is the outlier by a wide margin and is the same 5/100 at both scales, which suggests its
remaining failures are structural rather than sampling-related — its 3-command blitz, its own
generators and step twins. That is now the largest single block of work and the obvious next target.

**Next:** bb2016's lowest failing seed at argmax.

## ITER58 — bb2016 gave no movement phase to a move variant, and the game ended in silence

**bb2016 argmax: 5 → 13/100.** Scale 1.0: 5 → 7/100.

bb2016 seed 2 diverged at step 38, and the state strings matched everywhere they overlapped —
because Rust had only 38 steps against Java's 164. The `LOOP` trace named it in one line:

```
LOOP applied=Activate(home_03,HandOffMove) prompt_after=None finished=false
```

No prompt and not finished, so the parity loop's `current_prompt().is_none()` exit fired and the
game stopped mid-drive without a word. `bb2016/StepInitSelecting::execute_step` opens the movement
phase on

```rust
if action == Some(PlayerAction::Move) {
```

where Java gates it on `PlayerAction.isMoving()` — which lists `HAND_OVER_MOVE` and `PASS_MOVE`
alongside `MOVE`. Those two variants exist *precisely* to buy a movement phase before the give or
the throw, and bb2016 was the one edition that gave them none. `is_moving()` was already ported
faithfully on the Rust enum; the call site simply did not use it.

Rust stores a declared bb2016 blitz as `Blitz` rather than `BlitzMove`, and Java's `isMoving()` does
not list `BLITZ` either, so widening the gate to the real predicate leaves the blitz path untouched
— confirmed by `--agent random` and the fourteen-class rung staying 100/100 in all three editions.

**Why bb2016 alone.** The random contract declares the IMMEDIATE `HandOver`/`Pass`, never a move
variant, so no gate before the heuristic could reach this line. ITER49 taught the agent to declare
the MOVE variants — and bb2025 and bb2020 handled them, so the campaign moved on with bb2016 still
silently ending 95 of its 100 games at the first give.

**The signal worth remembering:** a *shorter* Rust log with no state mismatch is not a subtle
divergence — it is a stall, and `LOOP applied=… prompt_after=None` names the exact action that
caused it. That is much faster than diffing states, and this iteration's first three commands were
the whole diagnosis.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2016 5 → 13/100**; bb2020 90/100, bb2025 90/100
  (untouched — the fix is in a bb2016-only file).
- scale 1.0: **bb2016 5 → 7/100**; bb2025 94/100, bb2020 93/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,659 / 0** (+1, bite-checked — it fails on exactly the
  `== Move` comparison). `mvn -o -pl ffb-ai test`: **34 / 0**. The two Java trees agree.

**Next:** bb2016's new lowest failing seed. 87 reds remain there against 10 in each of the other two,
so bb2016 stays the target — and the stall signature above is worth checking for first each time,
since one more dead action would look identical.

## ITER59 — the bb2016 give had nowhere to land: 13 → 72/100

**bb2016 argmax: 13 → 72/100.** Scale 1.0: 7 → 10/100.

ITER58 ended by suggesting the stall signature be checked first each time. Doing that turned the
whole edition into one measurement: of bb2016's 87 reds, **82 reported `rust=None`** — Rust ran out
of steps rather than disagreeing about one. Sampling ten of them for the last applied action:

```
13 HandOff→home_05    16 HandOff→away_03    20 HandOff→away_08
14 HandOff→away_04    18 HandOff→home_02    21 HandOff→away_07
```

Six of ten stalled on the terminal give — the command ITER58 had just made reachable.

`bb2016/StepInitMoving`'s `Action::HandOff` arm dispatched the action and set nothing else.
`StepInitPassing` opens with

```rust
if game.thrower_id.is_none() || game.thrower_action.is_none() { ... }   // bare cont(), no prompt
```

so it parked and the game stopped. Java sets both from `CLIENT_HAND_OVER` inside `StepInitPassing`;
Rust sees that command one step earlier, so the arm has to set them — which **the bb2025 twin has
always done**, comment and all, twenty lines away in the mirror file:

```rust
game.pass_coordinate = Some(c);
game.thrower_id = game.acting_player.player_id.clone();
game.thrower_action = Some(PlayerAction::HandOver);
... .publish(StepParameter::CatcherId(Some(receiver_id.clone())))
```

Ported it verbatim. One arm, twenty-eight lines, and bb2016 went from 13 to 72.

**Two editions of the same file drifting apart is now the third instance** — ITER53 (the chain-push
defender, where bb2016 was the *correct* one and bb2020/bb2025 wrong) and ITER55 (two copies of the
block weighting) were the others. The direction reverses; the shape does not. When a bb2016 twin
misbehaves, diffing it against the bb2025 file is worth doing before anything else.

**And the stall check earns its place in the loop.** Two commands turned "87 failing seeds" into
"one dead command", which is a far better starting point than the shallowest seed's state diff.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2016 13 → 72/100**; bb2020 90/100, bb2025 90/100
  (untouched — bb2016-only file).
- scale 1.0: **bb2016 7 → 10/100**; bb2025 94/100, bb2020 93/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,660 / 0** (+1, bite-checked). `mvn -o -pl ffb-ai test`:
  **34 / 0**. The two Java trees agree.

**Next:** re-run the stall census on bb2016's remaining 28 reds — the same two commands — before
picking a seed. `Action::Pass` in the same file sets no thrower state either and is the obvious
candidate, but no sampled seed stalled on it, so measure before fixing.

## ITER60 — the pass, measured before it was fixed

**bb2016 argmax: 72 → 77/100. bb2016 scale 1.0: 10 → 81/100.**

ITER59 predicted this one from the code — `Action::Pass` in the same file set no thrower state
either — and explicitly said to measure before fixing, since no sampled seed had stalled on it. The
census says: of bb2016's 28 remaining reds, **12 are stalls**, and of six sampled, two ended on
`Pass(...)` and three on `EndPlayerAction`.

So the prediction was right and worth acting on, and the discipline was also right: `EndPlayerAction`
is the larger group and would have been missed by fixing the predicted thing and moving on.

The fix is the give's, one arm down, ported from the bb2025 twin. The only difference is that a pass
takes its catcher from the target SQUARE rather than a receiver id. `HAIL_MARY_PASS` keeps its bare
dispatch, as in bb2025 — it is thrown at a square and its own step sets what it needs.

**The sampled gate moved eight times as far as argmax** (10 → 81 against 72 → 77). Worth
remembering: a stall ends the game, so a single dead command destroys every seed that reaches it,
and how many seeds reach it depends on the policy. Reading the two gates' deltas as one number would
have understated this fix badly.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2016 72 → 77/100**; bb2020 90/100, bb2025 90/100
  (untouched — bb2016-only file).
- scale 1.0: **bb2016 10 → 81/100**; bb2025 94/100, bb2020 93/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,661 / 0** (+1, bite-checked). `mvn -o -pl ffb-ai test`:
  **34 / 0**. The two Java trees agree.

### Where the campaign stands

| | argmax | scale 1.0 |
|---|---|---|
| bb2025 | 90 | 94 |
| bb2020 | 90 | 93 |
| bb2016 | 77 | 81 |

**Next:** the `EndPlayerAction` stalls — three of the six sampled, and now the largest identified
group. A stall on a DESELECT is a different shape from a dead command: the action is universal, so
it is the state it lands in that has no continuation. Census first, as usual.

## ITER61 — bb2016 had no arm for a deselect: 77 → 83/100

**bb2016 argmax: 77 → 83/100.** Scale 1.0 unchanged at 81/100.

The census after ITER60: 23 reds, 6 stalls, and **all six ended on `EndPlayerAction`** — the group
ITER60 flagged. The context named it:

```
LOOP applied=Activate(home_11,Move) prompt_after=Some(Move { player_id: "home_11", squares: [] })
LOOP applied=EndPlayerAction        prompt_after=None finished=false
```

A `Move` prompt with an **empty** square list — the player has nowhere to go — answered with a
deselect, and nothing follows.

Java's bb2016 `StepInitSelecting` handles `CLIENT_ACTING_PLAYER` with no player id in the `else` of
its guard:

```java
} else {
    fEndPlayerAction = true;
}
commandStatus = StepCommandStatus.EXECUTE_STEP;
```

Rust had **no arm for it at all**, so the command fell through to `_ => {}` and returned a bare
`cont()` with no prompt.

**Why it hid.** The deselect only reaches this step while it is still waiting for its second
command, which happens only when the player never moved. After any real movement the deselect lands
in `StepInitMoving`, which has always handled it — so the identical command works a few lines
earlier in the same game. The failing case needs a player with zero legal destinations, which the
random agent, moving one square per activation into open space, essentially never produces.

Note this is **not** a copy of the bb2025 arm: bb2025's twin calls `change_player_action_to_none`,
matching ITS Java. The two editions genuinely differ here, so ITER59's "diff against the mirror"
heuristic would have given the wrong answer — the Java source is still the authority, and checking
it took one grep.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2016 77 → 83/100**; bb2020 90/100, bb2025 90/100
  (untouched — bb2016-only file).
- scale 1.0: bb2016 81/100 (unchanged), bb2025 94/100, bb2020 93/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,662 / 0** (+1, bite-checked). `mvn -o -pl ffb-ai test`:
  **34 / 0**. The two Java trees agree.

**Next:** bb2016's stalls should now be gone or nearly so — re-run the census. If it is clean, the
remaining 17 bb2016 reds are genuine state divergences and the edition finally looks like the other
two, at which point the three lists are worth comparing for shared seeds again (ITER54's trick).

## ITER62 — the agent read the wrong field for a bribe: bb2020 and bb2025 both 90 → 97

**argmax: bb2025 90 → 97/100, bb2020 90 → 97/100, bb2016 83 → 86/100.**

The census first: **no stalls remain in any edition** (bb2016 17 reds, bb2020 10, bb2025 10, all
genuine state divergences). So the three red lists were worth comparing again, and they split
cleanly:

- bb2016's 17 reds are all **bb2016-only** seeds.
- bb2020 and bb2025 share **8 seeds at identical steps** — 14/178, 20/114, 30/177, 32/187, 35/114,
  58/163, 98/166 — plus 62 at different steps.

Took the shared seed 20. Both sides activate `home_03` at step 114; Java declares `FOUL_MOVE` and
Rust `Move`. The candidate diff was a single row:

```
JAVA h01 foul tgt=a02 0.064770        RUST h01 foul tgt=a02 0.003657
```

At argmax, where both coverage terms are zero, so this is the raw `foulWeight`. Probing its inputs
on both sides gave it immediately: **Java saw `bribes=1`, Rust saw `bribes=0`**, and a bribe swaps
the ejection cost from 0.45 to 0.07.

The **"Get the Ref"** kickoff hands +1 Bribe the Ref to *both* teams. Rust's engine implements it
faithfully — into `turn_data_*.inducement_set`, exactly where Java's `handleGetTheRef` writes it.
The agent was reading `Team::bribes`, a *different* field that only the inducement-PURCHASE step
writes, so it stayed 0 for a granted bribe and every foul after that kickoff was priced as if
ejection were seven times more costly.

**My ITER45 Java port was the correct one here.** It read the inducement set because that is what
Java does; the Rust original read a field that happens to agree only when bribes are bought rather
than granted. Worth noting because the reflex all campaign long has been "Rust is the reference, the
port is suspect" — the port being right is what exposed this.

### Gates

- `--heur-classes all`, 100 seeds, argmax: **bb2025 90 → 97/100**, **bb2020 90 → 97/100**,
  **bb2016 83 → 86/100**.
- scale 1.0: bb2025 94/100, bb2020 93/100, bb2016 83/100.
- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,663 / 0** (+1, bite-checked — with the old field the two
  weights are byte-identical). `mvn -o -pl ffb-ai test`: **34 / 0**. The two Java trees agree.

### Where the campaign stands

| | argmax | scale 1.0 |
|---|---|---|
| bb2025 | 97 | 94 |
| bb2020 | 97 | 93 |
| bb2016 | 86 | 83 |

**Next:** re-compare the three red lists — 3/3/14 now, and if bb2020 and bb2025 still share their
remaining three, that is one more shared cause. bb2016's 14 are its own problem and are the bulk of
what is left.

## ITER63 — the foul re-picked its own victim: bb2020 reaches 100/100

**bb2020 argmax: 97 → 100/100 — the first edition to go green on the full class set.**
bb2025 97 → 99/100, bb2016 86 → 87/100.

The red lists after ITER62 left two shared seeds (14 and 32, identical steps in both editions). Took
seed 14. Both sides foul with `away_07` at step 178 on the same dice — and a *different victim* ends
up KO'd: Java `home_11`, Rust `home_03`.

Diffing the foul candidates across every decision in the game showed them **identical** until
decision 192, long after the divergence. That is what pointed at the delivery rather than the
weights: both agents scored the same options and picked the same one, and the harness then fouled
someone else.

`sendFoulAction` re-picked the victim with `actionRng` from its own coordinate-sorted list, ignoring
`heuristicTarget` — the same defect `sendBlockAction` had fixed back at ITER45 and the bb2016 blitz
at ITER50.

**Applying ITER57's lesson, I checked the whole class before fixing one member:**

| sender | honours `heuristicTarget` | reachable by the heuristic |
|---|---|---|
| `sendBlockAction` | yes | yes |
| `sendFoulAction` | **no** | **yes** ← the bug |
| `sendPassAction` | no | no — ITER49 routes ball actions via `sendMoveAction`/`sendPlanTerminal` |
| `sendHandOverAction` | no | no — same |
| `sendThrowTeamMateAction` | no | no — no TTM on a lineman roster |
| `sendBlitzTargetSelection` | n/a | yes, via its own plan replay |

So exactly one arm needed fixing, and the other four are unreachable rather than latent — which is
worth knowing, because two of them will become reachable the moment the campaign moves past lineman
rosters.

### Regression test

None, for the same reason as ITER46 and ITER50: the property is which branch a dispatcher takes,
and the `ffb-ai` test tree has no live-`Game` scaffolding. The table above is the guard that
actually applies here — a mechanical check over the whole class rather than a test of the one
member — and the sweep is the evidence.

### Gates

| | argmax | scale 1.0 | scale 1e6 |
|---|---|---|---|
| bb2025 | 99 | 99 | 93 |
| bb2020 | **100** | 97 | 96 |
| bb2016 | 87 | 83 | 85 |

- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,663 / 0** (no Rust change). `mvn -o -pl ffb-ai test`:
  **34 / 0**. The two Java trees agree.

**Next:** bb2025 has ONE red left at argmax and one at scale 1.0 — take it. bb2016's 13 and the
sampled-gate gaps are what remains after that. Note that argmax, 1.0 and 1e6 no longer move
together: 1e6 is now the weakest column in two editions, and the goal needs all three.

## ITER64 — the plan was scored with the weather and rebuilt without it

**bb2020: 100/100 at ALL THREE scales. bb2025: 100/100 at argmax and 1.0, 98/100 uniform.**
bb2016 87 → 88 argmax.

bb2025's last argmax red was seed 73. Both sides move `away_04` at step 37 on a board they agree
about, and the ball carrier ends at (19,12) for Java and (20,11) for Rust, with Java rolling one
extra die. The weather was **Blizzard**.

`ActivationChoice.choose` passes the real edition and weather when it SCORES a destination:

```java
new Reach.MoverSpec(m.home, m.ag, e.dodge, e.sureFeet), bb2016, blizzard, teamReRoll
```

Both of `ActivationDriver`'s path REBUILDS — `recordPlan` and `replan` — passed:

```java
new Reach.MoverSpec(m.home, m.ag, false, false), false, false, teamReRoll
```

Four wrong arguments. A plan was therefore chosen under one set of movement rules and its path
re-derived under another; in a blizzard the rush target is 3 rather than 2, so the squares past a
player's MA are worth materially less and a different route wins. Both rebuilds now go through one
`searchFor` that reads all four facts from the game.

**The test was wrong before it was right, and the reason matters.** The obvious assertion — that a
blizzard changes the reach — fails: it changes neither the reached SET nor `cost`. A rush is still
*allowed* in a blizzard, just likelier to fail, so the difference lands entirely in the quantised
`key` that the search minimises and `pathTo` walks back. Comparing the field a reader would reach
for first finds nothing, which is precisely why a constant `false` here looked free for 64
iterations. The test now asserts both halves: `cost` identical, `key` different.

### Gates

| | argmax | scale 1.0 | scale 1e6 |
|---|---|---|---|
| bb2025 | **100** | **100** | 98 |
| bb2020 | **100** | **100** | **100** |
| bb2016 | 88 | 84 | 88 |

- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in bb2016, bb2020 and bb2025.
- `cargo test --workspace --release`: **14,663 / 0** (no Rust change). `mvn -o -pl ffb-ai test`:
  **35 / 0** (+1). The two Java trees agree.

**bb2020 is DONE** — 100/100 at all three scales, which is one third of the campaign goal.

**Next:** bb2025's two uniform-only reds (1e6), then bb2016, which is now the whole remaining
problem at 88/84/88. Its 12 argmax reds are all bb2016-only seeds.

## ITER65 — the BB2025 stalling rule was dead in three places at once

**bb2025 uniform (1e6): 98 → 99/100.** Everything else unchanged; bb2020 remains 100/100 at all
three scales.

bb2025's uniform reds were seeds 26 and 82. Seed 26's states matched everywhere and only
`rng_calls` differed — 46 against 45 — and the dice trace named the extra die outright:

```
JAVA_DIE rng=46 d6=4 from=...bb2025.shared.StallingExtension.handleStaller:73
```

Java rolls a d6 at a stalling player; Rust rolled nothing. **Three independent defects were
stopping it**, and each one alone would have been enough:

1. **The option was off.** `UtilServerStartGame:301-303` builds `ENABLE_STALLING_CHECK` set to
   FALSE and then does *not* add it — the `addOption` line is commented out — so Java falls back to
   `GameOptionFactory`'s default, which is **true**. Rust treats an unset option as disabled. This
   is the third instance of that exact trap in `runner.rs`, alongside `mbStacksAgainstChainsaw` and
   `clawDoesNotStack`, and the comment there now says so.
2. **Guard 4 was a stub.** `is_considered_stalling` ended with `false` and a note that
   `PathFinderWithPassBlockSupport` was not translated. It *is*:
   `ffb_model::util::pathfinding::path_finder_with_pass_block_support`, whose
   `get_shortest_path_for_player` takes exactly Java's four arguments. Ported-but-unreached code
   again — the fourth time this campaign.
3. **`check_for_staller` had the condition inverted.** Java's guard is
   `!gameState.isStalling() && isConsideredStalling()` and its body *ends* with
   `gameState.stallingDetected()`. Rust required `game.stalling` to be true already and never set
   it — so the flag was never raised and `StepStallingPlayer`, a faithful port that reads exactly
   that flag, could never fire.

**A test encoded the bug.** `staller_detected_report_added_when_stalling` set `game.stalling = true`
and asserted a report — the inverted behaviour, written from the Rust code rather than from Java. It
failed the moment the condition was corrected, which is the useful thing a test can do here;
rewritten as `check_for_staller_raises_the_flag_for_a_lone_carrier`, which asserts the flag goes UP,
that a second call is a no-op while it is up, and that the option gates it.

Writing that test also corrected my own reading of the rule: `hasOpenPathToEndzone` asks the
pathfinder for a route of at most **MA** squares, so "open path" means *could have scored this
turn and chose not to*. My first version put the carrier 20 squares out and no carrier is ever
stalling from there.

### Gates

| | argmax | scale 1.0 | scale 1e6 |
|---|---|---|---|
| bb2025 | **100** | **100** | 99 |
| bb2020 | **100** | **100** | **100** |
| bb2016 | 88 | 84 | 88 |

- Fourteen-class rung: **100/100 in bb2016, bb2020 and bb2025**.
- `--agent random` lineman tier-3: **100/100** in all three — checked deliberately, since enabling
  the stalling option changes engine behaviour for every agent, not just the heuristic.
- `cargo test --workspace --release`: **14,663 / 0**. `mvn -o -pl ffb-ai test`: **35 / 0**. The two
  Java trees agree.

**Next:** bb2025's last uniform red is seed 82 at step 94. After that bb2016 is the whole remaining
problem at 88/84/88.

## ITER66 — the rock is thrown but never lands (partial; gate unchanged)

**No gate moved.** bb2025 uniform stays 99/100, everything else as ITER65 left it. Recorded because
the work is verified and the remaining step is now named precisely — the same shape as ITER54.

bb2025's last uniform red is seed 82. At step 94 Java rolls **12** dice where Rust rolled 4, and the
trace shows what they are:

```
139 d6  block          142 d6  StallingExtension.handleStaller   <- ITER65 got this far
140 d6  armour         143 d26 rollXCoordinate                    <- the rock's start square
141 d6  armour         144-147 d6  armour + injury
                       148-149 casualty (d16 + d6)
                       150 d8  (the ball, after the carrier leaves)
```

ITER65 made the staller roll happen; its *consequences* were still a stub, whose note said the
branch was "unreachable in headless" — true before ITER65 and false after. Ported Java's
`if (successful)` branch: the rock's start square, `InjuryTypeThrowARockStalling` via
`handle_injury_by_name`, and a `SteadyFootingContext` wrapping the drop. **Rust now rolls 11 of the
12.**

Two things found on the way:

- **`rollXCoordinate()` is `rollDice(26) - 1`**, giving x in [0, 25]. Rust's bb2020 twin rolls
  `die(24)` with a comment claiming [1..24]. That is wrong, but its branch does not fire in the
  current gate, so it is left alone rather than fixed blind — noted here so it is not mistaken for
  a model to copy.
- `handle_staller` now returns `(GameEvent, Option<SteadyFootingContext>)`, because the extension
  cannot publish step parameters and both callers must.

**What is still missing.** The injury is rolled but never applied: Java ends with `away_04`
seriously injured off the pitch and the ball bounced to (2,14); Rust leaves him Standing at (1,13)
holding it. The 12th die is that bounce. `StepForgoneStalling` publishes the context (probe
confirms `ctx=true`), and its sequence has `SteadyFooting` immediately after with a matching
`ApothecaryMode::HitPlayer`, so the next thing to check is whether
`SteadyFootingContext::from_drop_player` carries the apothecary mode through to
`get_apothecary_mode()` — `StepSteadyFooting::set_parameter` drops the context on the floor when it
does not match.

### Gates (all unchanged, verified not regressed)

| | argmax | scale 1.0 | scale 1e6 |
|---|---|---|---|
| bb2025 | 100 | 100 | 99 |
| bb2020 | 100 | 100 | 100 |
| bb2016 | 88 | 84 | 88 |

- Fourteen-class rung **100/100** and `--agent random` **100/100** in all three editions.
- `cargo test --workspace --release`: **14,663 / 0**. `mvn -o -pl ffb-ai test`: **35 / 0**.

**Next:** the apothecary-mode question above — one grep — then seed 82 should close and bb2025 joins
bb2020 as a finished edition.
