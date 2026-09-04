# Event coverage — HeuristicAgent, halfling v halfling, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=halfling scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11527 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 544 | ok |  |
| action Blitz | 751 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 233 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 426 | ok |  |
| dodge failure | 279 | ok |  |
| GFI rolls | 5873 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 205 | ok |  |
| pickup failure | 125 | ok | turnover + scatter |
| catch success | 259 | ok |  |
| catch failure | 190 | ok |  |
| ball scatters | 510 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 8 | ok | ball out of bounds |
| pass rolls | 723 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 523 | ok |  |
| block 2 dice | 208 | ok |  |
| block 2 dice against | 99 | ok | defender's choice |
| block 3 dice | 381 | ok | needs ST5+ differential via assists |
| block result Skull | 181 | ok |  |
| block result BothDown | 183 | ok |  |
| block result Pushback | 450 | ok |  |
| block result PowPushback | 151 | ok |  |
| block result Pow | 246 | ok |  |
| pushbacks | 751 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1111 | ok |  |
| armor held | 1202 | ok |  |
| stunned | 454 | ok | injury 2-7 |
| KO | 303 | ok |  |
| casualty (d16) | 305 | ok |  |
| death | 22 | ok | d16 = 15-16 only |
| fouls | 233 | ok |  |
| argue the call | 55 | ok | referee spotted a foul (doubles) |
| argue success | 14 | ok | d6 = 6 only |
| players ejected | 42 | ok |  |
| touchdowns | 7 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 27 | ok | kickoff event roll of 8 only |
| kickoff events | 207 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 24
- Charge: 20
- Cheering Fans: 42
- Dodgy Snack: 8
- Get the Ref: 7
- High Kick: 26
- Pitch Invasion: 7
- Quick Snap: 22
- Solid Defence: 15
- Time-out: 9
- Weather Change: 27

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/halfling_vs_halfling/seed_*_rust_events.jsonl)

Total events: 84026

```
  48025 playerMoved
  13599 playerAction
   5873 goForItRoll
   3389 turnEnd
   2264 injury
   1211 blockRoll
   1211 block
   1111 playerFellDown
    751 pushback
    723 passRoll
    705 dodgeRoll
    608 apothecaryRoll
    510 scatterBall
    449 catchRoll
    444 standUpRoll
    379 spellEffectRoll
    330 pickupRoll
    233 refereeSpotsFoul
    233 foul
    207 kickoffScatter
    207 kickoffResultEvent
    205 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    158 throwTeamMateRoll
    118 skillUse
     87 handOver
     68 rightStuffRoll
     55 argueTheCall
     42 playerEjected
     42 cheeringFans
     28 kickoffExtraReRoll
     27 weatherChange
     22 quickSnapRoll
     20 allYouCanEatRoll
     15 solidDefenceRoll
     13 kickoffPitchInvasionStun
      9 kickoffTimeout
      9 dodgySnackRoll
      9 bombOutOfBounds
      8 throwIn
      8 kickoffDodgySnack
      7 touchdown
      7 playerNote
      7 kickoffPitchInvasion
```

## Player actions declared

```
  11261 Move
    751 BlitzMove
    544 Block
    233 Foul
    221 ThrowBomb
    200 ThrowTeamMate
    170 PassMove
    123 AllYouCanEat
     96 HandOverMove
```

## Skill uses / re-rolls seen

```
```
