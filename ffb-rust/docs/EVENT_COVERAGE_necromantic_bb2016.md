# Event coverage — HeuristicAgent, necromantic v necromantic, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=necromantic scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12055 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 647 | ok |  |
| action Blitz | 889 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 297 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 479 | ok |  |
| dodge failure | 315 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 210 | ok |  |
| pickup failure | 161 | ok | turnover + scatter |
| catch success | 90 | ok |  |
| catch failure | 125 | ok |  |
| ball scatters | 574 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 10 | ok | ball out of bounds |
| pass rolls | 143 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 936 | ok |  |
| block 2 dice | 532 | ok |  |
| block 2 dice against | 257 | ok | defender's choice |
| block 3 dice | 2 | ok | needs ST5+ differential via assists |
| block result Skull | 281 | ok |  |
| block result BothDown | 251 | ok |  |
| block result Pushback | 580 | ok |  |
| block result PowPushback | 295 | ok |  |
| block result Pow | 320 | ok |  |
| pushbacks | 1194 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 892 | ok |  |
| armor held | 1792 | ok |  |
| stunned | 444 | ok | injury 2-7 |
| KO | 169 | ok |  |
| casualty (d16) | 134 | ok |  |
| death | 21 | ok | d16 = 15-16 only |
| fouls | 297 | ok |  |
| argue the call | 50 | ok | referee spotted a foul (doubles) |
| argue success | 11 | ok | d6 = 6 only |
| players ejected | 51 | ok |  |
| touchdowns | 28 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 40 | ok | kickoff event roll of 8 only |
| kickoff events | 223 | ok | per-result table below |

## Kickoff results

- Blitz: 10
- Brilliant Coaching: 26
- Cheering Fans: 38
- Get the Ref: 3
- High Kick: 26
- Perfect Defence: 24
- Pitch Invasion: 4
- Quick Snap: 31
- Riot: 10
- Throw a Rock: 11
- Weather Change: 40

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/necromantic_vs_necromantic/seed_*_rust_events.jsonl)

Total events: 29823

```
  13888 playerAction
   3407 turnEnd
   2539 injury
   1727 blockRoll
   1727 block
   1194 pushback
    892 playerFellDown
    794 dodgeRoll
    574 scatterBall
    371 pickupRoll
    297 refereeSpotsFoul
    297 foul
    223 kickoffScatter
    223 kickoffResultEvent
    215 catchRoll
    210 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    143 passRoll
    119 regenerationRoll
     77 handOver
     64 kickoffExtraReRollBb2016
     51 playerEjected
     50 argueTheCall
     40 weatherChange
     28 touchdown
     26 skillUse
     12 playerAdded
     11 kickoffThrowARockBb2016
     10 throwIn
     10 kickoffRiot
      4 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  11831 Move
    889 Blitz
    647 Block
    297 Foul
    146 PassMove
     78 HandOverMove
```

## Skill uses / re-rolls seen

```
     26 Dodge used=true
```
