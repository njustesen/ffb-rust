# T3 lineman coverage — 1 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 269 | ok |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 2 | ok |  |
| action Blitz | 3 | ok |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 3 | ok |  |
| action Pass | 1 | ok | needs a ball carrier |
| action HandOver | 1 | ok | needs carrier + adjacent teammate |
| dodge success | 1 | ok |  |
| dodge failure | 7 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 0 | **MISSING** |  |
| pickup failure | 0 | **MISSING** | turnover + scatter |
| catch success | 1 | ok |  |
| catch failure | 2 | ok |  |
| ball scatters | 2 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 1 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 5 | ok |  |
| block 2 dice | 0 | **MISSING** |  |
| block 2 dice against | 0 | **MISSING** | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 1 | ok |  |
| block result BothDown | 0 | **MISSING** |  |
| block result Pushback | 2 | ok |  |
| block result PowPushback | 1 | ok |  |
| block result Pow | 1 | ok |  |
| pushbacks | 4 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 10 | ok |  |
| armor held | 6 | ok |  |
| stunned | 5 | ok | injury 2-7 |
| KO | 2 | ok |  |
| casualty (d16) | 0 | **MISSING** |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 3 | ok |  |
| argue the call | 1 | ok | referee spotted a foul (doubles) |
| argue success | 0 | absent (optional) | d6 = 6 only |
| players ejected | 1 | ok |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 2 | ok |  |
| weather changes | 2 | ok | kickoff event roll of 8 only |
| kickoff events | 2 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 1
- Weather Change: 1

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
