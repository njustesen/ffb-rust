# T3 lineman coverage — 7 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 1866 | ok |  |
| action StandUp | 0 | **MISSING** | prone player stands (mapped from Move choice) |
| action Block | 21 | ok |  |
| action Blitz | 47 | ok |  |
| action StandUpBlitz | 0 | **MISSING** | prone + adjacent + blitz available |
| action Foul | 13 | ok |  |
| action Pass | 2 | ok | needs a ball carrier |
| action HandOver | 1 | ok | needs carrier + adjacent teammate |
| dodge success | 26 | ok |  |
| dodge failure | 31 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 0 | **MISSING** |  |
| pickup failure | 3 | ok | turnover + scatter |
| catch success | 1 | ok |  |
| catch failure | 2 | ok |  |
| ball scatters | 19 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 2 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 50 | ok |  |
| block 2 dice | 14 | ok |  |
| block 2 dice against | 4 | ok | defender's choice |
| block 3 dice | 0 | absent (optional) | needs ST5+ differential via assists |
| block result Skull | 12 | ok |  |
| block result BothDown | 14 | ok |  |
| block result Pushback | 23 | ok |  |
| block result PowPushback | 8 | ok |  |
| block result Pow | 11 | ok |  |
| pushbacks | 42 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 64 | ok |  |
| armor held | 58 | ok |  |
| stunned | 26 | ok | injury 2-7 |
| KO | 14 | ok |  |
| casualty (d16) | 5 | ok |  |
| death | 0 | absent (optional) | d16 = 15-16 only |
| fouls | 13 | ok |  |
| argue the call | 2 | ok | referee spotted a foul (doubles) |
| argue success | 0 | absent (optional) | d6 = 6 only |
| players ejected | 2 | ok |  |
| touchdowns | 0 | **MISSING** |  |
| half starts | 14 | ok |  |
| weather changes | 2 | ok | kickoff event roll of 8 only |
| kickoff events | 14 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 4
- Charge: 2
- Cheering Fans: 2
- High Kick: 1
- Quick Snap: 1
- Solid Defence: 2
- Weather Change: 2

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
