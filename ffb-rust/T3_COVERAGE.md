# T3 amazon_vs_amazon coverage — 95/100 seeds pass (seeds 45, 59, 77, 83, 93 failing)

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
| dodge success | 274 | ok |  |
| dodge failure | 219 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 27 | ok |  |
| pickup failure | 16 | ok | turnover + scatter |
| catch success | 22 | ok |  |
| catch failure | 23 | ok |  |
| ball scatters | 0 | **MISSING** | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 68 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 368 | ok |  |
| block 2 dice | 68 | ok |  |
| block 2 dice against | 35 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 82 | ok |  |
| block result BothDown | 77 | ok |  |
| block result Pushback | 162 | ok |  |
| block result PowPushback | 72 | ok |  |
| block result Pow | 78 | ok |  |
| pushbacks | 0 | **MISSING** |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 0 | **MISSING** |  |
| armor held | 189 | ok |  |
| stunned | 165 | ok | injury 2-7 |
| KO | 65 | ok |  |
| casualty (d16) | 41 | ok |  |
| death | 4 | ok | d16 = 15-16 only |
| fouls | 79 | ok |  |
| argue the call | 0 | **MISSING** | referee spotted a foul (doubles) |
| argue success | 0 | absent (optional) | d6 = 6 only |
| players ejected | 0 | **MISSING** |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 88 | ok |  |
| weather changes | 54 | ok | kickoff event roll of 8 only |
| kickoff events | 88 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 13
- Charge: 8
- Cheering Fans: 14
- Dodgy Snack: 4
- Get the Ref: 4
- High Kick: 10
- Pitch Invasion: 7
- Quick Snap: 13
- Solid Defence: 3
- Time-out: 2
- Weather Change: 10

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
