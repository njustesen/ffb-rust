# Event coverage — HeuristicAgent, nurgle v nurgle, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=nurgle scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11956 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 673 | ok |  |
| action Blitz | 916 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 257 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 322 | ok |  |
| dodge failure | 345 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 216 | ok |  |
| pickup failure | 154 | ok | turnover + scatter |
| catch success | 81 | ok |  |
| catch failure | 113 | ok |  |
| ball scatters | 580 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 10 | ok | ball out of bounds |
| pass rolls | 151 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 491 | ok |  |
| block 2 dice | 564 | ok |  |
| block 2 dice against | 234 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 173 | ok |  |
| block result BothDown | 188 | ok |  |
| block result Pushback | 440 | ok |  |
| block result PowPushback | 235 | ok |  |
| block result Pow | 253 | ok |  |
| pushbacks | 926 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1018 | ok |  |
| armor held | 1796 | ok |  |
| stunned | 304 | ok | injury 2-7 |
| KO | 118 | ok |  |
| casualty (d16) | 84 | ok |  |
| death | 18 | ok | d16 = 15-16 only |
| fouls | 233 | ok |  |
| argue the call | 44 | ok | referee spotted a foul (doubles) |
| argue success | 5 | ok | d6 = 6 only |
| players ejected | 44 | ok |  |
| touchdowns | 12 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 34 | ok | kickoff event roll of 8 only |
| kickoff events | 211 | ok | per-result table below |

## Kickoff results

- Blitz: 16
- Brilliant Coaching: 24
- Cheering Fans: 38
- Get the Ref: 4
- High Kick: 20
- Perfect Defence: 24
- Pitch Invasion: 2
- Quick Snap: 26
- Riot: 11
- Throw a Rock: 12
- Weather Change: 34

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/nurgle_vs_nurgle/seed_*_rust_events.jsonl)

Total events: 30899

```
  13802 playerAction
   3389 turnEnd
   2302 injury
   1677 confusionRoll
   1289 blockRoll
   1289 block
   1083 foulAppearanceRoll
   1018 playerFellDown
    926 pushback
    667 dodgeRoll
    580 scatterBall
    370 pickupRoll
    233 refereeSpotsFoul
    233 foul
    216 ballPickedUp
    211 kickoffScatter
    211 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    194 catchRoll
    151 passRoll
     91 skillUse
     70 handOver
     62 regenerationRoll
     62 kickoffExtraReRollBb2016
     44 playerEjected
     44 argueTheCall
     34 weatherChange
     12 touchdown
     12 kickoffThrowARockBb2016
     11 kickoffRiot
     10 throwIn
      4 playerAdded
      2 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  11718 Move
    916 Blitz
    673 Block
    257 Foul
    163 PassMove
     75 HandOverMove
```

## Skill uses / re-rolls seen

```
     91 Horns used=true
```
