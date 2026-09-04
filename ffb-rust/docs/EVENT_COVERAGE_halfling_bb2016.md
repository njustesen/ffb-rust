# Event coverage — HeuristicAgent, halfling v halfling, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=halfling scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12304 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 610 | ok |  |
| action Blitz | 936 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 356 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 374 | ok |  |
| dodge failure | 212 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 203 | ok |  |
| pickup failure | 129 | ok | turnover + scatter |
| catch success | 127 | ok |  |
| catch failure | 126 | ok |  |
| ball scatters | 532 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 6 | ok | ball out of bounds |
| pass rolls | 184 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 655 | ok |  |
| block 2 dice | 219 | ok |  |
| block 2 dice against | 124 | ok | defender's choice |
| block 3 dice | 367 | ok | needs ST5+ differential via assists |
| block result Skull | 200 | ok |  |
| block result BothDown | 226 | ok |  |
| block result Pushback | 471 | ok |  |
| block result PowPushback | 200 | ok |  |
| block result Pow | 268 | ok |  |
| pushbacks | 938 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 793 | ok |  |
| armor held | 1337 | ok |  |
| stunned | 577 | ok | injury 2-7 |
| KO | 216 | ok |  |
| casualty (d16) | 174 | ok |  |
| death | 23 | ok | d16 = 15-16 only |
| fouls | 356 | ok |  |
| argue the call | 65 | ok | referee spotted a foul (doubles) |
| argue success | 11 | ok | d6 = 6 only |
| players ejected | 67 | ok |  |
| touchdowns | 11 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 25 | ok | kickoff event roll of 8 only |
| kickoff events | 210 | ok | per-result table below |

## Kickoff results

- Blitz: 17
- Brilliant Coaching: 22
- Cheering Fans: 33
- Get the Ref: 7
- High Kick: 23
- Perfect Defence: 22
- Pitch Invasion: 5
- Quick Snap: 31
- Riot: 17
- Throw a Rock: 8
- Weather Change: 25

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/halfling_vs_halfling/seed_*_rust_events.jsonl)

Total events: 30673

```
  14745 playerAction
   3380 turnEnd
   2304 injury
   1365 blockRoll
   1365 block
    938 pushback
    917 standUpRoll
    793 playerFellDown
    586 dodgeRoll
    532 scatterBall
    390 apothecaryRoll
    356 refereeSpotsFoul
    356 foul
    332 pickupRoll
    253 catchRoll
    210 kickoffScatter
    210 kickoffResultEvent
    203 ballPickedUp
    201 throwTeamMateRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    184 passRoll
    105 skillUse
     89 handOver
     67 playerEjected
     65 argueTheCall
     55 kickoffExtraReRollBb2016
     25 weatherChange
     17 kickoffRiot
     11 touchdown
      8 kickoffThrowARockBb2016
      6 throwIn
      5 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  12031 Move
    936 Blitz
    610 Block
    539 ThrowTeamMate
    356 Foul
    184 PassMove
     89 HandOverMove
```

## Skill uses / re-rolls seen

```
```
