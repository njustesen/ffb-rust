# T3 lineman coverage — 10 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 2633 | ok |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 33 | ok |  |
| action Blitz | 73 | ok |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 22 | ok |  |
| action Pass | 2 | ok | needs a ball carrier |
| action HandOver | 2 | ok | needs carrier + adjacent teammate |
| dodge success | 40 | ok |  |
| dodge failure | 44 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 4 | ok |  |
| pickup failure | 2 | ok | turnover + scatter |
| catch success | 1 | ok |  |
| catch failure | 5 | ok |  |
| ball scatters | 31 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 2 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 77 | ok |  |
| block 2 dice | 19 | ok |  |
| block 2 dice against | 9 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 15 | ok |  |
| block result BothDown | 22 | ok |  |
| block result Pushback | 38 | ok |  |
| block result PowPushback | 15 | ok |  |
| block result Pow | 15 | ok |  |
| pushbacks | 68 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 96 | ok |  |
| armor held | 87 | ok |  |
| stunned | 40 | ok | injury 2-7 |
| KO | 21 | ok |  |
| casualty (d16) | 7 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 22 | ok |  |
| argue the call | 4 | ok | referee spotted a foul (doubles) |
| argue success | 0 | absent (optional) | d6 = 6 only |
| players ejected | 4 | ok |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 20 | ok |  |
| weather changes | 2 | ok | kickoff event roll of 8 only |
| kickoff events | 20 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 6
- Charge: 2
- Cheering Fans: 4
- Dodgy Snack: 1
- High Kick: 2
- Quick Snap: 1
- Solid Defence: 2
- Weather Change: 2

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
