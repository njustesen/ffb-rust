# Event coverage — HeuristicAgent, orc v orc, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=orc scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12398 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 719 | ok |  |
| action Blitz | 907 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 268 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 355 | ok |  |
| dodge failure | 388 | ok |  |
| GFI rolls | 5432 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 221 | ok |  |
| pickup failure | 160 | ok | turnover + scatter |
| catch success | 90 | ok |  |
| catch failure | 111 | ok |  |
| ball scatters | 565 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 17 | ok | ball out of bounds |
| pass rolls | 172 | ok |  |
| pass deviates | 24 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 530 | ok |  |
| block 2 dice | 583 | ok |  |
| block 2 dice against | 315 | ok | defender's choice |
| block 3 dice | 57 | ok | needs ST5+ differential via assists |
| block result Skull | 203 | ok |  |
| block result BothDown | 237 | ok |  |
| block result Pushback | 501 | ok |  |
| block result PowPushback | 250 | ok |  |
| block result Pow | 294 | ok |  |
| pushbacks | 1040 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1571 | ok |  |
| armor held | 1771 | ok |  |
| stunned | 238 | ok | injury 2-7 |
| KO | 149 | ok |  |
| casualty (d16) | 110 | ok |  |
| death | 10 | ok | d16 = 15-16 only |
| fouls | 239 | ok |  |
| argue the call | 39 | ok | referee spotted a foul (doubles) |
| argue success | 3 | ok | d6 = 6 only |
| players ejected | 41 | ok |  |
| touchdowns | 19 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 28 | ok | kickoff event roll of 8 only |
| kickoff events | 216 | ok | per-result table below |

## Kickoff results

- Blitz: 22
- Brilliant Coaching: 27
- Cheering Fans: 42
- Get the Ref: 6
- High Kick: 28
- Officious Ref: 8
- Pitch Invasion: 8
- Quick Snap: 23
- Solid Defence: 15
- Time-out: 9
- Weather Change: 28

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/orc_vs_orc/seed_*_rust_events.jsonl)

Total events: 92364

```
  54759 playerMoved
  14560 playerAction
   5432 goForItRoll
   3398 turnEnd
   2268 injury
   1668 confusionRoll
   1571 playerFellDown
   1485 blockRoll
   1485 block
   1040 pushback
    743 dodgeRoll
    565 scatterBall
    381 pickupRoll
    259 apothecaryRoll
    239 refereeSpotsFoul
    239 foul
    221 ballPickedUp
    216 kickoffScatter
    216 kickoffResultEvent
    201 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    172 passRoll
     71 animosityRoll
     68 handOver
     47 skillUse
     42 cheeringFans
     41 playerEjected
     39 argueTheCall
     38 prayerRoll
     36 alwaysHungry
     28 weatherChange
     28 throwTeamMateRoll
     24 passDeviate
     23 quickSnapRoll
     22 kickoffExtraReRoll
     22 blitzRoll
     19 touchdown
     17 throwIn
     16 rightStuffRoll
     15 solidDefenceRoll
     15 kickoffPitchInvasionStun
      9 kickoffTimeout
      9 escapeRoll
      8 kickoffPitchInvasion
      8 kickoffOfficiousRef
      1 trapDoor
```

## Player actions declared

```
  12163 Move
    907 BlitzMove
    719 Block
    268 ThrowTeamMate
    268 Foul
    164 PassMove
     71 HandOverMove
```

## Skill uses / re-rolls seen

```
     47 Dodge used=true
```
