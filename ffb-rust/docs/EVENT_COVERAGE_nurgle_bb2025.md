# Event coverage — HeuristicAgent, nurgle v nurgle, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=nurgle scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12370 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 665 | ok |  |
| action Blitz | 960 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 262 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 316 | ok |  |
| dodge failure | 374 | ok |  |
| GFI rolls | 5863 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 184 | ok |  |
| pickup failure | 166 | ok | turnover + scatter |
| catch success | 66 | ok |  |
| catch failure | 131 | ok |  |
| ball scatters | 569 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 6 | ok | ball out of bounds |
| pass rolls | 154 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 509 | ok |  |
| block 2 dice | 579 | ok |  |
| block 2 dice against | 252 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 180 | ok |  |
| block result BothDown | 219 | ok |  |
| block result Pushback | 438 | ok |  |
| block result PowPushback | 239 | ok |  |
| block result Pow | 264 | ok |  |
| pushbacks | 795 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1698 | ok |  |
| armor held | 1844 | ok |  |
| stunned | 292 | ok | injury 2-7 |
| KO | 123 | ok |  |
| casualty (d16) | 84 | ok |  |
| death | 17 | ok | d16 = 15-16 only |
| fouls | 245 | ok |  |
| argue the call | 43 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 36 | ok |  |
| touchdowns | 8 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 25 | ok | kickoff event roll of 8 only |
| kickoff events | 207 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 25
- Charge: 20
- Cheering Fans: 35
- Dodgy Snack: 6
- Get the Ref: 10
- High Kick: 30
- Pitch Invasion: 6
- Quick Snap: 21
- Solid Defence: 20
- Time-out: 9
- Weather Change: 25

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/nurgle_vs_nurgle/seed_*_rust_events.jsonl)

Total events: 87667

```
  49621 playerMoved
  14257 playerAction
   5863 goForItRoll
   3389 turnEnd
   2343 injury
   1698 playerFellDown
   1687 confusionRoll
   1340 blockRoll
   1340 block
   1140 foulAppearanceRoll
    795 pushback
    690 dodgeRoll
    569 scatterBall
    350 pickupRoll
    245 refereeSpotsFoul
    245 foul
    207 kickoffScatter
    207 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    197 catchRoll
    184 ballPickedUp
    154 passRoll
     99 passBlock
     71 skillUse
     58 handOver
     48 regenerationRoll
     43 argueTheCall
     36 playerEjected
     35 cheeringFans
     29 kickoffExtraReRoll
     25 weatherChange
     21 quickSnapRoll
     20 solidDefenceRoll
     12 kickoffPitchInvasionStun
      9 kickoffTimeout
      8 touchdown
      8 playerNote
      6 throwIn
      6 kickoffPitchInvasion
      6 kickoffDodgySnack
      6 dodgySnackRoll
```

## Player actions declared

```
  12159 Move
    960 BlitzMove
    665 Block
    262 Foul
    150 PassMove
     61 HandOverMove
```

## Skill uses / re-rolls seen

```
     71 Horns used=true
```
