# Norse — heuristic-agent parity campaign

**🏁 CLOSED 2026-09-05 (ITER1).** Nine gates 100/100, three random controls 100/100. The single red
was a **Java parity-HARNESS gap**, not a Rust engine bug. Started after nurgle (`117a532cd`).

## Surface

Lineman (Block, Drunkard, Thick Skull), Beer Boar (Dodge, No Hands, Pick-me-up, Stunty, Titchy),
Berzerker (Block, **Frenzy**, Jump Up), Valkyrie (Catch, Dauntless, Pass, Strip Ball), Ulfwerener
(**Frenzy**), Yhetee (Claws, Disturbing Presence, **Frenzy**, Loner 4, Unchannelled Fury).

Frenzy on three positions plus a negatrait (Unchannelled Fury) — the mandatory-follow-up and
second-block machinery is heavily exercised here, and that is exactly where the red lived.

## Baseline (measured on `e025e6f0d`, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | 99 (seed 90) | **100** |
| bb2025 | **100** | **100** | **100** |

## ITER1 — bb2020 @0 seed 90: the Stiletto prayer turns a Frenzy follow-up into a block-kind ask

### What diverged

`first_state_divergence.sh` put the resolving activation at **i=87** (`home_03, Block`), declared
identically by both engines. The state at i=88 differed in exactly three tokens:

```
R  h1 t8/7 active=home   a03:7,0,Stunned   h02:8,1,Standing   (home's turn continues)
J  h1 t8/8 active=away   a03:8,1,Standing  h02:8,2,Standing   (turn passed to away)
```

`FFB_TRACE=1` on both sides showed the whole story:

* Rust: `Push(8,1)` → follow-up → **second** `BlockChoice dice=[6,2]` → `Push(7,0)` → defender
  Stunned → home keeps activating. 8 rng calls.
* Java: `JAVA_BLOCKROLL nDice=2`, two d6, `JAVA_PUSHBACK pushed=…Away4 to=(8,1)`, then
  **`UNHANDLED_STEP: INIT_BLOCKING turnMode=SELECT_BLOCK_KIND`** and the runner's default
  `ClientCommandEndTurn`. 2 rng calls, no second block, turn over.

### Root cause

`home_03` is a plain **Ulfwerener — Frenzy and nothing else**. But a temporary-skills probe on the
unhandled step printed:

```
PROBE_SKILLS Stab[alt=true,altBlitz=false,used=false] Frenzy[alt=false,altBlitz=false,used=false]
```

Home had drawn the **Stiletto** prayer (`JAVA_PRAYERPICK side=home … pick=3`), which hands one random
player a **temporary Stab**. `Stab` carries `providesBlockAlternative`, so on the Frenzy second block
`StepEndBlocking` (bb2020) line 282 computes

```java
boolean askForBlockKind = UtilCards.hasUnusedSkillWithProperty(actingPlayer.getPlayer(), providesBlockAlternative)
    || (UtilCards.hasUnusedSkillWithProperty(actingPlayer.getPlayer(), providesBlockAlternativeDuringBlitz) && isBlitz);
```

= **true**, and pushes the block sequence with `ASK_FOR_BLOCK_KIND`. `StepInitBlocking` (bb2020) line
199 then sets `TurnMode.SELECT_BLOCK_KIND` and **returns without a next action**, waiting for a
`CLIENT_BLOCK` naming the block kind. `ParityRunner` had **no `INIT_BLOCKING` case**, so it fell
through to the `UNHANDLED_STEP` default and ended the turn in the middle of a frenzy follow-up.

The *engine* was right on both sides of the wire; only the harness could not answer.

### The fix (Java harness only — no Rust engine change)

`ParityRunner.handleStep` gains an `INIT_BLOCKING` case. When the step is parked in
`SELECT_BLOCK_KIND` it answers with the **plain block** on the same defender —
`ClientCommandBlock(actingPlayerId, game.getDefenderId(), false, false, false, false, false)` — which
is what every other block this runner sends looks like. `StepInitBlocking.handleCommand` takes the
`CLIENT_BLOCK`, `executeStep()` restores `lastTurnMode`, `askForBlockKind` is already consumed, and
the step proceeds exactly as an `askForBlockKind=false` push would have — i.e. Rust's behaviour.
Any other `INIT_BLOCKING` state still prints `UNHANDLED_STEP` and end-turns, so nothing else moved.

The answer costs **no RNG and no state change**, which is why the fix is a pure harness repair: the
transient `SELECT_BLOCK_KIND` turn mode is invisible to the per-activation state hash.

### Conclusions of mine that turned out WRONG (recorded per the campaign rule)

1. **"a pushback-direction / block-result divergence at the sideline"** (the ITER0 localisation in
   this very file). It was neither: both engines pushed to the *same* square `(8,1)` on the first
   block. The whole difference was that Java never threw the *second* (Frenzy) block at all. The
   `(7,0)` Stunned token was Rust's second push, not a different first push.
2. **"h02 moved but home_03 was the actor"** — the state-string labels `h00..h10` are **positional
   indices**, `home_NN` is a **jersey number**, so `home_03` is `h02`. Two minutes were lost to an
   apparently impossible move. (Already a known lesson; it bit again.)
3. **"askForBlockKind must be false, the Ulfwerener has only Frenzy"** — true of the roster, false of
   the live player. `UtilCards.hasUnusedSkillWithProperty` reads
   `getSkillsIncludingTemporaryOnes()`, and the Stiletto prayer had added Stab. Reading the roster
   XML three times could never have found this; the probe on the live step did it in one run.

### Known Rust fidelity gap left open (deliberate, filed in BACKLOG)

Rust's `step/bb2020/block/step_end_blocking.rs` and its bb2025 twin **do not port the
`askForBlockKind` computation** at all — the Frenzy re-push always goes out with
`ask_for_block_kind: false`. `StepInitBlocking` *does* implement the flag (both editions), it simply
never receives it. With both harnesses now answering "plain block", this is observationally inert
for parity, so it was NOT changed under a gate this iteration: porting it would require a new
`AgentPrompt` and a matching answer in both agents for zero measurable difference. Recorded as a
translation deviation rather than fixed.

## Gate (2026-09-05, this iteration's binary + jar)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** (was 99) | **100** |
| bb2025 | **100** | **100** | **100** |

Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`): bb2016 **100/100**,
bb2020 **100/100**, bb2025 **100/100**.

`cargo test -p ffb-engine`: **7422 passed, 0 failed** (unchanged — no Rust code changed).
`mvn -o -pl ffb-ai test`: **39 passed, 0 failed**. `check_java_trees.py`: trees agree.

Closed-roster regressions, bb2025 @1.0 seeds 1-100, all **100/100**: nurgle, necromantic, nippon,
lizardman, khemri, human, goblin, amazon, chaos, dwarf. Because the fix is in the edition-shared
runner, bb2020 @1.0 was also spot-checked: dwarf, goblin, chaos **100/100** each.

Coverage harvested ×3 → `docs/EVENT_COVERAGE_norse_{bb2016,bb2020,bb2025}.md`.
