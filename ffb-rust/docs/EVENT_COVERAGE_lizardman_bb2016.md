# Event coverage — HeuristicAgent, lizardman v lizardman, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=lizardman scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11616 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 797 | ok |  |
| action Blitz | 1007 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 285 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 262 | ok |  |
| dodge failure | 439 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 182 | ok |  |
| pickup failure | 257 | ok | turnover + scatter |
| catch success | 55 | ok |  |
| catch failure | 111 | ok |  |
| ball scatters | 649 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 7 | ok | ball out of bounds |
| pass rolls | 119 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 656 | ok |  |
| block 2 dice | 662 | ok |  |
| block 2 dice against | 333 | ok | defender's choice |
| block 3 dice | 90 | ok | needs ST5+ differential via assists |
| block result Skull | 260 | ok |  |
| block result BothDown | 284 | ok |  |
| block result Pushback | 608 | ok |  |
| block result PowPushback | 277 | ok |  |
| block result Pow | 312 | ok |  |
| pushbacks | 1192 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 813 | ok |  |
| armor held | 1911 | ok |  |
| stunned | 279 | ok | injury 2-7 |
| KO | 126 | ok |  |
| casualty (d16) | 156 | ok |  |
| death | 24 | ok | d16 = 15-16 only |
| fouls | 273 | ok |  |
| argue the call | 44 | ok | referee spotted a foul (doubles) |
| argue success | 5 | ok | d6 = 6 only |
| players ejected | 46 | ok |  |
| touchdowns | 26 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 45 | ok | kickoff event roll of 8 only |
| kickoff events | 226 | ok | per-result table below |

## Kickoff results

- Blitz: 11
- Brilliant Coaching: 25
- Cheering Fans: 31
- Get the Ref: 5
- High Kick: 23
- Perfect Defence: 22
- Pitch Invasion: 6
- Quick Snap: 29
- Riot: 9
- Throw a Rock: 20
- Weather Change: 45

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/lizardman_vs_lizardman/seed_*_rust_events.jsonl)

Total events: 31015

```
  13705 playerAction
   3408 turnEnd
   2472 injury
   1741 blockRoll
   1741 block
   1439 confusionRoll
   1192 pushback
    813 playerFellDown
    701 dodgeRoll
    649 scatterBall
    439 pickupRoll
    282 apothecaryRoll
    273 refereeSpotsFoul
    273 foul
    226 kickoffScatter
    226 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    182 ballPickedUp
    166 catchRoll
    119 passRoll
     58 skillUse
     56 kickoffExtraReRollBb2016
     51 handOver
     46 playerEjected
     45 weatherChange
     44 argueTheCall
     26 touchdown
     20 kickoffThrowARockBb2016
      9 kickoffRiot
      7 throwIn
      6 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  11444 Move
   1007 Blitz
    797 Block
    285 Foul
    121 PassMove
     51 HandOverMove
```

## Skill uses / re-rolls seen

```
     58 Dodge used=true
```
