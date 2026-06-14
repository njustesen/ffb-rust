# T3 lineman coverage — 1 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 0 | **MISSING** |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 0 | **MISSING** |  |
| action Blitz | 0 | **MISSING** |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 0 | **MISSING** |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 0 | **MISSING** |  |
| dodge failure | 0 | **MISSING** |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 0 | **MISSING** |  |
| pickup failure | 0 | **MISSING** | turnover + scatter |
| catch success | 0 | **MISSING** |  |
| catch failure | 0 | **MISSING** |  |
| ball scatters | 0 | **MISSING** | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 0 | **MISSING** |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 0 | **MISSING** |  |
| block 2 dice | 0 | **MISSING** |  |
| block 2 dice against | 0 | **MISSING** | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 0 | **MISSING** |  |
| block result BothDown | 0 | **MISSING** |  |
| block result Pushback | 0 | **MISSING** |  |
| block result PowPushback | 0 | **MISSING** |  |
| block result Pow | 0 | **MISSING** |  |
| pushbacks | 0 | **MISSING** |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 0 | **MISSING** |  |
| armor held | 0 | **MISSING** |  |
| stunned | 0 | **MISSING** | injury 2-7 |
| KO | 0 | **MISSING** |  |
| casualty (d16) | 0 | **MISSING** |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 0 | **MISSING** |  |
| argue the call | 0 | **MISSING** | referee spotted a foul (doubles) |
| argue success | 0 | absent (optional) | d6 = 6 only |
| players ejected | 0 | **MISSING** |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 2 | ok |  |
| weather changes | 1 | ok | kickoff event roll of 8 only |
| kickoff events | 2 | ok | per-result table below |

## Kickoff results

- Cheering Fans: 1
- Time-out: 1

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
