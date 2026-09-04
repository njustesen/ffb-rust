# Event coverage — HeuristicAgent, human v human, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=human scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12141 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 886 | ok |  |
| action Blitz | 993 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 249 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 698 | ok |  |
| dodge failure | 446 | ok |  |
| GFI rolls | 4883 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 253 | ok |  |
| pickup failure | 138 | ok | turnover + scatter |
| catch success | 168 | ok |  |
| catch failure | 115 | ok |  |
| ball scatters | 582 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 8 | ok | ball out of bounds |
| pass rolls | 184 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 684 | ok |  |
| block 2 dice | 699 | ok |  |
| block 2 dice against | 418 | ok | defender's choice |
| block 3 dice | 28 | ok | needs ST5+ differential via assists |
| block result Skull | 267 | ok |  |
| block result BothDown | 285 | ok |  |
| block result Pushback | 619 | ok |  |
| block result PowPushback | 330 | ok |  |
| block result Pow | 328 | ok |  |
| pushbacks | 1272 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1602 | ok |  |
| armor held | 1498 | ok |  |
| stunned | 432 | ok | injury 2-7 |
| KO | 203 | ok |  |
| casualty (d16) | 135 | ok |  |
| death | 25 | ok | d16 = 15-16 only |
| fouls | 235 | ok |  |
| argue the call | 48 | ok | referee spotted a foul (doubles) |
| argue success | 5 | ok | d6 = 6 only |
| players ejected | 44 | ok |  |
| touchdowns | 62 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 28 | ok | kickoff event roll of 8 only |
| kickoff events | 256 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 47
- Charge: 24
- Cheering Fans: 46
- Dodgy Snack: 7
- Get the Ref: 7
- High Kick: 34
- Pitch Invasion: 7
- Quick Snap: 22
- Solid Defence: 22
- Time-out: 12
- Weather Change: 28

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/human_vs_human/seed_*_rust_events.jsonl)

Total events: 100608

```
  61838 playerMoved
  14808 playerAction
   4883 goForItRoll
   3436 turnEnd
   2268 injury
   1829 blockRoll
   1829 block
   1602 playerFellDown
   1416 confusionRoll
   1272 pushback
   1144 dodgeRoll
    582 scatterBall
    391 pickupRoll
    338 apothecaryRoll
    283 catchRoll
    256 kickoffScatter
    256 kickoffResultEvent
    253 ballPickedUp
    235 refereeSpotsFoul
    235 foul
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    184 passRoll
    110 handOver
     95 skillUse
     63 playerNote
     62 touchdown
     59 kickoffExtraReRoll
     48 argueTheCall
     46 cheeringFans
     44 playerEjected
     28 weatherChange
     22 solidDefenceRoll
     22 quickSnapRoll
     13 kickoffPitchInvasionStun
     12 kickoffTimeout
     11 throwTeamMateRoll
      8 throwIn
      7 kickoffPitchInvasion
      7 kickoffDodgySnack
      7 dodgySnackRoll
      6 rightStuffRoll
```

## Player actions declared

```
  11850 Move
    993 BlitzMove
    886 Block
    287 RaidingParty
    252 ThrowTeamMate
    249 Foul
    176 PassMove
    115 HandOverMove
```

## Skill uses / re-rolls seen

```
```
