# Event coverage — HeuristicAgent, vampire v vampire, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-07 by `MATCHUP=vampire scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 48/100 passed, 52 FAILED.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 9404 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 475 | ok |  |
| action Blitz | 647 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 135 | ok |  |
| action Pass | 24 | ok | needs a ball carrier |
| action HandOver | 3 | ok | needs carrier + adjacent teammate |
| dodge success | 455 | ok |  |
| dodge failure | 107 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 171 | ok |  |
| pickup failure | 78 | ok | turnover + scatter |
| catch success | 100 | ok |  |
| catch failure | 68 | ok |  |
| ball scatters | 447 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 5 | ok | ball out of bounds |
| pass rolls | 116 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 580 | ok |  |
| block 2 dice | 267 | ok |  |
| block 2 dice against | 151 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 150 | ok |  |
| block result BothDown | 169 | ok |  |
| block result Pushback | 335 | ok |  |
| block result PowPushback | 164 | ok |  |
| block result Pow | 180 | ok |  |
| pushbacks | 678 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 510 | ok |  |
| armor held | 1005 | ok |  |
| stunned | 304 | ok | injury 2-7 |
| KO | 106 | ok |  |
| casualty (d16) | 76 | ok |  |
| death | 19 | ok | d16 = 15-16 only |
| fouls | 135 | ok |  |
| argue the call | 24 | ok | referee spotted a foul (doubles) |
| argue success | 6 | ok | d6 = 6 only |
| players ejected | 24 | ok |  |
| touchdowns | 15 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 30 | ok | kickoff event roll of 8 only |
| kickoff events | 214 | ok | per-result table below |

## Kickoff results

- Blitz: 12
- Brilliant Coaching: 28
- Cheering Fans: 38
- Get the Ref: 5
- High Kick: 29
- Perfect Defence: 17
- Pitch Invasion: 5
- Quick Snap: 26
- Riot: 12
- Throw a Rock: 12
- Weather Change: 30

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT

## GameEvent catalog (from parity/bb2016/vampire_vs_vampire/seed_*_rust_events.jsonl)

Total events: 27082

```
  10688 playerAction
   4389 bloodLustRoll
   3394 turnEnd
   1491 injury
    998 blockRoll
    998 block
    678 pushback
    628 biteSpectator
    562 dodgeRoll
    510 playerFellDown
    447 scatterBall
    249 pickupRoll
    214 kickoffScatter
    214 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    171 ballPickedUp
    168 catchRoll
    135 refereeSpotsFoul
    135 foul
    116 passRoll
     66 kickoffExtraReRollBb2016
     51 handOver
     41 regenerationRoll
     30 weatherChange
     24 playerEjected
     24 argueTheCall
     15 touchdown
     12 playerAdded
     12 kickoffThrowARockBb2016
     12 kickoffRiot
      5 throwIn
      5 kickoffPitchInvasionBb2016
```

## Player actions declared

```
   9248 Move
    647 Blitz
    475 Block
    135 Foul
    107 PassMove
     49 HandOverMove
     24 Pass
      3 HandOver
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
