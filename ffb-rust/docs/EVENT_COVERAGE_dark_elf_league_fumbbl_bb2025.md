# Event coverage — HeuristicAgent, dark_elf_league_fumbbl v dark_elf_league_fumbbl, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=dark_elf_league_fumbbl scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12638 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 848 | ok |  |
| action Blitz | 940 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 307 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 1084 | ok |  |
| dodge failure | 245 | ok |  |
| GFI rolls | 4210 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 302 | ok |  |
| pickup failure | 88 | ok | turnover + scatter |
| catch success | 170 | ok |  |
| catch failure | 51 | ok |  |
| ball scatters | 575 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 9 | ok | ball out of bounds |
| pass rolls | 198 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1526 | ok |  |
| block 2 dice | 359 | ok |  |
| block 2 dice against | 151 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 332 | ok |  |
| block result BothDown | 366 | ok |  |
| block result Pushback | 680 | ok |  |
| block result PowPushback | 333 | ok |  |
| block result Pow | 325 | ok |  |
| pushbacks | 1334 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1338 | ok |  |
| armor held | 1418 | ok |  |
| stunned | 454 | ok | injury 2-7 |
| KO | 176 | ok |  |
| casualty (d16) | 144 | ok |  |
| death | 18 | ok | d16 = 15-16 only |
| fouls | 307 | ok |  |
| argue the call | 63 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 56 | ok |  |
| touchdowns | 76 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 24 | ok | kickoff event roll of 8 only |
| kickoff events | 265 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 46
- Charge: 28
- Cheering Fans: 39
- Dodgy Snack: 18
- Get the Ref: 9
- High Kick: 32
- Pitch Invasion: 7
- Quick Snap: 27
- Solid Defence: 23
- Time-out: 12
- Weather Change: 24

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/dark_elf_league_fumbbl_vs_dark_elf_league_fumbbl/seed_*_rust_events.jsonl)

Total events: 105237

```
  68274 playerMoved
  14733 playerAction
   4210 goForItRoll
   3449 turnEnd
   2192 injury
   2036 blockRoll
   2036 block
   1338 playerFellDown
   1334 pushback
   1329 dodgeRoll
    575 scatterBall
    390 pickupRoll
    307 refereeSpotsFoul
    307 foul
    302 ballPickedUp
    265 kickoffScatter
    265 kickoffResultEvent
    255 skillUse
    221 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    198 passRoll
    105 handOver
     76 touchdown
     76 playerNote
     63 argueTheCall
     56 playerEjected
     54 kickoffExtraReRoll
     39 cheeringFans
     27 quickSnapRoll
     24 weatherChange
     23 solidDefenceRoll
     20 dodgySnackRoll
     18 kickoffDodgySnack
     12 kickoffTimeout
     12 kickoffPitchInvasionStun
      9 throwIn
      7 kickoffPitchInvasion
```

## Player actions declared

```
  12346 Move
    940 BlitzMove
    848 Block
    307 Foul
    183 PassMove
    109 HandOverMove
```

## Skill uses / re-rolls seen

```
    130 DumpOff used=false
    125 Dodge used=true
```
