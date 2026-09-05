# Event coverage — HeuristicAgent, norse v norse, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=norse scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11260 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 736 | ok |  |
| action Blitz | 1005 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 340 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 439 | ok |  |
| dodge failure | 326 | ok |  |
| GFI rolls | 4515 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 203 | ok |  |
| pickup failure | 128 | ok | turnover + scatter |
| catch success | 81 | ok |  |
| catch failure | 83 | ok |  |
| ball scatters | 583 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 18 | ok | ball out of bounds |
| pass rolls | 131 | ok |  |
| pass deviates | 35 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 675 | ok |  |
| block 2 dice | 807 | ok |  |
| block 2 dice against | 405 | ok | defender's choice |
| block 3 dice | 208 | ok | needs ST5+ differential via assists |
| block result Skull | 291 | ok |  |
| block result BothDown | 303 | ok |  |
| block result Pushback | 705 | ok |  |
| block result PowPushback | 352 | ok |  |
| block result Pow | 444 | ok |  |
| pushbacks | 1498 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1974 | ok |  |
| armor held | 1755 | ok |  |
| stunned | 612 | ok | injury 2-7 |
| KO | 234 | ok |  |
| casualty (d16) | 206 | ok |  |
| death | 26 | ok | d16 = 15-16 only |
| fouls | 292 | ok |  |
| argue the call | 63 | ok | referee spotted a foul (doubles) |
| argue success | 16 | ok | d6 = 6 only |
| players ejected | 52 | ok |  |
| touchdowns | 21 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 25 | ok | kickoff event roll of 8 only |
| kickoff events | 219 | ok | per-result table below |

## Kickoff results

- Blitz: 23
- Brilliant Coaching: 26
- Cheering Fans: 43
- Get the Ref: 6
- High Kick: 28
- Officious Ref: 14
- Pitch Invasion: 7
- Quick Snap: 24
- Solid Defence: 15
- Time-out: 8
- Weather Change: 25

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/norse_vs_norse/seed_*_rust_events.jsonl)

Total events: 89909

```
  52028 playerMoved
  13341 playerAction
   4515 goForItRoll
   3403 turnEnd
   2807 injury
   2095 blockRoll
   2095 block
   1974 playerFellDown
   1498 pushback
   1369 confusionRoll
    765 dodgeRoll
    583 scatterBall
    440 apothecaryRoll
    331 pickupRoll
    292 refereeSpotsFoul
    292 foul
    219 kickoffScatter
    219 kickoffResultEvent
    203 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    164 catchRoll
    131 passRoll
     67 handOver
     63 argueTheCall
     52 playerEjected
     48 dauntlessRoll
     43 cheeringFans
     35 passDeviate
     33 prayerRoll
     25 weatherChange
     24 quickSnapRoll
     23 blitzRoll
     22 kickoffExtraReRoll
     21 touchdown
     18 throwIn
     15 solidDefenceRoll
     14 skillUse
     14 kickoffOfficiousRef
     11 kickoffPitchInvasionStun
      8 kickoffTimeout
      7 kickoffPitchInvasion
      2 trapDoor
```

## Player actions declared

```
  11059 Move
   1005 BlitzMove
    736 Block
    340 Foul
    131 PassMove
     70 HandOverMove
```

## Skill uses / re-rolls seen

```
     14 Dodge used=true
```
