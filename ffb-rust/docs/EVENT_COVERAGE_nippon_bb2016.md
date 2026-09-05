# Event coverage — HeuristicAgent, nippon v nippon, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=nippon scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13285 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 943 | ok |  |
| action Blitz | 1026 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 295 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 682 | ok |  |
| dodge failure | 335 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 272 | ok |  |
| pickup failure | 193 | ok | turnover + scatter |
| catch success | 142 | ok |  |
| catch failure | 132 | ok |  |
| ball scatters | 654 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 14 | ok | ball out of bounds |
| pass rolls | 177 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 941 | ok |  |
| block 2 dice | 759 | ok |  |
| block 2 dice against | 260 | ok | defender's choice |
| block 3 dice | 9 | ok | needs ST5+ differential via assists |
| block result Skull | 264 | ok |  |
| block result BothDown | 375 | ok |  |
| block result Pushback | 642 | ok |  |
| block result PowPushback | 318 | ok |  |
| block result Pow | 370 | ok |  |
| pushbacks | 1322 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 713 | ok |  |
| armor held | 1729 | ok |  |
| stunned | 305 | ok | injury 2-7 |
| KO | 144 | ok |  |
| casualty (d16) | 104 | ok |  |
| death | 20 | ok | d16 = 15-16 only |
| fouls | 295 | ok |  |
| argue the call | 58 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 60 | ok |  |
| touchdowns | 60 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 37 | ok | kickoff event roll of 8 only |
| kickoff events | 253 | ok | per-result table below |

## Kickoff results

- Blitz: 20
- Brilliant Coaching: 28
- Cheering Fans: 50
- Get the Ref: 7
- High Kick: 29
- Perfect Defence: 27
- Pitch Invasion: 5
- Quick Snap: 29
- Riot: 11
- Throw a Rock: 10
- Weather Change: 37

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/nippon_vs_nippon/seed_*_rust_events.jsonl)

Total events: 32588

```
  15549 playerAction
   3435 turnEnd
   2282 injury
   1969 blockRoll
   1969 block
   1322 pushback
   1017 dodgeRoll
    713 playerFellDown
    654 scatterBall
    465 pickupRoll
    295 refereeSpotsFoul
    295 foul
    274 catchRoll
    272 ballPickedUp
    253 kickoffScatter
    253 kickoffResultEvent
    248 apothecaryRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    177 passRoll
    110 skillUse
    103 handOver
     78 kickoffExtraReRollBb2016
     60 touchdown
     60 playerEjected
     58 argueTheCall
     37 weatherChange
     14 throwIn
     11 kickoffRiot
     10 kickoffThrowARockBb2016
      5 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  13012 Move
   1026 Blitz
    943 Block
    295 Foul
    167 PassMove
    106 HandOverMove
```

## Skill uses / re-rolls seen

```
    110 Dodge used=true
```
