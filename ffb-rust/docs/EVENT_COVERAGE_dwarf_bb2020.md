# Event coverage — HeuristicAgent, dwarf v dwarf, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-02 by `MATCHUP=dwarf scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12560 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 727 | ok |  |
| action Blitz | 883 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 268 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 414 | ok |  |
| dodge failure | 293 | ok |  |
| GFI rolls | 6921 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 187 | ok |  |
| pickup failure | 151 | ok | turnover + scatter |
| catch success | 75 | ok |  |
| catch failure | 106 | ok |  |
| ball scatters | 546 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 12 | ok | ball out of bounds |
| pass rolls | 140 | ok |  |
| pass deviates | 36 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 862 | ok |  |
| block 2 dice | 258 | ok |  |
| block 2 dice against | 96 | ok | defender's choice |
| block 3 dice | 394 | ok | needs ST5+ differential via assists |
| block result Skull | 233 | ok |  |
| block result BothDown | 231 | ok |  |
| block result Pushback | 541 | ok |  |
| block result PowPushback | 305 | ok |  |
| block result Pow | 300 | ok |  |
| pushbacks | 1116 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1659 | ok |  |
| armor held | 1761 | ok |  |
| stunned | 318 | ok | injury 2-7 |
| KO | 54 | ok |  |
| casualty (d16) | 86 | ok |  |
| death | 11 | ok | d16 = 15-16 only |
| fouls | 268 | ok |  |
| argue the call | 61 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 60 | ok |  |
| touchdowns | 12 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 27 | ok | kickoff event roll of 8 only |
| kickoff events | 212 | ok | per-result table below |

## Kickoff results

- Blitz: 21
- Brilliant Coaching: 29
- Cheering Fans: 33
- Get the Ref: 8
- High Kick: 31
- Officious Ref: 8
- Pitch Invasion: 7
- Quick Snap: 20
- Solid Defence: 16
- Time-out: 12
- Weather Change: 27

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/dwarf_vs_dwarf/seed_*_rust_events.jsonl)

Total events: 84870

```
  47763 playerMoved
  14438 playerAction
   6921 goForItRoll
   3392 turnEnd
   2219 injury
   1659 playerFellDown
   1610 blockRoll
   1610 block
   1116 pushback
    707 dodgeRoll
    546 scatterBall
    338 pickupRoll
    268 refereeSpotsFoul
    268 foul
    212 kickoffScatter
    212 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    187 ballPickedUp
    181 catchRoll
    140 passRoll
     69 handOver
     61 argueTheCall
     60 playerEjected
     36 passDeviate
     33 cheeringFans
     28 prayerRoll
     27 weatherChange
     24 kickoffExtraReRoll
     21 skillUse
     21 blitzRoll
     20 quickSnapRoll
     16 solidDefenceRoll
     14 kickoffPitchInvasionStun
     12 touchdown
     12 throwIn
     12 kickoffTimeout
      8 kickoffOfficiousRef
      7 kickoffPitchInvasion
      2 trapDoor
```

## Player actions declared

```
  12354 Move
    883 BlitzMove
    727 Block
    268 Foul
    134 PassMove
     72 HandOverMove
```

## Skill uses / re-rolls seen

```
```
