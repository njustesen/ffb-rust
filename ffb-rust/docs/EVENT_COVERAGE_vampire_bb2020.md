# Event coverage — HeuristicAgent, vampire v vampire, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-07 by `MATCHUP=vampire scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 9992 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 552 | ok |  |
| action Blitz | 703 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 223 | ok |  |
| action Pass | 22 | ok | needs a ball carrier |
| action HandOver | 3 | ok | needs carrier + adjacent teammate |
| dodge success | 461 | ok |  |
| dodge failure | 155 | ok |  |
| GFI rolls | 3644 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 192 | ok |  |
| pickup failure | 93 | ok | turnover + scatter |
| catch success | 97 | ok |  |
| catch failure | 72 | ok |  |
| ball scatters | 490 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 16 | ok | ball out of bounds |
| pass rolls | 146 | ok |  |
| pass deviates | 42 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 491 | ok |  |
| block 2 dice | 623 | ok |  |
| block 2 dice against | 254 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 188 | ok |  |
| block result BothDown | 186 | ok |  |
| block result Pushback | 471 | ok |  |
| block result PowPushback | 262 | ok |  |
| block result Pow | 261 | ok |  |
| pushbacks | 1028 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1221 | ok |  |
| armor held | 1161 | ok |  |
| stunned | 371 | ok | injury 2-7 |
| KO | 129 | ok |  |
| casualty (d16) | 123 | ok |  |
| death | 16 | ok | d16 = 15-16 only |
| fouls | 223 | ok |  |
| argue the call | 54 | ok | referee spotted a foul (doubles) |
| argue success | 13 | ok | d6 = 6 only |
| players ejected | 44 | ok |  |
| touchdowns | 20 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 25 | ok | kickoff event roll of 8 only |
| kickoff events | 218 | ok | per-result table below |

## Kickoff results

- Blitz: 25
- Brilliant Coaching: 33
- Cheering Fans: 35
- Get the Ref: 5
- High Kick: 24
- Officious Ref: 9
- Pitch Invasion: 7
- Quick Snap: 22
- Solid Defence: 23
- Time-out: 10
- Weather Change: 25

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2020/vampire_vs_vampire/seed_*_rust_events.jsonl)

Total events: 81359

```
  45630 playerMoved
  11495 playerAction
   5359 bloodLustRoll
   3644 goForItRoll
   3402 turnEnd
   1784 injury
   1368 blockRoll
   1368 block
   1221 playerFellDown
   1093 biteSpectator
   1028 pushback
    616 dodgeRoll
    490 scatterBall
    285 pickupRoll
    223 refereeSpotsFoul
    223 foul
    218 kickoffScatter
    218 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    192 ballPickedUp
    169 catchRoll
    146 passRoll
     73 regenerationRoll
     54 handOver
     54 argueTheCall
     47 skillUse
     44 playerEjected
     42 passDeviate
     35 cheeringFans
     29 kickoffExtraReRoll
     26 prayerRoll
     25 weatherChange
     25 blitzRoll
     23 solidDefenceRoll
     22 quickSnapRoll
     20 touchdown
     16 throwIn
     11 playerAdded
     11 kickoffPitchInvasionStun
     10 kickoffTimeout
      9 kickoffOfficiousRef
      7 kickoffPitchInvasion
      4 trapDoor
```

## Player actions declared

```
   9819 Move
    703 BlitzMove
    552 Block
    223 Foul
    119 PassMove
     54 HandOverMove
     22 Pass
      3 HandOver
```

## Skill uses / re-rolls seen

```
     42 Juggernaut used=true
      5 Juggernaut used=false
```
