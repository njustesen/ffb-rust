# Event coverage — HeuristicAgent, wood_elf v wood_elf, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=wood_elf scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12726 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 920 | ok |  |
| action Blitz | 989 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 273 | ok |  |
| action Pass | 34 | ok | needs a ball carrier |
| action HandOver | 5 | ok | needs carrier + adjacent teammate |
| dodge success | 995 | ok |  |
| dodge failure | 178 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 246 | ok |  |
| pickup failure | 88 | ok | turnover + scatter |
| catch success | 191 | ok |  |
| catch failure | 85 | ok |  |
| ball scatters | 515 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 9 | ok | ball out of bounds |
| pass rolls | 178 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 762 | ok |  |
| block 2 dice | 641 | ok |  |
| block 2 dice against | 354 | ok | defender's choice |
| block 3 dice | 103 | ok | needs ST5+ differential via assists |
| block result Skull | 254 | ok |  |
| block result BothDown | 321 | ok |  |
| block result Pushback | 676 | ok |  |
| block result PowPushback | 272 | ok |  |
| block result Pow | 337 | ok |  |
| pushbacks | 1279 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 647 | ok |  |
| armor held | 1277 | ok |  |
| stunned | 415 | ok | injury 2-7 |
| KO | 182 | ok |  |
| casualty (d16) | 144 | ok |  |
| death | 33 | ok | d16 = 15-16 only |
| fouls | 273 | ok |  |
| argue the call | 56 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 60 | ok |  |
| touchdowns | 98 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 49 | ok | kickoff event roll of 8 only |
| kickoff events | 288 | ok | per-result table below |

## Kickoff results

- Blitz: 20
- Brilliant Coaching: 36
- Cheering Fans: 45
- Get the Ref: 8
- High Kick: 39
- Perfect Defence: 27
- Pitch Invasion: 4
- Quick Snap: 31
- Riot: 17
- Throw a Rock: 12
- Weather Change: 49

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2016/wood_elf_vs_wood_elf/seed_*_rust_events.jsonl)

Total events: 31747

```
  15201 playerAction
   3474 turnEnd
   2018 injury
   1860 blockRoll
   1860 block
   1279 pushback
   1173 dodgeRoll
    647 playerFellDown
    515 scatterBall
    334 pickupRoll
    327 standUpRoll
    288 kickoffScatter
    288 kickoffResultEvent
    276 catchRoll
    273 refereeSpotsFoul
    273 foul
    246 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    178 passRoll
    154 skillUse
     98 touchdown
     97 handOver
     81 kickoffExtraReRollBb2016
     60 playerEjected
     56 argueTheCall
     49 weatherChange
     17 kickoffRiot
     12 kickoffThrowARockBb2016
      9 throwIn
      4 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  12478 Move
    989 Blitz
    920 Block
    273 Foul
    254 ThrowTeamMate
    152 PassMove
     96 HandOverMove
     34 Pass
      5 HandOver
```

## Skill uses / re-rolls seen

```
    154 Dodge used=true
```
