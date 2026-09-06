# Event coverage — HeuristicAgent, underworld v underworld, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=underworld scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12541 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 837 | ok |  |
| action Blitz | 970 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 268 | ok |  |
| action Pass | 48 | ok | needs a ball carrier |
| action HandOver | 2 | ok | needs carrier + adjacent teammate |
| dodge success | 551 | ok |  |
| dodge failure | 407 | ok |  |
| GFI rolls | 4404 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 254 | ok |  |
| pickup failure | 144 | ok | turnover + scatter |
| catch success | 114 | ok |  |
| catch failure | 102 | ok |  |
| ball scatters | 577 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 22 | ok | ball out of bounds |
| pass rolls | 173 | ok |  |
| pass deviates | 45 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 585 | ok |  |
| block 2 dice | 827 | ok |  |
| block 2 dice against | 332 | ok | defender's choice |
| block 3 dice | 191 | ok | needs ST5+ differential via assists |
| block result Skull | 229 | ok |  |
| block result BothDown | 302 | ok |  |
| block result Pushback | 659 | ok |  |
| block result PowPushback | 305 | ok |  |
| block result Pow | 440 | ok |  |
| pushbacks | 1399 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1820 | ok |  |
| armor held | 1530 | ok |  |
| stunned | 505 | ok | injury 2-7 |
| KO | 244 | ok |  |
| casualty (d16) | 227 | ok |  |
| death | 21 | ok | d16 = 15-16 only |
| fouls | 229 | ok |  |
| argue the call | 54 | ok | referee spotted a foul (doubles) |
| argue success | 12 | ok | d6 = 6 only |
| players ejected | 47 | ok |  |
| touchdowns | 62 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 26 | ok | kickoff event roll of 8 only |
| kickoff events | 258 | ok | per-result table below |

## Kickoff results

- Blitz: 25
- Brilliant Coaching: 39
- Cheering Fans: 50
- Get the Ref: 9
- High Kick: 36
- Officious Ref: 10
- Pitch Invasion: 11
- Quick Snap: 28
- Solid Defence: 15
- Time-out: 9
- Weather Change: 26

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2020/underworld_vs_underworld/seed_*_rust_events.jsonl)

Total events: 104074

```
  65164 playerMoved
  14666 playerAction
   4404 goForItRoll
   3448 turnEnd
   2506 injury
   1935 blockRoll
   1935 block
   1820 playerFellDown
   1511 animalSavagery
   1399 pushback
    958 dodgeRoll
    577 scatterBall
    471 apothecaryRoll
    398 pickupRoll
    258 kickoffScatter
    258 kickoffResultEvent
    254 ballPickedUp
    229 refereeSpotsFoul
    229 foul
    216 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    173 passRoll
     82 skillUse
     75 handOver
     62 touchdown
     54 argueTheCall
     50 cheeringFans
     47 playerEjected
     45 passDeviate
     44 prayerRoll
     33 kickoffExtraReRoll
     28 quickSnapRoll
     26 weatherChange
     25 blitzRoll
     22 throwIn
     22 kickoffPitchInvasionStun
     15 solidDefenceRoll
     11 kickoffPitchInvasion
     10 kickoffOfficiousRef
      9 kickoffTimeout
      5 trapDoor
```

## Player actions declared

```
  12350 Move
    970 BlitzMove
    837 Block
    268 Foul
    116 PassMove
     75 HandOverMove
     48 Pass
      2 HandOver
```

## Skill uses / re-rolls seen

```
     82 Dodge used=true
```
