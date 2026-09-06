# Event coverage — HeuristicAgent, slann_fumbbl v slann_fumbbl, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=slann_fumbbl scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12274 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 723 | ok |  |
| action Blitz | 849 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 237 | ok |  |
| action Pass | 29 | ok | needs a ball carrier |
| action HandOver | 5 | ok | needs carrier + adjacent teammate |
| dodge success | 563 | ok |  |
| dodge failure | 384 | ok |  |
| GFI rolls | 4605 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 243 | ok |  |
| pickup failure | 149 | ok | turnover + scatter |
| catch success | 105 | ok |  |
| catch failure | 93 | ok |  |
| ball scatters | 624 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 10 | ok | ball out of bounds |
| pass rolls | 166 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 539 | ok |  |
| block 2 dice | 484 | ok |  |
| block 2 dice against | 251 | ok | defender's choice |
| block 3 dice | 222 | ok | needs ST5+ differential via assists |
| block result Skull | 214 | ok |  |
| block result BothDown | 226 | ok |  |
| block result Pushback | 476 | ok |  |
| block result PowPushback | 277 | ok |  |
| block result Pow | 303 | ok |  |
| pushbacks | 1052 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1566 | ok |  |
| armor held | 1201 | ok |  |
| stunned | 616 | ok | injury 2-7 |
| KO | 250 | ok |  |
| casualty (d16) | 170 | ok |  |
| death | 21 | ok | d16 = 15-16 only |
| fouls | 227 | ok |  |
| argue the call | 51 | ok | referee spotted a foul (doubles) |
| argue success | 14 | ok | d6 = 6 only |
| players ejected | 42 | ok |  |
| touchdowns | 42 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 22 | ok | kickoff event roll of 8 only |
| kickoff events | 235 | ok | per-result table below |

## Kickoff results

- Blitz: 23
- Brilliant Coaching: 43
- Cheering Fans: 38
- Get the Ref: 7
- High Kick: 28
- Officious Ref: 10
- Pitch Invasion: 6
- Quick Snap: 29
- Solid Defence: 19
- Time-out: 10
- Weather Change: 22

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2020/slann_fumbbl_vs_slann_fumbbl/seed_*_rust_events.jsonl)

Total events: 98085

```
  61498 playerMoved
  14117 playerAction
   4605 goForItRoll
   3419 turnEnd
   2237 injury
   1627 confusionRoll
   1566 playerFellDown
   1496 blockRoll
   1496 block
   1052 pushback
    947 dodgeRoll
    624 scatterBall
    420 apothecaryRoll
    392 pickupRoll
    243 ballPickedUp
    235 kickoffScatter
    235 kickoffResultEvent
    227 refereeSpotsFoul
    227 foul
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    198 catchRoll
    166 passRoll
     75 handOver
     51 argueTheCall
     42 touchdown
     42 playerEjected
     38 kickoffExtraReRoll
     38 cheeringFans
     31 prayerRoll
     29 quickSnapRoll
     23 blitzRoll
     22 weatherChange
     19 solidDefenceRoll
     11 kickoffPitchInvasionStun
     10 throwIn
     10 kickoffTimeout
     10 kickoffOfficiousRef
      6 kickoffPitchInvasion
      1 trapDoor
```

## Player actions declared

```
  12053 Move
    849 BlitzMove
    723 Block
    237 Foul
    147 PassMove
     74 HandOverMove
     29 Pass
      5 HandOver
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
