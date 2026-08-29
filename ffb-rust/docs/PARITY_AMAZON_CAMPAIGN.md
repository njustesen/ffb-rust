# Parity campaign — HeuristicAgent on amazon vs amazon (all three rulesets)

Started 2026-08-29, immediately after the lineman heuristic campaign closed at ITER70
(`3b3746b12`, nine gates at 100/100).

**Command**: `/amz-iter` — `.claude/commands/amz-iter.md` holds the goal, the rules, the root-cause
procedure and the standing no-regression gate. Read it every iteration; read the TAIL of this file
for the frontier.

## Goal

Nine `PARITY: 100/100 games match` — amazon v amazon, tier 3, seeds 1-100, `--agent heuristic
--heur-classes all`, across bb2016 / bb2020 / bb2025 x `--heur-scale` 0 / 1.0 / 1e6 — **plus** two
things the lineman campaign did not have to deliver:

- the agent uses the amazon skills it now has (Dodge everywhere, Block, Catch, Pass, Safe Pass,
  On the Ball, Hit and Run, Jump Up, Defensive) in its scoring, kept fast and simple;
- the event coverage is analysed, and any skill present on the pitch but absent from the event
  stream is explained — agent gap, engine bug, or genuinely dead path.

## Why this roster

Lineman rosters carry no skills at all, so 70 iterations of parity work never executed a skill
re-roll, a block-die choice under Block, a catch under Catch, or a throw under Pass/Safe Pass.
Amazons carry one skill on every player in every edition, and the three editions carry *different*
ones — bb2016 has a Catcher with Catch and a Blitzer with Block; bb2020/bb2025 replace those with
On the Ball, Safe Pass, Hit and Run, Jump Up and Defensive. The editions therefore differ
structurally, not just by generator.

## Status

| amazon v amazon, `--heur-classes all`, seeds 1-100 | sampled (1.0) |
|---|---|
| bb2016 | 52/100 |
| bb2020 | 55/100 |
| bb2025 | 45/100 |

Control: `--agent random` amazon is **100/100 in all three editions**, so the roster itself is
parity-clean and every red below belongs to the heuristic.

## Iterations

## ITER0 — baseline, and the control that scopes the problem

**Measured** (fresh JVM, `--tier 3 --seeds 1-100 --no-abort`):

| | random | heuristic `--heur-scale 1.0 --heur-classes all` |
|---|---|---|
| bb2016 | 100/100 | **46/100** (54 FAILED) |
| bb2020 | 100/100 | **0/100** |
| bb2025 | 100/100 | **0/100** |

Timing, 100 seeds: random `rust_total` 9-14s; heuristic 36-37s against `java_total` ~72s. That is
the number to compare future iterations against.

The random control matters more than the heuristic numbers do: it says the amazon team data, the
roster loading, the skills on the pitch and the engine paths they reach are all already in parity.
Everything red is the agent, or the agent's seam with the harness.

**Roster note that the census turned up, and that cost the first hypothesis:** the bb2025 amazon
spec fields the STAR PLAYER Estelle la Veneaux at jersey 2 (`data/teams/bb2025/team_amazon.json`
`stars`), so its `players` list runs 1, 13, 3, 4 ... 12 and the star fills the gap. bb2016 and
bb2020 field no star and number 1-12 in order. All three editions therefore field 12 or 13 players
with contiguous jerseys — the roster ORDER differs between the Rust spec and the Java XML, the
jersey numbers do not.

**Also observed, not a parity failure:** bb2016 prints `UNHANDLED_DIALOG: WINNINGS_RE_ROLL
turnMode=END_GAME` many times per run. It is an end-of-game dialog neither agent answers, and the
sweep is green through it in the lineman campaign too. Logged here so the next reader does not
re-diagnose it.

**Next:** bb2025 seed 2, the lowest-numbered seed with a clean early divergence — first mismatch at
step index 6 with **identical state hashes on both sides** (`50981e42f474ac9f`), Java activating the
star `...Home2` and Rust activating `home_01`. Same board, same legal options, different pick: an
agent disagreement, not an engine one.

## ITER1 — canonical order imposed in the Java chooser (correct port, NO gate movement)

