# Event coverage — HeuristicAgent, high_elf v high_elf, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=high_elf scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13461 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 760 | ok |  |
| action Blitz | 938 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 253 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 1003 | ok |  |
| dodge failure | 208 | ok |  |
| GFI rolls | 5327 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 302 | ok |  |
| pickup failure | 65 | ok | turnover + scatter |
| catch success | 246 | ok |  |
| catch failure | 98 | ok |  |
| ball scatters | 561 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 32 | ok | ball out of bounds |
| pass rolls | 247 | ok |  |
| pass deviates | 58 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1223 | ok |  |
| block 2 dice | 370 | ok |  |
| block 2 dice against | 105 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 262 | ok |  |
| block result BothDown | 280 | ok |  |
| block result Pushback | 552 | ok |  |
| block result PowPushback | 328 | ok |  |
| block result Pow | 276 | ok |  |
| pushbacks | 1148 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1495 | ok |  |
| armor held | 1582 | ok |  |
| stunned | 381 | ok | injury 2-7 |
| KO | 164 | ok |  |
| casualty (d16) | 103 | ok |  |
| death | 11 | ok | d16 = 15-16 only |
| fouls | 253 | ok |  |
| argue the call | 46 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 44 | ok |  |
| touchdowns | 73 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 29 | ok | kickoff event roll of 8 only |
| kickoff events | 263 | ok | per-result table below |

## Kickoff results

- Blitz: 27
- Brilliant Coaching: 36
- Cheering Fans: 50
- Get the Ref: 5
- High Kick: 36
- Officious Ref: 10
- Pitch Invasion: 9
- Quick Snap: 30
- Solid Defence: 21
- Time-out: 10
- Weather Change: 29

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/high_elf_vs_high_elf/seed_*_rust_events.jsonl)

Total events: 106999

```
  69200 playerMoved
  15412 playerAction
   5327 goForItRoll
   3447 turnEnd
   2230 injury
   1698 blockRoll
   1698 block
   1495 playerFellDown
   1211 dodgeRoll
   1148 pushback
    561 scatterBall
    367 pickupRoll
    344 catchRoll
    302 ballPickedUp
    263 kickoffScatter
    263 kickoffResultEvent
    253 refereeSpotsFoul
    253 foul
    247 passRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    164 handOver
     73 touchdown
     58 passDeviate
     50 cheeringFans
     46 argueTheCall
     44 playerEjected
     36 prayerRoll
     32 throwIn
     30 quickSnapRoll
     29 weatherChange
     27 blitzRoll
     25 kickoffExtraReRoll
     21 solidDefenceRoll
     16 kickoffPitchInvasionStun
     10 kickoffTimeout
     10 kickoffOfficiousRef
      9 kickoffPitchInvasion
```

## Player actions declared

```
  13068 Move
    938 BlitzMove
    760 Block
    253 Foul
    224 PassMove
    169 HandOverMove
```

## Skill uses / re-rolls seen

```
```
