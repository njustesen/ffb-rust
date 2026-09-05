# Event coverage — HeuristicAgent, necromantic v necromantic, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=necromantic scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12075 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 691 | ok |  |
| action Blitz | 888 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 274 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 488 | ok |  |
| dodge failure | 326 | ok |  |
| GFI rolls | 5926 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 171 | ok |  |
| pickup failure | 147 | ok | turnover + scatter |
| catch success | 59 | ok |  |
| catch failure | 78 | ok |  |
| ball scatters | 622 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 9 | ok | ball out of bounds |
| pass rolls | 106 | ok |  |
| pass deviates | 19 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 964 | ok |  |
| block 2 dice | 559 | ok |  |
| block 2 dice against | 233 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 258 | ok |  |
| block result BothDown | 263 | ok |  |
| block result Pushback | 621 | ok |  |
| block result PowPushback | 297 | ok |  |
| block result Pow | 317 | ok |  |
| pushbacks | 1129 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1660 | ok |  |
| armor held | 1754 | ok |  |
| stunned | 411 | ok | injury 2-7 |
| KO | 162 | ok |  |
| casualty (d16) | 107 | ok |  |
| death | 14 | ok | d16 = 15-16 only |
| fouls | 274 | ok |  |
| argue the call | 50 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 46 | ok |  |
| touchdowns | 26 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 33 | ok | kickoff event roll of 8 only |
| kickoff events | 221 | ok | per-result table below |

## Kickoff results

- Blitz: 21
- Brilliant Coaching: 30
- Cheering Fans: 38
- Get the Ref: 5
- High Kick: 23
- Officious Ref: 5
- Pitch Invasion: 7
- Quick Snap: 24
- Solid Defence: 22
- Time-out: 13
- Weather Change: 33

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/necromantic_vs_necromantic/seed_*_rust_events.jsonl)

Total events: 90375

```
  53863 playerMoved
  13928 playerAction
   5926 goForItRoll
   3399 turnEnd
   2434 injury
   1756 blockRoll
   1756 block
   1660 playerFellDown
   1129 pushback
    814 dodgeRoll
    622 scatterBall
    318 pickupRoll
    274 refereeSpotsFoul
    274 foul
    221 kickoffScatter
    221 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    196 foulAppearanceRoll
    171 ballPickedUp
    137 catchRoll
    106 passRoll
    105 regenerationRoll
     59 handOver
     50 argueTheCall
     46 playerEjected
     38 cheeringFans
     33 weatherChange
     30 prayerRoll
     28 kickoffExtraReRoll
     26 touchdown
     24 quickSnapRoll
     22 solidDefenceRoll
     21 blitzRoll
     19 passDeviate
     17 skillUse
     16 kickoffPitchInvasionStun
     13 kickoffTimeout
      9 throwIn
      7 kickoffPitchInvasion
      5 kickoffOfficiousRef
      2 trapDoor
```

## Player actions declared

```
  11910 Move
    888 BlitzMove
    691 Block
    274 Foul
    106 PassMove
     59 HandOverMove
```

## Skill uses / re-rolls seen

```
     17 Dodge used=true
```
