# Event coverage — HeuristicAgent, wood_elf v wood_elf, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=wood_elf scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12843 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 860 | ok |  |
| action Blitz | 1000 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 270 | ok |  |
| action Pass | 30 | ok | needs a ball carrier |
| action HandOver | 8 | ok | needs carrier + adjacent teammate |
| dodge success | 904 | ok |  |
| dodge failure | 294 | ok |  |
| GFI rolls | 4143 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 296 | ok |  |
| pickup failure | 57 | ok | turnover + scatter |
| catch success | 202 | ok |  |
| catch failure | 83 | ok |  |
| ball scatters | 550 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 42 | ok | ball out of bounds |
| pass rolls | 188 | ok |  |
| pass deviates | 68 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 895 | ok |  |
| block 2 dice | 562 | ok |  |
| block 2 dice against | 274 | ok | defender's choice |
| block 3 dice | 56 | ok | needs ST5+ differential via assists |
| block result Skull | 256 | ok |  |
| block result BothDown | 282 | ok |  |
| block result Pushback | 622 | ok |  |
| block result PowPushback | 277 | ok |  |
| block result Pow | 350 | ok |  |
| pushbacks | 1103 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1472 | ok |  |
| armor held | 1415 | ok |  |
| stunned | 440 | ok | injury 2-7 |
| KO | 209 | ok |  |
| casualty (d16) | 141 | ok |  |
| death | 12 | ok | d16 = 15-16 only |
| fouls | 270 | ok |  |
| argue the call | 54 | ok | referee spotted a foul (doubles) |
| argue success | 11 | ok | d6 = 6 only |
| players ejected | 45 | ok |  |
| touchdowns | 89 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 34 | ok | kickoff event roll of 8 only |
| kickoff events | 277 | ok | per-result table below |

## Kickoff results

- Blitz: 25
- Brilliant Coaching: 40
- Cheering Fans: 45
- Get the Ref: 9
- High Kick: 36
- Officious Ref: 11
- Pitch Invasion: 8
- Quick Snap: 28
- Solid Defence: 25
- Time-out: 16
- Weather Change: 34

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2020/wood_elf_vs_wood_elf/seed_*_rust_events.jsonl)

Total events: 106240

```
  69346 playerMoved
  15212 playerAction
   4143 goForItRoll
   3457 turnEnd
   2205 injury
   1787 blockRoll
   1787 block
   1472 playerFellDown
   1198 dodgeRoll
   1103 pushback
    550 scatterBall
    356 standUpRoll
    353 pickupRoll
    296 ballPickedUp
    285 catchRoll
    277 kickoffScatter
    277 kickoffResultEvent
    270 refereeSpotsFoul
    270 foul
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    188 passRoll
    145 handOver
     89 touchdown
     85 skillUse
     68 passDeviate
     54 argueTheCall
     45 playerEjected
     45 cheeringFans
     42 throwIn
     34 weatherChange
     34 prayerRoll
     31 kickoffExtraReRoll
     28 quickSnapRoll
     25 solidDefenceRoll
     25 blitzRoll
     17 kickoffPitchInvasionStun
     16 kickoffTimeout
     11 kickoffOfficiousRef
      8 kickoffPitchInvasion
      6 trapDoor
```

## Player actions declared

```
  12549 Move
   1000 BlitzMove
    860 Block
    270 Foul
    201 ThrowTeamMate
    156 PassMove
    138 HandOverMove
     30 Pass
      8 HandOver
```

## Skill uses / re-rolls seen

```
     85 Dodge used=true
```
