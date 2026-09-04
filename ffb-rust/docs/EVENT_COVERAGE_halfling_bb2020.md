# Event coverage — HeuristicAgent, halfling v halfling, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=halfling scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12105 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 623 | ok |  |
| action Blitz | 934 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 316 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 406 | ok |  |
| dodge failure | 367 | ok |  |
| GFI rolls | 5965 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 214 | ok |  |
| pickup failure | 89 | ok | turnover + scatter |
| catch success | 132 | ok |  |
| catch failure | 144 | ok |  |
| ball scatters | 524 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 22 | ok | ball out of bounds |
| pass rolls | 192 | ok |  |
| pass deviates | 50 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 659 | ok |  |
| block 2 dice | 249 | ok |  |
| block 2 dice against | 111 | ok | defender's choice |
| block 3 dice | 366 | ok | needs ST5+ differential via assists |
| block result Skull | 203 | ok |  |
| block result BothDown | 204 | ok |  |
| block result Pushback | 455 | ok |  |
| block result PowPushback | 200 | ok |  |
| block result Pow | 323 | ok |  |
| pushbacks | 725 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1450 | ok |  |
| armor held | 1457 | ok |  |
| stunned | 372 | ok | injury 2-7 |
| KO | 231 | ok |  |
| casualty (d16) | 241 | ok |  |
| death | 11 | ok | d16 = 15-16 only |
| fouls | 316 | ok |  |
| argue the call | 64 | ok | referee spotted a foul (doubles) |
| argue success | 13 | ok | d6 = 6 only |
| players ejected | 57 | ok |  |
| touchdowns | 8 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 26 | ok | kickoff event roll of 8 only |
| kickoff events | 207 | ok | per-result table below |

## Kickoff results

- Blitz: 20
- Brilliant Coaching: 26
- Cheering Fans: 34
- Get the Ref: 6
- High Kick: 25
- Officious Ref: 7
- Pitch Invasion: 8
- Quick Snap: 25
- Solid Defence: 18
- Time-out: 12
- Weather Change: 26

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/halfling_vs_halfling/seed_*_rust_events.jsonl)

Total events: 85343

```
  48032 playerMoved
  14546 playerAction
   5965 goForItRoll
   3383 turnEnd
   2301 injury
   1450 playerFellDown
   1385 blockRoll
   1385 block
    911 standUpRoll
    773 dodgeRoll
    725 pushback
    524 scatterBall
    472 apothecaryRoll
    316 refereeSpotsFoul
    316 foul
    303 pickupRoll
    276 catchRoll
    214 ballPickedUp
    207 kickoffScatter
    207 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    192 passRoll
    163 throwTeamMateRoll
    106 skillUse
    101 handOver
     73 rightStuffRoll
     64 argueTheCall
     57 playerEjected
     50 passDeviate
     34 cheeringFans
     27 prayerRoll
     26 weatherChange
     25 quickSnapRoll
     22 throwIn
     21 kickoffExtraReRoll
     20 blitzRoll
     18 solidDefenceRoll
     17 kickoffPitchInvasionStun
     12 kickoffTimeout
      8 touchdown
      8 kickoffPitchInvasion
      7 kickoffOfficiousRef
      1 trapDoor
```

## Player actions declared

```
  11817 Move
    934 BlitzMove
    623 Block
    568 ThrowTeamMate
    316 Foul
    182 PassMove
    106 HandOverMove
```

## Skill uses / re-rolls seen

```
```
