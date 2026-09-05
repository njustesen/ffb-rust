# Event coverage — HeuristicAgent, norse v norse, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=norse scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11210 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 740 | ok |  |
| action Blitz | 1044 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 302 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 423 | ok |  |
| dodge failure | 336 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 222 | ok |  |
| pickup failure | 157 | ok | turnover + scatter |
| catch success | 83 | ok |  |
| catch failure | 109 | ok |  |
| ball scatters | 576 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 11 | ok | ball out of bounds |
| pass rolls | 149 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 886 | ok |  |
| block 2 dice | 795 | ok |  |
| block 2 dice against | 495 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 363 | ok |  |
| block result BothDown | 296 | ok |  |
| block result Pushback | 754 | ok |  |
| block result PowPushback | 364 | ok |  |
| block result Pow | 399 | ok |  |
| pushbacks | 1511 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 770 | ok |  |
| armor held | 1623 | ok |  |
| stunned | 539 | ok | injury 2-7 |
| KO | 217 | ok |  |
| casualty (d16) | 191 | ok |  |
| death | 37 | ok | d16 = 15-16 only |
| fouls | 259 | ok |  |
| argue the call | 45 | ok | referee spotted a foul (doubles) |
| argue success | 6 | ok | d6 = 6 only |
| players ejected | 48 | ok |  |
| touchdowns | 40 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 40 | ok | kickoff event roll of 8 only |
| kickoff events | 232 | ok | per-result table below |

## Kickoff results

- Blitz: 15
- Brilliant Coaching: 30
- Cheering Fans: 42
- Get the Ref: 2
- High Kick: 24
- Perfect Defence: 21
- Pitch Invasion: 3
- Quick Snap: 30
- Riot: 12
- Throw a Rock: 13
- Weather Change: 40

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/norse_vs_norse/seed_*_rust_events.jsonl)

Total events: 31674

```
  13296 playerAction
   3408 turnEnd
   2570 injury
   2176 blockRoll
   2176 block
   1511 pushback
   1508 confusionRoll
    770 playerFellDown
    759 dodgeRoll
    576 scatterBall
    379 pickupRoll
    259 refereeSpotsFoul
    259 foul
    232 kickoffScatter
    232 kickoffResultEvent
    222 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    192 catchRoll
    149 passRoll
     72 kickoffExtraReRollBb2016
     65 handOver
     51 dauntlessRoll
     48 playerEjected
     45 argueTheCall
     40 weatherChange
     40 touchdown
     13 kickoffThrowARockBb2016
     12 kickoffRiot
     11 throwIn
      3 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  10990 Move
   1044 Blitz
    740 Block
    302 Foul
    149 PassMove
     71 HandOverMove
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
