# Event coverage — HeuristicAgent, dwarf v dwarf, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-02 by `MATCHUP=dwarf scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11889 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 705 | ok |  |
| action Blitz | 1005 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 300 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 430 | ok |  |
| dodge failure | 318 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 232 | ok |  |
| pickup failure | 120 | ok | turnover + scatter |
| catch success | 90 | ok |  |
| catch failure | 106 | ok |  |
| ball scatters | 588 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 9 | ok | ball out of bounds |
| pass rolls | 157 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1213 | ok |  |
| block 2 dice | 307 | ok |  |
| block 2 dice against | 140 | ok | defender's choice |
| block 3 dice | 306 | ok | needs ST5+ differential via assists |
| block result Skull | 302 | ok |  |
| block result BothDown | 261 | ok |  |
| block result Pushback | 647 | ok |  |
| block result PowPushback | 377 | ok |  |
| block result Pow | 379 | ok |  |
| pushbacks | 1409 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 986 | ok |  |
| armor held | 1831 | ok |  |
| stunned | 424 | ok | injury 2-7 |
| KO | 69 | ok |  |
| casualty (d16) | 112 | ok |  |
| death | 15 | ok | d16 = 15-16 only |
| fouls | 300 | ok |  |
| argue the call | 59 | ok | referee spotted a foul (doubles) |
| argue success | 11 | ok | d6 = 6 only |
| players ejected | 63 | ok |  |
| touchdowns | 17 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 30 | ok | kickoff event roll of 8 only |
| kickoff events | 214 | ok | per-result table below |

## Kickoff results

- Blitz: 11
- Brilliant Coaching: 27
- Cheering Fans: 42
- Get the Ref: 5
- High Kick: 26
- Perfect Defence: 20
- Pitch Invasion: 4
- Quick Snap: 26
- Riot: 12
- Throw a Rock: 11
- Weather Change: 30

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/dwarf_vs_dwarf/seed_*_rust_events.jsonl)

Total events: 30509

```
  13907 playerAction
   3394 turnEnd
   2436 injury
   1966 blockRoll
   1966 block
   1409 pushback
    986 playerFellDown
    748 dodgeRoll
    588 scatterBall
    352 pickupRoll
    300 refereeSpotsFoul
    300 foul
    232 ballPickedUp
    214 kickoffScatter
    214 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    196 catchRoll
    174 dauntlessRoll
    157 passRoll
     85 handOver
     69 kickoffExtraReRollBb2016
     63 playerEjected
     59 argueTheCall
     30 weatherChange
     17 touchdown
     12 kickoffRiot
     11 skillUse
     11 kickoffThrowARockBb2016
      9 throwIn
      4 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  11638 Move
   1005 Blitz
    705 Block
    300 Foul
    161 PassMove
     90 HandOverMove
      8 HailMaryPass
```

## Skill uses / re-rolls seen

```
```
