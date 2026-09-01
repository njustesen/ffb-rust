# Event coverage — HeuristicAgent, dark_elf v dark_elf, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-01 by `MATCHUP=dark_elf scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12291 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 665 | ok |  |
| action Blitz | 840 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 234 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 739 | ok |  |
| dodge failure | 258 | ok |  |
| GFI rolls | 4483 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 292 | ok |  |
| pickup failure | 82 | ok | turnover + scatter |
| catch success | 176 | ok |  |
| catch failure | 72 | ok |  |
| ball scatters | 546 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 27 | ok | ball out of bounds |
| pass rolls | 204 | ok |  |
| pass deviates | 58 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1067 | ok |  |
| block 2 dice | 307 | ok |  |
| block 2 dice against | 92 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 206 | ok |  |
| block result BothDown | 237 | ok |  |
| block result Pushback | 480 | ok |  |
| block result PowPushback | 272 | ok |  |
| block result Pow | 271 | ok |  |
| pushbacks | 1009 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1396 | ok |  |
| armor held | 1338 | ok |  |
| stunned | 470 | ok | injury 2-7 |
| KO | 191 | ok |  |
| casualty (d16) | 134 | ok |  |
| death | 16 | ok | d16 = 15-16 only |
| fouls | 234 | ok |  |
| argue the call | 49 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 47 | ok |  |
| touchdowns | 57 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 199 | ok |  |
| weather changes | 37 | ok | kickoff event roll of 8 only |
| kickoff events | 247 | ok | per-result table below |

## Kickoff results

- Blitz: 19
- Brilliant Coaching: 35
- Cheering Fans: 39
- Get the Ref: 9
- High Kick: 33
- Officious Ref: 8
- Pitch Invasion: 6
- Quick Snap: 27
- Solid Defence: 21
- Time-out: 13
- Weather Change: 37

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/dark_elf_vs_dark_elf/seed_*_rust_events.jsonl)

Total events: 99099

```
  64010 playerMoved
  14235 playerAction
   4483 goForItRoll
   3326 turnEnd
   2133 injury
   1466 blockRoll
   1466 block
   1396 playerFellDown
   1009 pushback
    997 dodgeRoll
    546 scatterBall
    374 pickupRoll
    341 skillUse
    328 foulAppearanceRoll
    292 ballPickedUp
    248 catchRoll
    247 kickoffScatter
    247 kickoffResultEvent
    234 refereeSpotsFoul
    234 foul
    204 passRoll
    199 startHalf
    188 winningsRoll
    188 mvpRoll
    114 passBlock
    110 handOver
     58 passDeviate
     57 touchdown
     49 argueTheCall
     47 playerEjected
     39 cheeringFans
     37 weatherChange
     33 prayerRoll
     30 kickoffExtraReRoll
     27 throwIn
     27 quickSnapRoll
     21 solidDefenceRoll
     19 blitzRoll
     13 kickoffTimeout
     11 kickoffPitchInvasionStun
      8 kickoffOfficiousRef
      6 kickoffPitchInvasion
      2 trapDoor
```

## Player actions declared

```
  11984 Move
    840 BlitzMove
    665 Block
    234 Foul
    193 PassMove
    138 BlackInk
    114 HandOverMove
     67 MultipleBlock
```

## Skill uses / re-rolls seen

```
```
