# Event coverage — HeuristicAgent, chaos_pact v chaos_pact, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-01 by `MATCHUP=chaos_pact scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12023 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 658 | ok |  |
| action Blitz | 1019 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 322 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 300 | ok |  |
| dodge failure | 292 | ok |  |
| GFI rolls | 4556 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 217 | ok |  |
| pickup failure | 134 | ok | turnover + scatter |
| catch success | 91 | ok |  |
| catch failure | 98 | ok |  |
| ball scatters | 563 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 14 | ok | ball out of bounds |
| pass rolls | 143 | ok |  |
| pass deviates | 34 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 688 | ok |  |
| block 2 dice | 804 | ok |  |
| block 2 dice against | 238 | ok | defender's choice |
| block 3 dice | 66 | ok | needs ST5+ differential via assists |
| block result Skull | 231 | ok |  |
| block result BothDown | 288 | ok |  |
| block result Pushback | 594 | ok |  |
| block result PowPushback | 324 | ok |  |
| block result Pow | 359 | ok |  |
| pushbacks | 1275 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1927 | ok |  |
| armor held | 1894 | ok |  |
| stunned | 469 | ok | injury 2-7 |
| KO | 176 | ok |  |
| casualty (d16) | 156 | ok |  |
| death | 15 | ok | d16 = 15-16 only |
| fouls | 240 | ok |  |
| argue the call | 49 | ok | referee spotted a foul (doubles) |
| argue success | 4 | ok | d6 = 6 only |
| players ejected | 47 | ok |  |
| touchdowns | 26 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 27 | ok | kickoff event roll of 8 only |
| kickoff events | 222 | ok | per-result table below |

## Kickoff results

- Blitz: 18
- Brilliant Coaching: 28
- Cheering Fans: 36
- Get the Ref: 4
- High Kick: 32
- Officious Ref: 8
- Pitch Invasion: 9
- Quick Snap: 28
- Solid Defence: 18
- Time-out: 14
- Weather Change: 27

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/chaos_pact_vs_chaos_pact/seed_*_rust_events.jsonl)

Total events: 93483

```
  53289 playerMoved
  14309 playerAction
   4556 goForItRoll
   3398 turnEnd
   2695 injury
   2567 confusionRoll
   1927 playerFellDown
   1796 blockRoll
   1796 block
   1333 animalSavagery
   1275 pushback
    592 dodgeRoll
    563 scatterBall
    351 pickupRoll
    343 skillUse
    240 refereeSpotsFoul
    240 foul
    222 kickoffScatter
    222 kickoffResultEvent
    217 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    189 catchRoll
    143 passRoll
    124 animosityRoll
     72 handOver
     49 argueTheCall
     47 playerEjected
     36 cheeringFans
     34 passDeviate
     31 prayerRoll
     28 quickSnapRoll
     27 weatherChange
     26 touchdown
     22 kickoffExtraReRoll
     18 solidDefenceRoll
     18 kickoffPitchInvasionStun
     18 blitzRoll
     15 throwTeamMateRoll
     14 throwIn
     14 kickoffTimeout
      9 kickoffPitchInvasion
      8 rightStuffRoll
      8 kickoffOfficiousRef
      2 trapDoor
```

## Player actions declared

```
  11781 Move
   1019 BlitzMove
    658 Block
    322 Foul
    287 ThrowTeamMate
    160 PassMove
     82 HandOverMove
```

## Skill uses / re-rolls seen

```
```
