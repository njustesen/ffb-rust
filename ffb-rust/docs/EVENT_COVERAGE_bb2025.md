# Event coverage — HeuristicAgent, amazon v amazon, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-08-30 by `scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12684 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 788 | ok |  |
| action Blitz | 874 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 218 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 686 | ok |  |
| dodge failure | 392 | ok |  |
| GFI rolls | 4867 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 229 | ok |  |
| pickup failure | 121 | ok | turnover + scatter |
| catch success | 133 | ok |  |
| catch failure | 130 | ok |  |
| ball scatters | 555 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 10 | ok | ball out of bounds |
| pass rolls | 195 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 817 | ok |  |
| block 2 dice | 579 | ok |  |
| block 2 dice against | 266 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 254 | ok |  |
| block result BothDown | 267 | ok |  |
| block result Pushback | 577 | ok |  |
| block result PowPushback | 251 | ok |  |
| block result Pow | 313 | ok |  |
| pushbacks | 1132 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1125 | ok |  |
| armor held | 1136 | ok |  |
| stunned | 425 | ok | injury 2-7 |
| KO | 188 | ok |  |
| casualty (d16) | 125 | ok |  |
| death | 12 | ok | d16 = 15-16 only |
| fouls | 218 | ok |  |
| argue the call | 38 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 32 | ok |  |
| touchdowns | 43 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 29 | ok | kickoff event roll of 8 only |
| kickoff events | 237 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 37
- Charge: 22
- Cheering Fans: 46
- Dodgy Snack: 11
- Get the Ref: 4
- High Kick: 29
- Pitch Invasion: 11
- Quick Snap: 21
- Solid Defence: 17
- Time-out: 10
- Weather Change: 29

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/amazon_vs_amazon/seed_*_rust_events.jsonl)

Total events: 100157

```
  64068 playerMoved
  14742 playerAction
   4867 goForItRoll
   3425 turnEnd
   1874 injury
   1662 blockRoll
   1662 block
   1132 pushback
   1125 playerFellDown
   1078 dodgeRoll
    555 scatterBall
    350 pickupRoll
    313 apothecaryRoll
    263 catchRoll
    251 skillUse
    237 kickoffScatter
    237 kickoffResultEvent
    229 ballPickedUp
    218 refereeSpotsFoul
    218 foul
    216 hitAndRun
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    195 passRoll
    161 passBlock
     96 handOver
     46 cheeringFans
     43 touchdown
     43 playerNote
     40 kickoffExtraReRoll
     38 argueTheCall
     32 playerEjected
     29 weatherChange
     21 quickSnapRoll
     19 kickoffPitchInvasionStun
     17 solidDefenceRoll
     12 dodgySnackRoll
     11 kickoffPitchInvasion
     11 kickoffDodgySnack
     10 throwIn
     10 kickoffTimeout
      1 throwAtStallingPlayer
```

## Player actions declared

```
  12414 Move
    874 BlitzMove
    788 Block
    218 Foul
    178 BalefulHex
    171 PassMove
     99 HandOverMove
```

## Skill uses / re-rolls seen

```
```
