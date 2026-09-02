# Event coverage — HeuristicAgent, elf v elf, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-02 by `MATCHUP=elf scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13077 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 882 | ok |  |
| action Blitz | 953 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 265 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 1039 | ok |  |
| dodge failure | 234 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 280 | ok |  |
| pickup failure | 74 | ok | turnover + scatter |
| catch success | 257 | ok |  |
| catch failure | 70 | ok |  |
| ball scatters | 533 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 15 | ok | ball out of bounds |
| pass rolls | 234 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1347 | ok |  |
| block 2 dice | 374 | ok |  |
| block 2 dice against | 114 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 280 | ok |  |
| block result BothDown | 311 | ok |  |
| block result Pushback | 610 | ok |  |
| block result PowPushback | 316 | ok |  |
| block result Pow | 318 | ok |  |
| pushbacks | 1239 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 680 | ok |  |
| armor held | 1349 | ok |  |
| stunned | 513 | ok | injury 2-7 |
| KO | 208 | ok |  |
| casualty (d16) | 128 | ok |  |
| death | 30 | ok | d16 = 15-16 only |
| fouls | 265 | ok |  |
| argue the call | 57 | ok | referee spotted a foul (doubles) |
| argue success | 8 | ok | d6 = 6 only |
| players ejected | 59 | ok |  |
| touchdowns | 109 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 42 | ok | kickoff event roll of 8 only |
| kickoff events | 302 | ok | per-result table below |

## Kickoff results

- Blitz: 23
- Brilliant Coaching: 30
- Cheering Fans: 54
- Get the Ref: 7
- High Kick: 44
- Perfect Defence: 28
- Pitch Invasion: 5
- Quick Snap: 36
- Riot: 13
- Throw a Rock: 20
- Weather Change: 42

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/elf_vs_elf/seed_*_rust_events.jsonl)

Total events: 31695

```
  15177 playerAction
   3488 turnEnd
   2198 injury
   1835 blockRoll
   1835 block
   1273 dodgeRoll
   1239 pushback
    680 playerFellDown
    533 scatterBall
    354 pickupRoll
    327 catchRoll
    302 kickoffScatter
    302 kickoffResultEvent
    280 ballPickedUp
    265 refereeSpotsFoul
    265 foul
    234 passRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    109 touchdown
    103 handOver
     84 kickoffExtraReRollBb2016
     59 playerEjected
     57 argueTheCall
     42 weatherChange
     20 kickoffThrowARockBb2016
     15 throwIn
     13 kickoffRiot
      5 kickoffPitchInvasionBb2016
      1 playerNote
```

## Player actions declared

```
  12722 Move
    953 Blitz
    882 Block
    265 Foul
    249 PassMove
    106 HandOverMove
```

## Skill uses / re-rolls seen

```
```
