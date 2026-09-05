# Event coverage — HeuristicAgent, lineman v lineman, bb2025, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=lineman scripts/harvest_coverage.sh bb2025 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12334 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 668 | ok |  |
| action Blitz | 854 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 239 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 594 | ok |  |
| dodge failure | 329 | ok |  |
| GFI rolls | 5041 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 248 | ok |  |
| pickup failure | 132 | ok | turnover + scatter |
| catch success | 120 | ok |  |
| catch failure | 107 | ok |  |
| ball scatters | 571 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 13 | ok | ball out of bounds |
| pass rolls | 194 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1074 | ok |  |
| block 2 dice | 353 | ok |  |
| block 2 dice against | 95 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 231 | ok |  |
| block result BothDown | 252 | ok |  |
| block result Pushback | 466 | ok |  |
| block result PowPushback | 258 | ok |  |
| block result Pow | 315 | ok |  |
| pushbacks | 1031 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1549 | ok |  |
| armor held | 1312 | ok |  |
| stunned | 559 | ok | injury 2-7 |
| KO | 248 | ok |  |
| casualty (d16) | 160 | ok |  |
| death | 18 | ok | d16 = 15-16 only |
| fouls | 239 | ok |  |
| argue the call | 44 | ok | referee spotted a foul (doubles) |
| argue success | 8 | ok | d6 = 6 only |
| players ejected | 38 | ok |  |
| touchdowns | 31 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 34 | ok | kickoff event roll of 8 only |
| kickoff events | 226 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 31
- Charge: 17
- Cheering Fans: 38
- Dodgy Snack: 12
- Get the Ref: 5
- High Kick: 25
- Pitch Invasion: 9
- Quick Snap: 26
- Solid Defence: 19
- Time-out: 10
- Weather Change: 34

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2025/lineman_vs_lineman/seed_*_rust_events.jsonl)

Total events: 96093

```
  60701 playerMoved
  14095 playerAction
   5041 goForItRoll
   3406 turnEnd
   2279 injury
   1549 playerFellDown
   1522 blockRoll
   1522 block
   1031 pushback
    923 dodgeRoll
    571 scatterBall
    408 apothecaryRoll
    380 pickupRoll
    248 ballPickedUp
    239 refereeSpotsFoul
    239 foul
    227 catchRoll
    226 kickoffScatter
    226 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    194 passRoll
     94 handOver
     44 argueTheCall
     38 playerEjected
     38 cheeringFans
     36 kickoffExtraReRoll
     34 weatherChange
     31 touchdown
     31 playerNote
     26 quickSnapRoll
     19 solidDefenceRoll
     16 kickoffPitchInvasionStun
     15 dodgySnackRoll
     13 throwIn
     12 kickoffDodgySnack
     10 kickoffTimeout
      9 kickoffPitchInvasion
```

## Player actions declared

```
  12057 Move
    854 BlitzMove
    668 Block
    239 Foul
    181 PassMove
     96 HandOverMove
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
