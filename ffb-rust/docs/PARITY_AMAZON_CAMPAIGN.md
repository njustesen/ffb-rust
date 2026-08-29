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
| bb2016 | 39/100 |
| bb2020 | 14/100 |
| bb2025 | 0/100 |

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
