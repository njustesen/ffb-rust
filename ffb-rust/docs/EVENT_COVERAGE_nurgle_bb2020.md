# Event coverage — HeuristicAgent, nurgle v nurgle, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=nurgle scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12384 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 666 | ok |  |
| action Blitz | 878 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 258 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 307 | ok |  |
| dodge failure | 375 | ok |  |
| GFI rolls | 5777 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 180 | ok |  |
| pickup failure | 163 | ok | turnover + scatter |
| catch success | 75 | ok |  |
| catch failure | 107 | ok |  |
| ball scatters | 561 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 27 | ok | ball out of bounds |
| pass rolls | 135 | ok |  |
| pass deviates | 55 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 494 | ok |  |
| block 2 dice | 541 | ok |  |
| block 2 dice against | 239 | ok | defender's choice |
| block 3 dice | 1 | ok | needs ST5+ differential via assists |
| block result Skull | 161 | ok |  |
| block result BothDown | 219 | ok |  |
| block result Pushback | 429 | ok |  |
| block result PowPushback | 220 | ok |  |
| block result Pow | 246 | ok |  |
| pushbacks | 894 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1633 | ok |  |
| armor held | 1760 | ok |  |
| stunned | 287 | ok | injury 2-7 |
| KO | 114 | ok |  |
| casualty (d16) | 85 | ok |  |
| death | 11 | ok | d16 = 15-16 only |
| fouls | 232 | ok |  |
| argue the call | 43 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 39 | ok |  |
| touchdowns | 12 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 26 | ok | kickoff event roll of 8 only |
| kickoff events | 208 | ok | per-result table below |

## Kickoff results

- Blitz: 18
- Brilliant Coaching: 30
- Cheering Fans: 33
- Get the Ref: 7
- High Kick: 26
- Officious Ref: 10
- Pitch Invasion: 7
- Quick Snap: 22
- Solid Defence: 14
- Time-out: 15
- Weather Change: 26

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/nurgle_vs_nurgle/seed_*_rust_events.jsonl)

Total events: 87456

```
  49975 playerMoved
  14186 playerAction
   5777 goForItRoll
   3378 turnEnd
   2246 injury
   1723 confusionRoll
   1633 playerFellDown
   1275 blockRoll
   1275 block
    956 foulAppearanceRoll
    894 pushback
    682 dodgeRoll
    561 scatterBall
    343 pickupRoll
    232 refereeSpotsFoul
    232 foul
    208 kickoffScatter
    208 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    182 catchRoll
    180 ballPickedUp
    135 passRoll
     68 handOver
     60 regenerationRoll
     59 skillUse
     55 passDeviate
     43 argueTheCall
     39 playerEjected
     33 cheeringFans
     28 kickoffExtraReRoll
     27 throwIn
     27 prayerRoll
     26 weatherChange
     22 quickSnapRoll
     18 blitzRoll
     15 kickoffTimeout
     14 solidDefenceRoll
     12 touchdown
     12 kickoffPitchInvasionStun
     10 kickoffOfficiousRef
      7 kickoffPitchInvasion
```

## Player actions declared

```
  12175 Move
    878 BlitzMove
    666 Block
    258 Foul
    134 PassMove
     75 HandOverMove
```

## Skill uses / re-rolls seen

```
     59 Horns used=true
```
