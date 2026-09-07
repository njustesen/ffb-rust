# Event coverage — HeuristicAgent, undead v undead, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-07 by `MATCHUP=undead scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12472 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 694 | ok |  |
| action Blitz | 885 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 261 | ok |  |
| action Pass | 17 | ok | needs a ball carrier |
| action HandOver | 4 | ok | needs carrier + adjacent teammate |
| dodge success | 374 | ok |  |
| dodge failure | 414 | ok |  |
| GFI rolls | 6209 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 207 | ok |  |
| pickup failure | 158 | ok | turnover + scatter |
| catch success | 95 | ok |  |
| catch failure | 125 | ok |  |
| ball scatters | 576 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 12 | ok | ball out of bounds |
| pass rolls | 157 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 751 | ok |  |
| block 2 dice | 566 | ok |  |
| block 2 dice against | 237 | ok | defender's choice |
| block 3 dice | 4 | ok | needs ST5+ differential via assists |
| block result Skull | 220 | ok |  |
| block result BothDown | 221 | ok |  |
| block result Pushback | 555 | ok |  |
| block result PowPushback | 266 | ok |  |
| block result Pow | 296 | ok |  |
| pushbacks | 1116 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1643 | ok |  |
| armor held | 1580 | ok |  |
| stunned | 462 | ok | injury 2-7 |
| KO | 87 | ok |  |
| casualty (d16) | 153 | ok |  |
| death | 20 | ok | d16 = 15-16 only |
| fouls | 261 | ok |  |
| argue the call | 52 | ok | referee spotted a foul (doubles) |
| argue success | 11 | ok | d6 = 6 only |
| players ejected | 42 | ok |  |
| touchdowns | 21 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 27 | ok | kickoff event roll of 8 only |
| kickoff events | 217 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 27
- Charge: 20
- Cheering Fans: 36
- Dodgy Snack: 9
- Get the Ref: 4
- High Kick: 32
- Pitch Invasion: 7
- Quick Snap: 23
- Solid Defence: 20
- Time-out: 12
- Weather Change: 27

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2025/undead_vs_undead/seed_*_rust_events.jsonl)

Total events: 92002

```
  55424 playerMoved
  14333 playerAction
   6209 goForItRoll
   3393 turnEnd
   2282 injury
   1643 playerFellDown
   1558 blockRoll
   1558 block
   1116 pushback
    788 dodgeRoll
    576 scatterBall
    365 pickupRoll
    261 refereeSpotsFoul
    261 foul
    220 catchRoll
    217 kickoffScatter
    217 kickoffResultEvent
    207 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    162 regenerationRoll
    157 passRoll
     80 handOver
     52 argueTheCall
     42 playerEjected
     36 cheeringFans
     35 skillUse
     32 kickoffExtraReRoll
     27 weatherChange
     23 quickSnapRoll
     21 touchdown
     21 playerNote
     20 solidDefenceRoll
     14 kickoffPitchInvasionStun
     12 throwIn
     12 kickoffTimeout
      9 kickoffDodgySnack
      9 dodgySnackRoll
      7 kickoffPitchInvasion
      3 playerAdded
```

## Player actions declared

```
  12260 Move
    885 BlitzMove
    694 Block
    261 Foul
    131 PassMove
     81 HandOverMove
     17 Pass
      4 HandOver
```

## Skill uses / re-rolls seen

```
     19 Dodge used=true
      8 Tackle used=true
      8 Dodge used=false
```
