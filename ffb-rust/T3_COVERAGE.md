# T3 amazon_vs_amazon coverage — 100/100 seeds pass

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
| dodge success | 649 | ok |  |
| dodge failure | 491 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 53 | ok |  |
| pickup failure | 26 | ok | turnover + scatter |
| catch success | 51 | ok |  |
| catch failure | 50 | ok |  |
| ball scatters | 0 | **MISSING** | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 145 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 830 | ok |  |
| block 2 dice | 171 | ok |  |
| block 2 dice against | 78 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 172 | ok |  |
| block result BothDown | 201 | ok |  |
| block result Pushback | 364 | ok |  |
| block result PowPushback | 173 | ok |  |
| block result Pow | 169 | ok |  |
| pushbacks | 0 | **MISSING** |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 0 | **MISSING** |  |
| armor held | 421 | ok |  |
| stunned | 388 | ok | injury 2-7 |
| KO | 146 | ok |  |
| casualty (d16) | 101 | ok |  |
| death | 9 | ok | d16 = 15-16 only |
| fouls | 167 | ok |  |
| argue the call | 0 | **MISSING** | referee spotted a foul (doubles) |
| argue success | 0 | absent (optional) | d6 = 6 only |
| players ejected | 0 | **MISSING** |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 200 | ok |  |
| weather changes | 123 | ok | kickoff event roll of 8 only |
| kickoff events | 200 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 30
- Charge: 19
- Cheering Fans: 31
- Dodgy Snack: 8
- Get the Ref: 6
- High Kick: 27
- Pitch Invasion: 12
- Quick Snap: 22
- Solid Defence: 12
- Time-out: 10
- Weather Change: 23

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
