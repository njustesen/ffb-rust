# Event coverage — HeuristicAgent, skaven v skaven, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=skaven scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12370 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 976 | ok |  |
| action Blitz | 1026 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 291 | ok |  |
| action Pass | 34 | ok | needs a ball carrier |
| action HandOver | 4 | ok | needs carrier + adjacent teammate |
| dodge success | 748 | ok |  |
| dodge failure | 339 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 303 | ok |  |
| pickup failure | 112 | ok | turnover + scatter |
| catch success | 130 | ok |  |
| catch failure | 111 | ok |  |
| ball scatters | 630 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 13 | ok | ball out of bounds |
| pass rolls | 174 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 777 | ok |  |
| block 2 dice | 775 | ok |  |
| block 2 dice against | 394 | ok | defender's choice |
| block 3 dice | 139 | ok | needs ST5+ differential via assists |
| block result Skull | 297 | ok |  |
| block result BothDown | 314 | ok |  |
| block result Pushback | 734 | ok |  |
| block result PowPushback | 330 | ok |  |
| block result Pow | 410 | ok |  |
| pushbacks | 1466 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 698 | ok |  |
| armor held | 1445 | ok |  |
| stunned | 464 | ok | injury 2-7 |
| KO | 235 | ok |  |
| casualty (d16) | 168 | ok |  |
| death | 32 | ok | d16 = 15-16 only |
| fouls | 260 | ok |  |
| argue the call | 51 | ok | referee spotted a foul (doubles) |
| argue success | 8 | ok | d6 = 6 only |
| players ejected | 51 | ok |  |
| touchdowns | 87 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 44 | ok | kickoff event roll of 8 only |
| kickoff events | 278 | ok | per-result table below |

## Kickoff results

- Blitz: 17
- Brilliant Coaching: 40
- Cheering Fans: 40
- Get the Ref: 6
- High Kick: 31
- Perfect Defence: 25
- Pitch Invasion: 5
- Quick Snap: 33
- Riot: 16
- Throw a Rock: 21
- Weather Change: 44

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2016/skaven_vs_skaven/seed_*_rust_events.jsonl)

Total events: 33526

```
  14701 playerAction
   3454 turnEnd
   2312 injury
   2085 blockRoll
   2085 block
   1678 confusionRoll
   1466 pushback
   1087 dodgeRoll
    698 playerFellDown
    630 scatterBall
    415 pickupRoll
    303 ballPickedUp
    278 kickoffScatter
    278 kickoffResultEvent
    260 refereeSpotsFoul
    260 foul
    241 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    174 passRoll
     87 touchdown
     81 handOver
     80 kickoffExtraReRollBb2016
     71 skillUse
     51 playerEjected
     51 argueTheCall
     44 weatherChange
     21 kickoffThrowARockBb2016
     16 kickoffRiot
     13 throwIn
      5 kickoffPitchInvasionBb2016
      1 playerNote
```

## Player actions declared

```
  12154 Move
   1026 Blitz
    976 Block
    291 Foul
    134 PassMove
     82 HandOverMove
     34 Pass
      4 HandOver
```

## Skill uses / re-rolls seen

```
     71 Dodge used=true
```
