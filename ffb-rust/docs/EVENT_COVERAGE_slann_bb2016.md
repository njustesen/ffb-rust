# Event coverage — HeuristicAgent, slann v slann, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=slann scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12134 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 736 | ok |  |
| action Blitz | 953 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 271 | ok |  |
| action Pass | 30 | ok | needs a ball carrier |
| action HandOver | 3 | ok | needs carrier + adjacent teammate |
| dodge success | 476 | ok |  |
| dodge failure | 363 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 221 | ok |  |
| pickup failure | 105 | ok | turnover + scatter |
| catch success | 120 | ok |  |
| catch failure | 96 | ok |  |
| ball scatters | 529 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 10 | ok | ball out of bounds |
| pass rolls | 152 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 632 | ok |  |
| block 2 dice | 585 | ok |  |
| block 2 dice against | 357 | ok | defender's choice |
| block 3 dice | 63 | ok | needs ST5+ differential via assists |
| block result Skull | 281 | ok |  |
| block result BothDown | 241 | ok |  |
| block result Pushback | 541 | ok |  |
| block result PowPushback | 294 | ok |  |
| block result Pow | 280 | ok |  |
| pushbacks | 1112 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 764 | ok |  |
| armor held | 1647 | ok |  |
| stunned | 447 | ok | injury 2-7 |
| KO | 171 | ok |  |
| casualty (d16) | 117 | ok |  |
| death | 25 | ok | d16 = 15-16 only |
| fouls | 261 | ok |  |
| argue the call | 54 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 55 | ok |  |
| touchdowns | 47 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 37 | ok | kickoff event roll of 8 only |
| kickoff events | 239 | ok | per-result table below |

## Kickoff results

- Blitz: 14
- Brilliant Coaching: 35
- Cheering Fans: 40
- Get the Ref: 4
- High Kick: 22
- Perfect Defence: 20
- Pitch Invasion: 7
- Quick Snap: 33
- Riot: 10
- Throw a Rock: 17
- Weather Change: 37

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2016/slann_vs_slann/seed_*_rust_events.jsonl)

Total events: 30959

```
  14127 playerAction
   3423 turnEnd
   2382 injury
   1637 blockRoll
   1637 block
   1578 confusionRoll
   1112 pushback
    839 dodgeRoll
    764 playerFellDown
    529 scatterBall
    326 pickupRoll
    261 refereeSpotsFoul
    261 foul
    239 kickoffScatter
    239 kickoffResultEvent
    221 ballPickedUp
    216 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    152 passRoll
     76 handOver
     75 kickoffExtraReRollBb2016
     55 playerEjected
     54 argueTheCall
     47 touchdown
     37 weatherChange
     28 jumpRoll
     17 kickoffThrowARockBb2016
     10 throwIn
     10 kickoffRiot
      7 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  11930 Move
    953 Blitz
    736 Block
    271 Foul
    130 PassMove
     74 HandOverMove
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
