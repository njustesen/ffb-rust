# T3 lineman coverage — 100 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 8829 | ok |  |
| action StandUp | 451 | ok | prone player stands (mapped from Move choice) |
| action Block | 276 | ok |  |
| action Blitz | 240 | ok |  |
| action StandUpBlitz | 72 | ok | prone + adjacent + blitz available |
| action Foul | 277 | ok |  |
| action Pass | 107 | ok | needs a ball carrier |
| action HandOver | 47 | ok | needs carrier + adjacent teammate |
| dodge success | 787 | ok |  |
| dodge failure | 538 | ok |  |
| GFI rolls | 16610 | ok |  |
| pickup success | 79 | ok |  |
| pickup failure | 49 | ok | turnover + scatter |
| catch success | 62 | ok |  |
| catch failure | 58 | ok |  |
| ball scatters | 339 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 1 | ok | ball out of bounds |
| pass rolls | 131 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 399 | ok |  |
| block 2 dice | 73 | ok |  |
| block 2 dice against | 44 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 85 | ok |  |
| block result BothDown | 87 | ok |  |
| block result Pushback | 186 | ok |  |
| block result PowPushback | 71 | ok |  |
| block result Pow | 87 | ok |  |
| pushbacks | 344 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 2610 | ok |  |
| armor held | 1781 | ok |  |
| stunned | 748 | ok | injury 2-7 |
| KO | 315 | ok |  |
| casualty (d16) | 219 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 277 | ok |  |
| argue the call | 68 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 62 | ok |  |
| touchdowns | 4 | ok |  |
| half starts | 200 | ok |  |
| weather changes | 26 | ok | kickoff event roll of 8 only |
| kickoff events | 203 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 38
- Charge: 10
- Cheering Fans: 35
- Dodgy Snack: 14
- Get the Ref: 4
- High Kick: 24
- Pitch Invasion: 3
- Quick Snap: 20
- Solid Defence: 18
- Time-out: 11
- Weather Change: 26

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: ALL REQUIRED ITEMS PRESENT
