# Event coverage — HeuristicAgent, lizardman v lizardman, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=lizardman scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11764 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 847 | ok |  |
| action Blitz | 914 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 249 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 360 | ok |  |
| dodge failure | 527 | ok |  |
| GFI rolls | 3920 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 156 | ok |  |
| pickup failure | 212 | ok | turnover + scatter |
| catch success | 70 | ok |  |
| catch failure | 105 | ok |  |
| ball scatters | 594 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 5 | ok | ball out of bounds |
| pass rolls | 120 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 473 | ok |  |
| block 2 dice | 746 | ok |  |
| block 2 dice against | 373 | ok | defender's choice |
| block 3 dice | 107 | ok | needs ST5+ differential via assists |
| block result Skull | 251 | ok |  |
| block result BothDown | 276 | ok |  |
| block result Pushback | 575 | ok |  |
| block result PowPushback | 270 | ok |  |
| block result Pow | 327 | ok |  |
| pushbacks | 1236 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1511 | ok |  |
| armor held | 1644 | ok |  |
| stunned | 288 | ok | injury 2-7 |
| KO | 155 | ok |  |
| casualty (d16) | 127 | ok |  |
| death | 8 | ok | d16 = 15-16 only |
| fouls | 240 | ok |  |
| argue the call | 49 | ok | referee spotted a foul (doubles) |
| argue success | 8 | ok | d6 = 6 only |
| players ejected | 42 | ok |  |
| touchdowns | 25 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 22 | ok | kickoff event roll of 8 only |
| kickoff events | 219 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 30
- Charge: 18
- Cheering Fans: 39
- Dodgy Snack: 12
- Get the Ref: 8
- High Kick: 29
- Pitch Invasion: 7
- Quick Snap: 25
- Solid Defence: 17
- Time-out: 12
- Weather Change: 22

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/lizardman_vs_lizardman/seed_*_rust_events.jsonl)

Total events: 94084

```
  58655 playerMoved
  13775 playerAction
   3920 goForItRoll
   3399 turnEnd
   2214 injury
   1699 blockRoll
   1699 block
   1511 playerFellDown
   1458 confusionRoll
   1236 pushback
    887 dodgeRoll
    594 scatterBall
    368 pickupRoll
    240 refereeSpotsFoul
    240 foul
    219 kickoffScatter
    219 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    194 skillUse
    175 catchRoll
    156 ballPickedUp
    120 passRoll
    111 passBlock
     52 handOver
     49 argueTheCall
     42 playerEjected
     39 cheeringFans
     35 kickoffExtraReRoll
     25 touchdown
     25 quickSnapRoll
     25 playerNote
     22 weatherChange
     17 solidDefenceRoll
     15 kickoffPitchInvasionStun
     13 dodgySnackRoll
     12 kickoffTimeout
     12 kickoffDodgySnack
      7 kickoffPitchInvasion
      5 throwIn
```

## Player actions declared

```
  11590 Move
    914 BlitzMove
    847 Block
    249 Foul
    120 PassMove
     54 HandOverMove
      1 LookIntoMyEyes
```

## Skill uses / re-rolls seen

```
    123 Dodge used=true
     69 Juggernaut used=true
      2 Juggernaut used=false
```
