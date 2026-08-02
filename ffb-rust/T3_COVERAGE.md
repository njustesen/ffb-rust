# T3 lineman coverage — 25 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 6610 | ok |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 109 | ok |  |
| action Blitz | 157 | ok |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 53 | ok |  |
| action Pass | 16 | ok | needs a ball carrier |
| action HandOver | 11 | ok | needs carrier + adjacent teammate |
| dodge success | 105 | ok |  |
| dodge failure | 99 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 11 | ok |  |
| pickup failure | 11 | ok | turnover + scatter |
| catch success | 13 | ok |  |
| catch failure | 16 | ok |  |
| ball scatters | 86 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 2 | ok | ball out of bounds |
| pass rolls | 16 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 199 | ok |  |
| block 2 dice | 44 | ok |  |
| block 2 dice against | 18 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 40 | ok |  |
| block result BothDown | 41 | ok |  |
| block result Pushback | 99 | ok |  |
| block result PowPushback | 39 | ok |  |
| block result Pow | 42 | ok |  |
| pushbacks | 180 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 221 | ok |  |
| armor held | 209 | ok |  |
| stunned | 81 | ok | injury 2-7 |
| KO | 41 | ok |  |
| casualty (d16) | 24 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 53 | ok |  |
| argue the call | 11 | ok | referee spotted a foul (doubles) |
| argue success | 1 | ok | d6 = 6 only |
| players ejected | 11 | ok |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 50 | ok |  |
| weather changes | 6 | ok | kickoff event roll of 8 only |
| kickoff events | 50 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 10
- Charge: 5
- Cheering Fans: 8
- Get the Ref: 1
- High Kick: 4
- Pitch Invasion: 2
- Quick Snap: 8
- Solid Defence: 5
- Time-out: 1
- Weather Change: 6

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
