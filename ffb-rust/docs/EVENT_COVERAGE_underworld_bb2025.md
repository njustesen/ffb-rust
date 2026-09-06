# Event coverage — HeuristicAgent, underworld v underworld, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=underworld scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13023 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 855 | ok |  |
| action Blitz | 970 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 237 | ok |  |
| action Pass | 33 | ok | needs a ball carrier |
| action HandOver | 2 | ok | needs carrier + adjacent teammate |
| dodge success | 667 | ok |  |
| dodge failure | 345 | ok |  |
| GFI rolls | 5115 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 260 | ok |  |
| pickup failure | 151 | ok | turnover + scatter |
| catch success | 134 | ok |  |
| catch failure | 99 | ok |  |
| ball scatters | 562 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 10 | ok | ball out of bounds |
| pass rolls | 179 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 493 | ok |  |
| block 2 dice | 504 | ok |  |
| block 2 dice against | 284 | ok | defender's choice |
| block 3 dice | 384 | ok | needs ST5+ differential via assists |
| block result Skull | 227 | ok |  |
| block result BothDown | 300 | ok |  |
| block result Pushback | 579 | ok |  |
| block result PowPushback | 236 | ok |  |
| block result Pow | 323 | ok |  |
| pushbacks | 1132 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1338 | ok |  |
| armor held | 1167 | ok |  |
| stunned | 425 | ok | injury 2-7 |
| KO | 237 | ok |  |
| casualty (d16) | 190 | ok |  |
| death | 20 | ok | d16 = 15-16 only |
| fouls | 209 | ok |  |
| argue the call | 40 | ok | referee spotted a foul (doubles) |
| argue success | 8 | ok | d6 = 6 only |
| players ejected | 32 | ok |  |
| touchdowns | 67 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 34 | ok | kickoff event roll of 8 only |
| kickoff events | 261 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 47
- Charge: 23
- Cheering Fans: 43
- Dodgy Snack: 11
- Get the Ref: 4
- High Kick: 28
- Pitch Invasion: 8
- Quick Snap: 34
- Solid Defence: 19
- Time-out: 10
- Weather Change: 34

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2025/underworld_vs_underworld/seed_*_rust_events.jsonl)

Total events: 105261

```
  66301 playerMoved
  15396 playerAction
   5115 goForItRoll
   3453 turnEnd
   2019 injury
   1811 confusionRoll
   1665 blockRoll
   1665 block
   1338 playerFellDown
   1132 pushback
   1012 dodgeRoll
    562 scatterBall
    426 apothecaryRoll
    411 pickupRoll
    261 kickoffScatter
    261 kickoffResultEvent
    260 ballPickedUp
    233 catchRoll
    209 refereeSpotsFoul
    209 foul
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    179 passRoll
     93 skillUse
     92 handOver
     68 playerNote
     67 touchdown
     55 kickoffExtraReRoll
     43 cheeringFans
     40 argueTheCall
     34 weatherChange
     34 quickSnapRoll
     33 alwaysHungry
     32 playerEjected
     28 throwTeamMateRoll
     19 solidDefenceRoll
     18 rightStuffRoll
     16 kickoffPitchInvasionStun
     12 dodgySnackRoll
     11 kickoffDodgySnack
     11 animosityRoll
     10 throwIn
     10 kickoffTimeout
      8 kickoffPitchInvasion
      7 escapeRoll
      2 regenerationRoll
```

## Player actions declared

```
  12797 Move
    970 BlitzMove
    855 Block
    276 ThrowTeamMate
    237 Foul
    134 PassMove
     92 HandOverMove
     33 Pass
      2 HandOver
```

## Skill uses / re-rolls seen

```
     93 Dodge used=true
```
