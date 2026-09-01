---
description: Run one iteration of the FFB chaos heuristic-agent parity campaign (all three rulesets)
---

Run ONE iteration of the **chaos-vs-chaos** heuristic-agent parity campaign, then stop and report.

**The procedure IS `.claude/commands/amz-iter.md`** — the three loops (inner: one seed +
`first_state_divergence.sh` + trace diff; middle: `frontier.sh` + 20-seed probe ×3; outer: the full
standing gate), the unit-port rule, the tool rule, the fault patterns, the traps, the non-negotiable
rules. Read it every iteration. This file carries only what is chaos-specific.

Ledger (**read its TAIL first**): `ffb-rust/docs/PARITY_CHAOS_CAMPAIGN.md` — goal, roster table,
status, per-iteration record.

## Chaos-specific mechanics

Every tool takes `MATCHUP=chaos`:

```bash
MATCHUP=chaos sh scripts/first_state_divergence.sh <edition> <seed>
MATCHUP=chaos sh scripts/frontier.sh <edition> <seed...>
MATCHUP=chaos sh scripts/harvest_coverage.sh <edition>       # run ALONE
```

Target gate: chaos ×3 editions ×3 scales. **Standing regression gate**: amazon ×3 AND lineman ×3
at scale 1.0 still 100/100 (`gate.sh` runs those two; re-run the full nine amazon + nine lineman
before any push), `--agent random` chaos + amazon + lineman ×3 editions 100/100 (isolated root),
`cargo test -p ffb-engine`, trees agree, `rust_total=` not blown up.

## What to expect (check by name before waiting for seeds to surface it)

- **Horns**: +1 ST only on a BLITZ — block-dice count depends on the declared action. Both agents'
  dice expectations and the engine's `find_block_dice` must agree per action.
- **Frenzy**: mandatory follow-up (no FollowUp prompt where the skill forces it) + the SECOND block
  after a pushback — a whole step chain (`Frenzy` steps / `StepBlockRoll` re-entry). Expect
  window-style unit bugs here: open/re-open/close translated separately.
- **Mighty Blow (+1)**: bb2020/25 CHOOSE armour-or-injury; Java may dialog or auto-apply per
  edition — find which before assuming (the Safe Pass lesson, ITER30).
- **Loner 4+**: fires on every Minotaur team re-roll the heuristic spends. `use_reroll`'s
  `loner_roll` path is live for the first time under the heuristic.
- **Wild Animal (bb2016) / Unchannelled Fury (bb2020/25)**: Select-sequence negatrait steps with
  GOTO_LABEL_ON_FAILURE — under an agent that MOVES, a failed activation roll mid-plan is new.
  The kickoff-return window now runs the REAL Select (ITER29), so these fire inside windows too.
- **ST5/ST4**: 3-die blocks (`block 3 dice` was `absent (optional)` all campaign) and `-2d` against
  the Minotaur.
- The bb2016 negatrait differs from bb2020/25 — a fix in one edition's twin is not a fix in the
  other; `make_step` vs `make_step_for` tables (amz-iter "fault patterns").

## Agent quality (the second half)

Where the skills change costs the agent already models, price them — simply:
- Horns: blitz block-dice estimate uses ST+1 (both agents, goldens updated deliberately).
- Frenzy: the follow-up is not a choice; the second block is a cost/benefit the block scorer
  already approximates — do not add a planner for it.
- Mighty Blow: prefer the MB blitzer for blocks the scorer already wants.
- Loner: discount the team re-roll's value for Loner players (`p_with_reroll` with 0.5 weight).
Every scoring change lands in BOTH agents with the cross-language goldens bit-identical.

## Coverage (the third half)

`MATCHUP=chaos harvest_coverage.sh` per edition when parity is green → `docs/EVENT_COVERAGE_chaos_<ed>.md`
+ a summary section in `docs/EVENT_COVERAGE.md`. Mind BACKLOG §E6/E7: most skill uses are NOT
evented — parity plus traces is the proof; say per skill which of the four buckets it is in
(exercised+evented / exercised-unevented / agent never creates it / genuinely dead).

## Stopping

As amz-iter.md: never stop the loop on your own judgement; switch tactics. When the goal is met:
docs, ledger, coverage, memory, commit with explicit paths, **push**.
