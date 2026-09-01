# Event coverage — HeuristicAgent, dark_elf v dark_elf, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-01 by `MATCHUP=dark_elf scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12625 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 840 | ok |  |
| action Blitz | 944 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 245 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 763 | ok |  |
| dodge failure | 326 | ok |  |
| GFI rolls | 4529 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 304 | ok |  |
| pickup failure | 97 | ok | turnover + scatter |
| catch success | 147 | ok |  |
| catch failure | 93 | ok |  |
| ball scatters | 614 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 17 | ok | ball out of bounds |
| pass rolls | 180 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 950 | ok |  |
| block 2 dice | 667 | ok |  |
| block 2 dice against | 258 | ok | defender's choice |
| block 3 dice | 4 | ok | needs ST5+ differential via assists |
| block result Skull | 266 | ok |  |
| block result BothDown | 286 | ok |  |
| block result Pushback | 649 | ok |  |
| block result PowPushback | 332 | ok |  |
| block result Pow | 346 | ok |  |
| pushbacks | 1353 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1521 | ok |  |
| armor held | 1447 | ok |  |
| stunned | 434 | ok | injury 2-7 |
| KO | 202 | ok |  |
| casualty (d16) | 136 | ok |  |
| death | 12 | ok | d16 = 15-16 only |
| fouls | 245 | ok |  |
| argue the call | 54 | ok | referee spotted a foul (doubles) |
| argue success | 11 | ok | d6 = 6 only |
| players ejected | 44 | ok |  |
| touchdowns | 61 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 28 | ok | kickoff event roll of 8 only |
| kickoff events | 247 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 39
- Charge: 26
- Cheering Fans: 38
- Dodgy Snack: 10
- Get the Ref: 5
- High Kick: 31
- Pitch Invasion: 9
- Quick Snap: 20
- Solid Defence: 29
- Time-out: 12
- Weather Change: 28

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/dark_elf_vs_dark_elf/seed_*_rust_events.jsonl)

Total events: 103840

```
  66750 playerMoved
  14885 playerAction
   4529 goForItRoll
   3423 turnEnd
   2219 injury
   1879 blockRoll
   1879 block
   1521 playerFellDown
   1353 pushback
   1089 dodgeRoll
    614 scatterBall
    401 pickupRoll
    343 skillUse
    304 ballPickedUp
    247 kickoffScatter
    247 kickoffResultEvent
    245 refereeSpotsFoul
    245 foul
    240 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    180 passRoll
     93 handOver
     82 hitAndRun
     63 playerNote
     61 touchdown
     54 argueTheCall
     44 playerEjected
     41 kickoffExtraReRoll
     38 cheeringFans
     29 solidDefenceRoll
     28 weatherChange
     20 quickSnapRoll
     20 kickoffPitchInvasionStun
     17 throwIn
     13 regenerationRoll
     12 kickoffTimeout
     12 dodgySnackRoll
     10 kickoffDodgySnack
      9 kickoffPitchInvasion
      1 throwAtStallingPlayer
```

## Player actions declared

```
  12348 Move
    944 BlitzMove
    840 Block
    245 Foul
    188 AutoGazeZoat
    180 PassMove
     97 HandOverMove
     43 Punt
```

## Skill uses / re-rolls seen

```
```
