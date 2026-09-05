# Event coverage — HeuristicAgent, necromantic v necromantic, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=necromantic scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12005 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 673 | ok |  |
| action Blitz | 895 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 262 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 514 | ok |  |
| dodge failure | 312 | ok |  |
| GFI rolls | 5304 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 155 | ok |  |
| pickup failure | 113 | ok | turnover + scatter |
| catch success | 63 | ok |  |
| catch failure | 58 | ok |  |
| ball scatters | 576 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 16 | ok | ball out of bounds |
| pass rolls | 108 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 899 | ok |  |
| block 2 dice | 594 | ok |  |
| block 2 dice against | 237 | ok | defender's choice |
| block 3 dice | 1 | ok | needs ST5+ differential via assists |
| block result Skull | 226 | ok |  |
| block result BothDown | 290 | ok |  |
| block result Pushback | 568 | ok |  |
| block result PowPushback | 313 | ok |  |
| block result Pow | 334 | ok |  |
| pushbacks | 1082 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1738 | ok |  |
| armor held | 1801 | ok |  |
| stunned | 425 | ok | injury 2-7 |
| KO | 165 | ok |  |
| casualty (d16) | 115 | ok |  |
| death | 14 | ok | d16 = 15-16 only |
| fouls | 262 | ok |  |
| argue the call | 41 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 34 | ok |  |
| touchdowns | 31 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 24 | ok | kickoff event roll of 8 only |
| kickoff events | 227 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 32
- Charge: 20
- Cheering Fans: 44
- Dodgy Snack: 9
- Get the Ref: 6
- High Kick: 28
- Pitch Invasion: 9
- Quick Snap: 23
- Solid Defence: 17
- Time-out: 15
- Weather Change: 24

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/necromantic_vs_necromantic/seed_*_rust_events.jsonl)

Total events: 90448

```
  54788 playerMoved
  13835 playerAction
   5304 goForItRoll
   3397 turnEnd
   2506 injury
   1738 playerFellDown
   1731 blockRoll
   1731 block
   1082 pushback
    826 dodgeRoll
    576 scatterBall
    268 pickupRoll
    262 refereeSpotsFoul
    262 foul
    227 kickoffScatter
    227 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    155 ballPickedUp
    129 foulAppearanceRoll
    121 catchRoll
    117 regenerationRoll
    108 passRoll
     50 handOver
     44 cheeringFans
     41 argueTheCall
     38 kickoffExtraReRoll
     36 skillUse
     34 playerEjected
     31 touchdown
     31 playerNote
     25 kickoffPitchInvasionStun
     24 weatherChange
     23 quickSnapRoll
     17 solidDefenceRoll
     16 throwIn
     15 kickoffTimeout
      9 kickoffPitchInvasion
      9 kickoffDodgySnack
      9 dodgySnackRoll
      6 playerAdded
```

## Player actions declared

```
  11850 Move
    895 BlitzMove
    673 Block
    262 Foul
    104 PassMove
     51 HandOverMove
```

## Skill uses / re-rolls seen

```
     36 Dodge used=true
```
