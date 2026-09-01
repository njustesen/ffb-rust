# Event coverage — HeuristicAgent, dark_elf v dark_elf, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-01 by `MATCHUP=dark_elf scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12713 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 799 | ok |  |
| action Blitz | 900 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 290 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 1019 | ok |  |
| dodge failure | 144 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 244 | ok |  |
| pickup failure | 60 | ok | turnover + scatter |
| catch success | 202 | ok |  |
| catch failure | 60 | ok |  |
| ball scatters | 476 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 10 | ok | ball out of bounds |
| pass rolls | 183 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1476 | ok |  |
| block 2 dice | 344 | ok |  |
| block 2 dice against | 132 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 292 | ok |  |
| block result BothDown | 334 | ok |  |
| block result Pushback | 656 | ok |  |
| block result PowPushback | 322 | ok |  |
| block result Pow | 348 | ok |  |
| pushbacks | 1318 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 640 | ok |  |
| armor held | 1415 | ok |  |
| stunned | 464 | ok | injury 2-7 |
| KO | 188 | ok |  |
| casualty (d16) | 138 | ok |  |
| death | 32 | ok | d16 = 15-16 only |
| fouls | 290 | ok |  |
| argue the call | 55 | ok | referee spotted a foul (doubles) |
| argue success | 12 | ok | d6 = 6 only |
| players ejected | 55 | ok |  |
| touchdowns | 71 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 42 | ok | kickoff event roll of 8 only |
| kickoff events | 262 | ok | per-result table below |

## Kickoff results

- Blitz: 15
- Brilliant Coaching: 29
- Cheering Fans: 45
- Get the Ref: 6
- High Kick: 33
- Perfect Defence: 22
- Pitch Invasion: 7
- Quick Snap: 27
- Riot: 19
- Throw a Rock: 17
- Weather Change: 42

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/dark_elf_vs_dark_elf/seed_*_rust_events.jsonl)

Total events: 31204

```
  14702 playerAction
   3436 turnEnd
   2205 injury
   1952 blockRoll
   1952 block
   1318 pushback
   1163 dodgeRoll
    640 playerFellDown
    476 scatterBall
    304 pickupRoll
    290 refereeSpotsFoul
    290 foul
    262 kickoffScatter
    262 kickoffResultEvent
    262 catchRoll
    244 ballPickedUp
    209 skillUse
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    183 passRoll
    104 handOver
     74 kickoffExtraReRollBb2016
     71 touchdown
     55 playerEjected
     55 argueTheCall
     42 weatherChange
     19 kickoffRiot
     17 kickoffThrowARockBb2016
     10 throwIn
      7 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  12404 Move
    900 Blitz
    799 Block
    290 Foul
    203 PassMove
    106 HandOverMove
```

## Skill uses / re-rolls seen

```
```