**Hypothesis** (from the ITER0 next-step): Rust sorts its candidate list by `canon_key` = `(side,
nr)` before enumerating a single plan (`c1.sort_by_key(...)` in `handle_activate`), while the Java
port iterated whatever order the harness handed it — ROSTER order, per
`computeEligiblePlayers`. `ActivationChoice`'s own class doc claims it walks the canonically-sorted
list; it did not. The draw and the declaration grouping are POSITIONAL, so a differently ordered
list picks a different candidate out of identical weights.

**Fixed** as a 1:1 port of Rust's sort: `ActivationChoice.choose` now sorts a copy of the eligible
list by `(side, nr)` before tier 1. Regression test `eligibleListOrderDoesNotChangeTheDecision`
feeds the same board in roster order and in canonical order and asserts the decisions match at all
three scales.

**The test was vacuous on its first draft and that is worth recording**: a fixture with a ball
carrier in it answers the same way whatever order the list is in, because one candidate dominates.
Rebuilt with a loose ball nobody is near and six numbered 1, 13, 3, 4, 5, 6 — it then failed against
the un-fixed code at argmax (`home_13/Block/away_02` vs `home_03/Block/away_04`), which is the proof
the test is worth having. Never trust a regression test that has not been seen to fail.

**Gate — the target did NOT move:**

| | before | after |
|---|---|---|
| bb2016 amazon | 46/100 | 46/100 |
| bb2020 amazon | 0/100 | 0/100 |
| bb2025 amazon | 0/100 | 0/100 |
| bb2016/20/25 lineman heuristic 1.0 | 100/100 | **100/100** |
| `mvn -o -pl ffb-ai test` | 35/0 | **36/0** |

`rust_total` 36.6-43.4s, unchanged. Java trees synced, jar rebuilt, gates run on a fresh JVM.

**Why it is kept anyway, stated plainly rather than dressed up as a win:** for these three matchups
the harness's roster order already IS jersey order, so the sort is a no-op today and closes no seed.
It is kept because it is what Rust does, because the class documented itself as doing it, and
because the property now has a test that fails without it. It is a latent-correctness port, not
progress against the goal, and the ledger says so.

**What the iteration actually refuted:** the candidate-ORDER explanation for bb2025 seed 2 step 6.
Same board, same canonical order on both sides, and Java still activates the star while Rust
activates `home_01`. So the disagreement is in the WEIGHTS or their inputs, not the sequence.

**Next:** dump both agents' full candidate lists at bb2025 seed 2 step 6 and diff them — the
playbook's highest-yield tool, and the one that distinguishes "scored differently" from "scored
identically and declared differently". The prime suspect is the star player herself: Estelle's
attributes and skills feed `ValueModel.Mover` on both sides, and the Java `Eligible` is built from
`getMovementWithModifiers`/`getAgilityWithModifiers`/`getStrengthWithModifiers` plus five
name-matched skills, any one of which a star can carry differently from a rostered player. Check the
INPUTS before the arithmetic.

## ITER2 — the SkillUse prompt: a class the lineman campaign proved unreachable

**The measurement chain, because two hypotheses died on the way and the tooling lied once.**

ITER1 left bb2025 seed 2 diverging at a decision where both sides held the same state hash. Probes
were added to BOTH agents to dump the activation candidate lists (`FFB_CANDSUM`, `FFB_CAND=<k>`,
`FFB_DRAWS`), and they settled it in three steps:

1. Decisions 1-6 enumerate **the same players with the same option counts**; the sets first differ
   at 7, which is AFTER the divergence, not at it.
2. At the diverging decision the two lists are **bit-identical** — same order, same players, same
   actions, same raw float weights, 1850 rows each. So the scoring agrees completely and the DRAW
   disagrees.
3. The draw counters said why: entering that decision Rust had spent **26** sampler draws and Java
   **24**. They were equal at the previous decision, so two draws went missing inside the
   activation between them.

Windowing both prompt streams by draw count named it outright: Rust's `skill` prompt spends 2
draws, Java's `SKILL_USE` spends 0.

