# Event coverage — HeuristicAgent, ogre v ogre, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=ogre scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11052 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 605 | ok |  |
| action Blitz | 785 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 226 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 313 | ok |  |
| dodge failure | 232 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 171 | ok |  |
| pickup failure | 124 | ok | turnover + scatter |
| catch success | 62 | ok |  |
| catch failure | 80 | ok |  |
| ball scatters | 512 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 12 | ok | ball out of bounds |
| pass rolls | 123 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 641 | ok |  |
| block 2 dice | 173 | ok |  |
| block 2 dice against | 62 | ok | defender's choice |
| block 3 dice | 327 | ok | needs ST5+ differential via assists |
| block result Skull | 164 | ok |  |
| block result BothDown | 197 | ok |  |
| block result Pushback | 426 | ok |  |
| block result PowPushback | 175 | ok |  |
| block result Pow | 241 | ok |  |
| pushbacks | 841 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 874 | ok |  |
| armor held | 1370 | ok |  |
| stunned | 523 | ok | injury 2-7 |
| KO | 293 | ok |  |
| casualty (d16) | 268 | ok |  |
| death | 37 | ok | d16 = 15-16 only |
| fouls | 200 | ok |  |
| argue the call | 41 | ok | referee spotted a foul (doubles) |
| argue success | 6 | ok | d6 = 6 only |
| players ejected | 41 | ok |  |
| touchdowns | 10 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 37 | ok | kickoff event roll of 8 only |
| kickoff events | 209 | ok | per-result table below |

## Kickoff results

- Blitz: 11
- Brilliant Coaching: 27
- Cheering Fans: 33
- Get the Ref: 4
- High Kick: 26
- Perfect Defence: 21
- Pitch Invasion: 3
- Quick Snap: 26
- Riot: 10
- Throw a Rock: 11
- Weather Change: 37

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/ogre_vs_ogre/seed_*_rust_events.jsonl)

Total events: 37468

```
  14442 playerAction
   8798 confusionRoll
   3389 turnEnd
   2454 injury
   1203 blockRoll
   1203 block
    874 playerFellDown
    841 pushback
    751 throwTeamMateRoll
    545 dodgeRoll
    512 scatterBall
    295 pickupRoll
    209 kickoffScatter
    209 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 refereeSpotsFoul
    200 mvpRoll
    200 foul
    171 ballPickedUp
    142 catchRoll
    123 passRoll
     60 kickoffExtraReRollBb2016
     49 handOver
     41 playerEjected
     41 argueTheCall
     37 weatherChange
     33 skillUse
     12 throwIn
     11 kickoffThrowARockBb2016
     10 touchdown
     10 kickoffRiot
      3 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  10872 Move
   1774 ThrowTeamMate
    785 Blitz
    605 Block
    226 Foul
    127 PassMove
     53 HandOverMove
```

## Skill uses / re-rolls seen

```
     33 Dodge used=true
```
