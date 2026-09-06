# Event coverage — HeuristicAgent, slann v slann, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-06 by `MATCHUP=slann scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12501 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 718 | ok |  |
| action Blitz | 818 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 250 | ok |  |
| action Pass | 25 | ok | needs a ball carrier |
| action HandOver | 5 | ok | needs carrier + adjacent teammate |
| dodge success | 502 | ok |  |
| dodge failure | 388 | ok |  |
| GFI rolls | 4639 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 222 | ok |  |
| pickup failure | 147 | ok | turnover + scatter |
| catch success | 131 | ok |  |
| catch failure | 119 | ok |  |
| ball scatters | 576 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 30 | ok | ball out of bounds |
| pass rolls | 167 | ok |  |
| pass deviates | 48 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 528 | ok |  |
| block 2 dice | 470 | ok |  |
| block 2 dice against | 278 | ok | defender's choice |
| block 3 dice | 183 | ok | needs ST5+ differential via assists |
| block result Skull | 206 | ok |  |
| block result BothDown | 216 | ok |  |
| block result Pushback | 480 | ok |  |
| block result PowPushback | 255 | ok |  |
| block result Pow | 302 | ok |  |
| pushbacks | 1029 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1502 | ok |  |
| armor held | 1160 | ok |  |
| stunned | 589 | ok | injury 2-7 |
| KO | 252 | ok |  |
| casualty (d16) | 171 | ok |  |
| death | 19 | ok | d16 = 15-16 only |
| fouls | 240 | ok |  |
| argue the call | 58 | ok | referee spotted a foul (doubles) |
| argue success | 13 | ok | d6 = 6 only |
| players ejected | 50 | ok |  |
| touchdowns | 33 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 27 | ok | kickoff event roll of 8 only |
| kickoff events | 228 | ok | per-result table below |

## Kickoff results

- Blitz: 27
- Brilliant Coaching: 39
- Cheering Fans: 31
- Get the Ref: 7
- High Kick: 27
- Officious Ref: 14
- Pitch Invasion: 5
- Quick Snap: 21
- Solid Defence: 17
- Time-out: 13
- Weather Change: 27

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2020/slann_vs_slann/seed_*_rust_events.jsonl)

Total events: 98658

```
  62054 playerMoved
  14317 playerAction
   4639 goForItRoll
   3402 turnEnd
   2172 injury
   1651 confusionRoll
   1502 playerFellDown
   1459 blockRoll
   1459 block
   1029 pushback
    890 dodgeRoll
    576 scatterBall
    423 apothecaryRoll
    369 pickupRoll
    250 catchRoll
    240 refereeSpotsFoul
    240 foul
    228 kickoffScatter
    228 kickoffResultEvent
    222 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    167 passRoll
     99 handOver
     58 argueTheCall
     50 playerEjected
     48 passDeviate
     35 kickoffExtraReRoll
     33 touchdown
     31 cheeringFans
     30 throwIn
     27 weatherChange
     27 blitzRoll
     23 prayerRoll
     21 quickSnapRoll
     17 solidDefenceRoll
     14 kickoffOfficiousRef
     13 kickoffTimeout
      9 kickoffPitchInvasionStun
      5 kickoffPitchInvasion
      1 trapDoor
```

## Player actions declared

```
  12267 Move
    818 BlitzMove
    718 Block
    250 Foul
    135 PassMove
     99 HandOverMove
     25 Pass
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
