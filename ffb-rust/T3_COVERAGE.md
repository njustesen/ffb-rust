# T3 lineman coverage — 55 games

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
| dodge success | 225 | ok |  |
| dodge failure | 189 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 33 | ok |  |
| pickup failure | 18 | ok | turnover + scatter |
| catch success | 22 | ok |  |
| catch failure | 23 | ok |  |
| ball scatters | 0 | **MISSING** | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 48 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 427 | ok |  |
| block 2 dice | 98 | ok |  |
| block 2 dice against | 49 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 91 | ok |  |
| block result BothDown | 95 | ok |  |
| block result Pushback | 212 | ok |  |
| block result PowPushback | 85 | ok |  |
| block result Pow | 91 | ok |  |
| pushbacks | 0 | **MISSING** |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 0 | **MISSING** |  |
| armor held | 447 | ok |  |
| stunned | 184 | ok | injury 2-7 |
| KO | 78 | ok |  |
| casualty (d16) | 56 | ok |  |
| death | 5 | ok | d16 = 15-16 only |
| fouls | 119 | ok |  |
| argue the call | 0 | **MISSING** | referee spotted a foul (doubles) |
| argue success | 0 | absent (optional) | d6 = 6 only |
| players ejected | 0 | **MISSING** |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 110 | ok |  |
| weather changes | 67 | ok | kickoff event roll of 8 only |
| kickoff events | 110 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 18
- Charge: 12
- Cheering Fans: 16
- Dodgy Snack: 3
- Get the Ref: 2
- High Kick: 13
- Pitch Invasion: 4
- Quick Snap: 17
- Solid Defence: 9
- Time-out: 4
- Weather Change: 12

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
