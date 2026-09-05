# Event coverage — HeuristicAgent, chaos_dwarf v chaos_dwarf, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=chaos_dwarf scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12087 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 656 | ok |  |
| action Blitz | 940 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 278 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 286 | ok |  |
| dodge failure | 278 | ok |  |
| GFI rolls | 5759 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 160 | ok |  |
| pickup failure | 152 | ok | turnover + scatter |
| catch success | 50 | ok |  |
| catch failure | 92 | ok |  |
| ball scatters | 527 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 9 | ok | ball out of bounds |
| pass rolls | 104 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 509 | ok |  |
| block 2 dice | 814 | ok |  |
| block 2 dice against | 320 | ok | defender's choice |
| block 3 dice | 19 | ok | needs ST5+ differential via assists |
| block result Skull | 193 | ok |  |
| block result BothDown | 262 | ok |  |
| block result Pushback | 556 | ok |  |
| block result PowPushback | 333 | ok |  |
| block result Pow | 318 | ok |  |
| pushbacks | 1203 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1831 | ok |  |
| armor held | 1992 | ok |  |
| stunned | 444 | ok | injury 2-7 |
| KO | 68 | ok |  |
| casualty (d16) | 108 | ok |  |
| death | 12 | ok | d16 = 15-16 only |
| fouls | 230 | ok |  |
| argue the call | 39 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 34 | ok |  |
| touchdowns | 8 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 29 | ok | kickoff event roll of 8 only |
| kickoff events | 205 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 27
- Charge: 21
- Cheering Fans: 35
- Dodgy Snack: 10
- Get the Ref: 4
- High Kick: 26
- Pitch Invasion: 9
- Quick Snap: 25
- Solid Defence: 11
- Time-out: 8
- Weather Change: 29

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/chaos_dwarf_vs_chaos_dwarf/seed_*_rust_events.jsonl)

Total events: 86654

```
  48763 playerMoved
  14137 playerAction
   5759 goForItRoll
   3389 turnEnd
   2612 injury
   1831 playerFellDown
   1662 blockRoll
   1662 block
   1475 confusionRoll
   1203 pushback
    564 dodgeRoll
    527 scatterBall
    368 skillUse
    312 pickupRoll
    230 refereeSpotsFoul
    230 foul
    205 kickoffScatter
    205 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    174 thenIStartedBlastin
    160 ballPickedUp
    142 catchRoll
    104 passRoll
     52 handOver
     39 argueTheCall
     35 cheeringFans
     34 playerEjected
     32 kickoffExtraReRoll
     29 weatherChange
     25 quickSnapRoll
     16 kickoffPitchInvasionStun
     13 dodgySnackRoll
     11 solidDefenceRoll
     10 kickoffDodgySnack
      9 throwIn
      9 playerNote
      9 kickoffPitchInvasion
      8 touchdown
      8 kickoffTimeout
      1 throwAtStallingPlayer
```

## Player actions declared

```
  11920 Move
    940 BlitzMove
    656 Block
    278 Foul
    175 ThenIStartedBlastin
    114 PassMove
     53 HandOverMove
      1 HailMaryPass
```

## Skill uses / re-rolls seen

```
    368 Horns used=true
```
