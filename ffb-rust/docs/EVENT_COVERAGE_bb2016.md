# Event coverage — HeuristicAgent, amazon v amazon, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-08-30 by `scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13427 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 924 | ok |  |
| action Blitz | 947 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 222 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 822 | ok |  |
| dodge failure | 91 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 267 | ok |  |
| pickup failure | 163 | ok | turnover + scatter |
| catch success | 171 | ok |  |
| catch failure | 151 | ok |  |
| ball scatters | 609 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 6 | ok | ball out of bounds |
| pass rolls | 219 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1393 | ok |  |
| block 2 dice | 370 | ok |  |
| block 2 dice against | 108 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 294 | ok |  |
| block result BothDown | 297 | ok |  |
| block result Pushback | 632 | ok |  |
| block result PowPushback | 304 | ok |  |
| block result Pow | 344 | ok |  |
| pushbacks | 1270 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 643 | ok |  |
| armor held | 980 | ok |  |
| stunned | 384 | ok | injury 2-7 |
| KO | 190 | ok |  |
| casualty (d16) | 136 | ok |  |
| death | 24 | ok | d16 = 15-16 only |
| fouls | 222 | ok |  |
| argue the call | 48 | ok | referee spotted a foul (doubles) |
| argue success | 10 | ok | d6 = 6 only |
| players ejected | 48 | ok |  |
| touchdowns | 60 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 48 | ok | kickoff event roll of 8 only |
| kickoff events | 254 | ok | per-result table below |

## Kickoff results

- Blitz: 19
- Brilliant Coaching: 21
- Cheering Fans: 43
- Get the Ref: 7
- High Kick: 23
- Perfect Defence: 27
- Pitch Invasion: 6
- Quick Snap: 33
- Riot: 13
- Throw a Rock: 14
- Weather Change: 48

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/amazon_vs_amazon/seed_*_rust_events.jsonl)

Total events: 31658

```
  15520 playerAction
   3432 turnEnd
   1871 blockRoll
   1871 block
   1690 injury
   1270 pushback
    913 dodgeRoll
    643 playerFellDown
    609 scatterBall
    430 pickupRoll
    326 apothecaryRoll
    322 catchRoll
    304 skillUse
    267 ballPickedUp
    254 kickoffScatter
    254 kickoffResultEvent
    222 refereeSpotsFoul
    222 foul
    219 passRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    112 handOver
     64 kickoffExtraReRollBb2016
     60 touchdown
     48 weatherChange
     48 playerEjected
     48 argueTheCall
     14 kickoffThrowARockBb2016
     13 kickoffRiot
      6 throwIn
      6 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  13104 Move
    947 Blitz
    924 Block
    222 Foul
    210 PassMove
    113 HandOverMove
```

## Skill uses / re-rolls seen

```
```
