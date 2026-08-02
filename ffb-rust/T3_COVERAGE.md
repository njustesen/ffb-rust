# T3 lineman coverage — 100 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 26297 | ok |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 448 | ok |  |
| action Blitz | 607 | ok |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 232 | ok |  |
| action Pass | 90 | ok | needs a ball carrier |
| action HandOver | 39 | ok | needs carrier + adjacent teammate |
| dodge success | 422 | ok |  |
| dodge failure | 356 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 63 | ok |  |
| pickup failure | 31 | ok | turnover + scatter |
| catch success | 53 | ok |  |
| catch failure | 56 | ok |  |
| ball scatters | 326 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 2 | ok | ball out of bounds |
| pass rolls | 89 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 790 | ok |  |
| block 2 dice | 170 | ok |  |
| block 2 dice against | 82 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 182 | ok |  |
| block result BothDown | 173 | ok |  |
| block result Pushback | 367 | ok |  |
| block result PowPushback | 159 | ok |  |
| block result Pow | 161 | ok |  |
| pushbacks | 687 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 849 | ok |  |
| armor held | 834 | ok |  |
| stunned | 353 | ok | injury 2-7 |
| KO | 151 | ok |  |
| casualty (d16) | 98 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 232 | ok |  |
| argue the call | 43 | ok | referee spotted a foul (doubles) |
| argue success | 6 | ok | d6 = 6 only |
| players ejected | 39 | ok |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 200 | ok |  |
| weather changes | 26 | ok | kickoff event roll of 8 only |
| kickoff events | 200 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 33
- Charge: 20
- Cheering Fans: 26
- Dodgy Snack: 5
- Get the Ref: 5
- High Kick: 23
- Pitch Invasion: 5
- Quick Snap: 33
- Solid Defence: 15
- Time-out: 9
- Weather Change: 26

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