**Root cause.** `PromptClass::SKILL_USE` is scored by Rust and had **no arm at all** in
`HeuristicDriver`, so it fell through to the random contract's fixed "always use the skill" rule.
That rule answers without touching the sampler. The class was unreachable for all 70 lineman
iterations — a team with no skills is never offered one — so nothing ever caught it. Every amazon
carries Dodge, and the first failed dodge desynchronised the two RNG streams for the rest of the
game.

**Fixed** by porting Rust's arm 1:1 into `HeuristicDriver.useSkill` (the weight table: Dodge 0.95,
Fend/QuickBite/AnimalSavagery 0.85, Juggernaut 0.80, HitAndRun 0.70, Wrestle 0.55, everything else
0.50; two options; `pick(0.20)`), and routing `ParityRunner`'s SKILL_USE arm through it whenever the
class is switched on. Deliberately **no** carve-out for the four skills the random contract declines
for harness reasons (DumpOff, PrimalSavagery, SafePairOfHands, Swoop) — Rust's heuristic has none,
and inventing one here would be a divergence from the agent this is a port of. None of the four
exists on an amazon roster.

`SkillUseTest` pins the weight table, the **draw cost** (2 live, 0 at argmax) and that a 0.95/0.05
split actually declines sometimes — without that last one, the sampled arm would be
indistinguishable from the fixed rule it replaced, which is precisely how this hid.

**Gate:**

