# Event coverage — HeuristicAgent, renegades v renegades, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=renegades scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12535 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 707 | ok |  |
| action Blitz | 937 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 309 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 360 | ok |  |
| dodge failure | 294 | ok |  |
| GFI rolls | 4658 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 202 | ok |  |
| pickup failure | 107 | ok | turnover + scatter |
| catch success | 93 | ok |  |
| catch failure | 85 | ok |  |
| ball scatters | 537 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 6 | ok | ball out of bounds |
| pass rolls | 134 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 616 | ok |  |
| block 2 dice | 688 | ok |  |
| block 2 dice against | 202 | ok | defender's choice |
| block 3 dice | 29 | ok | needs ST5+ differential via assists |
| block result Skull | 192 | ok |  |
| block result BothDown | 219 | ok |  |
| block result Pushback | 503 | ok |  |
| block result PowPushback | 310 | ok |  |
| block result Pow | 311 | ok |  |
| pushbacks | 1123 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1687 | ok |  |
| armor held | 1641 | ok |  |
| stunned | 411 | ok | injury 2-7 |
| KO | 140 | ok |  |
| casualty (d16) | 141 | ok |  |
| death | 22 | ok | d16 = 15-16 only |
| fouls | 234 | ok |  |
| argue the call | 43 | ok | referee spotted a foul (doubles) |
| argue success | 6 | ok | d6 = 6 only |
| players ejected | 39 | ok |  |
| touchdowns | 26 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 30 | ok | kickoff event roll of 8 only |
| kickoff events | 224 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 33
- Charge: 19
- Cheering Fans: 36
- Dodgy Snack: 11
- Get the Ref: 7
- High Kick: 29
- Pitch Invasion: 7
- Quick Snap: 23
- Solid Defence: 19
- Time-out: 10
- Weather Change: 30

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/renegades_vs_renegades/seed_*_rust_events.jsonl)

Total events: 95296

```
  54969 playerMoved
  15056 playerAction
   4658 goForItRoll
   4206 confusionRoll
   3408 turnEnd
   2333 injury
   1687 playerFellDown
   1535 blockRoll
   1535 block
   1123 pushback
    654 dodgeRoll
    537 scatterBall
    344 skillUse
    309 pickupRoll
    281 apothecaryRoll
    234 refereeSpotsFoul
    234 foul
    224 kickoffScatter
    224 kickoffResultEvent
    212 animosityRoll
    202 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    178 catchRoll
    134 passRoll
     75 handOver
     43 argueTheCall
     39 playerEjected
     38 kickoffExtraReRoll
     36 cheeringFans
     30 weatherChange
     26 touchdown
     26 playerNote
     23 quickSnapRoll
     19 solidDefenceRoll
     12 dodgySnackRoll
     11 kickoffPitchInvasionStun
     11 kickoffDodgySnack
     10 kickoffTimeout
      7 kickoffPitchInvasion
      6 throwIn
      4 regenerationRoll
      1 throwTeamMateRoll
      1 rightStuffRoll
      1 alwaysHungry
```

## Player actions declared

```
  12280 Move
    937 BlitzMove
    707 Block
    568 ThrowTeamMate
    309 Foul
    166 PassMove
     89 HandOverMove
```

## Skill uses / re-rolls seen

```
    341 Horns used=true
      3 Dodge used=true
```
