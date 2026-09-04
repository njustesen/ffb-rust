# Event coverage — HeuristicAgent, goblin v goblin, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=goblin scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11063 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 654 | ok |  |
| action Blitz | 823 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 224 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 344 | ok |  |
| dodge failure | 156 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 211 | ok |  |
| pickup failure | 111 | ok | turnover + scatter |
| catch success | 228 | ok |  |
| catch failure | 198 | ok |  |
| ball scatters | 521 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 9 | ok | ball out of bounds |
| pass rolls | 823 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 505 | ok |  |
| block 2 dice | 304 | ok |  |
| block 2 dice against | 115 | ok | defender's choice |
| block 3 dice | 290 | ok | needs ST5+ differential via assists |
| block result Skull | 159 | ok |  |
| block result BothDown | 186 | ok |  |
| block result Pushback | 412 | ok |  |
| block result PowPushback | 194 | ok |  |
| block result Pow | 263 | ok |  |
| pushbacks | 880 | ok |  |
| crowd surfs | 646 | ok | push off pitch — board-position dependent |
| players fell | 640 | ok |  |
| armor held | 1484 | ok |  |
| stunned | 362 | ok | injury 2-7 |
| KO | 255 | ok |  |
| casualty (d16) | 269 | ok |  |
| death | 40 | ok | d16 = 15-16 only |
| fouls | 182 | ok |  |
| argue the call | 36 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 42 | ok |  |
| touchdowns | 22 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 34 | ok | kickoff event roll of 8 only |
| kickoff events | 216 | ok | per-result table below |

## Kickoff results

- Blitz: 16
- Brilliant Coaching: 28
- Cheering Fans: 38
- Get the Ref: 4
- High Kick: 26
- Perfect Defence: 22
- Pitch Invasion: 2
- Quick Snap: 27
- Riot: 8
- Throw a Rock: 11
- Weather Change: 34

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/goblin_vs_goblin/seed_*_rust_events.jsonl)

Total events: 32995

```
  13840 playerAction
   3400 turnEnd
   3167 confusionRoll
   2370 injury
   1214 blockRoll
   1214 block
    880 pushback
    823 passRoll
    646 scatterPlayer
    640 playerFellDown
    521 scatterBall
    500 dodgeRoll
    482 spellEffectRoll
    431 apothecaryRoll
    426 catchRoll
    322 pickupRoll
    216 kickoffScatter
    216 kickoffResultEvent
    211 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    182 refereeSpotsFoul
    182 foul
     93 skillUse
     89 throwTeamMateRoll
     74 handOver
     66 kickoffExtraReRollBb2016
     42 playerEjected
     36 argueTheCall
     34 weatherChange
     22 touchdown
     13 chainsawRoll
     13 bombOutOfBounds
     11 kickoffThrowARockBb2016
      9 throwIn
      8 kickoffRiot
      2 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  10831 Move
    823 Blitz
    654 Block
    599 ThrowTeamMate
    477 ThrowBomb
    224 Foul
    157 PassMove
     75 HandOverMove
```

## Skill uses / re-rolls seen

```
```
