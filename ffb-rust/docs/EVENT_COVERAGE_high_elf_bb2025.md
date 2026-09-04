# Event coverage — HeuristicAgent, high_elf v high_elf, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=high_elf scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13552 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 811 | ok |  |
| action Blitz | 933 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 238 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 1064 | ok |  |
| dodge failure | 230 | ok |  |
| GFI rolls | 4828 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 302 | ok |  |
| pickup failure | 73 | ok | turnover + scatter |
| catch success | 232 | ok |  |
| catch failure | 68 | ok |  |
| ball scatters | 569 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 8 | ok | ball out of bounds |
| pass rolls | 180 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1239 | ok |  |
| block 2 dice | 417 | ok |  |
| block 2 dice against | 88 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 247 | ok |  |
| block result BothDown | 276 | ok |  |
| block result Pushback | 590 | ok |  |
| block result PowPushback | 304 | ok |  |
| block result Pow | 327 | ok |  |
| pushbacks | 1214 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1440 | ok |  |
| armor held | 1424 | ok |  |
| stunned | 348 | ok | injury 2-7 |
| KO | 150 | ok |  |
| casualty (d16) | 81 | ok |  |
| death | 5 | ok | d16 = 15-16 only |
| fouls | 238 | ok |  |
| argue the call | 44 | ok | referee spotted a foul (doubles) |
| argue success | 10 | ok | d6 = 6 only |
| players ejected | 34 | ok |  |
| touchdowns | 115 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 35 | ok | kickoff event roll of 8 only |
| kickoff events | 302 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 51
- Charge: 22
- Cheering Fans: 48
- Dodgy Snack: 13
- Get the Ref: 10
- High Kick: 38
- Pitch Invasion: 8
- Quick Snap: 30
- Solid Defence: 30
- Time-out: 17
- Weather Change: 35

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/high_elf_vs_high_elf/seed_*_rust_events.jsonl)

Total events: 109524

```
  71762 playerMoved
  15534 playerAction
   4828 goForItRoll
   3488 turnEnd
   2003 injury
   1744 blockRoll
   1744 block
   1440 playerFellDown
   1294 dodgeRoll
   1214 pushback
    569 scatterBall
    375 pickupRoll
    302 kickoffScatter
    302 kickoffResultEvent
    302 ballPickedUp
    300 catchRoll
    238 refereeSpotsFoul
    238 foul
    231 apothecaryRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    180 passRoll
    141 handOver
    117 playerNote
    115 touchdown
    107 skillUse
     59 kickoffExtraReRoll
     48 cheeringFans
     44 argueTheCall
     35 weatherChange
     34 playerEjected
     30 solidDefenceRoll
     30 quickSnapRoll
     17 kickoffTimeout
     15 dodgySnackRoll
     14 kickoffPitchInvasionStun
     13 kickoffDodgySnack
      8 throwIn
      8 kickoffPitchInvasion
      1 throwAtStallingPlayer
```

## Player actions declared

```
  13234 Move
    933 BlitzMove
    811 Block
    238 Foul
    175 PassMove
    143 HandOverMove
```

## Skill uses / re-rolls seen

```
```
