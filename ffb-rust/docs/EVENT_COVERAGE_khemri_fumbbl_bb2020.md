# Event coverage — HeuristicAgent, khemri_fumbbl v khemri_fumbbl, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=khemri_fumbbl scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 13490 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 692 | ok |  |
| action Blitz | 819 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 224 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 900 | ok |  |
| dodge failure | 230 | ok |  |
| GFI rolls | 6328 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 296 | ok |  |
| pickup failure | 77 | ok | turnover + scatter |
| catch success | 127 | ok |  |
| catch failure | 55 | ok |  |
| ball scatters | 555 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 7 | ok | ball out of bounds |
| pass rolls | 184 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 781 | ok |  |
| block 2 dice | 506 | ok |  |
| block 2 dice against | 224 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 205 | ok |  |
| block result BothDown | 219 | ok |  |
| block result Pushback | 521 | ok |  |
| block result PowPushback | 281 | ok |  |
| block result Pow | 285 | ok |  |
| pushbacks | 1081 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1459 | ok |  |
| armor held | 1032 | ok |  |
| stunned | 636 | ok | injury 2-7 |
| KO | 180 | ok |  |
| casualty (d16) | 147 | ok |  |
| death | 19 | ok | d16 = 15-16 only |
| fouls | 224 | ok |  |
| argue the call | 51 | ok | referee spotted a foul (doubles) |
| argue success | 10 | ok | d6 = 6 only |
| players ejected | 45 | ok |  |
| touchdowns | 33 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 24 | ok | kickoff event roll of 8 only |
| kickoff events | 227 | ok | per-result table below |

## Kickoff results

- Blitz: 21
- Brilliant Coaching: 35
- Cheering Fans: 41
- Get the Ref: 8
- High Kick: 32
- Officious Ref: 6
- Pitch Invasion: 7
- Quick Snap: 22
- Solid Defence: 13
- Time-out: 18
- Weather Change: 24

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/khemri_fumbbl_vs_khemri_fumbbl/seed_*_rust_events.jsonl)

Total events: 99100

```
  61745 playerMoved
  15225 playerAction
   6328 goForItRoll
   3395 turnEnd
   1995 injury
   1511 blockRoll
   1511 block
   1459 playerFellDown
   1130 dodgeRoll
   1081 pushback
    555 scatterBall
    373 pickupRoll
    296 ballPickedUp
    227 kickoffScatter
    227 kickoffResultEvent
    224 refereeSpotsFoul
    224 foul
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    184 passRoll
    182 catchRoll
    147 regenerationRoll
    109 handOver
     51 argueTheCall
     45 playerEjected
     41 cheeringFans
     33 touchdown
     33 prayerRoll
     33 kickoffExtraReRoll
     24 weatherChange
     22 quickSnapRoll
     21 blitzRoll
     18 kickoffTimeout
     16 kickoffPitchInvasionStun
     13 solidDefenceRoll
      7 throwIn
      7 kickoffPitchInvasion
      6 kickoffOfficiousRef
      2 trapDoor
```

## Player actions declared

```
  13185 Move
    819 BlitzMove
    692 Block
    224 Foul
    193 PassMove
    112 HandOverMove
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
