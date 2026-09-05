# Event coverage — HeuristicAgent, khemri_fumbbl v khemri_fumbbl, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=khemri_fumbbl scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12991 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 681 | ok |  |
| action Blitz | 888 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 270 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 775 | ok |  |
| dodge failure | 194 | ok |  |
| GFI rolls | 6477 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 300 | ok |  |
| pickup failure | 76 | ok | turnover + scatter |
| catch success | 134 | ok |  |
| catch failure | 38 | ok |  |
| ball scatters | 538 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 7 | ok | ball out of bounds |
| pass rolls | 199 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 743 | ok |  |
| block 2 dice | 602 | ok |  |
| block 2 dice against | 224 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 218 | ok |  |
| block result BothDown | 235 | ok |  |
| block result Pushback | 536 | ok |  |
| block result PowPushback | 272 | ok |  |
| block result Pow | 308 | ok |  |
| pushbacks | 1110 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1559 | ok |  |
| armor held | 1379 | ok |  |
| stunned | 579 | ok | injury 2-7 |
| KO | 173 | ok |  |
| casualty (d16) | 142 | ok |  |
| death | 21 | ok | d16 = 15-16 only |
| fouls | 270 | ok |  |
| argue the call | 57 | ok | referee spotted a foul (doubles) |
| argue success | 12 | ok | d6 = 6 only |
| players ejected | 46 | ok |  |
| touchdowns | 21 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 24 | ok | kickoff event roll of 8 only |
| kickoff events | 216 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 29
- Charge: 22
- Cheering Fans: 40
- Dodgy Snack: 9
- Get the Ref: 4
- High Kick: 34
- Pitch Invasion: 5
- Quick Snap: 22
- Solid Defence: 15
- Time-out: 12
- Weather Change: 24

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/khemri_fumbbl_vs_khemri_fumbbl/seed_*_rust_events.jsonl)

Total events: 94603

```
  57102 playerMoved
  14830 playerAction
   6477 goForItRoll
   3396 turnEnd
   2273 injury
   1569 blockRoll
   1569 block
   1559 playerFellDown
   1110 pushback
    969 dodgeRoll
    538 scatterBall
    376 pickupRoll
    300 ballPickedUp
    270 refereeSpotsFoul
    270 foul
    216 kickoffScatter
    216 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    199 passRoll
    172 catchRoll
    151 regenerationRoll
    107 handOver
     57 argueTheCall
     46 playerEjected
     40 cheeringFans
     36 kickoffExtraReRoll
     24 weatherChange
     22 quickSnapRoll
     21 touchdown
     21 playerNote
     15 solidDefenceRoll
     12 kickoffTimeout
     10 dodgySnackRoll
      9 kickoffPitchInvasionStun
      9 kickoffDodgySnack
      7 throwIn
      5 kickoffPitchInvasion
```

## Player actions declared

```
  12669 Move
    888 BlitzMove
    681 Block
    270 Foul
    214 PassMove
    108 HandOverMove
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
