# Event coverage — HeuristicAgent, undead v undead, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-07 by `MATCHUP=undead scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12552 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 722 | ok |  |
| action Blitz | 935 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 262 | ok |  |
| action Pass | 24 | ok | needs a ball carrier |
| action HandOver | 4 | ok | needs carrier + adjacent teammate |
| dodge success | 495 | ok |  |
| dodge failure | 392 | ok |  |
| GFI rolls | 6140 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 211 | ok |  |
| pickup failure | 132 | ok | turnover + scatter |
| catch success | 101 | ok |  |
| catch failure | 114 | ok |  |
| ball scatters | 552 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 16 | ok | ball out of bounds |
| pass rolls | 153 | ok |  |
| pass deviates | 41 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 856 | ok |  |
| block 2 dice | 520 | ok |  |
| block 2 dice against | 269 | ok | defender's choice |
| block 3 dice | 3 | ok | needs ST5+ differential via assists |
| block result Skull | 248 | ok |  |
| block result BothDown | 254 | ok |  |
| block result Pushback | 562 | ok |  |
| block result PowPushback | 282 | ok |  |
| block result Pow | 302 | ok |  |
| pushbacks | 1141 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1587 | ok |  |
| armor held | 1547 | ok |  |
| stunned | 418 | ok | injury 2-7 |
| KO | 152 | ok |  |
| casualty (d16) | 124 | ok |  |
| death | 18 | ok | d16 = 15-16 only |
| fouls | 262 | ok |  |
| argue the call | 52 | ok | referee spotted a foul (doubles) |
| argue success | 11 | ok | d6 = 6 only |
| players ejected | 47 | ok |  |
| touchdowns | 28 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 26 | ok | kickoff event roll of 8 only |
| kickoff events | 226 | ok | per-result table below |

## Kickoff results

- Blitz: 21
- Brilliant Coaching: 35
- Cheering Fans: 32
- Get the Ref: 7
- High Kick: 29
- Officious Ref: 10
- Pitch Invasion: 6
- Quick Snap: 26
- Solid Defence: 21
- Time-out: 13
- Weather Change: 26

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2020/undead_vs_undead/seed_*_rust_events.jsonl)

Total events: 93961

```
  57070 playerMoved
  14499 playerAction
   6140 goForItRoll
   3412 turnEnd
   2241 injury
   1648 blockRoll
   1648 block
   1587 playerFellDown
   1141 pushback
    887 dodgeRoll
    552 scatterBall
    343 pickupRoll
    262 refereeSpotsFoul
    262 foul
    226 kickoffScatter
    226 kickoffResultEvent
    215 catchRoll
    211 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    153 passRoll
     81 regenerationRoll
     79 handOver
     72 skillUse
     52 argueTheCall
     47 playerEjected
     41 passDeviate
     32 cheeringFans
     30 kickoffExtraReRoll
     28 touchdown
     26 weatherChange
     26 quickSnapRoll
     26 prayerRoll
     21 solidDefenceRoll
     21 blitzRoll
     16 throwIn
     13 kickoffTimeout
     11 kickoffPitchInvasionStun
     10 kickoffOfficiousRef
      6 kickoffPitchInvasion
```

## Player actions declared

```
  12347 Move
    935 BlitzMove
    722 Block
    262 Foul
    129 PassMove
     76 HandOverMove
     24 Pass
      4 HandOver
```

## Skill uses / re-rolls seen

```
     72 Dodge used=true
```
