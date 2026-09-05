# Event coverage — HeuristicAgent, lizardman v lizardman, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=lizardman scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11807 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 847 | ok |  |
| action Blitz | 1062 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 279 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 457 | ok |  |
| dodge failure | 566 | ok |  |
| GFI rolls | 3675 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 226 | ok |  |
| pickup failure | 153 | ok | turnover + scatter |
| catch success | 81 | ok |  |
| catch failure | 116 | ok |  |
| ball scatters | 590 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 12 | ok | ball out of bounds |
| pass rolls | 158 | ok |  |
| pass deviates | 43 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 664 | ok |  |
| block 2 dice | 685 | ok |  |
| block 2 dice against | 338 | ok | defender's choice |
| block 3 dice | 163 | ok | needs ST5+ differential via assists |
| block result Skull | 266 | ok |  |
| block result BothDown | 342 | ok |  |
| block result Pushback | 579 | ok |  |
| block result PowPushback | 286 | ok |  |
| block result Pow | 377 | ok |  |
| pushbacks | 1232 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1567 | ok |  |
| armor held | 1797 | ok |  |
| stunned | 328 | ok | injury 2-7 |
| KO | 161 | ok |  |
| casualty (d16) | 167 | ok |  |
| death | 12 | ok | d16 = 15-16 only |
| fouls | 268 | ok |  |
| argue the call | 48 | ok | referee spotted a foul (doubles) |
| argue success | 6 | ok | d6 = 6 only |
| players ejected | 47 | ok |  |
| touchdowns | 48 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 29 | ok | kickoff event roll of 8 only |
| kickoff events | 246 | ok | per-result table below |

## Kickoff results

- Blitz: 22
- Brilliant Coaching: 30
- Cheering Fans: 46
- Get the Ref: 8
- High Kick: 29
- Officious Ref: 9
- Pitch Invasion: 11
- Quick Snap: 28
- Solid Defence: 23
- Time-out: 11
- Weather Change: 29

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/lizardman_vs_lizardman/seed_*_rust_events.jsonl)

Total events: 101311

```
  64535 playerMoved
  13995 playerAction
   3675 goForItRoll
   3427 turnEnd
   2453 injury
   1850 blockRoll
   1850 block
   1567 playerFellDown
   1433 confusionRoll
   1232 pushback
   1023 dodgeRoll
    590 scatterBall
    379 pickupRoll
    328 apothecaryRoll
    268 refereeSpotsFoul
    268 foul
    246 kickoffScatter
    246 kickoffResultEvent
    226 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    197 catchRoll
    158 passRoll
    123 skillUse
    113 passBlock
     66 handOver
     48 touchdown
     48 argueTheCall
     47 playerEjected
     46 cheeringFans
     43 passDeviate
     37 prayerRoll
     29 weatherChange
     28 quickSnapRoll
     28 kickoffExtraReRoll
     23 solidDefenceRoll
     22 blitzRoll
     21 kickoffPitchInvasionStun
     12 throwIn
     11 kickoffTimeout
     11 kickoffPitchInvasion
      9 kickoffOfficiousRef
```

## Player actions declared

```
  11595 Move
   1062 BlitzMove
    847 Block
    279 Foul
    142 PassMove
     70 HandOverMove
```

## Skill uses / re-rolls seen

```
    123 Dodge used=true
```