| | ITER1 | ITER2 |
|---|---|---|
| bb2016 amazon | 46/100 | **39/100** |
| bb2020 amazon | 0/100 | **14/100** |
| bb2025 amazon | 0/100 | 0/100 (seed 2's first divergence moved from step 6 to step **39**) |
| lineman heuristic 1.0 x3 | 100/100 | **100/100** |
| `cargo test -p ffb-engine` | 7342/0 | **7342/0** |
| `mvn -o -pl ffb-ai test` | 36/0 | **39/0** |

**bb2016 went DOWN by 7 and the fix was kept anyway. The evidence for that call:**

Diffing the failing seed sets (never the counts) gives 10 newly broken, 3 newly fixed. Taking the
lowest newly-broken seed, bb2016 seed 13, the two agents agree for 116 activations and then diverge
with **identical candidate lists** and Java **two draws ahead** — one whole extra sampled prompt.
Windowing the prompt streams names it: Java is offered `SKILL_USE skill=Pass` for the Amazon
Thrower, and **Rust's bb2016 engine never emits that prompt at all**.

So the asymmetry is in the ENGINE and it was there before this iteration. Answering the prompt for
free used to hide it; sampling it exposes it. Reverting would re-hide a real Rust engine bug to buy
back seven seeds, which is the wrong trade, and the aggregate across the three editions rises 46 ->
53 regardless.

**A tooling lesson worth more than the seeds.** The first bb2016 measurement showed a **one**-draw
gap at decision 114 and pointed at the wrong place. Rust's epsilon branch draws twice but only one
of them goes through `unit()`, while Java counts both — so my own instrument was off by one on every
epsilon hit. Corrected, the same seed reads a clean two-draw gap at decision 117. An instrument that
disagrees with itself by exactly one is indistinguishable from a real one-draw divergence.

The probes are kept, documented and env-gated (`FFB_CANDSUM`, `FFB_CAND=<k>`, `FFB_DRAWS`) rather
than removed: they are the campaign's highest-yield tool and cost one integer increment per draw.

**Next (ITER3):** port Java's bb2016 Pass-skill re-roll prompt into the Rust engine. Java asks the
coach; Rust decides for itself (the lineman campaign's `2244d941` made the Pass re-roll auto-use).
Java is the truth, so Rust must emit the prompt. That is a Rust ENGINE fix with a colocated
regression test, and it should recover most of the 10 bb2016 seeds as well as being right.

**Also queued, seen in the same runs:** Rust declares `Activate(home_02, BalefulHex)` for the bb2025
star while Java declares MOVE for someone else. `ParityRunner.actionFromName` has no case for it and
its `default` returns `PlayerAction.MOVE` — the identical shape as the `HandOver`/`HandOffMove` bug
already recorded in `ActivationChoice.moveVariant`. The agent picks the right thing and the harness
declares something else, so no scoring diff can show it.

## ITER3 — Rust spent the Pass skill instead of offering it (and the fix first landed in a dead file)

**The bug.** After a failed pass, Java (every edition, `StepPass`) does:

```java
ReRollSource passingReroll = UtilCards.getRerollSource(game.getThrower(), PASS);
if (passingReroll != null && !state.passSkillUsed) { showDialog(new DialogSkillUseParameter(...)); }
else { askForReRollIfAvailable(...); }
```

It ASKS. Rust auto-USED the skill and re-entered the step, with a comment explaining why that was
equivalent: "Java offers it as a SKILL_USE that ParityRunner ALWAYS uses". True of the RANDOM
contract, which answers for free. ITER2 made the heuristic SCORE `SkillUse`, so Java now spends two
sampler draws there — and a prompt Rust never emits cannot be answered, so the two RNG streams part
on the first failed pass by a thrower with the Pass skill. No lineman has it. Every Amazon Thrower
has it, in all three editions.

**Fixed** in `step/bb2025/pass/step_pass.rs` — the step that actually runs for all three editions —
by replacing the auto-use at all three failure sites (SAVED_FUMBLE, FUMBLE, INACCURATE/WILDLY) with
the offer, and porting `AbstractPassBehaviour`'s answer: the re-rolled action is set either way, the
SOURCE only on accept, so a decline goes straight to the failed-pass path rather than falling back
on the team re-roll. Added `find_player_reroll_source`, the twin of Java's
`UtilCards.getRerollSource(PLAYER, action)` — no "already used" filter, REGULAR usage types only,
which is a different question from the existing `find_skill_reroll_source`.

**Two process failures worth more than the fix, both caught by measurement rather than by review.**

1. **The fix first landed in a dead file.** `step/bb2016/pass/step_pass.rs` is the obvious home for a
   bb2016 pass bug, and it is never dispatched: `driver.rs` routes `StepId::Pass` to the bb2025 step
   for every edition. The gate came back with a byte-identical failing seed set — same 61 seeds,
   same first-diff step on every one — which is the signature of a change that did not execute. The
   file now carries a note saying so. (This is the campaign's "ported-but-unreached" pattern, and the
   TTM lesson that edition-gating inside the shared step beats routing to a dead twin.)
2. **A concurrent run of the same matchup corrupted a measurement.** I started the random control
   while the bb2016 heuristic gate was still running; they share
   `parity/bb2016/amazon_vs_amazon/seed_N_*.jsonl`, and the gate reported **12/100**. Re-run alone:
   **52/100**. The rule against this is in the command file; the cost of breaking it is a number
   that looks like a catastrophic regression and would have got a correct fix reverted.

**An existing test asserted the bug** (`fumble_auto_uses_free_pass_skill_reroll_without_prompt`) and
failed the moment the code was corrected — the useful half of the trap. Read against the Java, it
was wrong; rewritten as three tests: the offer is made, accepting spends the skill while declining
does not, and the offer happens only once per step.

**Gate:**

| | ITER2 | ITER3 |
|---|---|---|
| bb2016 amazon | 39/100 | **52/100** |
| bb2020 amazon | 14/100 | 14/100 |
| bb2025 amazon | 0/100 | 0/100 |
| lineman heuristic 1.0 x3 | 100/100 | **100/100** |
| `cargo test -p ffb-engine` | 7342/0 | **7344/0** |

`rust_total` 34.1s for 100 bb2016 seeds, down from 36.6s — the agent no longer re-enters the step to
re-roll silently. No Java change this iteration, so the jar and the Java tests are untouched.

**Next:** bb2020 is the edition holding still at 14/100 while bb2016 moves, so it has a cause of its
own; bb2025's 0/100 has the `BalefulHex` declaration bug queued from ITER2
(`ParityRunner.actionFromName` has no case for it and defaults to MOVE). Take the bb2025 one first:
it is a known, named defect with a one-line shape, and 0/100 means every seed is blocked behind it.

## ITER4 — a mistranslated star declaration fixed; a snapshot hypothesis refuted by measurement

**Fixed: `actionFromName` turned every unlisted action into MOVE.** `nameForAgent` ends
`default: return a.name()`, so an action outside its nine special cases reaches the agent as
`BALEFUL_HEX`; `actionFromName` ended `default: return PlayerAction.MOVE`, so it came back as a
plain move. The bb2025 amazon roster fields the star Estelle la Veneaux, the agent picked her
BALEFUL_HEX, and the harness declared MOVE while Rust declared the real action — the two engines
taking different branches out of an IDENTICAL decision. Same shape as the `HandOver` bug already
recorded in `ActivationChoice.moveVariant`: the agent is right and the harness mistranslates it.

The default now round-trips through `PlayerAction.valueOf(name)`, which is exactly the inverse of
what `nameForAgent` emits, and an unmappable name prints `UNMAPPED_AGENT_ACTION` instead of silently
becoming a move. Verified live: Java's log now records `Activate(...,BALEFUL_HEX)` against Rust's
`Activate(home_02,BalefulHex)`, and bb2025 seed 2's first divergence moved from step 39 to 40. **No
gate movement** — like ITER1, a real mistranslation fixed with no seed closed, and recorded as such
rather than counted as progress.

**Refuted: the eligible-list snapshot.** Root-causing bb2025 seed 1 gave a clean signal — at
activation 19 the draw counts AGREE (51 each) and the candidate lists differ by exactly one option:
Rust offers `home_06` a FOUL, Java does not. Dumping both eligible lists (`RELIG`/`JELIG`, new,
env-gated) shows why: Rust has `[Move, Foul]` for that player and Java has `[Move]`.

`RandomAgent` snapshots the eligible list at turn start and says so in a comment — "NOT the engine's
live per-activation list, so an action offered at turn start survives even if its target is knocked
down later in the same turn" — while the heuristic reads the live list. That is a textbook instance
of the campaign's own "contract rules that live in the harness LOOP, not the scorer" pattern, and it
looked certain.

It measured **bb2016 52 -> 14/100** and was reverted. So the two lists differ for a reason other
than the one I assumed: both sides compute FOUL from the same predicate (adjacent PRONE/STUNNED
opponent, foul unused) and both re-filter stale actions per activation, yet Java's list lacks it.
Either the snapshots are taken at different MOMENTS, or Java's `computeEligiblePlayers` and Rust's
`legal_activate_player_actions` disagree about this player. That is the next thing to settle, and it
now has an exact repro rather than a theory.

The failed experiment is worth its cost: it converted "the heuristic should snapshot" from a
plausible-looking certainty into a measured falsehood, and it left the diagnostic behind.

**Gate:**

| | ITER3 | ITER4 |
|---|---|---|
| bb2016 amazon | 52/100 | 52/100 |
| bb2020 amazon | 14/100 | 14/100 |
| bb2025 amazon | 0/100 | 0/100 |
| lineman heuristic 1.0 x3 | 100/100 | **100/100** |
| `cargo test -p ffb-engine` | 7344/0 | **7344/0** |
| `mvn -o -pl ffb-ai test` | 39/0 | **39/0** |

**Next:** settle why Java's eligible list lacks FOUL for `home_06` at bb2025 seed 1 activation 19.
Print the turn key and the board at the moment each side takes its snapshot; if they coincide, the
fault is in one of the two eligibility computations and Java is the truth.

## ITER5 — the pass-block window, which the heuristic had never heard of

**bb2020 14 -> 55/100**, and bb2025 records its first passing seed of the campaign. The largest
single move so far, and it came from a grep that returned zero.

**How it was found.** ITER4's refuted snapshot left an exact repro, so this iteration started by
re-measuring it rather than re-theorising. Dumping both eligible lists at every activation of
bb2025 seed 1 showed the frozen and live lists **agree byte-for-byte at the turn's first
activation** — Java `Move|Block|Blitz|BALEFUL_HEX`, Rust `Move|Block|Blitz|BalefulHex`, same eleven
players. So the freeze moments coincide and the snapshot hypothesis could not explain the gap.

Re-applying the snapshot anyway and censusing the damage was what paid: **17 failures, 0 stalls** —
all real divergences. That ruled out the obvious mechanism (a frozen list going stale and being
refused by the engine) and sent me to read `RandomAgent`'s activation branch line by line against
the heuristic's. It carries TWO pass-block rules that the heuristic does not, and
`grep -c PassBlock heuristic_agent.rs` returned **0**.

1. **The action filter.** Java `filterStaleActions`: in a non-REGULAR window the list shrinks to
   MOVE plus the UseSkill specials. A window Block/Blitz/Foul is a declare-then-deselect no-op, and
   a window BLITZ against the suspended thrower re-fires `CONFIRM_END_ACTION` forever.
2. **The move deselect.** The engine never re-presents `INIT_SELECTING` phase 2 for a pass-block
   window mover, so `ParityRunner` deselects immediately: the mover activates and never moves.
   `RandomAgent`'s comment for this one names **amazon seeds 8/11**, where the On-the-Ball defender
   stays put in Java.

That comment is the whole story. **On the Ball is the Amazon Thrower's skill in bb2020 and bb2025** —
the two editions that were stuck at 14 and 0 while bb2016 moved freely with every other fix. bb2016
amazons have no On the Ball, so bb2016 has no pass-block windows, and its number does not move here
(52 -> 52). The edition split was the clue and it was visible in the ITER0 table all along.

**The A/B that settled what to keep**, seeds 1-20:

| | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| baseline (ITER4) | 7 | 3 | 0 |
| snapshot + pass-block rules | 3 | 2 | 0 |
| **pass-block rules only** | **7** | **9** | **1** |

So the snapshot is harmful on its own terms and the pass-block rules are the win. Kept the rules,
dropped the snapshot for the second and last time — with a comment in the code recording that Java
freezes and Rust does not, that the lists agree at turn start, and that freezing measured worse
twice. That is a known, documented, deliberate divergence rather than an oversight.

**Gate:**

| | ITER4 | ITER5 |
|---|---|---|
| bb2016 amazon | 52/100 | 52/100 |
| bb2020 amazon | 14/100 | **55/100** |
| bb2025 amazon | 0/100 | **1/100** |
| lineman heuristic 1.0 x3 | 100/100 | **100/100** |
| `cargo test -p ffb-engine` | 7344/0 | **7345/0** |

Regression test `a_move_prompt_in_a_pass_block_window_deselects` pins both halves: the window
deselects, and the rule does not leak into a regular turn — without that second assertion the fix
would freeze every activation in the game and still pass.

**Next:** bb2025 at 1/100 is now the outlier, and it is the only edition with a STAR on the roster.
bb2020 and bb2016 sit at 55 and 52 with the same skills minus Estelle, so the bb2025 gap is most
likely hers — the `BalefulHex` declaration now translates correctly (ITER4) but nothing has verified
what the two engines DO with it. Start there: run seed 1 to the first divergence and check whether
the star's action resolves identically on both sides.

## ITER6 — Sidestep asked instead of assumed (bb2025 only, and no seed closed)

**The bug, and it is ITER3's twin.** bb2025 seed 5 diverges at step 2. The two agents agree exactly
for two activations and then Java spends two draws Rust does not; windowing the prompt streams names
it: `SKILL_USE skill=Sidestep pid=...Home2`. Java's `SidestepBehaviour` shows a
`DialogSkillUseParameter` during `StepPushback` and waits; Rust's hook auto-answered TRUE inline,
with a comment explaining that the parity harness always uses the skill so the round-trip is
unobservable.

That is exactly the reasoning ITER3 disproved for the Pass skill. It holds for the RANDOM contract,
which answers a SKILL_USE for free, and fails for the heuristic, which SCORES the class and spends
two sampler draws on it.

**Why this was bb2025's blocker specifically.** Sidestep is bb2025-only (bb2016 and bb2020 spell it
`Side Step` and have their own behaviour), and on the amazon roster exactly one player carries it —
the star Estelle la Veneaux, who is only on the bb2025 team. bb2016 and bb2020 have no carrier, so
this could not have been their problem, and indeed their numbers do not move by a single seed.

**Fixed** by parking the request: a step HOOK cannot raise a prompt, so `SidestepBehaviour` sets
`pending_skill_use` and returns true (Java returns true when it shows the dialog), `StepPushback`
turns that into `AgentPrompt::SkillUse`, and the answer is filed back into `side_stepping` exactly as
Java's `handleCommandHook` does. Another test asserted the old auto-use and failed on the corrected
code; rewritten as two — the undecided case must ASK and decide nothing, and the accepted case
switches the pushback mode.

**Gate — no seed closed, and the depth says why the fix is kept:**

| | ITER5 | ITER6 |
|---|---|---|
| bb2016 amazon | 52/100 | 52/100 |
| bb2020 amazon | 55/100 | 55/100 |
| bb2025 amazon | 1/100 | 1/100 |
| lineman heuristic 1.0 x3 | 100/100 | **100/100** |
| `cargo test -p ffb-engine` | 7345/0 | **7346/0** |

The pass count cannot see this fix, so measure what it can: across bb2025's 99 failures, **49 seeds
now diverge LATER and 0 earlier**, median first-diff step **4 -> 9**. bb2016 and bb2020 are untouched
to the step — 0 deeper, 0 shallower — which is exactly the signature of a change confined to the one
edition that has the skill. Seed 5's own alignment moved from activation 3 to activation 5, with six
Sidestep prompts now firing where none did.

A seed with several independent causes closes only when the last of them goes, so "seeds passed" is
a lagging indicator mid-campaign. Kept on that evidence, and the entry says plainly that no seed was
closed.

**Also found, not fixed:** `bb2016/side_step_behaviour.rs` has the identical auto-use
(`side_stepping.insert(id, true)`), and so does its Stand Firm sibling by the same comment's
admission. No amazon carries Side Step in bb2016/bb2020, so it is unreachable on this roster and
fixing it blind would be an unmeasured change to two green-ish editions. Logged for the roster that
does carry it.

**Next:** bb2025 seed 5 now diverges at activation 5 with the draws AGREEING and Rust enumerating
**54 more candidates** than Java (2025 vs 1971) — the same shape as the ITER4 FOUL, an option Rust
offers and Java does not, but fifty of them. Dump both lists there and diff by declaration.

## ITER7 — a star special is a COMMAND PAIR, not a declaration (bb2025 1 -> 45/100)

**The measurement that named it, in one line.** At the diverging activation Java's pre-hash and
post-hash are *identical* (`439b40e2...` -> `439b40e2...`) while Rust's board changed. Java did not
merely resolve the Baleful Hex differently — it did **nothing at all**. A step that changes no state
has not run.

Why: Java's `StepInitSelecting` dispatches every star special exclusively from `CLIENT_USE_SKILL`.
The client sends `ActingPlayer(MOVE)` and then `ClientCommandUseSkill(<skill>)`, and it is the
skill's PROPERTY (`canMakeOpponentMissTurn`) that sets `fDispatchPlayerAction = BALEFUL_HEX`.
Sending `ActingPlayer(BALEFUL_HEX)` on its own is accepted and goes nowhere. Rust's engine takes the
declaration directly, so the agent's identical choice reached one engine and evaporated in the other.

The random path has always sent the pair — its own inline chain, a few hundred lines above the
heuristic branch, which is why `--agent random` was 100/100 on this roster all along. The heuristic
branch sent the bare declaration. Extracted that chain as `sendStarSpecialDeclaration` and called it
from the heuristic branch, covering all five of the family (Baleful Hex, Look Into My Eyes, Catch of
the Day, Then I Started Blastin', Raiding Party) so the two paths cannot drift again.

**This supersedes half of ITER4.** That iteration made `actionFromName` return the real
`PlayerAction.BALEFUL_HEX` instead of defaulting to MOVE, and reported honestly that it closed no
seed. It was necessary but not sufficient: the name was right and the COMMAND was still wrong. Two
correct-looking fixes were needed before either could show a number.

**Gate:**

| | ITER6 | ITER7 |
|---|---|---|
| bb2016 amazon | 52/100 | 52/100 |
| bb2020 amazon | 55/100 | 55/100 |
| bb2025 amazon | 1/100 | **45/100** |
| lineman heuristic 1.0 x3 | 100/100 | **100/100** |
| `mvn -o -pl ffb-ai test` | 39/0 | **39/0** |

bb2016 and bb2020 are untouched to the seed, as they must be: neither fields a star.

**Next:** the three editions are finally in the same range (52 / 55 / 45) and no single cause
dominates any of them. Census each edition's remaining failures for a SHARED first-diff shape before
picking one — the ITER5 lesson was that the edition split itself was the clue, and the same table
should now be read for what the three have in common.
