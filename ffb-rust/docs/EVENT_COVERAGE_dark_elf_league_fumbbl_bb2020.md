# Event coverage — HeuristicAgent, dark_elf_league_fumbbl v dark_elf_league_fumbbl, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-02 by `MATCHUP=dark_elf_league_fumbbl scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13501 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 820 | ok |  |
| action Blitz | 895 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 255 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 963 | ok |  |
| dodge failure | 214 | ok |  |
| GFI rolls | 5192 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 283 | ok |  |
| pickup failure | 72 | ok | turnover + scatter |
| catch success | 199 | ok |  |
| catch failure | 79 | ok |  |
| ball scatters | 544 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 25 | ok | ball out of bounds |
| pass rolls | 226 | ok |  |
| pass deviates | 70 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1312 | ok |  |
| block 2 dice | 386 | ok |  |
| block 2 dice against | 133 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 270 | ok |  |
| block result BothDown | 312 | ok |  |
| block result Pushback | 613 | ok |  |
| block result PowPushback | 303 | ok |  |
| block result Pow | 333 | ok |  |
| pushbacks | 1239 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1399 | ok |  |
| armor held | 1445 | ok |  |
| stunned | 410 | ok | injury 2-7 |
| KO | 176 | ok |  |
| casualty (d16) | 133 | ok |  |
| death | 14 | ok | d16 = 15-16 only |
| fouls | 255 | ok |  |
| argue the call | 52 | ok | referee spotted a foul (doubles) |
| argue success | 8 | ok | d6 = 6 only |
| players ejected | 50 | ok |  |
| touchdowns | 66 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 35 | ok | kickoff event roll of 8 only |
| kickoff events | 259 | ok | per-result table below |

## Kickoff results

- Blitz: 24
- Brilliant Coaching: 31
- Cheering Fans: 35
- Get the Ref: 8
- High Kick: 30
- Officious Ref: 11
- Pitch Invasion: 9
- Quick Snap: 37
- Solid Defence: 21
- Time-out: 18
- Weather Change: 35

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/dark_elf_league_fumbbl_vs_dark_elf_league_fumbbl/seed_*_rust_events.jsonl)

Total events: 107883

```
  69942 playerMoved
  15471 playerAction
   5192 goForItRoll
   3427 turnEnd
   2164 injury
   1831 blockRoll
   1831 block
   1399 playerFellDown
   1239 pushback
   1177 dodgeRoll
    544 scatterBall
    355 pickupRoll
    283 ballPickedUp
    278 catchRoll
    259 kickoffScatter
    259 kickoffResultEvent
    255 refereeSpotsFoul
    255 foul
    253 skillUse
    226 passRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    114 handOver
     70 passDeviate
     66 touchdown
     52 argueTheCall
     50 playerEjected
     37 quickSnapRoll
     35 weatherChange
     35 cheeringFans
     29 prayerRoll
     27 kickoffExtraReRoll
     25 throwIn
     24 blitzRoll
     21 solidDefenceRoll
     19 kickoffPitchInvasionStun
     18 kickoffTimeout
     11 kickoffOfficiousRef
      9 kickoffPitchInvasion
      1 trapDoor
```

## Player actions declared

```
  13175 Move
    895 BlitzMove
    820 Block
    255 Foul
    211 PassMove
    115 HandOverMove
```

## Skill uses / re-rolls seen

```
```
