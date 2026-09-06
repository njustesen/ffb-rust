# Event coverage — HeuristicAgent, underworld v underworld, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=underworld scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11433 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 694 | ok |  |
| action Blitz | 918 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 285 | ok |  |
| action Pass | 38 | ok | needs a ball carrier |
| action HandOver | 3 | ok | needs carrier + adjacent teammate |
| dodge success | 513 | ok |  |
| dodge failure | 328 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 224 | ok |  |
| pickup failure | 114 | ok | turnover + scatter |
| catch success | 102 | ok |  |
| catch failure | 91 | ok |  |
| ball scatters | 561 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 2 | ok | ball out of bounds |
| pass rolls | 158 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 816 | ok |  |
| block 2 dice | 644 | ok |  |
| block 2 dice against | 221 | ok | defender's choice |
| block 3 dice | 71 | ok | needs ST5+ differential via assists |
| block result Skull | 267 | ok |  |
| block result BothDown | 271 | ok |  |
| block result Pushback | 592 | ok |  |
| block result PowPushback | 302 | ok |  |
| block result Pow | 320 | ok |  |
| pushbacks | 1208 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 754 | ok |  |
| armor held | 1645 | ok |  |
| stunned | 385 | ok | injury 2-7 |
| KO | 199 | ok |  |
| casualty (d16) | 158 | ok |  |
| death | 27 | ok | d16 = 15-16 only |
| fouls | 241 | ok |  |
| argue the call | 43 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 43 | ok |  |
| touchdowns | 41 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 32 | ok | kickoff event roll of 8 only |
| kickoff events | 234 | ok | per-result table below |

## Kickoff results

- Blitz: 12
- Brilliant Coaching: 29
- Cheering Fans: 44
- Get the Ref: 5
- High Kick: 33
- Perfect Defence: 23
- Pitch Invasion: 3
- Quick Snap: 30
- Riot: 9
- Throw a Rock: 14
- Weather Change: 32

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2016/underworld_vs_underworld/seed_*_rust_events.jsonl)

Total events: 32212

```
  13917 playerAction
   3416 turnEnd
   2777 confusionRoll
   2387 injury
   1752 blockRoll
   1752 block
   1208 pushback
    841 dodgeRoll
    754 playerFellDown
    561 scatterBall
    338 pickupRoll
    241 refereeSpotsFoul
    241 foul
    234 kickoffScatter
    234 kickoffResultEvent
    224 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    193 catchRoll
    158 passRoll
     73 kickoffExtraReRollBb2016
     62 handOver
     43 playerEjected
     43 argueTheCall
     41 touchdown
     32 weatherChange
     22 throwTeamMateRoll
     21 regenerationRoll
     18 skillUse
     14 kickoffThrowARockBb2016
      9 kickoffRiot
      3 kickoffPitchInvasionBb2016
      2 throwIn
      1 playerNote
```

## Player actions declared

```
  11242 Move
    918 Blitz
    694 Block
    546 ThrowTeamMate
    285 Foul
    128 PassMove
     63 HandOverMove
     38 Pass
      3 HandOver
```

## Skill uses / re-rolls seen

```
     18 Dodge used=true
```
