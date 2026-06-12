# T3 lineman coverage — 1 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 249 | ok |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 4 | ok |  |
| action Blitz | 9 | ok |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 2 | ok |  |
| action Pass | 0 | **MISSING** | needs a ball carrier |
| action HandOver | 0 | **MISSING** | needs carrier + adjacent teammate |
| dodge success | 9 | ok |  |
| dodge failure | 2 | ok |  |
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
| block 1 die | 11 | ok |  |
| block 2 dice | 2 | ok |  |
| block 2 dice against | 0 | **MISSING** | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 5 | ok |  |
| block result BothDown | 5 | ok |  |
| block result Pushback | 3 | ok |  |
| block result PowPushback | 0 | **MISSING** |  |
| block result Pow | 0 | **MISSING** |  |
| pushbacks | 3 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 17 | ok |  |
| armor held | 15 | ok |  |
| stunned | 3 | ok | injury 2-7 |
| KO | 0 | **MISSING** |  |
| casualty (d16) | 1 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 2 | ok |  |
| argue the call | 0 | **MISSING** | referee spotted a foul (doubles) |
| argue success | 0 | absent (optional) | d6 = 6 only |
| players ejected | 0 | **MISSING** |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 2 | ok |  |
| weather changes | 1 | ok | kickoff event roll of 8 only |
| kickoff events | 2 | ok | per-result table below |

## Kickoff results

- Cheering Fans: 2

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
