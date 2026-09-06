# Event coverage — HeuristicAgent, skaven v skaven, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=skaven scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12534 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 855 | ok |  |
| action Blitz | 1025 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 278 | ok |  |
| action Pass | 34 | ok | needs a ball carrier |
| action HandOver | 8 | ok | needs carrier + adjacent teammate |
| dodge success | 605 | ok |  |
| dodge failure | 429 | ok |  |
| GFI rolls | 3536 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 272 | ok |  |
| pickup failure | 128 | ok | turnover + scatter |
| catch success | 134 | ok |  |
| catch failure | 101 | ok |  |
| ball scatters | 594 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 7 | ok | ball out of bounds |
| pass rolls | 166 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 760 | ok |  |
| block 2 dice | 769 | ok |  |
| block 2 dice against | 306 | ok | defender's choice |
| block 3 dice | 118 | ok | needs ST5+ differential via assists |
| block result Skull | 255 | ok |  |
| block result BothDown | 275 | ok |  |
| block result Pushback | 681 | ok |  |
| block result PowPushback | 328 | ok |  |
| block result Pow | 414 | ok |  |
| pushbacks | 1417 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1833 | ok |  |
| armor held | 1537 | ok |  |
| stunned | 540 | ok | injury 2-7 |
| KO | 238 | ok |  |
| casualty (d16) | 171 | ok |  |
| death | 19 | ok | d16 = 15-16 only |
| fouls | 239 | ok |  |
| argue the call | 44 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 38 | ok |  |
| touchdowns | 74 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 30 | ok | kickoff event roll of 8 only |
| kickoff events | 266 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 41
- Charge: 22
- Cheering Fans: 53
- Dodgy Snack: 16
- Get the Ref: 8
- High Kick: 33
- Pitch Invasion: 5
- Quick Snap: 30
- Solid Defence: 20
- Time-out: 8
- Weather Change: 30

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2025/skaven_vs_skaven/seed_*_rust_events.jsonl)

Total events: 107390

```
  69264 playerMoved
  14734 playerAction
   3536 goForItRoll
   3450 turnEnd
   2486 injury
   1953 blockRoll
   1953 block
   1833 playerFellDown
   1424 animalSavagery
   1417 pushback
   1034 dodgeRoll
    594 scatterBall
    409 apothecaryRoll
    400 pickupRoll
    272 ballPickedUp
    266 kickoffScatter
    266 kickoffResultEvent
    239 refereeSpotsFoul
    239 foul
    235 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    166 passRoll
    100 handOver
     74 touchdown
     74 playerNote
     53 cheeringFans
     49 skillUse
     46 kickoffExtraReRoll
     44 argueTheCall
     38 playerEjected
     30 weatherChange
     30 quickSnapRoll
     20 solidDefenceRoll
     17 dodgySnackRoll
     16 kickoffDodgySnack
      9 kickoffPitchInvasionStun
      8 kickoffTimeout
      7 throwIn
      5 kickoffPitchInvasion
```

## Player actions declared

```
  12307 Move
   1025 BlitzMove
    855 Block
    278 Foul
    124 PassMove
    103 HandOverMove
     34 Pass
      8 HandOver
```

## Skill uses / re-rolls seen

```
     49 Dodge used=true
```
