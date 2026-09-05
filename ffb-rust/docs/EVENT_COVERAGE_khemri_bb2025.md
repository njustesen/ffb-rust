# Event coverage — HeuristicAgent, khemri v khemri, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=khemri scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12300 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 675 | ok |  |
| action Blitz | 978 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 304 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 229 | ok |  |
| dodge failure | 442 | ok |  |
| GFI rolls | 5759 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 178 | ok |  |
| pickup failure | 254 | ok | turnover + scatter |
| catch success | 72 | ok |  |
| catch failure | 129 | ok |  |
| ball scatters | 614 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 6 | ok | ball out of bounds |
| pass rolls | 151 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 779 | ok |  |
| block 2 dice | 619 | ok |  |
| block 2 dice against | 251 | ok | defender's choice |
| block 3 dice | 4 | ok | needs ST5+ differential via assists |
| block result Skull | 225 | ok |  |
| block result BothDown | 258 | ok |  |
| block result Pushback | 551 | ok |  |
| block result PowPushback | 288 | ok |  |
| block result Pow | 331 | ok |  |
| pushbacks | 1168 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1772 | ok |  |
| armor held | 1969 | ok |  |
| stunned | 376 | ok | injury 2-7 |
| KO | 94 | ok |  |
| casualty (d16) | 97 | ok |  |
| death | 16 | ok | d16 = 15-16 only |
| fouls | 304 | ok |  |
| argue the call | 60 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 51 | ok |  |
| touchdowns | 6 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 18 | ok | kickoff event roll of 8 only |
| kickoff events | 205 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 27
- Charge: 21
- Cheering Fans: 35
- Dodgy Snack: 10
- Get the Ref: 6
- High Kick: 30
- Pitch Invasion: 5
- Quick Snap: 28
- Solid Defence: 15
- Time-out: 10
- Weather Change: 18

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/khemri_vs_khemri/seed_*_rust_events.jsonl)

Total events: 88588

```
  52081 playerMoved
  14257 playerAction
   5759 goForItRoll
   3385 turnEnd
   2536 injury
   1772 playerFellDown
   1653 blockRoll
   1653 block
   1168 pushback
    671 dodgeRoll
    614 scatterBall
    432 pickupRoll
    304 refereeSpotsFoul
    304 foul
    205 kickoffScatter
    205 kickoffResultEvent
    201 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    178 ballPickedUp
    151 passRoll
    100 regenerationRoll
     60 argueTheCall
     58 handOver
     51 playerEjected
     35 cheeringFans
     30 kickoffExtraReRoll
     28 quickSnapRoll
     18 weatherChange
     15 solidDefenceRoll
     12 dodgySnackRoll
     10 kickoffTimeout
     10 kickoffDodgySnack
      9 kickoffPitchInvasionStun
      6 touchdown
      6 throwIn
      6 playerNote
      5 kickoffPitchInvasion
```

## Player actions declared

```
  12103 Move
    978 BlitzMove
    675 Block
    304 Foul
    138 PassMove
     59 HandOverMove
```

## Skill uses / re-rolls seen

```
```
