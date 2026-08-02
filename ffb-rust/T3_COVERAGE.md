# T3 lineman coverage — 13 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 3377 | ok |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 48 | ok |  |
| action Blitz | 89 | ok |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 31 | ok |  |
| action Pass | 6 | ok | needs a ball carrier |
| action HandOver | 3 | ok | needs carrier + adjacent teammate |
| dodge success | 53 | ok |  |
| dodge failure | 59 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 8 | ok |  |
| pickup failure | 2 | ok | turnover + scatter |
| catch success | 4 | ok |  |
| catch failure | 7 | ok |  |
| ball scatters | 44 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 6 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 99 | ok |  |
| block 2 dice | 27 | ok |  |
| block 2 dice against | 10 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 17 | ok |  |
| block result BothDown | 28 | ok |  |
| block result Pushback | 52 | ok |  |
| block result PowPushback | 21 | ok |  |
| block result Pow | 18 | ok |  |
| pushbacks | 91 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 126 | ok |  |
| armor held | 122 | ok |  |
| stunned | 45 | ok | injury 2-7 |
| KO | 27 | ok |  |
| casualty (d16) | 8 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 31 | ok |  |
| argue the call | 8 | ok | referee spotted a foul (doubles) |
| argue success | 1 | ok | d6 = 6 only |
| players ejected | 7 | ok |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 26 | ok |  |
| weather changes | 2 | ok | kickoff event roll of 8 only |
| kickoff events | 26 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 6
- Charge: 3
- Cheering Fans: 5
- Dodgy Snack: 1
- Get the Ref: 1
- High Kick: 2
- Pitch Invasion: 1
- Quick Snap: 2
- Solid Defence: 3
- Weather Change: 2

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
