# Event coverage — HeuristicAgent, slann v slann, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=slann scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11965 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 731 | ok |  |
| action Blitz | 823 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 220 | ok |  |
| action Pass | 32 | ok | needs a ball carrier |
| action HandOver | 5 | ok | needs carrier + adjacent teammate |
| dodge success | 614 | ok |  |
| dodge failure | 315 | ok |  |
| GFI rolls | 3876 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 238 | ok |  |
| pickup failure | 155 | ok | turnover + scatter |
| catch success | 93 | ok |  |
| catch failure | 78 | ok |  |
| ball scatters | 589 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 22 | ok | ball out of bounds |
| pass rolls | 142 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 528 | ok |  |
| block 2 dice | 558 | ok |  |
| block 2 dice against | 344 | ok | defender's choice |
| block 3 dice | 60 | ok | needs ST5+ differential via assists |
| block result Skull | 206 | ok |  |
| block result BothDown | 221 | ok |  |
| block result Pushback | 514 | ok |  |
| block result PowPushback | 295 | ok |  |
| block result Pow | 254 | ok |  |
| pushbacks | 1060 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1533 | ok |  |
| armor held | 1192 | ok |  |
| stunned | 572 | ok | injury 2-7 |
| KO | 233 | ok |  |
| casualty (d16) | 179 | ok |  |
| death | 21 | ok | d16 = 15-16 only |
| fouls | 213 | ok |  |
| argue the call | 46 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 37 | ok |  |
| touchdowns | 33 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 21 | ok | kickoff event roll of 8 only |
| kickoff events | 225 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 31
- Charge: 21
- Cheering Fans: 39
- Dodgy Snack: 14
- Get the Ref: 6
- High Kick: 27
- Pitch Invasion: 10
- Quick Snap: 24
- Solid Defence: 20
- Time-out: 12
- Weather Change: 21

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2025/slann_vs_slann/seed_*_rust_events.jsonl)

Total events: 96705

```
  61948 playerMoved
  13776 playerAction
   3876 goForItRoll
   3409 turnEnd
   2176 injury
   1539 confusionRoll
   1533 playerFellDown
   1490 blockRoll
   1490 block
   1060 pushback
    929 dodgeRoll
    589 scatterBall
    393 pickupRoll
    238 ballPickedUp
    225 kickoffScatter
    225 kickoffResultEvent
    213 refereeSpotsFoul
    213 foul
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    171 catchRoll
    142 passRoll
     84 handOver
     46 argueTheCall
     39 cheeringFans
     38 kickoffExtraReRoll
     37 playerEjected
     34 playerNote
     33 touchdown
     24 quickSnapRoll
     22 throwIn
     21 weatherChange
     20 solidDefenceRoll
     19 kickoffPitchInvasionStun
     17 dodgySnackRoll
     14 kickoffDodgySnack
     12 kickoffTimeout
     10 kickoffPitchInvasion
```

## Player actions declared

```
  11768 Move
    823 BlitzMove
    731 Block
    220 Foul
    114 PassMove
     83 HandOverMove
     32 Pass
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
