# Event coverage — HeuristicAgent, dwarf v dwarf, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=dwarf scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12221 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 725 | ok |  |
| action Blitz | 988 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 275 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 445 | ok |  |
| dodge failure | 238 | ok |  |
| GFI rolls | 6206 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 196 | ok |  |
| pickup failure | 185 | ok | turnover + scatter |
| catch success | 79 | ok |  |
| catch failure | 107 | ok |  |
| ball scatters | 590 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 8 | ok | ball out of bounds |
| pass rolls | 136 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1002 | ok |  |
| block 2 dice | 322 | ok |  |
| block 2 dice against | 108 | ok | defender's choice |
| block 3 dice | 362 | ok | needs ST5+ differential via assists |
| block result Skull | 236 | ok |  |
| block result BothDown | 248 | ok |  |
| block result Pushback | 582 | ok |  |
| block result PowPushback | 348 | ok |  |
| block result Pow | 380 | ok |  |
| pushbacks | 1149 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 2000 | ok |  |
| armor held | 1925 | ok |  |
| stunned | 421 | ok | injury 2-7 |
| KO | 78 | ok |  |
| casualty (d16) | 121 | ok |  |
| death | 13 | ok | d16 = 15-16 only |
| fouls | 275 | ok |  |
| argue the call | 55 | ok | referee spotted a foul (doubles) |
| argue success | 4 | ok | d6 = 6 only |
| players ejected | 51 | ok |  |
| touchdowns | 8 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 32 | ok | kickoff event roll of 8 only |
| kickoff events | 208 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 31
- Charge: 20
- Cheering Fans: 36
- Dodgy Snack: 8
- Get the Ref: 6
- High Kick: 22
- Pitch Invasion: 9
- Quick Snap: 18
- Solid Defence: 16
- Time-out: 10
- Weather Change: 32

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/dwarf_vs_dwarf/seed_*_rust_events.jsonl)

Total events: 89524

```
  51387 playerMoved
  14770 playerAction
   6206 goForItRoll
   3388 turnEnd
   2545 injury
   2000 playerFellDown
   1794 blockRoll
   1794 block
   1149 pushback
    683 dodgeRoll
    590 scatterBall
    381 pickupRoll
    275 refereeSpotsFoul
    275 foul
    249 kegThrow
    208 kickoffScatter
    208 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    196 ballPickedUp
    186 catchRoll
    136 passRoll
     86 dauntlessRoll
     80 handOver
     55 argueTheCall
     51 playerEjected
     37 kickoffExtraReRoll
     36 cheeringFans
     32 weatherChange
     18 quickSnapRoll
     17 skillUse
     16 solidDefenceRoll
     16 kickoffPitchInvasionStun
     10 kickoffTimeout
      9 kickoffPitchInvasion
      9 dodgySnackRoll
      8 touchdown
      8 throwIn
      8 playerNote
      8 kickoffDodgySnack
```

## Player actions declared

```
  11995 Move
    988 BlitzMove
    725 Block
    385 ThrowKeg
    275 Foul
    176 WisdomOfTheWhiteDwarf
    142 PassMove
     84 HandOverMove
```

## Skill uses / re-rolls seen

```
     17 Juggernaut used=true
```
