# Event coverage — HeuristicAgent, slann v slann, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=slann scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11953 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 731 | ok |  |
| action Blitz | 831 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 217 | ok |  |
| action Pass | 31 | ok | needs a ball carrier |
| action HandOver | 5 | ok | needs carrier + adjacent teammate |
| dodge success | 610 | ok |  |
| dodge failure | 311 | ok |  |
| GFI rolls | 3873 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 237 | ok |  |
| pickup failure | 157 | ok | turnover + scatter |
| catch success | 93 | ok |  |
| catch failure | 78 | ok |  |
| ball scatters | 590 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 23 | ok | ball out of bounds |
| pass rolls | 141 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 532 | ok |  |
| block 2 dice | 560 | ok |  |
| block 2 dice against | 346 | ok | defender's choice |
| block 3 dice | 60 | ok | needs ST5+ differential via assists |
| block result Skull | 209 | ok |  |
| block result BothDown | 220 | ok |  |
| block result Pushback | 518 | ok |  |
| block result PowPushback | 295 | ok |  |
| block result Pow | 256 | ok |  |
| pushbacks | 1067 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1536 | ok |  |
| armor held | 1189 | ok |  |
| stunned | 571 | ok | injury 2-7 |
| KO | 235 | ok |  |
| casualty (d16) | 182 | ok |  |
| death | 21 | ok | d16 = 15-16 only |
| fouls | 210 | ok |  |
| argue the call | 45 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 36 | ok |  |
| touchdowns | 33 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 20 | ok | kickoff event roll of 8 only |
| kickoff events | 225 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 31
- Charge: 22
- Cheering Fans: 39
- Dodgy Snack: 14
- Get the Ref: 7
- High Kick: 27
- Pitch Invasion: 10
- Quick Snap: 24
- Solid Defence: 19
- Time-out: 12
- Weather Change: 20

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2025/slann_vs_slann/seed_*_rust_events.jsonl)

Total events: 96638

```
  61870 playerMoved
  13768 playerAction
   3873 goForItRoll
   3409 turnEnd
   2177 injury
   1542 confusionRoll
   1536 playerFellDown
   1498 blockRoll
   1498 block
   1067 pushback
    921 dodgeRoll
    590 scatterBall
    394 pickupRoll
    237 ballPickedUp
    225 kickoffScatter
    225 kickoffResultEvent
    210 refereeSpotsFoul
    210 foul
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    171 catchRoll
    141 passRoll
     84 handOver
     45 argueTheCall
     39 cheeringFans
     38 kickoffExtraReRoll
     36 playerEjected
     34 playerNote
     33 touchdown
     24 quickSnapRoll
     23 throwIn
     20 weatherChange
     19 solidDefenceRoll
     19 kickoffPitchInvasionStun
     17 dodgySnackRoll
     14 kickoffDodgySnack
     12 kickoffTimeout
     10 kickoffPitchInvasion
      9 jumpRoll
```

## Player actions declared

```
  11756 Move
    831 BlitzMove
    731 Block
    217 Foul
    114 PassMove
     83 HandOverMove
     31 Pass
      5 HandOver
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
