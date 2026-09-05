# Event coverage — HeuristicAgent, ogre v ogre, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=ogre scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11685 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 598 | ok |  |
| action Blitz | 809 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 203 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 368 | ok |  |
| dodge failure | 260 | ok |  |
| GFI rolls | 5505 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 195 | ok |  |
| pickup failure | 111 | ok | turnover + scatter |
| catch success | 85 | ok |  |
| catch failure | 82 | ok |  |
| ball scatters | 506 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 9 | ok | ball out of bounds |
| pass rolls | 148 | ok |  |
| pass deviates | 40 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 645 | ok |  |
| block 2 dice | 186 | ok |  |
| block 2 dice against | 65 | ok | defender's choice |
| block 3 dice | 356 | ok | needs ST5+ differential via assists |
| block result Skull | 181 | ok |  |
| block result BothDown | 187 | ok |  |
| block result Pushback | 409 | ok |  |
| block result PowPushback | 202 | ok |  |
| block result Pow | 273 | ok |  |
| pushbacks | 880 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1384 | ok |  |
| armor held | 1320 | ok |  |
| stunned | 459 | ok | injury 2-7 |
| KO | 258 | ok |  |
| casualty (d16) | 264 | ok |  |
| death | 22 | ok | d16 = 15-16 only |
| fouls | 173 | ok |  |
| argue the call | 33 | ok | referee spotted a foul (doubles) |
| argue success | 6 | ok | d6 = 6 only |
| players ejected | 31 | ok |  |
| touchdowns | 9 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 26 | ok | kickoff event roll of 8 only |
| kickoff events | 207 | ok | per-result table below |

## Kickoff results

- Blitz: 21
- Brilliant Coaching: 28
- Cheering Fans: 31
- Get the Ref: 8
- High Kick: 26
- Officious Ref: 10
- Pitch Invasion: 6
- Quick Snap: 26
- Solid Defence: 17
- Time-out: 8
- Weather Change: 26

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/ogre_vs_ogre/seed_*_rust_events.jsonl)

Total events: 91434

```
  47672 playerMoved
  14926 playerAction
   7566 confusionRoll
   5505 goForItRoll
   3390 turnEnd
   2301 injury
   1384 playerFellDown
   1252 blockRoll
   1252 block
    880 pushback
    660 throwTeamMateRoll
    628 dodgeRoll
    522 apothecaryRoll
    506 scatterBall
    350 rightStuffRoll
    306 pickupRoll
    207 kickoffScatter
    207 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    195 ballPickedUp
    173 refereeSpotsFoul
    173 foul
    167 catchRoll
    148 passRoll
     77 handOver
     48 skillUse
     40 passDeviate
     33 argueTheCall
     31 playerEjected
     31 cheeringFans
     26 weatherChange
     26 quickSnapRoll
     26 prayerRoll
     24 kickoffExtraReRoll
     21 blitzRoll
     17 solidDefenceRoll
     15 kickoffPitchInvasionStun
     10 kickoffOfficiousRef
      9 touchdown
      9 throwIn
      8 kickoffTimeout
      6 kickTeamMateFumble
      6 kickoffPitchInvasion
      1 trapDoor
```

## Player actions declared

```
  11465 Move
   1269 ThrowTeamMate
    809 BlitzMove
    598 Block
    362 KickTeamMate
    203 Foul
    140 PassMove
     80 HandOverMove
```

## Skill uses / re-rolls seen

```
     48 Dodge used=true
```
