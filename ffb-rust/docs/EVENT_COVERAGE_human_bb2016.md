# Event coverage — HeuristicAgent, human v human, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=human scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12609 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 892 | ok |  |
| action Blitz | 1047 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 267 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 691 | ok |  |
| dodge failure | 341 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 253 | ok |  |
| pickup failure | 134 | ok | turnover + scatter |
| catch success | 186 | ok |  |
| catch failure | 162 | ok |  |
| ball scatters | 616 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 14 | ok | ball out of bounds |
| pass rolls | 166 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 819 | ok |  |
| block 2 dice | 622 | ok |  |
| block 2 dice against | 356 | ok | defender's choice |
| block 3 dice | 65 | ok | needs ST5+ differential via assists |
| block result Skull | 266 | ok |  |
| block result BothDown | 291 | ok |  |
| block result Pushback | 690 | ok |  |
| block result PowPushback | 282 | ok |  |
| block result Pow | 333 | ok |  |
| pushbacks | 1300 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 770 | ok |  |
| armor held | 1453 | ok |  |
| stunned | 397 | ok | injury 2-7 |
| KO | 158 | ok |  |
| casualty (d16) | 135 | ok |  |
| death | 26 | ok | d16 = 15-16 only |
| fouls | 262 | ok |  |
| argue the call | 63 | ok | referee spotted a foul (doubles) |
| argue success | 15 | ok | d6 = 6 only |
| players ejected | 66 | ok |  |
| touchdowns | 80 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 52 | ok | kickoff event roll of 8 only |
| kickoff events | 272 | ok | per-result table below |

## Kickoff results

- Blitz: 21
- Brilliant Coaching: 24
- Cheering Fans: 41
- Get the Ref: 8
- High Kick: 32
- Perfect Defence: 28
- Pitch Invasion: 2
- Quick Snap: 35
- Riot: 16
- Throw a Rock: 13
- Weather Change: 52

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/human_vs_human/seed_*_rust_events.jsonl)

Total events: 32908

```
  15099 playerAction
   3456 turnEnd
   2143 injury
   1862 blockRoll
   1862 block
   1405 confusionRoll
   1300 pushback
   1032 dodgeRoll
    770 playerFellDown
    616 scatterBall
    387 pickupRoll
    348 catchRoll
    272 kickoffScatter
    272 kickoffResultEvent
    262 refereeSpotsFoul
    262 foul
    253 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    166 passRoll
    115 handOver
     80 touchdown
     66 playerEjected
     65 kickoffExtraReRollBb2016
     63 argueTheCall
     55 skillUse
     52 weatherChange
     16 kickoffRiot
     14 throwIn
     13 kickoffThrowARockBb2016
      2 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  12308 Move
   1047 Blitz
    892 Block
    284 ThrowTeamMate
    267 Foul
    179 PassMove
    122 HandOverMove
```

## Skill uses / re-rolls seen

```
```
