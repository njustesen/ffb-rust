# Event coverage — HeuristicAgent, chaos_pact v chaos_pact, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-01 by `MATCHUP=chaos_pact scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12310 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 643 | ok |  |
| action Blitz | 960 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 303 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 362 | ok |  |
| dodge failure | 315 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 215 | ok |  |
| pickup failure | 126 | ok | turnover + scatter |
| catch success | 121 | ok |  |
| catch failure | 121 | ok |  |
| ball scatters | 555 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 5 | ok | ball out of bounds |
| pass rolls | 172 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 634 | ok |  |
| block 2 dice | 592 | ok |  |
| block 2 dice against | 208 | ok | defender's choice |
| block 3 dice | 17 | ok | needs ST5+ differential via assists |
| block result Skull | 189 | ok |  |
| block result BothDown | 238 | ok |  |
| block result Pushback | 473 | ok |  |
| block result PowPushback | 266 | ok |  |
| block result Pow | 285 | ok |  |
| pushbacks | 1019 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 853 | ok |  |
| armor held | 1617 | ok |  |
| stunned | 411 | ok | injury 2-7 |
| KO | 144 | ok |  |
| casualty (d16) | 131 | ok |  |
| death | 22 | ok | d16 = 15-16 only |
| fouls | 222 | ok |  |
| argue the call | 41 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 41 | ok |  |
| touchdowns | 21 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 35 | ok | kickoff event roll of 8 only |
| kickoff events | 213 | ok | per-result table below |

## Kickoff results

- Blitz: 15
- Brilliant Coaching: 28
- Cheering Fans: 33
- Get the Ref: 3
- High Kick: 24
- Perfect Defence: 24
- Pitch Invasion: 3
- Quick Snap: 28
- Riot: 10
- Throw a Rock: 10
- Weather Change: 35

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/chaos_pact_vs_chaos_pact/seed_*_rust_events.jsonl)

Total events: 34310

```
  14787 playerAction
   4536 confusionRoll
   3393 turnEnd
   2303 injury
   1451 blockRoll
   1451 block
   1019 pushback
    853 playerFellDown
    677 dodgeRoll
    555 scatterBall
    341 pickupRoll
    275 apothecaryRoll
    264 skillUse
    242 catchRoll
    222 refereeSpotsFoul
    222 foul
    215 ballPickedUp
    213 kickoffScatter
    213 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    172 passRoll
     75 handOver
     61 kickoffExtraReRollBb2016
     41 playerEjected
     41 argueTheCall
     35 weatherChange
     21 touchdown
     10 kickoffThrowARockBb2016
     10 kickoffRiot
      5 throwIn
      4 throwTeamMateRoll
      3 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  12048 Move
    960 Blitz
    643 Block
    571 ThrowTeamMate
    303 Foul
    183 PassMove
     79 HandOverMove
```

## Skill uses / re-rolls seen

```
```
