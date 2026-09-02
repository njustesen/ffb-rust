# Event coverage — HeuristicAgent, dark_elf_league_fumbbl v dark_elf_league_fumbbl, bb2016, --heur-scale 1.0, seeds 1-100

Harvested 2026-09-02 by `MATCHUP=dark_elf_league_fumbbl scripts/harvest_coverage.sh bb2016 1.0`. Parity for the run: `PARITY: 100/100 games match, but required coverage items are MISSING.`.

## Tier-3 checklist (as written by the run)

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 11513 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 818 | ok |  |
| action Blitz | 1034 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 324 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 435 | ok |  |
| dodge failure | 314 | ok |  |
| GFI rolls | 0 | BLOCKED (needs a decision) | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| pickup success | 226 | ok |  |
| pickup failure | 223 | ok | turnover + scatter |
| catch success | 89 | ok |  |
| catch failure | 151 | ok |  |
| ball scatters | 665 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 12 | ok | ball out of bounds |
| pass rolls | 124 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 1596 | ok |  |
| block 2 dice | 396 | ok |  |
| block 2 dice against | 144 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 352 | ok |  |
| block result BothDown | 368 | ok |  |
| block result Pushback | 707 | ok |  |
| block result PowPushback | 340 | ok |  |
| block result Pow | 369 | ok |  |
| pushbacks | 1410 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 688 | ok |  |
| armor held | 1830 | ok |  |
| stunned | 308 | ok | injury 2-7 |
| KO | 135 | ok |  |
| casualty (d16) | 109 | ok |  |
| death | 20 | ok | d16 = 15-16 only |
| fouls | 324 | ok |  |
| argue the call | 80 | ok | referee spotted a foul (doubles) |
| argue success | 11 | ok | d6 = 6 only |
| players ejected | 86 | ok |  |
| touchdowns | 47 | ok | BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG. |
| half starts | 200 | ok |  |
| weather changes | 43 | ok | kickoff event roll of 8 only |
| kickoff events | 241 | ok | per-result table below |

## Kickoff results

- Blitz: 19
- Brilliant Coaching: 26
- Cheering Fans: 33
- Get the Ref: 4
- High Kick: 28
- Perfect Defence: 28
- Pitch Invasion: 4
- Quick Snap: 28
- Riot: 17
- Throw a Rock: 11
- Weather Change: 43

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING

## GameEvent catalog (from parity/bb2016/dark_elf_league_fumbbl_vs_dark_elf_league_fumbbl/seed_*_rust_events.jsonl)

Total events: 30744

```
  13689 playerAction
   3415 turnEnd
   2382 injury
   2136 blockRoll
   2136 block
   1410 pushback
    749 dodgeRoll
    688 playerFellDown
    665 scatterBall
    449 pickupRoll
    324 refereeSpotsFoul
    324 foul
    263 skillUse
    241 kickoffScatter
    241 kickoffResultEvent
    240 catchRoll
    226 ballPickedUp
    200 winningsRoll
    200 startHalf
    200 mvpRoll
    124 passRoll
     86 playerEjected
     83 handOver
     80 argueTheCall
     59 kickoffExtraReRollBb2016
     47 touchdown
     43 weatherChange
     17 kickoffRiot
     12 throwIn
     11 kickoffThrowARockBb2016
      4 kickoffPitchInvasionBb2016
```

## Player actions declared

```
  11294 Move
   1034 Blitz
    818 Block
    324 Foul
    131 PassMove
     88 HandOverMove
```

## Skill uses / re-rolls seen

```
```
