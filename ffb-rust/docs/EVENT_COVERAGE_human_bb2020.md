# Event coverage — HeuristicAgent, human v human, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=human scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12895 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 887 | ok |  |
| action Blitz | 963 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 254 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 639 | ok |  |
| dodge failure | 349 | ok |  |
| GFI rolls | 5135 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 260 | ok |  |
| pickup failure | 124 | ok | turnover + scatter |
| catch success | 137 | ok |  |
| catch failure | 94 | ok |  |
| ball scatters | 558 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 16 | ok | ball out of bounds |
| pass rolls | 178 | ok |  |
| pass deviates | 56 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 754 | ok |  |
| block 2 dice | 595 | ok |  |
| block 2 dice against | 366 | ok | defender's choice |
| block 3 dice | 67 | ok | needs ST5+ differential via assists |
| block result Skull | 262 | ok |  |
| block result BothDown | 276 | ok |  |
| block result Pushback | 626 | ok |  |
| block result PowPushback | 301 | ok |  |
| block result Pow | 317 | ok |  |
| pushbacks | 1241 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1515 | ok |  |
| armor held | 1411 | ok |  |
| stunned | 392 | ok | injury 2-7 |
| KO | 180 | ok |  |
| casualty (d16) | 150 | ok |  |
| death | 12 | ok | d16 = 15-16 only |
| fouls | 245 | ok |  |
| argue the call | 51 | ok | referee spotted a foul (doubles) |
| argue success | 15 | ok | d6 = 6 only |
| players ejected | 41 | ok |  |
| touchdowns | 56 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 27 | ok | kickoff event roll of 8 only |
| kickoff events | 251 | ok | per-result table below |

## Kickoff results

- Blitz: 26
- Brilliant Coaching: 37
- Cheering Fans: 47
- Get the Ref: 9
- High Kick: 30
- Officious Ref: 11
- Pitch Invasion: 7
- Quick Snap: 25
- Solid Defence: 21
- Time-out: 11
- Weather Change: 27

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/human_vs_human/seed_*_rust_events.jsonl)

Total events: 103867

```
  64899 playerMoved
  15329 playerAction
   5135 goForItRoll
   3429 turnEnd
   2133 injury
   1782 blockRoll
   1782 block
   1515 playerFellDown
   1442 confusionRoll
   1241 pushback
    988 dodgeRoll
    558 scatterBall
    384 pickupRoll
    330 apothecaryRoll
    260 ballPickedUp
    251 kickoffScatter
    251 kickoffResultEvent
    245 refereeSpotsFoul
    245 foul
    231 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    178 passRoll
     95 handOver
     56 touchdown
     56 passDeviate
     51 argueTheCall
     47 cheeringFans
     41 skillUse
     41 playerEjected
     39 prayerRoll
     32 kickoffExtraReRoll
     27 weatherChange
     26 blitzRoll
     25 quickSnapRoll
     24 throwTeamMateRoll
     21 solidDefenceRoll
     17 kickoffPitchInvasionStun
     16 throwIn
     11 rightStuffRoll
     11 kickoffTimeout
     11 kickoffOfficiousRef
      7 kickoffPitchInvasion
      5 trapDoor
```

## Player actions declared

```
  12628 Move
    963 BlitzMove
    887 Block
    330 ThrowTeamMate
    254 Foul
    166 PassMove
    101 HandOverMove
```

## Skill uses / re-rolls seen

```
```
