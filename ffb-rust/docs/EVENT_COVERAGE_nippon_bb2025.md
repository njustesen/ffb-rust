# Event coverage — HeuristicAgent, nippon v nippon, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=nippon scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13344 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 923 | ok |  |
| action Blitz | 969 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 278 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 863 | ok |  |
| dodge failure | 330 | ok |  |
| GFI rolls | 4674 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 278 | ok |  |
| pickup failure | 141 | ok | turnover + scatter |
| catch success | 145 | ok |  |
| catch failure | 90 | ok |  |
| ball scatters | 601 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 9 | ok | ball out of bounds |
| pass rolls | 177 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 922 | ok |  |
| block 2 dice | 733 | ok |  |
| block 2 dice against | 222 | ok | defender's choice |
| block 3 dice | 15 | ok | needs ST5+ differential via assists |
| block result Skull | 245 | ok |  |
| block result BothDown | 368 | ok |  |
| block result Pushback | 624 | ok |  |
| block result PowPushback | 259 | ok |  |
| block result Pow | 396 | ok |  |
| pushbacks | 1270 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1418 | ok |  |
| armor held | 1377 | ok |  |
| stunned | 449 | ok | injury 2-7 |
| KO | 193 | ok |  |
| casualty (d16) | 113 | ok |  |
| death | 13 | ok | d16 = 15-16 only |
| fouls | 278 | ok |  |
| argue the call | 64 | ok | referee spotted a foul (doubles) |
| argue success | 8 | ok | d6 = 6 only |
| players ejected | 56 | ok |  |
| touchdowns | 71 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 26 | ok | kickoff event roll of 8 only |
| kickoff events | 262 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 48
- Charge: 30
- Cheering Fans: 36
- Dodgy Snack: 13
- Get the Ref: 5
- High Kick: 32
- Pitch Invasion: 8
- Quick Snap: 35
- Solid Defence: 17
- Time-out: 12
- Weather Change: 26

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/nippon_vs_nippon/seed_*_rust_events.jsonl)

Total events: 110558

```
  72720 playerMoved
  15514 playerAction
   4674 goForItRoll
   3442 turnEnd
   2132 injury
   1892 blockRoll
   1892 block
   1418 playerFellDown
   1270 pushback
   1193 dodgeRoll
    601 scatterBall
    419 pickupRoll
    306 apothecaryRoll
    278 refereeSpotsFoul
    278 foul
    278 ballPickedUp
    262 kickoffScatter
    262 kickoffResultEvent
    235 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    177 passRoll
    108 handOver
    106 skillUse
     72 playerNote
     71 touchdown
     64 argueTheCall
     56 playerEjected
     54 kickoffExtraReRoll
     36 cheeringFans
     35 quickSnapRoll
     26 weatherChange
     17 solidDefenceRoll
     15 kickoffPitchInvasionStun
     13 kickoffDodgySnack
     13 dodgySnackRoll
     12 kickoffTimeout
      9 throwIn
      8 kickoffPitchInvasion
```

## Player actions declared

```
  13062 Move
    969 BlitzMove
    923 Block
    278 Foul
    172 PassMove
    110 HandOverMove
```

## Skill uses / re-rolls seen

```
    106 Dodge used=true
```
