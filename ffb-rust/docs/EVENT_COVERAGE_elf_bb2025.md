# Event coverage — HeuristicAgent, elf v elf, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-02 by `MATCHUP=elf scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12955 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 812 | ok |  |
| action Blitz | 978 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 254 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 998 | ok |  |
| dodge failure | 227 | ok |  |
| GFI rolls | 4708 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 311 | ok |  |
| pickup failure | 73 | ok | turnover + scatter |
| catch success | 208 | ok |  |
| catch failure | 69 | ok |  |
| ball scatters | 574 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 14 | ok | ball out of bounds |
| pass rolls | 220 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1294 | ok |  |
| block 2 dice | 380 | ok |  |
| block 2 dice against | 116 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 268 | ok |  |
| block result BothDown | 291 | ok |  |
| block result Pushback | 602 | ok |  |
| block result PowPushback | 305 | ok |  |
| block result Pow | 324 | ok |  |
| pushbacks | 1225 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1538 | ok |  |
| armor held | 1380 | ok |  |
| stunned | 473 | ok | injury 2-7 |
| KO | 212 | ok |  |
| casualty (d16) | 138 | ok |  |
| death | 14 | ok | d16 = 15-16 only |
| fouls | 254 | ok |  |
| argue the call | 46 | ok | referee spotted a foul (doubles) |
| argue success | 10 | ok | d6 = 6 only |
| players ejected | 37 | ok |  |
| touchdowns | 87 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 41 | ok | kickoff event roll of 8 only |
| kickoff events | 277 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 40
- Charge: 26
- Cheering Fans: 42
- Dodgy Snack: 6
- Get the Ref: 8
- High Kick: 41
- Pitch Invasion: 9
- Quick Snap: 24
- Solid Defence: 24
- Time-out: 16
- Weather Change: 41

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/elf_vs_elf/seed_*_rust_events.jsonl)

Total events: 105874

```
  68880 playerMoved
  15027 playerAction
   4708 goForItRoll
   3453 turnEnd
   2203 injury
   1790 blockRoll
   1790 block
   1538 playerFellDown
   1225 pushback
   1225 dodgeRoll
    574 scatterBall
    384 pickupRoll
    311 ballPickedUp
    277 kickoffScatter
    277 kickoffResultEvent
    277 catchRoll
    254 refereeSpotsFoul
    254 foul
    220 passRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    104 handOver
     87 touchdown
     87 playerNote
     48 kickoffExtraReRoll
     46 argueTheCall
     42 cheeringFans
     41 weatherChange
     37 playerEjected
     24 solidDefenceRoll
     24 quickSnapRoll
     16 kickoffTimeout
     16 kickoffPitchInvasionStun
     14 throwIn
      9 kickoffPitchInvasion
      6 kickoffDodgySnack
      6 dodgySnackRoll
```

## Player actions declared

```
  12629 Move
    978 BlitzMove
    812 Block
    254 Foul
    217 PassMove
    109 HandOverMove
     28 HailMaryPass
```

## Skill uses / re-rolls seen

```
```
