# Event coverage — HeuristicAgent, skaven v skaven, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=skaven scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12513 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 804 | ok |  |
| action Blitz | 989 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 322 | ok |  |
| action Pass | 39 | ok | needs a ball carrier |
| action HandOver | 3 | ok | needs carrier + adjacent teammate |
| dodge success | 595 | ok |  |
| dodge failure | 433 | ok |  |
| GFI rolls | 4078 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 248 | ok |  |
| pickup failure | 125 | ok | turnover + scatter |
| catch success | 124 | ok |  |
| catch failure | 91 | ok |  |
| ball scatters | 549 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 14 | ok | ball out of bounds |
| pass rolls | 165 | ok |  |
| pass deviates | 35 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 898 | ok |  |
| block 2 dice | 690 | ok |  |
| block 2 dice against | 263 | ok | defender's choice |
| block 3 dice | 43 | ok | needs ST5+ differential via assists |
| block result Skull | 278 | ok |  |
| block result BothDown | 304 | ok |  |
| block result Pushback | 618 | ok |  |
| block result PowPushback | 318 | ok |  |
| block result Pow | 376 | ok |  |
| pushbacks | 1299 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1796 | ok |  |
| armor held | 1626 | ok |  |
| stunned | 549 | ok | injury 2-7 |
| KO | 210 | ok |  |
| casualty (d16) | 181 | ok |  |
| death | 27 | ok | d16 = 15-16 only |
| fouls | 298 | ok |  |
| argue the call | 69 | ok | referee spotted a foul (doubles) |
| argue success | 15 | ok | d6 = 6 only |
| players ejected | 63 | ok |  |
| touchdowns | 58 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 27 | ok | kickoff event roll of 8 only |
| kickoff events | 254 | ok | per-result table below |

## Kickoff results

- Blitz: 22
- Brilliant Coaching: 41
- Cheering Fans: 37
- Get the Ref: 6
- High Kick: 39
- Officious Ref: 12
- Pitch Invasion: 10
- Quick Snap: 28
- Solid Defence: 21
- Time-out: 11
- Weather Change: 27

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2020/skaven_vs_skaven/seed_*_rust_events.jsonl)

Total events: 105645

```
  67344 playerMoved
  14670 playerAction
   4078 goForItRoll
   3436 turnEnd
   2566 injury
   1894 blockRoll
   1894 block
   1796 playerFellDown
   1401 animalSavagery
   1299 pushback
   1028 dodgeRoll
    549 scatterBall
    391 apothecaryRoll
    373 pickupRoll
    298 refereeSpotsFoul
    298 foul
    254 kickoffScatter
    254 kickoffResultEvent
    248 ballPickedUp
    215 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    165 passRoll
     79 handOver
     69 argueTheCall
     63 playerEjected
     58 touchdown
     37 cheeringFans
     35 passDeviate
     35 kickoffExtraReRoll
     32 prayerRoll
     28 quickSnapRoll
     27 weatherChange
     22 blitzRoll
     21 solidDefenceRoll
     18 kickoffPitchInvasionStun
     17 skillUse
     14 throwIn
     12 kickoffOfficiousRef
     11 kickoffTimeout
     10 kickoffPitchInvasion
      6 trapDoor
```

## Player actions declared

```
  12319 Move
    989 BlitzMove
    804 Block
    322 Foul
    118 PassMove
     76 HandOverMove
     39 Pass
      3 HandOver
```

## Skill uses / re-rolls seen

```
     17 Dodge used=true
```
