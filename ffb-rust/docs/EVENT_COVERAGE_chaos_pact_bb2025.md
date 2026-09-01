# Event coverage — HeuristicAgent, chaos_pact v chaos_pact, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-01 by `MATCHUP=chaos_pact scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12398 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 650 | ok |  |
| action Blitz | 798 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 265 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 576 | ok |  |
| dodge failure | 203 | ok |  |
| GFI rolls | 4845 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 237 | ok |  |
| pickup failure | 108 | ok | turnover + scatter |
| catch success | 96 | ok |  |
| catch failure | 93 | ok |  |
| ball scatters | 552 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 6 | ok | ball out of bounds |
| pass rolls | 163 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 579 | ok |  |
| block 2 dice | 600 | ok |  |
| block 2 dice against | 203 | ok | defender's choice |
| block 3 dice | 23 | ok | needs ST5+ differential via assists |
| block result Skull | 177 | ok |  |
| block result BothDown | 221 | ok |  |
| block result Pushback | 464 | ok |  |
| block result PowPushback | 269 | ok |  |
| block result Pow | 274 | ok |  |
| pushbacks | 1005 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1570 | ok |  |
| armor held | 1208 | ok |  |
| stunned | 591 | ok | injury 2-7 |
| KO | 207 | ok |  |
| casualty (d16) | 204 | ok |  |
| death | 27 | ok | d16 = 15-16 only |
| fouls | 235 | ok |  |
| argue the call | 49 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 40 | ok |  |
| touchdowns | 32 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 29 | ok | kickoff event roll of 8 only |
| kickoff events | 221 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 35
- Charge: 24
- Cheering Fans: 32
- Dodgy Snack: 7
- Get the Ref: 7
- High Kick: 30
- Pitch Invasion: 7
- Quick Snap: 22
- Solid Defence: 21
- Time-out: 7
- Weather Change: 29

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/chaos_pact_vs_chaos_pact/seed_*_rust_events.jsonl)

Total events: 95588

```
  57356 playerMoved
  14606 playerAction
   4845 goForItRoll
   3407 turnEnd
   2844 confusionRoll
   2210 injury
   1570 playerFellDown
   1405 blockRoll
   1405 block
   1005 pushback
    779 dodgeRoll
    552 scatterBall
    411 apothecaryRoll
    345 pickupRoll
    271 skillUse
    237 ballPickedUp
    235 refereeSpotsFoul
    235 foul
    221 kickoffScatter
    221 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    189 catchRoll
    163 passRoll
     76 handOver
     49 argueTheCall
     46 animosityRoll
     40 playerEjected
     39 kickoffExtraReRoll
     32 touchdown
     32 playerNote
     32 cheeringFans
     29 weatherChange
     22 quickSnapRoll
     21 solidDefenceRoll
     13 kickoffPitchInvasionStun
      8 dodgySnackRoll
      7 kickoffTimeout
      7 kickoffPitchInvasion
      7 kickoffDodgySnack
      6 throwIn
      4 throwTeamMateRoll
      3 regenerationRoll
      2 alwaysHungry
      1 rightStuffRoll
```

## Player actions declared

```
  12142 Move
    798 BlitzMove
    650 Block
    495 ThrowTeamMate
    265 Foul
    170 PassMove
     86 HandOverMove
```

## Skill uses / re-rolls seen

```
```
