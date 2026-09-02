# Event coverage — HeuristicAgent, elf v elf, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-02 by `MATCHUP=elf scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13372 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 875 | ok |  |
| action Blitz | 920 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 267 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 1022 | ok |  |
| dodge failure | 216 | ok |  |
| GFI rolls | 5239 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 312 | ok |  |
| pickup failure | 88 | ok | turnover + scatter |
| catch success | 233 | ok |  |
| catch failure | 108 | ok |  |
| ball scatters | 592 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 25 | ok | ball out of bounds |
| pass rolls | 243 | ok |  |
| pass deviates | 72 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1337 | ok |  |
| block 2 dice | 361 | ok |  |
| block 2 dice against | 97 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 254 | ok |  |
| block result BothDown | 289 | ok |  |
| block result Pushback | 578 | ok |  |
| block result PowPushback | 325 | ok |  |
| block result Pow | 349 | ok |  |
| pushbacks | 1246 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1483 | ok |  |
| armor held | 1338 | ok |  |
| stunned | 480 | ok | injury 2-7 |
| KO | 209 | ok |  |
| casualty (d16) | 133 | ok |  |
| death | 15 | ok | d16 = 15-16 only |
| fouls | 267 | ok |  |
| argue the call | 65 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 65 | ok |  |
| touchdowns | 78 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 32 | ok | kickoff event roll of 8 only |
| kickoff events | 267 | ok | per-result table below |

## Kickoff results

- Blitz: 22
- Brilliant Coaching: 47
- Cheering Fans: 44
- Get the Ref: 6
- High Kick: 29
- Officious Ref: 13
- Pitch Invasion: 7
- Quick Snap: 25
- Solid Defence: 29
- Time-out: 13
- Weather Change: 32

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/elf_vs_elf/seed_*_rust_events.jsonl)

Total events: 108333

```
  69867 playerMoved
  15434 playerAction
   5239 goForItRoll
   3457 turnEnd
   2160 injury
   1795 blockRoll
   1795 block
   1483 playerFellDown
   1246 pushback
   1238 dodgeRoll
    592 scatterBall
    400 pickupRoll
    342 apothecaryRoll
    341 catchRoll
    312 ballPickedUp
    267 refereeSpotsFoul
    267 kickoffScatter
    267 kickoffResultEvent
    267 foul
    243 passRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    137 handOver
     78 touchdown
     72 passDeviate
     65 playerEjected
     65 argueTheCall
     44 cheeringFans
     39 prayerRoll
     38 kickoffExtraReRoll
     32 weatherChange
     29 solidDefenceRoll
     25 throwIn
     25 quickSnapRoll
     22 blitzRoll
     14 kickoffPitchInvasionStun
     13 kickoffTimeout
     13 kickoffOfficiousRef
      7 kickoffPitchInvasion
      3 trapDoor
```

## Player actions declared

```
  13011 Move
    920 BlitzMove
    875 Block
    267 Foul
    221 PassMove
    140 HandOverMove
```

## Skill uses / re-rolls seen

```
```
