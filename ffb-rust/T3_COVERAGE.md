# T3 lineman coverage — 3 games

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
| dodge success | 24 | ok |  |
| dodge failure | 26 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 0 | **MISSING** |  |
| pickup failure | 3 | ok | turnover + scatter |
| catch success | 0 | **MISSING** |  |
| catch failure | 0 | **MISSING** |  |
| ball scatters | 0 | **MISSING** | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 0 | **MISSING** |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 28 | ok |  |
| block 2 dice | 4 | ok |  |
| block 2 dice against | 1 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 6 | ok |  |
| block result BothDown | 7 | ok |  |
| block result Pushback | 11 | ok |  |
| block result PowPushback | 1 | ok |  |
| block result Pow | 8 | ok |  |
| pushbacks | 0 | **MISSING** |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 0 | **MISSING** |  |
| armor held | 20 | ok |  |
| stunned | 14 | ok | injury 2-7 |
| KO | 6 | ok |  |
| casualty (d16) | 2 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 6 | ok |  |
| argue the call | 0 | **MISSING** | referee spotted a foul (doubles) |
| argue success | 0 | absent (optional) | d6 = 6 only |
| players ejected | 0 | **MISSING** |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 6 | ok |  |
| weather changes | 3 | ok | kickoff event roll of 8 only |
| kickoff events | 6 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 2
- Cheering Fans: 1
- Dodgy Snack: 1
- High Kick: 1
- Quick Snap: 1

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
