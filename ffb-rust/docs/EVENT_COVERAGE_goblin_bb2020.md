# Event coverage — HeuristicAgent, goblin v goblin, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-04 by `MATCHUP=goblin scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 10843 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 716 | ok |  |
| action Blitz | 835 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 214 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 425 | ok |  |
| dodge failure | 269 | ok |  |
| GFI rolls | 4232 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 209 | ok |  |
| pickup failure | 121 | ok | turnover + scatter |
| catch success | 209 | ok |  |
| catch failure | 179 | ok |  |
| ball scatters | 548 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 14 | ok | ball out of bounds |
| pass rolls | 746 | ok |  |
| pass deviates | 157 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 615 | ok |  |
| block 2 dice | 297 | ok |  |
| block 2 dice against | 111 | ok | defender's choice |
| block 3 dice | 412 | ok | needs ST5+ differential via assists |
| block result Skull | 191 | ok |  |
| block result BothDown | 250 | ok |  |
| block result Pushback | 478 | ok |  |
| block result PowPushback | 196 | ok |  |
| block result Pow | 320 | ok |  |
| pushbacks | 1013 | ok |  |
| crowd surfs | 721 | ok | push off pitch — board-position dependent |
| players fell | 1270 | ok |  |
| armor held | 1365 | ok |  |
| stunned | 398 | ok | injury 2-7 |
| KO | 362 | ok |  |
| casualty (d16) | 245 | ok |  |
| death | 30 | ok | d16 = 15-16 only |
| fouls | 200 | ok |  |
| argue the call | 43 | ok | referee spotted a foul (doubles) |
| argue success | 10 | ok | d6 = 6 only |
| players ejected | 46 | ok |  |
| touchdowns | 42 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 33 | ok | kickoff event roll of 8 only |
| kickoff events | 234 | ok | per-result table below |

## Kickoff results

- Blitz: 19
- Brilliant Coaching: 39
- Cheering Fans: 34
- Get the Ref: 6
- High Kick: 28
- Officious Ref: 12
- Pitch Invasion: 8
- Quick Snap: 26
- Solid Defence: 17
- Time-out: 12
- Weather Change: 33

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/goblin_vs_goblin/seed_*_rust_events.jsonl)

Total events: 88243

```
  51030 playerMoved
  13331 playerAction
   4232 goForItRoll
   3410 turnEnd
   2370 injury
   1748 confusionRoll
   1435 blockRoll
   1435 block
   1270 playerFellDown
   1013 pushback
    746 passRoll
    721 scatterPlayer
    694 dodgeRoll
    548 scatterBall
    489 apothecaryRoll
    388 catchRoll
    380 spellEffectRoll
    330 pickupRoll
    234 kickoffScatter
    234 kickoffResultEvent
    209 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 refereeSpotsFoul
    200 mvpRoll
    200 foul
    157 passDeviate
     99 skillUse
     92 alwaysHungry
     83 throwTeamMateRoll
     75 handOver
     46 playerEjected
     43 argueTheCall
     42 touchdown
     42 rightStuffRoll
     34 cheeringFans
     33 weatherChange
     33 kickoffExtraReRoll
     31 bombOutOfBounds
     27 prayerRoll
     26 quickSnapRoll
     19 escapeRoll
     19 blitzRoll
     17 solidDefenceRoll
     15 kickoffPitchInvasionStun
     14 throwIn
     12 kickoffTimeout
     12 kickoffOfficiousRef
     11 swoopPlayer
      8 kickoffPitchInvasion
      6 trapDoor
```

## Player actions declared

```
  10620 Move
    835 BlitzMove
    716 Block
    517 ThrowBomb
    214 Foul
    206 ThrowTeamMate
    146 PassMove
     77 HandOverMove
```

## Skill uses / re-rolls seen

```
```
