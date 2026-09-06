# Event coverage — HeuristicAgent, wood_elf v wood_elf, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=wood_elf scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13077 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 807 | ok |  |
| action Blitz | 880 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 217 | ok |  |
| action Pass | 34 | ok | needs a ball carrier |
| action HandOver | 4 | ok | needs carrier + adjacent teammate |
| dodge success | 943 | ok |  |
| dodge failure | 286 | ok |  |
| GFI rolls | 4020 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 305 | ok |  |
| pickup failure | 89 | ok | turnover + scatter |
| catch success | 206 | ok |  |
| catch failure | 93 | ok |  |
| ball scatters | 593 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 17 | ok | ball out of bounds |
| pass rolls | 200 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 652 | ok |  |
| block 2 dice | 759 | ok |  |
| block 2 dice against | 266 | ok | defender's choice |
| block 3 dice | 10 | ok | needs ST5+ differential via assists |
| block result Skull | 231 | ok |  |
| block result BothDown | 288 | ok |  |
| block result Pushback | 530 | ok |  |
| block result PowPushback | 301 | ok |  |
| block result Pow | 337 | ok |  |
| pushbacks | 1162 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1468 | ok |  |
| armor held | 1126 | ok |  |
| stunned | 501 | ok | injury 2-7 |
| KO | 313 | ok |  |
| casualty (d16) | 228 | ok |  |
| death | 21 | ok | d16 = 15-16 only |
| fouls | 217 | ok |  |
| argue the call | 39 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 31 | ok |  |
| touchdowns | 122 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 43 | ok | kickoff event roll of 8 only |
| kickoff events | 306 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 44
- Charge: 26
- Cheering Fans: 47
- Dodgy Snack: 14
- Get the Ref: 9
- High Kick: 42
- Pitch Invasion: 9
- Quick Snap: 34
- Solid Defence: 23
- Time-out: 15
- Weather Change: 43

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2025/wood_elf_vs_wood_elf/seed_*_rust_events.jsonl)

Total events: 110201

```
  73498 playerMoved
  15344 playerAction
   4020 goForItRoll
   3500 turnEnd
   2168 injury
   1687 blockRoll
   1687 block
   1468 playerFellDown
   1229 dodgeRoll
   1162 pushback
    593 scatterBall
    394 pickupRoll
    306 kickoffScatter
    306 kickoffResultEvent
    305 ballPickedUp
    299 catchRoll
    217 refereeSpotsFoul
    217 foul
    200 winningsRoll
    200 startHalf
    200 passRoll
    200 mvpRoll
    186 skillUse
    124 playerNote
    122 touchdown
    119 handOver
     87 passBlock
     49 kickoffExtraReRoll
     47 cheeringFans
     43 weatherChange
     39 argueTheCall
     34 quickSnapRoll
     31 playerEjected
     23 solidDefenceRoll
     19 kickoffPitchInvasionStun
     17 throwIn
     16 dodgySnackRoll
     15 kickoffTimeout
     14 kickoffDodgySnack
      9 kickoffPitchInvasion
      6 standUpRoll
      1 throwAtStallingPlayer
```

## Player actions declared

```
  12799 Move
    880 BlitzMove
    807 Block
    217 Foul
    206 FuriousOutburst
    161 PassMove
    117 HandOverMove
     66 ThrowTeamMate
     53 CatchOfTheDay
     34 Pass
      4 HandOver
```

## Skill uses / re-rolls seen

```
    116 Dodge used=true
     51 Wrestle used=true
     19 Wrestle used=false
```
