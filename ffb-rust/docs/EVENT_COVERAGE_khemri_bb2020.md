# Event coverage — HeuristicAgent, khemri v khemri, bb2020, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-05 by `MATCHUP=khemri scripts/harvest_coverage.sh bb2020 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12622 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 660 | ok |  |
| action Blitz | 927 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 292 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 239 | ok |  |
| dodge failure | 428 | ok |  |
| GFI rolls | 5831 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 181 | ok |  |
| pickup failure | 235 | ok | turnover + scatter |
| catch success | 76 | ok |  |
| catch failure | 104 | ok |  |
| ball scatters | 567 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 11 | ok | ball out of bounds |
| pass rolls | 162 | ok |  |
| pass deviates | 28 | ok | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 792 | ok |  |
| block 2 dice | 567 | ok |  |
| block 2 dice against | 223 | ok | defender's choice |
| block 3 dice | 5 | ok | needs ST5+ differential via assists |
| block result Skull | 216 | ok |  |
| block result BothDown | 254 | ok |  |
| block result Pushback | 544 | ok |  |
| block result PowPushback | 272 | ok |  |
| block result Pow | 301 | ok |  |
| pushbacks | 1112 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 1712 | ok |  |
| armor held | 1898 | ok |  |
| stunned | 389 | ok | injury 2-7 |
| KO | 79 | ok |  |
| casualty (d16) | 96 | ok |  |
| death | 11 | ok | d16 = 15-16 only |
| fouls | 292 | ok |  |
| argue the call | 57 | ok | referee spotted a foul (doubles) |
| argue success | 9 | ok | d6 = 6 only |
| players ejected | 53 | ok |  |
| touchdowns | 6 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 23 | ok | kickoff event roll of 8 only |
| kickoff events | 205 | ok | per-result table below |

## Kickoff results

- Blitz: 22
- Brilliant Coaching: 26
- Cheering Fans: 36
- Get the Ref: 5
- High Kick: 30
- Officious Ref: 12
- Pitch Invasion: 8
- Quick Snap: 20
- Solid Defence: 14
- Time-out: 9
- Weather Change: 23

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2020/khemri_vs_khemri/seed_*_rust_events.jsonl)

Total events: 89772

```
  53308 playerMoved
  14501 playerAction
   5831 goForItRoll
   3387 turnEnd
   2462 injury
   1712 playerFellDown
   1587 blockRoll
   1587 block
   1112 pushback
    667 dodgeRoll
    567 scatterBall
    416 pickupRoll
    292 refereeSpotsFoul
    292 foul
    205 kickoffScatter
    205 kickoffResultEvent
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    181 ballPickedUp
    180 catchRoll
    162 passRoll
     96 regenerationRoll
     57 argueTheCall
     53 playerEjected
     53 handOver
     36 cheeringFans
     30 prayerRoll
     28 passDeviate
     23 weatherChange
     23 kickoffExtraReRoll
     22 blitzRoll
     20 quickSnapRoll
     14 solidDefenceRoll
     14 kickoffPitchInvasionStun
     12 kickoffOfficiousRef
     11 throwIn
      9 kickoffTimeout
      8 kickoffPitchInvasion
      6 touchdown
      3 trapDoor
```

## Player actions declared

```
  12427 Move
    927 BlitzMove
    660 Block
    292 Foul
    142 PassMove
     53 HandOverMove
```

## Skill uses / re-rolls seen

```
(no skillUse events in this run)

Note: GameEvent::SkillUse is emitted by only five sites --
block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster
with none of those legitimately produces zero. Every other skill is
used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.
```
