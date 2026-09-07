# Event coverage — HeuristicAgent, undead v undead, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-07 by `MATCHUP=undead scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12078 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 704 | ok |  |
| action Blitz | 936 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 261 | ok |  |
| action Pass | 22 | ok | needs a ball carrier |
| action HandOver | 3 | ok | needs carrier + adjacent teammate |
| dodge success | 415 | ok |  |
| dodge failure | 347 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 217 | ok |  |
| pickup failure | 164 | ok | turnover + scatter |
| catch success | 112 | ok |  |
| catch failure | 126 | ok |  |
| ball scatters | 577 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 5 | ok | ball out of bounds |
| pass rolls | 146 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 766 | ok |  |
| block 2 dice | 580 | ok |  |
| block 2 dice against | 257 | ok | defender's choice |
| block 3 dice | 8 | ok | needs ST5+ differential via assists |
| block result Skull | 220 | ok |  |
| block result BothDown | 259 | ok |  |
| block result Pushback | 570 | ok |  |
| block result PowPushback | 248 | ok |  |
| block result Pow | 314 | ok |  |
| pushbacks | 1129 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 927 | ok |  |
| armor held | 1608 | ok |  |
| stunned | 390 | ok | injury 2-7 |
| KO | 173 | ok |  |
| casualty (d16) | 150 | ok |  |
| death | 26 | ok | d16 = 15-16 only |
| fouls | 261 | ok |  |
| argue the call | 62 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 62 | ok |  |
| touchdowns | 24 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 37 | ok | kickoff event roll of 8 only |
| kickoff events | 222 | ok | per-result table below |

## Kickoff results

- Blitz: 15
- Brilliant Coaching: 24
- Cheering Fans: 40
- Get the Ref: 7
- High Kick: 24
- Perfect Defence: 22
- Pitch Invasion: 1
- Quick Snap: 25
- Riot: 11
- Throw a Rock: 16
- Weather Change: 37

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2016/undead_vs_undead/seed_*_rust_events.jsonl)

Total events: 29441

```
  14004 playerAction
   3400 turnEnd
   2321 injury
   1611 blockRoll
   1611 block
   1129 pushback
    927 playerFellDown
    762 dodgeRoll
    577 scatterBall
    381 pickupRoll
    261 refereeSpotsFoul
    261 foul
    238 catchRoll
    222 kickoffScatter
    222 kickoffResultEvent
    217 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    146 passRoll
    105 regenerationRoll
    102 handOver
     64 kickoffExtraReRollBb2016
     62 playerEjected
     62 argueTheCall
     50 skillUse
     37 weatherChange
     24 touchdown
     16 kickoffThrowARockBb2016
     12 playerAdded
     11 kickoffRiot
      5 throwIn
      1 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  11843 Move
    936 Blitz
    704 Block
    261 Foul
    136 PassMove
     99 HandOverMove
     22 Pass
      3 HandOver
```

## Skill uses / re-rolls seen

```
     50 Dodge used=true
```
