# T3 lineman coverage — 29 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 7605 | ok |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 118 | ok |  |
| action Blitz | 189 | ok |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 70 | ok |  |
| action Pass | 19 | ok | needs a ball carrier |
| action HandOver | 9 | ok | needs carrier + adjacent teammate |
| dodge success | 124 | ok |  |
| dodge failure | 126 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 17 | ok |  |
| pickup failure | 6 | ok | turnover + scatter |
| catch success | 15 | ok |  |
| catch failure | 17 | ok |  |
| ball scatters | 103 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 17 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 229 | ok |  |
| block 2 dice | 53 | ok |  |
| block 2 dice against | 23 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 50 | ok |  |
| block result BothDown | 46 | ok |  |
| block result Pushback | 117 | ok |  |
| block result PowPushback | 48 | ok |  |
| block result Pow | 44 | ok |  |
| pushbacks | 209 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 264 | ok |  |
| armor held | 263 | ok |  |
| stunned | 90 | ok | injury 2-7 |
| KO | 53 | ok |  |
| casualty (d16) | 24 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 70 | ok |  |
| argue the call | 14 | ok | referee spotted a foul (doubles) |
| argue success | 2 | ok | d6 = 6 only |
| players ejected | 12 | ok |  |
| touchdowns | 1 | ok |  |
| half starts | 58 | ok |  |
| weather changes | 6 | ok | kickoff event roll of 8 only |
| kickoff events | 59 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 11
- Charge: 7
- Cheering Fans: 11
- Dodgy Snack: 2
- Get the Ref: 1
- High Kick: 6
- Pitch Invasion: 3
- Quick Snap: 7
- Solid Defence: 4
- Time-out: 1
- Weather Change: 6

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
