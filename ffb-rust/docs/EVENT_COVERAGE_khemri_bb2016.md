# Event coverage — HeuristicAgent, khemri v khemri, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=khemri scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12088 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 677 | ok |  |
| action Blitz | 955 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 259 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 258 | ok |  |
| dodge failure | 469 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 195 | ok |  |
| pickup failure | 219 | ok | turnover + scatter |
| catch success | 84 | ok |  |
| catch failure | 120 | ok |  |
| ball scatters | 592 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 9 | ok | ball out of bounds |
| pass rolls | 155 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 794 | ok |  |
| block 2 dice | 644 | ok |  |
| block 2 dice against | 192 | ok | defender's choice |
| block 3 dice | 2 | ok | needs ST5+ differential via assists |
| block result Skull | 228 | ok |  |
| block result BothDown | 243 | ok |  |
| block result Pushback | 543 | ok |  |
| block result PowPushback | 287 | ok |  |
| block result Pow | 331 | ok |  |
| pushbacks | 1157 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 941 | ok |  |
| armor held | 1908 | ok |  |
| stunned | 351 | ok | injury 2-7 |
| KO | 147 | ok |  |
| casualty (d16) | 108 | ok |  |
| death | 24 | ok | d16 = 15-16 only |
| fouls | 259 | ok |  |
| argue the call | 50 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 51 | ok |  |
| touchdowns | 9 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 33 | ok | kickoff event roll of 8 only |
| kickoff events | 209 | ok | per-result table below |

## Kickoff results

- Blitz: 14
- Brilliant Coaching: 19
- Cheering Fans: 36
- Get the Ref: 6
- High Kick: 25
- Perfect Defence: 21
- Pitch Invasion: 4
- Quick Snap: 35
- Riot: 7
- Throw a Rock: 9
- Weather Change: 33

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/khemri_vs_khemri/seed_*_rust_events.jsonl)

Total events: 29489

```
  13979 playerAction
   3395 turnEnd
   2514 injury
   1632 blockRoll
   1632 block
   1157 pushback
    941 playerFellDown
    727 dodgeRoll
    592 scatterBall
    414 pickupRoll
    259 refereeSpotsFoul
    259 foul
    209 kickoffScatter
    209 kickoffResultEvent
    204 catchRoll
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    195 ballPickedUp
    155 passRoll
    108 regenerationRoll
     81 handOver
     55 kickoffExtraReRollBb2016
     51 playerEjected
     50 argueTheCall
     33 weatherChange
      9 touchdown
      9 throwIn
      9 kickoffThrowARockBb2016
      7 kickoffRiot
      4 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  11863 Move
    955 Blitz
    677 Block
    259 Foul
    142 PassMove
     83 HandOverMove
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
