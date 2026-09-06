# Event coverage — HeuristicAgent, orc v orc, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=orc scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12491 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 774 | ok |  |
| action Blitz | 951 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 281 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 442 | ok |  |
| dodge failure | 390 | ok |  |
| GFI rolls | 5250 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 224 | ok |  |
| pickup failure | 128 | ok | turnover + scatter |
| catch success | 120 | ok |  |
| catch failure | 99 | ok |  |
| ball scatters | 524 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 7 | ok | ball out of bounds |
| pass rolls | 173 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 584 | ok |  |
| block 2 dice | 643 | ok |  |
| block 2 dice against | 340 | ok | defender's choice |
| block 3 dice | 30 | ok | needs ST5+ differential via assists |
| block result Skull | 221 | ok |  |
| block result BothDown | 273 | ok |  |
| block result Pushback | 534 | ok |  |
| block result PowPushback | 276 | ok |  |
| block result Pow | 293 | ok |  |
| pushbacks | 1103 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1613 | ok |  |
| armor held | 1732 | ok |  |
| stunned | 327 | ok | injury 2-7 |
| KO | 132 | ok |  |
| casualty (d16) | 158 | ok |  |
| death | 18 | ok | d16 = 15-16 only |
| fouls | 263 | ok |  |
| argue the call | 63 | ok | referee spotted a foul (doubles) |
| argue success | 13 | ok | d6 = 6 only |
| players ejected | 50 | ok |  |
| touchdowns | 20 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 28 | ok | kickoff event roll of 8 only |
| kickoff events | 215 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 26
- Charge: 19
- Cheering Fans: 39
- Dodgy Snack: 10
- Get the Ref: 8
- High Kick: 29
- Pitch Invasion: 8
- Quick Snap: 20
- Solid Defence: 18
- Time-out: 10
- Weather Change: 28

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/orc_vs_orc/seed_*_rust_events.jsonl)

Total events: 94610

```
  56636 playerMoved
  14776 playerAction
   5250 goForItRoll
   3399 turnEnd
   2349 injury
   1613 playerFellDown
   1597 blockRoll
   1597 block
   1583 confusionRoll
   1103 pushback
    832 dodgeRoll
    524 scatterBall
    352 pickupRoll
    289 apothecaryRoll
    263 refereeSpotsFoul
    263 foul
    224 ballPickedUp
    219 catchRoll
    215 kickoffScatter
    215 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    173 passRoll
     92 handOver
     63 argueTheCall
     50 playerEjected
     39 cheeringFans
     38 skillUse
     30 kickoffExtraReRoll
     28 weatherChange
     20 touchdown
     20 quickSnapRoll
     20 playerNote
     20 alwaysHungry
     18 solidDefenceRoll
     17 throwTeamMateRoll
     17 kickoffPitchInvasionStun
     14 rightStuffRoll
     10 kickoffTimeout
     10 kickoffDodgySnack
     10 dodgySnackRoll
      8 kickoffPitchInvasion
      7 throwIn
      4 escapeRoll
      3 regenerationRoll
```

## Player actions declared

```
  12237 Move
    951 BlitzMove
    774 Block
    281 Foul
    279 ThrowTeamMate
    159 PassMove
     95 HandOverMove
```

## Skill uses / re-rolls seen

```
     38 Dodge used=true
```
