# Event coverage — HeuristicAgent, renegades v renegades, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=renegades scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12062 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 674 | ok |  |
| action Blitz | 998 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 321 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 392 | ok |  |
| dodge failure | 321 | ok |  |
| GFI rolls | 4360 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 216 | ok |  |
| pickup failure | 122 | ok | turnover + scatter |
| catch success | 110 | ok |  |
| catch failure | 84 | ok |  |
| ball scatters | 559 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 23 | ok | ball out of bounds |
| pass rolls | 136 | ok |  |
| pass deviates | 43 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 697 | ok |  |
| block 2 dice | 820 | ok |  |
| block 2 dice against | 222 | ok | defender's choice |
| block 3 dice | 51 | ok | needs ST5+ differential via assists |
| block result Skull | 196 | ok |  |
| block result BothDown | 273 | ok |  |
| block result Pushback | 611 | ok |  |
| block result PowPushback | 344 | ok |  |
| block result Pow | 366 | ok |  |
| pushbacks | 1315 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1971 | ok |  |
| armor held | 1867 | ok |  |
| stunned | 497 | ok | injury 2-7 |
| KO | 164 | ok |  |
| casualty (d16) | 175 | ok |  |
| death | 23 | ok | d16 = 15-16 only |
| fouls | 251 | ok |  |
| argue the call | 51 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 46 | ok |  |
| touchdowns | 35 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 31 | ok | kickoff event roll of 8 only |
| kickoff events | 232 | ok | per-result table below |

## Kickoff results

- Blitz: 24
- Brilliant Coaching: 31
- Cheering Fans: 40
- Get the Ref: 6
- High Kick: 30
- Officious Ref: 8
- Pitch Invasion: 8
- Quick Snap: 24
- Solid Defence: 16
- Time-out: 14
- Weather Change: 31

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/renegades_vs_renegades/seed_*_rust_events.jsonl)

Total events: 96145

```
  55910 playerMoved
  14288 playerAction
   4360 goForItRoll
   3408 turnEnd
   2703 injury
   2660 confusionRoll
   1971 playerFellDown
   1790 blockRoll
   1790 block
   1315 pushback
   1246 animalSavagery
    713 dodgeRoll
    559 scatterBall
    349 skillUse
    338 pickupRoll
    251 refereeSpotsFoul
    251 foul
    232 kickoffScatter
    232 kickoffResultEvent
    216 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    194 catchRoll
    136 passRoll
     94 animosityRoll
     80 handOver
     51 argueTheCall
     46 playerEjected
     43 passDeviate
     40 cheeringFans
     35 touchdown
     32 prayerRoll
     31 weatherChange
     26 kickoffExtraReRoll
     24 quickSnapRoll
     24 blitzRoll
     23 throwIn
     16 solidDefenceRoll
     16 kickoffPitchInvasionStun
     14 kickoffTimeout
     10 throwTeamMateRoll
      8 rightStuffRoll
      8 kickoffPitchInvasion
      8 kickoffOfficiousRef
      4 trapDoor
```

## Player actions declared

```
  11819 Move
    998 BlitzMove
    674 Block
    321 Foul
    225 ThrowTeamMate
    154 PassMove
     89 HandOverMove
      8 Treacherous
```

## Skill uses / re-rolls seen

```
    319 Horns used=true
     30 Dodge used=true
```
