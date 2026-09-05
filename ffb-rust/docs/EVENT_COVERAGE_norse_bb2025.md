# Event coverage — HeuristicAgent, norse v norse, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=norse scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11264 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 724 | ok |  |
| action Blitz | 990 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 324 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 417 | ok |  |
| dodge failure | 276 | ok |  |
| GFI rolls | 4109 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 190 | ok |  |
| pickup failure | 104 | ok | turnover + scatter |
| catch success | 89 | ok |  |
| catch failure | 94 | ok |  |
| ball scatters | 563 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 10 | ok | ball out of bounds |
| pass rolls | 137 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 751 | ok |  |
| block 2 dice | 823 | ok |  |
| block 2 dice against | 359 | ok | defender's choice |
| block 3 dice | 113 | ok | needs ST5+ differential via assists |
| block result Skull | 287 | ok |  |
| block result BothDown | 303 | ok |  |
| block result Pushback | 661 | ok |  |
| block result PowPushback | 401 | ok |  |
| block result Pow | 394 | ok |  |
| pushbacks | 1450 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1938 | ok |  |
| armor held | 1747 | ok |  |
| stunned | 582 | ok | injury 2-7 |
| KO | 237 | ok |  |
| casualty (d16) | 183 | ok |  |
| death | 16 | ok | d16 = 15-16 only |
| fouls | 280 | ok |  |
| argue the call | 68 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 60 | ok |  |
| touchdowns | 22 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 21 | ok | kickoff event roll of 8 only |
| kickoff events | 219 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 38
- Charge: 22
- Cheering Fans: 37
- Dodgy Snack: 12
- Get the Ref: 8
- High Kick: 31
- Pitch Invasion: 9
- Quick Snap: 19
- Solid Defence: 11
- Time-out: 11
- Weather Change: 21

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/norse_vs_norse/seed_*_rust_events.jsonl)

Total events: 89555

```
  52532 playerMoved
  13302 playerAction
   4109 goForItRoll
   3405 turnEnd
   2749 injury
   2046 blockRoll
   2046 block
   1938 playerFellDown
   1450 pushback
   1375 confusionRoll
    693 dodgeRoll
    563 scatterBall
    420 apothecaryRoll
    294 pickupRoll
    280 refereeSpotsFoul
    280 foul
    219 kickoffScatter
    219 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    190 ballPickedUp
    183 catchRoll
    137 passRoll
     77 handOver
     68 argueTheCall
     65 dauntlessRoll
     60 playerEjected
     44 kickoffExtraReRoll
     37 cheeringFans
     22 touchdown
     22 playerNote
     21 weatherChange
     19 quickSnapRoll
     16 kickoffPitchInvasionStun
     12 kickoffDodgySnack
     12 dodgySnackRoll
     11 solidDefenceRoll
     11 kickoffTimeout
     10 throwIn
      9 skillUse
      9 kickoffPitchInvasion
```

## Player actions declared

```
  11046 Move
    990 BlitzMove
    724 Block
    324 Foul
    131 PassMove
     87 HandOverMove
```

## Skill uses / re-rolls seen

```
      9 Dodge used=true
```
