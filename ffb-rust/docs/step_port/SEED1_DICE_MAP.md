# SEED1_DICE_MAP.md — Java ground-truth dice stream, lineman seed 1 (BB2025)

Captured via `FFB_DICE_TRACE=1` against the Java ParityRunner jar. This is the exact porting
target for the Phase D batch-1 milestone (first-activation parity). Each `pos` is one
`GameRng` draw, in order; the Rust engine must reproduce this stream by construction (porting
steps in Java order). `caller` is the Java step that rolled it.

## Game start → first activation (the milestone window: pos 1–12)

| pos | sides | result | Java step / caller | meaning |
|----:|------:|-------:|--------------------|---------|
| 1 | d3 | 2 | StepSpectators.rollSpectators:66 | fanRollHome → fanFactorHome=2 |
| 2 | d3 | 2 | StepSpectators.rollSpectators:70 | fanRollAway → fanFactorAway=2 |
| 3 | d6 | 3 | StepWeather.rollWeather:54 | weather die 1 |
| 4 | d6 | 6 | StepWeather.rollWeather:54 | weather die 2 (3+6=9 → … ) |
| 5 | d2 | 2 | StepCoinChoice.handleCommand:66 (throwCoin) | coin flip |
| 6 | d8 | 3 | StepKickoffScatterRoll:132 rollScatterDirection | dir=EAST |
| 7 | d6 | 3 | StepKickoffScatterRoll:136 rollScatterDistance | dist=3 |
|   |    |   | JAVA_SCATTER half=1 start=(21,9) dir=EAST dist=3 → end=(24,9) | ball lands (24,9) |
| 8 | d6 | 4 | StepKickoffResultRoll:80 rollKickoff | kickoff die 1 |
| 9 | d6 | 2 | StepKickoffResultRoll:80 rollKickoff | kickoff die 2 (4+2=6 → CHEERING_FANS) |
| 10 | d6 | 5 | StepApplyKickoffResult.handleCheeringFans:478 | home cheering die |
| 11 | d6 | 2 | StepApplyKickoffResult.handleCheeringFans:484 | away cheering die |
| 12 | d8 | 7 | StepCatchScatterThrowIn.bounceBall:684 | ball bounce (no catcher under (24,9)) |
| — | — | — | **first ActivatePlayer (i:1, away player 3)** | milestone reached here |

### What this implies for the port (Kickoff sequence, lineman)
- **Setup** (steps 4–5) and **KickBall** (step 9 latch) consume **0 dice** but MUST run first:
  the kick target is **(21,9)** (away receiving — recall after ReceiveChoice the kicker sets up
  first; here home kicks to the away half). Placement + target feed the state hash.
- **KickoffScatterRoll**: d8 dir THEN d6 dist (order matters). Scatter (21,9) EAST by 3 → (24,9).
- **KickoffResultRoll**: 2d6, table → 6 = **CHEERING_FANS** (BB2025 `interpretRollKickoff`).
- **ApplyKickoffResult / handleCheeringFans**: d6 home then d6 away; +FAME; higher total grants
  a reroll (ties → none). (Seed 1: home 5 vs away 2 → home’s total higher; award per Java.)
- **CatchScatterThrowIn.bounceBall**: d8 to bounce the ball from its landing square (no receiver
  on (24,9) yet). After this the receiving team’s turn opens → InitSelecting → ActivatePlayer.

GameStart hash (i:0, fresh game) = `384ccaed1d572749` (✓ Rust matches as of Phase D step 0).
First-activation pre-hash (i:1) = the target to match next (regenerate from the fresh Java log:
`parity/lineman_vs_lineman/seed_1_java.jsonl` line i:1 `state_hash`).

## Beyond first activation (context only — pos 13+)
pos 13 rollBlockDice (away3 BLITZ → block), 14–15 dodge skill rolls, 16–21 armour/injury/casualty,
… then H2 kickoff at pos 33 (scatter d8+d6 (6,8)→(10,4)), pos 35–36 kickoff result (3+5=8 = BLITZ
or … ), pos 37–41 weather-change + scatter (a CHANGING_WEATHER result), etc. These belong to the
move/block/foul steps and H2 — port after first-activation parity holds.
