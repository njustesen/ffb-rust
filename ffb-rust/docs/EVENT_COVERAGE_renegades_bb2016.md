# Event coverage — HeuristicAgent, renegades v renegades, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=renegades scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11792 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 596 | ok |  |
| action Blitz | 898 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 265 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 636 | ok |  |
| dodge failure | 252 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 194 | ok |  |
| pickup failure | 95 | ok | turnover + scatter |
| catch success | 131 | ok |  |
| catch failure | 79 | ok |  |
| ball scatters | 501 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 12 | ok | ball out of bounds |
| pass rolls | 162 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 659 | ok |  |
| block 2 dice | 730 | ok |  |
| block 2 dice against | 219 | ok | defender's choice |
| block 3 dice | 71 | ok | needs ST5+ differential via assists |
| block result Skull | 214 | ok |  |
| block result BothDown | 247 | ok |  |
| block result Pushback | 566 | ok |  |
| block result PowPushback | 299 | ok |  |
| block result Pow | 353 | ok |  |
| pushbacks | 1212 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 864 | ok |  |
| armor held | 1971 | ok |  |
| stunned | 297 | ok | injury 2-7 |
| KO | 121 | ok |  |
| casualty (d16) | 107 | ok |  |
| death | 20 | ok | d16 = 15-16 only |
| fouls | 252 | ok |  |
| argue the call | 48 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 48 | ok |  |
| touchdowns | 21 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 29 | ok | kickoff event roll of 8 only |
| kickoff events | 213 | ok | per-result table below |

## Kickoff results

- Blitz: 14
- Brilliant Coaching: 20
- Cheering Fans: 33
- Get the Ref: 8
- High Kick: 27
- Perfect Defence: 17
- Pitch Invasion: 7
- Quick Snap: 29
- Riot: 17
- Throw a Rock: 12
- Weather Change: 29

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/renegades_vs_renegades/seed_*_rust_events.jsonl)

Total events: 32514

```
  14224 playerAction
   3379 turnEnd
   2505 confusionRoll
   2496 injury
   1679 blockRoll
   1679 block
   1212 pushback
    888 dodgeRoll
    864 playerFellDown
    501 scatterBall
    349 skillUse
    289 pickupRoll
    252 refereeSpotsFoul
    252 foul
    213 kickoffScatter
    213 kickoffResultEvent
    210 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    194 ballPickedUp
    162 passRoll
     74 handOver
     53 kickoffExtraReRollBb2016
     48 playerEjected
     48 argueTheCall
     29 weatherChange
     29 throwTeamMateRoll
     21 touchdown
     17 kickoffRiot
     12 throwIn
     12 kickoffThrowARockBb2016
      7 kickoffPitchInvasionBb2016
      3 regenerationRoll
```

## Player actions declared

```
  11531 Move
    898 Blitz
    673 ThrowTeamMate
    596 Block
    265 Foul
    178 PassMove
     83 HandOverMove
```

## Skill uses / re-rolls seen

```
    337 Horns used=true
     12 Dodge used=true
```
