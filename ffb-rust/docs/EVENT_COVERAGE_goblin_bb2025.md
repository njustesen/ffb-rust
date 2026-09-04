# Event coverage — HeuristicAgent, goblin v goblin, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=goblin scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 10832 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 679 | ok |  |
| action Blitz | 798 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 201 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 368 | ok |  |
| dodge failure | 287 | ok |  |
| GFI rolls | 4282 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 178 | ok |  |
| pickup failure | 102 | ok | turnover + scatter |
| catch success | 206 | ok |  |
| catch failure | 191 | ok |  |
| ball scatters | 504 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 8 | ok | ball out of bounds |
| pass rolls | 832 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 528 | ok |  |
| block 2 dice | 205 | ok |  |
| block 2 dice against | 91 | ok | defender's choice |
| block 3 dice | 471 | ok | needs ST5+ differential via assists |
| block result Skull | 171 | ok |  |
| block result BothDown | 193 | ok |  |
| block result Pushback | 441 | ok |  |
| block result PowPushback | 195 | ok |  |
| block result Pow | 295 | ok |  |
| pushbacks | 940 | ok |  |
| crowd surfs | 1389 | ok | push off pitch — board-position dependent |
| players fell | 1211 | ok |  |
| armor held | 1438 | ok |  |
| stunned | 360 | ok | injury 2-7 |
| KO | 340 | ok |  |
| casualty (d16) | 245 | ok |  |
| death | 30 | ok | d16 = 15-16 only |
| fouls | 159 | ok |  |
| argue the call | 34 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 27 | ok |  |
| touchdowns | 15 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 28 | ok | kickoff event roll of 8 only |
| kickoff events | 213 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 29
- Charge: 19
- Cheering Fans: 32
- Dodgy Snack: 13
- Get the Ref: 7
- High Kick: 28
- Pitch Invasion: 10
- Quick Snap: 20
- Solid Defence: 15
- Time-out: 12
- Weather Change: 28

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/goblin_vs_goblin/seed_*_rust_events.jsonl)

Total events: 87099

```
  48145 playerMoved
  13594 playerAction
   4282 goForItRoll
   3389 turnEnd
   3213 confusionRoll
   2383 injury
   1389 scatterPlayer
   1295 blockRoll
   1295 block
   1211 playerFellDown
    940 pushback
    832 passRoll
    655 dodgeRoll
    525 apothecaryRoll
    504 scatterBall
    468 spellEffectRoll
    397 catchRoll
    280 pickupRoll
    213 kickoffScatter
    213 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    178 ballPickedUp
    159 refereeSpotsFoul
    159 foul
    116 skillUse
    104 alwaysHungry
    101 throwTeamMateRoll
     64 handOver
     60 rightStuffRoll
     36 kickoffExtraReRoll
     34 argueTheCall
     32 cheeringFans
     28 weatherChange
     27 playerEjected
     20 quickSnapRoll
     18 escapeRoll
     16 kickoffPitchInvasionStun
     16 bombOutOfBounds
     15 touchdown
     15 solidDefenceRoll
     15 playerNote
     14 dodgySnackRoll
     13 kickoffDodgySnack
     12 kickoffTimeout
     10 kickoffPitchInvasion
      8 throwIn
      6 regenerationRoll
```

## Player actions declared

```
  10640 Move
    798 BlitzMove
    679 Block
    567 ThrowTeamMate
    517 ThrowBomb
    201 Foul
    124 PassMove
     68 HandOverMove
```

## Skill uses / re-rolls seen

```
```
