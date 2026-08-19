# T3 lineman coverage — 100 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 27680 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 529 | ok |  |
| action Blitz | 668 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 231 | ok |  |
| action Pass | 88 | ok | needs a ball carrier |
| action HandOver | 49 | ok | needs carrier + adjacent teammate |
| dodge success | 453 | ok |  |
| dodge failure | 401 | ok |  |
| GFI rolls | 0 | **MISSING** |  |
| pickup success | 52 | ok |  |
| pickup failure | 33 | ok | turnover + scatter |
| catch success | 48 | ok |  |
| catch failure | 57 | ok |  |
| ball scatters | 323 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 86 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 364 | ok |  |
| block 2 dice | 245 | ok |  |
| block 2 dice against | 265 | ok | defender's choice |
| block 3 dice | 162 | ok | needs ST5+ differential via assists |
| block result Skull | 176 | ok |  |
| block result BothDown | 164 | ok |  |
| block result Pushback | 365 | ok |  |
| block result PowPushback | 164 | ok |  |
| block result Pow | 167 | ok |  |
| pushbacks | 696 | ok |  |
| crowd surfs | 0 | absent (optional) | push off pitch — board-position dependent |
| players fell | 715 | ok |  |
| armor held | 905 | ok |  |
| stunned | 160 | ok | injury 2-7 |
| KO | 96 | ok |  |
| casualty (d16) | 66 | ok |  |
| death | 7 | ok | d16 = 15-16 only |
| fouls | 218 | ok |  |
| argue the call | 39 | ok | referee spotted a foul (doubles) |
| argue success | 7 | ok | d6 = 6 only |
| players ejected | 32 | ok |  |
| touchdowns | 0 | **MISSING** | NEVER exercised: the random agent does not score — needs a scoring-biased tier |
| half starts | 200 | ok |  |
| weather changes | 17 | ok | kickoff event roll of 8 only |
| kickoff events | 200 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 31
- Charge: 20
- Cheering Fans: 41
- Dodgy Snack: 5
- Get the Ref: 6
- High Kick: 31
- Pitch Invasion: 9
- Quick Snap: 18
- Solid Defence: 13
- Time-out: 9
- Weather Change: 17

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
