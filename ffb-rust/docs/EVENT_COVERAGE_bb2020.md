# Event coverage — HeuristicAgent, amazon v amazon, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-08-30 by `scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12469 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 802 | ok |  |
| action Blitz | 864 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 244 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 642 | ok |  |
| dodge failure | 359 | ok |  |
| GFI rolls | 4762 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 227 | ok |  |
| pickup failure | 123 | ok | turnover + scatter |
| catch success | 142 | ok |  |
| catch failure | 115 | ok |  |
| ball scatters | 539 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 13 | ok | ball out of bounds |
| pass rolls | 187 | ok |  |
| pass deviates | 35 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 715 | ok |  |
| block 2 dice | 700 | ok |  |
| block 2 dice against | 251 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 245 | ok |  |
| block result BothDown | 275 | ok |  |
| block result Pushback | 589 | ok |  |
| block result PowPushback | 234 | ok |  |
| block result Pow | 323 | ok |  |
| pushbacks | 1136 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1172 | ok |  |
| armor held | 1228 | ok |  |
| stunned | 427 | ok | injury 2-7 |
| KO | 182 | ok |  |
| casualty (d16) | 110 | ok |  |
| death | 8 | ok | d16 = 15-16 only |
| fouls | 244 | ok |  |
| argue the call | 48 | ok | referee spotted a foul (doubles) |
| argue success | 6 | ok | d6 = 6 only |
| players ejected | 46 | ok |  |
| touchdowns | 46 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 28 | ok | kickoff event roll of 8 only |
| kickoff events | 242 | ok | per-result table below |

## Kickoff results

- Blitz: 24
- Brilliant Coaching: 38
- Cheering Fans: 43
- Get the Ref: 3
- High Kick: 31
- Officious Ref: 6
- Pitch Invasion: 6
- Quick Snap: 30
- Solid Defence: 21
- Time-out: 12
- Weather Change: 28

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/amazon_vs_amazon/seed_*_rust_events.jsonl)

Total events: 97625

```
  61941 playerMoved
  14379 playerAction
   4762 goForItRoll
   3422 turnEnd
   1947 injury
   1666 blockRoll
   1666 block
   1172 playerFellDown
   1136 pushback
   1001 dodgeRoll
    539 scatterBall
    350 pickupRoll
    292 apothecaryRoll
    257 catchRoll
    244 refereeSpotsFoul
    244 foul
    242 kickoffScatter
    242 kickoffResultEvent
    234 skillUse
    227 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    187 passRoll
    185 hitAndRun
    157 passBlock
     90 handOver
     48 argueTheCall
     46 touchdown
     46 playerEjected
     43 cheeringFans
     36 prayerRoll
     35 passDeviate
     35 kickoffExtraReRoll
     30 quickSnapRoll
     28 weatherChange
     24 blitzRoll
     21 solidDefenceRoll
     13 throwIn
     12 kickoffTimeout
     12 kickoffPitchInvasionStun
      6 kickoffPitchInvasion
      6 kickoffOfficiousRef
      2 trapDoor
```

## Player actions declared

```
  12207 Move
    864 BlitzMove
    802 Block
    244 Foul
    169 PassMove
     93 HandOverMove
```

## Skill uses / re-rolls seen

```
```
