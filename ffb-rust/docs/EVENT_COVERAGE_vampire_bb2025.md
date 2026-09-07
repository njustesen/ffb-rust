# Event coverage — HeuristicAgent, vampire v vampire, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-07 by `MATCHUP=vampire scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 8739 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 519 | ok |  |
| action Blitz | 681 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 197 | ok |  |
| action Pass | 17 | ok | needs a ball carrier |
| action HandOver | 2 | ok | needs carrier + adjacent teammate |
| dodge success | 408 | ok |  |
| dodge failure | 161 | ok |  |
| GFI rolls | 2994 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 178 | ok |  |
| pickup failure | 74 | ok | turnover + scatter |
| catch success | 81 | ok |  |
| catch failure | 61 | ok |  |
| ball scatters | 458 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 4 | ok | ball out of bounds |
| pass rolls | 109 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 430 | ok |  |
| block 2 dice | 607 | ok |  |
| block 2 dice against | 267 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 188 | ok |  |
| block result BothDown | 180 | ok |  |
| block result Pushback | 411 | ok |  |
| block result PowPushback | 273 | ok |  |
| block result Pow | 252 | ok |  |
| pushbacks | 967 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1160 | ok |  |
| armor held | 1143 | ok |  |
| stunned | 310 | ok | injury 2-7 |
| KO | 139 | ok |  |
| casualty (d16) | 102 | ok |  |
| death | 10 | ok | d16 = 15-16 only |
| fouls | 197 | ok |  |
| argue the call | 47 | ok | referee spotted a foul (doubles) |
| argue success | 10 | ok | d6 = 6 only |
| players ejected | 37 | ok |  |
| touchdowns | 17 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 21 | ok | kickoff event roll of 8 only |
| kickoff events | 215 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 37
- Charge: 24
- Cheering Fans: 33
- Dodgy Snack: 7
- Get the Ref: 7
- High Kick: 29
- Pitch Invasion: 7
- Quick Snap: 23
- Solid Defence: 18
- Time-out: 9
- Weather Change: 21

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2025/vampire_vs_vampire/seed_*_rust_events.jsonl)

Total events: 74035

```
  39534 playerMoved
  10155 playerAction
   6592 bloodLustRoll
   3401 turnEnd
   2994 goForItRoll
   1694 injury
   1318 biteSpectator
   1304 blockRoll
   1304 block
   1160 playerFellDown
    967 pushback
    569 dodgeRoll
    458 scatterBall
    252 pickupRoll
    215 kickoffScatter
    215 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    197 refereeSpotsFoul
    197 foul
    178 ballPickedUp
    142 catchRoll
    109 passRoll
     78 regenerationRoll
     57 handOver
     47 argueTheCall
     43 kickoffExtraReRoll
     37 playerEjected
     36 skillUse
     33 cheeringFans
     23 quickSnapRoll
     21 weatherChange
     18 solidDefenceRoll
     17 touchdown
     17 playerNote
     12 kickoffPitchInvasionStun
      9 kickoffTimeout
      7 playerAdded
      7 kickoffPitchInvasion
      7 kickoffDodgySnack
      7 dodgySnackRoll
      4 throwIn
```

## Player actions declared

```
   8578 Move
    681 BlitzMove
    519 Block
    197 Foul
    100 PassMove
     61 HandOverMove
     17 Pass
      2 HandOver
```

## Skill uses / re-rolls seen

```
     34 Juggernaut used=true
      2 Juggernaut used=false
```
