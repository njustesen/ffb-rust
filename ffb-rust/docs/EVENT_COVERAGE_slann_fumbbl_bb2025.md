# Event coverage — HeuristicAgent, slann_fumbbl v slann_fumbbl, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=slann_fumbbl scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11622 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 788 | ok |  |
| action Blitz | 885 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 208 | ok |  |
| action Pass | 31 | ok | needs a ball carrier |
| action HandOver | 4 | ok | needs carrier + adjacent teammate |
| dodge success | 613 | ok |  |
| dodge failure | 287 | ok |  |
| GFI rolls | 3744 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 256 | ok |  |
| pickup failure | 155 | ok | turnover + scatter |
| catch success | 91 | ok |  |
| catch failure | 86 | ok |  |
| ball scatters | 607 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 10 | ok | ball out of bounds |
| pass rolls | 123 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 567 | ok |  |
| block 2 dice | 736 | ok |  |
| block 2 dice against | 280 | ok | defender's choice |
| block 3 dice | 90 | ok | needs ST5+ differential via assists |
| block result Skull | 234 | ok |  |
| block result BothDown | 270 | ok |  |
| block result Pushback | 537 | ok |  |
| block result PowPushback | 295 | ok |  |
| block result Pow | 337 | ok |  |
| pushbacks | 1162 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1590 | ok |  |
| armor held | 1231 | ok |  |
| stunned | 596 | ok | injury 2-7 |
| KO | 287 | ok |  |
| casualty (d16) | 195 | ok |  |
| death | 17 | ok | d16 = 15-16 only |
| fouls | 208 | ok |  |
| argue the call | 48 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 39 | ok |  |
| touchdowns | 43 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 28 | ok | kickoff event roll of 8 only |
| kickoff events | 239 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 33
- Charge: 27
- Cheering Fans: 37
- Dodgy Snack: 9
- Get the Ref: 9
- High Kick: 35
- Pitch Invasion: 7
- Quick Snap: 24
- Solid Defence: 15
- Time-out: 15
- Weather Change: 28

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2025/slann_fumbbl_vs_slann_fumbbl/seed_*_rust_events.jsonl)

Total events: 94738

```
  61195 playerMoved
  13538 playerAction
   3744 goForItRoll
   3409 turnEnd
   2309 injury
   1673 blockRoll
   1673 block
   1590 playerFellDown
   1162 pushback
    900 dodgeRoll
    607 scatterBall
    411 pickupRoll
    256 ballPickedUp
    239 kickoffScatter
    239 kickoffResultEvent
    208 refereeSpotsFoul
    208 foul
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    177 catchRoll
    123 passRoll
     94 handOver
     48 argueTheCall
     45 playerNote
     43 touchdown
     39 playerEjected
     38 kickoffExtraReRoll
     37 cheeringFans
     28 weatherChange
     24 quickSnapRoll
     15 solidDefenceRoll
     15 kickoffTimeout
     14 kickoffPitchInvasionStun
     10 throwIn
     10 dodgySnackRoll
      9 kickoffDodgySnack
      7 kickoffPitchInvasion
      1 throwAtStallingPlayer
```

## Player actions declared

```
  11419 Move
    885 BlitzMove
    788 Block
    208 Foul
    108 PassMove
     95 HandOverMove
     31 Pass
      4 HandOver
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
