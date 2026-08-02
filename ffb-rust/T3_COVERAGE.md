# T3 lineman coverage — 46 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 12227 | ok |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 206 | ok |  |
| action Blitz | 287 | ok |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 105 | ok |  |
| action Pass | 37 | ok | needs a ball carrier |
| action HandOver | 19 | ok | needs carrier + adjacent teammate |
| dodge success | 187 | ok |  |
| dodge failure | 158 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 24 | ok |  |
| pickup failure | 17 | ok | turnover + scatter |
| catch success | 27 | ok |  |
| catch failure | 22 | ok |  |
| ball scatters | 144 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 2 | ok | ball out of bounds |
| pass rolls | 37 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 364 | ok |  |
| block 2 dice | 80 | ok |  |
| block 2 dice against | 43 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 83 | ok |  |
| block result BothDown | 79 | ok |  |
| block result Pushback | 181 | ok |  |
| block result PowPushback | 70 | ok |  |
| block result Pow | 74 | ok |  |
| pushbacks | 325 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 381 | ok |  |
| armor held | 384 | ok |  |
| stunned | 154 | ok | injury 2-7 |
| KO | 67 | ok |  |
| casualty (d16) | 43 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 105 | ok |  |
| argue the call | 21 | ok | referee spotted a foul (doubles) |
| argue success | 1 | ok | d6 = 6 only |
| players ejected | 21 | ok |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 92 | ok |  |
| weather changes | 11 | ok | kickoff event roll of 8 only |
| kickoff events | 92 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 15
- Charge: 10
- Cheering Fans: 12
- Dodgy Snack: 3
- Get the Ref: 2
- High Kick: 9
- Pitch Invasion: 3
- Quick Snap: 17
- Solid Defence: 8
- Time-out: 2
- Weather Change: 11

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
