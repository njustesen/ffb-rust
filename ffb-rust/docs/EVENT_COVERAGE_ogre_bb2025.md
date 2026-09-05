# Event coverage — HeuristicAgent, ogre v ogre, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=ogre scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11154 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 617 | ok |  |
| action Blitz | 763 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 204 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 354 | ok |  |
| dodge failure | 261 | ok |  |
| GFI rolls | 5143 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 163 | ok |  |
| pickup failure | 109 | ok | turnover + scatter |
| catch success | 69 | ok |  |
| catch failure | 74 | ok |  |
| ball scatters | 509 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 11 | ok | ball out of bounds |
| pass rolls | 124 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 623 | ok |  |
| block 2 dice | 172 | ok |  |
| block 2 dice against | 66 | ok | defender's choice |
| block 3 dice | 332 | ok | needs ST5+ differential via assists |
| block result Skull | 183 | ok |  |
| block result BothDown | 169 | ok |  |
| block result Pushback | 386 | ok |  |
| block result PowPushback | 172 | ok |  |
| block result Pow | 283 | ok |  |
| pushbacks | 840 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1473 | ok |  |
| armor held | 1364 | ok |  |
| stunned | 498 | ok | injury 2-7 |
| KO | 279 | ok |  |
| casualty (d16) | 292 | ok |  |
| death | 23 | ok | d16 = 15-16 only |
| fouls | 176 | ok |  |
| argue the call | 37 | ok | referee spotted a foul (doubles) |
| argue success | 4 | ok | d6 = 6 only |
| players ejected | 35 | ok |  |
| touchdowns | 9 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 21 | ok | kickoff event roll of 8 only |
| kickoff events | 208 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 36
- Charge: 20
- Cheering Fans: 32
- Dodgy Snack: 7
- Get the Ref: 6
- High Kick: 27
- Pitch Invasion: 8
- Quick Snap: 26
- Solid Defence: 18
- Time-out: 7
- Weather Change: 21

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/ogre_vs_ogre/seed_*_rust_events.jsonl)

Total events: 88310

```
  44469 playerMoved
  14592 playerAction
   8721 confusionRoll
   5143 goForItRoll
   3394 turnEnd
   2433 injury
   1473 playerFellDown
   1193 blockRoll
   1193 block
    840 pushback
    822 throwTeamMateRoll
    615 dodgeRoll
    509 scatterBall
    441 rightStuffRoll
    272 pickupRoll
    208 kickoffScatter
    208 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    176 refereeSpotsFoul
    176 foul
    163 ballPickedUp
    143 catchRoll
    124 passRoll
     60 handOver
     42 kickoffExtraReRoll
     39 skillUse
     37 argueTheCall
     35 playerEjected
     32 cheeringFans
     26 quickSnapRoll
     21 weatherChange
     19 kickoffPitchInvasionStun
     18 solidDefenceRoll
     15 kickTeamMateFumble
     11 throwIn
      9 touchdown
      9 playerNote
      8 kickoffPitchInvasion
      7 kickoffTimeout
      7 kickoffDodgySnack
      7 dodgySnackRoll
```

## Player actions declared

```
  10965 Move
   1524 ThrowTeamMate
    763 BlitzMove
    617 Block
    330 KickTeamMate
    204 Foul
    125 PassMove
     64 HandOverMove
```

## Skill uses / re-rolls seen

```
     39 Dodge used=true
```
