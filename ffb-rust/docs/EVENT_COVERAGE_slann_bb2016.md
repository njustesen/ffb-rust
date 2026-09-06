# Event coverage — HeuristicAgent, slann v slann, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=slann scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12122 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 737 | ok |  |
| action Blitz | 956 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 270 | ok |  |
| action Pass | 30 | ok | needs a ball carrier |
| action HandOver | 3 | ok | needs carrier + adjacent teammate |
| dodge success | 486 | ok |  |
| dodge failure | 381 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 219 | ok |  |
| pickup failure | 107 | ok | turnover + scatter |
| catch success | 118 | ok |  |
| catch failure | 94 | ok |  |
| ball scatters | 533 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 11 | ok | ball out of bounds |
| pass rolls | 153 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 641 | ok |  |
| block 2 dice | 588 | ok |  |
| block 2 dice against | 353 | ok | defender's choice |
| block 3 dice | 61 | ok | needs ST5+ differential via assists |
| block result Skull | 284 | ok |  |
| block result BothDown | 238 | ok |  |
| block result Pushback | 545 | ok |  |
| block result PowPushback | 294 | ok |  |
| block result Pow | 282 | ok |  |
| pushbacks | 1118 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 780 | ok |  |
| armor held | 1663 | ok |  |
| stunned | 441 | ok | injury 2-7 |
| KO | 176 | ok |  |
| casualty (d16) | 117 | ok |  |
| death | 26 | ok | d16 = 15-16 only |
| fouls | 261 | ok |  |
| argue the call | 56 | ok | referee spotted a foul (doubles) |
| argue success | 5 | ok | d6 = 6 only |
| players ejected | 57 | ok |  |
| touchdowns | 42 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 39 | ok | kickoff event roll of 8 only |
| kickoff events | 234 | ok | per-result table below |

## Kickoff results

- Blitz: 14
- Brilliant Coaching: 34
- Cheering Fans: 37
- Get the Ref: 4
- High Kick: 20
- Perfect Defence: 19
- Pitch Invasion: 7
- Quick Snap: 33
- Riot: 10
- Throw a Rock: 17
- Weather Change: 39

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2016/slann_vs_slann/seed_*_rust_events.jsonl)

Total events: 30958

```
  14118 playerAction
   3418 turnEnd
   2397 injury
   1643 blockRoll
   1643 block
   1557 confusionRoll
   1118 pushback
    867 dodgeRoll
    780 playerFellDown
    533 scatterBall
    326 pickupRoll
    261 refereeSpotsFoul
    261 foul
    234 kickoffScatter
    234 kickoffResultEvent
    219 ballPickedUp
    212 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    153 passRoll
     74 handOver
     71 kickoffExtraReRollBb2016
     57 playerEjected
     56 argueTheCall
     42 touchdown
     39 weatherChange
     17 kickoffThrowARockBb2016
     11 throwIn
     10 kickoffRiot
      7 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  11916 Move
    956 Blitz
    737 Block
    270 Foul
    133 PassMove
     73 HandOverMove
     30 Pass
      3 HandOver
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
