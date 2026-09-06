# Event coverage — HeuristicAgent, orc v orc, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=orc scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12222 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 643 | ok |  |
| action Blitz | 821 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 255 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 351 | ok |  |
| dodge failure | 354 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 231 | ok |  |
| pickup failure | 138 | ok | turnover + scatter |
| catch success | 133 | ok |  |
| catch failure | 112 | ok |  |
| ball scatters | 580 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 8 | ok | ball out of bounds |
| pass rolls | 169 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 545 | ok |  |
| block 2 dice | 497 | ok |  |
| block 2 dice against | 324 | ok | defender's choice |
| block 3 dice | 2 | ok | needs ST5+ differential via assists |
| block result Skull | 183 | ok |  |
| block result BothDown | 211 | ok |  |
| block result Pushback | 451 | ok |  |
| block result PowPushback | 248 | ok |  |
| block result Pow | 275 | ok |  |
| pushbacks | 972 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1044 | ok |  |
| armor held | 1890 | ok |  |
| stunned | 234 | ok | injury 2-7 |
| KO | 126 | ok |  |
| casualty (d16) | 87 | ok |  |
| death | 18 | ok | d16 = 15-16 only |
| fouls | 228 | ok |  |
| argue the call | 39 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 40 | ok |  |
| touchdowns | 13 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 26 | ok | kickoff event roll of 8 only |
| kickoff events | 212 | ok | per-result table below |

## Kickoff results

- Blitz: 15
- Brilliant Coaching: 25
- Cheering Fans: 34
- Get the Ref: 5
- High Kick: 24
- Perfect Defence: 26
- Pitch Invasion: 5
- Quick Snap: 27
- Riot: 10
- Throw a Rock: 15
- Weather Change: 26

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/orc_vs_orc/seed_*_rust_events.jsonl)

Total events: 30298

```
  14245 playerAction
   3392 turnEnd
   2337 injury
   1454 confusionRoll
   1368 blockRoll
   1368 block
   1044 playerFellDown
    972 pushback
    705 dodgeRoll
    580 scatterBall
    369 pickupRoll
    245 catchRoll
    231 ballPickedUp
    228 refereeSpotsFoul
    228 foul
    212 kickoffScatter
    212 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    169 passRoll
    109 handOver
     59 kickoffExtraReRollBb2016
     40 playerEjected
     39 argueTheCall
     26 weatherChange
     15 kickoffThrowARockBb2016
     13 touchdown
     11 regenerationRoll
     10 kickoffRiot
      8 throwIn
      5 kickoffPitchInvasionBb2016
      2 throwTeamMateRoll
      2 skillUse
```

## Player actions declared

```
  11932 Move
    821 Blitz
    643 Block
    304 ThrowTeamMate
    255 Foul
    175 PassMove
    115 HandOverMove
```

## Skill uses / re-rolls seen

```
      2 Dodge used=true
```
