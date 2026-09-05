# Event coverage — HeuristicAgent, chaos v chaos, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=chaos scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12108 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 698 | ok |  |
| action Blitz | 1053 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 295 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 462 | ok |  |
| dodge failure | 295 | ok |  |
| GFI rolls | 4994 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 217 | ok |  |
| pickup failure | 121 | ok | turnover + scatter |
| catch success | 130 | ok |  |
| catch failure | 88 | ok |  |
| ball scatters | 556 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 8 | ok | ball out of bounds |
| pass rolls | 159 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 531 | ok |  |
| block 2 dice | 1038 | ok |  |
| block 2 dice against | 240 | ok | defender's choice |
| block 3 dice | 4 | ok | needs ST5+ differential via assists |
| block result Skull | 198 | ok |  |
| block result BothDown | 272 | ok |  |
| block result Pushback | 580 | ok |  |
| block result PowPushback | 358 | ok |  |
| block result Pow | 405 | ok |  |
| pushbacks | 1340 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1916 | ok |  |
| armor held | 1976 | ok |  |
| stunned | 445 | ok | injury 2-7 |
| KO | 112 | ok |  |
| casualty (d16) | 112 | ok |  |
| death | 12 | ok | d16 = 15-16 only |
| fouls | 256 | ok |  |
| argue the call | 57 | ok | referee spotted a foul (doubles) |
| argue success | 14 | ok | d6 = 6 only |
| players ejected | 44 | ok |  |
| touchdowns | 24 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 29 | ok | kickoff event roll of 8 only |
| kickoff events | 219 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 29
- Charge: 23
- Cheering Fans: 34
- Dodgy Snack: 13
- Get the Ref: 9
- High Kick: 25
- Pitch Invasion: 7
- Quick Snap: 23
- Solid Defence: 19
- Time-out: 8
- Weather Change: 29

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/chaos_vs_chaos/seed_*_rust_events.jsonl)

Total events: 92676

```
  54055 playerMoved
  14154 playerAction
   4994 goForItRoll
   3403 turnEnd
   2645 injury
   1916 playerFellDown
   1813 blockRoll
   1813 block
   1470 confusionRoll
   1340 pushback
    757 dodgeRoll
    609 skillUse
    556 scatterBall
    338 pickupRoll
    256 refereeSpotsFoul
    256 foul
    224 apothecaryRoll
    219 kickoffScatter
    219 kickoffResultEvent
    218 catchRoll
    217 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    159 passRoll
     90 handOver
     57 argueTheCall
     44 playerEjected
     34 kickoffExtraReRoll
     34 cheeringFans
     29 weatherChange
     25 playerNote
     24 touchdown
     23 quickSnapRoll
     19 solidDefenceRoll
     15 kickoffPitchInvasionStun
     15 dodgySnackRoll
     13 kickoffDodgySnack
      8 throwIn
      8 kickoffTimeout
      7 kickoffPitchInvasion
```

## Player actions declared

```
  11851 Move
   1053 BlitzMove
    698 Block
    295 Foul
    164 PassMove
     93 HandOverMove
```

## Skill uses / re-rolls seen

```
    609 Horns used=true
```
