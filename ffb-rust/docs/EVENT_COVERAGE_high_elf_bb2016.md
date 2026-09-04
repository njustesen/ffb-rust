# Event coverage — HeuristicAgent, high_elf v high_elf, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=high_elf scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12952 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 820 | ok |  |
| action Blitz | 964 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 259 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 1057 | ok |  |
| dodge failure | 227 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 290 | ok |  |
| pickup failure | 67 | ok | turnover + scatter |
| catch success | 247 | ok |  |
| catch failure | 103 | ok |  |
| ball scatters | 552 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 17 | ok | ball out of bounds |
| pass rolls | 222 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1321 | ok |  |
| block 2 dice | 354 | ok |  |
| block 2 dice against | 109 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 254 | ok |  |
| block result BothDown | 268 | ok |  |
| block result Pushback | 607 | ok |  |
| block result PowPushback | 345 | ok |  |
| block result Pow | 310 | ok |  |
| pushbacks | 1257 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 697 | ok |  |
| armor held | 1471 | ok |  |
| stunned | 410 | ok | injury 2-7 |
| KO | 173 | ok |  |
| casualty (d16) | 129 | ok |  |
| death | 27 | ok | d16 = 15-16 only |
| fouls | 259 | ok |  |
| argue the call | 54 | ok | referee spotted a foul (doubles) |
| argue success | 10 | ok | d6 = 6 only |
| players ejected | 54 | ok |  |
| touchdowns | 113 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 57 | ok | kickoff event roll of 8 only |
| kickoff events | 308 | ok | per-result table below |

## Kickoff results

- Blitz: 21
- Brilliant Coaching: 35
- Cheering Fans: 55
- Get the Ref: 14
- High Kick: 31
- Perfect Defence: 23
- Pitch Invasion: 5
- Quick Snap: 36
- Riot: 14
- Throw a Rock: 17
- Weather Change: 57

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/high_elf_vs_high_elf/seed_*_rust_events.jsonl)

Total events: 31822

```
  14995 playerAction
   3492 turnEnd
   2183 injury
   1784 blockRoll
   1784 block
   1284 dodgeRoll
   1257 pushback
    697 playerFellDown
    552 scatterBall
    357 pickupRoll
    350 catchRoll
    308 kickoffScatter
    308 kickoffResultEvent
    302 apothecaryRoll
    290 ballPickedUp
    259 refereeSpotsFoul
    259 foul
    222 passRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    116 handOver
    113 touchdown
     90 kickoffExtraReRollBb2016
     57 weatherChange
     54 playerEjected
     54 argueTheCall
     17 throwIn
     17 kickoffThrowARockBb2016
     14 kickoffRiot
      5 kickoffPitchInvasionBb2016
      2 safeThrowRoll
```

## Player actions declared

```
  12603 Move
    964 Blitz
    820 Block
    259 Foul
    233 PassMove
    116 HandOverMove
```

## Skill uses / re-rolls seen

```
```
