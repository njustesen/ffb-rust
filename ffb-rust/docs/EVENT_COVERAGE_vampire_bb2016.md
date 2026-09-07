# Event coverage — HeuristicAgent, vampire v vampire, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-07 by `MATCHUP=vampire scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 9415 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 484 | ok |  |
| action Blitz | 654 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 148 | ok |  |
| action Pass | 26 | ok | needs a ball carrier |
| action HandOver | 4 | ok | needs carrier + adjacent teammate |
| dodge success | 484 | ok |  |
| dodge failure | 120 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 172 | ok |  |
| pickup failure | 87 | ok | turnover + scatter |
| catch success | 128 | ok |  |
| catch failure | 76 | ok |  |
| ball scatters | 464 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 6 | ok | ball out of bounds |
| pass rolls | 131 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 624 | ok |  |
| block 2 dice | 300 | ok |  |
| block 2 dice against | 161 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 152 | ok |  |
| block result BothDown | 180 | ok |  |
| block result Pushback | 365 | ok |  |
| block result PowPushback | 189 | ok |  |
| block result Pow | 199 | ok |  |
| pushbacks | 751 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 523 | ok |  |
| armor held | 1085 | ok |  |
| stunned | 308 | ok | injury 2-7 |
| KO | 110 | ok |  |
| casualty (d16) | 82 | ok |  |
| death | 22 | ok | d16 = 15-16 only |
| fouls | 148 | ok |  |
| argue the call | 26 | ok | referee spotted a foul (doubles) |
| argue success | 5 | ok | d6 = 6 only |
| players ejected | 26 | ok |  |
| touchdowns | 21 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 30 | ok | kickoff event roll of 8 only |
| kickoff events | 220 | ok | per-result table below |

## Kickoff results

- Blitz: 11
- Brilliant Coaching: 30
- Cheering Fans: 39
- Get the Ref: 6
- High Kick: 30
- Perfect Defence: 19
- Pitch Invasion: 4
- Quick Snap: 26
- Riot: 14
- Throw a Rock: 11
- Weather Change: 30

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2016/vampire_vs_vampire/seed_*_rust_events.jsonl)

Total events: 27681

```
  10731 playerAction
   4399 bloodLustRoll
   3396 turnEnd
   1585 injury
   1085 blockRoll
   1085 block
    751 pushback
    631 biteSpectator
    604 dodgeRoll
    523 playerFellDown
    464 scatterBall
    259 pickupRoll
    220 kickoffScatter
    220 kickoffResultEvent
    204 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    172 ballPickedUp
    148 refereeSpotsFoul
    148 foul
    131 passRoll
     69 kickoffExtraReRollBb2016
     62 handOver
     40 regenerationRoll
     30 weatherChange
     26 playerEjected
     26 argueTheCall
     21 touchdown
     16 playerAdded
     14 kickoffRiot
     11 kickoffThrowARockBb2016
      6 throwIn
      4 kickoffPitchInvasionBb2016
```

## Player actions declared

```
   9235 Move
    654 Blitz
    484 Block
    148 Foul
    118 PassMove
     62 HandOverMove
     26 Pass
      4 HandOver
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
