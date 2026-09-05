# Event coverage — HeuristicAgent, nippon v nippon, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=nippon scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13481 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 906 | ok |  |
| action Blitz | 941 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 272 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 828 | ok |  |
| dodge failure | 312 | ok |  |
| GFI rolls | 4754 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 258 | ok |  |
| pickup failure | 138 | ok | turnover + scatter |
| catch success | 135 | ok |  |
| catch failure | 108 | ok |  |
| ball scatters | 580 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 18 | ok | ball out of bounds |
| pass rolls | 166 | ok |  |
| pass deviates | 23 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 918 | ok |  |
| block 2 dice | 694 | ok |  |
| block 2 dice against | 220 | ok | defender's choice |
| block 3 dice | 15 | ok | needs ST5+ differential via assists |
| block result Skull | 259 | ok |  |
| block result BothDown | 361 | ok |  |
| block result Pushback | 600 | ok |  |
| block result PowPushback | 268 | ok |  |
| block result Pow | 359 | ok |  |
| pushbacks | 1217 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1374 | ok |  |
| armor held | 1339 | ok |  |
| stunned | 440 | ok | injury 2-7 |
| KO | 199 | ok |  |
| casualty (d16) | 111 | ok |  |
| death | 12 | ok | d16 = 15-16 only |
| fouls | 272 | ok |  |
| argue the call | 62 | ok | referee spotted a foul (doubles) |
| argue success | 4 | ok | d6 = 6 only |
| players ejected | 61 | ok |  |
| touchdowns | 63 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 22 | ok | kickoff event roll of 8 only |
| kickoff events | 254 | ok | per-result table below |

## Kickoff results

- Blitz: 26
- Brilliant Coaching: 43
- Cheering Fans: 39
- Get the Ref: 6
- High Kick: 32
- Officious Ref: 10
- Pitch Invasion: 7
- Quick Snap: 34
- Solid Defence: 20
- Time-out: 15
- Weather Change: 22

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/nippon_vs_nippon/seed_*_rust_events.jsonl)

Total events: 110974

```
  73380 playerMoved
  15600 playerAction
   4754 goForItRoll
   3436 turnEnd
   2089 injury
   1847 blockRoll
   1847 block
   1374 playerFellDown
   1217 pushback
   1140 dodgeRoll
    580 scatterBall
    396 pickupRoll
    310 apothecaryRoll
    272 refereeSpotsFoul
    272 foul
    258 ballPickedUp
    254 kickoffScatter
    254 kickoffResultEvent
    243 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    166 passRoll
    103 handOver
     98 skillUse
     63 touchdown
     62 argueTheCall
     61 playerEjected
     39 cheeringFans
     37 kickoffExtraReRoll
     34 quickSnapRoll
     32 prayerRoll
     26 blitzRoll
     23 passDeviate
     22 weatherChange
     20 solidDefenceRoll
     18 throwIn
     15 kickoffTimeout
     14 kickoffPitchInvasionStun
     10 kickoffOfficiousRef
      7 kickoffPitchInvasion
      1 trapDoor
```

## Player actions declared

```
  13217 Move
    941 BlitzMove
    906 Block
    272 Foul
    160 PassMove
    104 HandOverMove
```

## Skill uses / re-rolls seen

```
     98 Dodge used=true
```
