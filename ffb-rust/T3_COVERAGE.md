# T3 lineman coverage — 100 games

| Item | Count | Status | Note |
|---|---:|---|---|
| action Move | 23565 | ok |  |
| action StandUp | 0 | absent (optional) | not a distinct action: mapped into the Move choice by both agents |
| action Block | 477 | ok |  |
| action Blitz | 676 | ok |  |
| action StandUpBlitz | 0 | absent (optional) | not a distinct action: mapped into the Blitz choice by both agents |
| action Foul | 208 | ok |  |
| action Pass | 66 | ok | needs a ball carrier |
| action HandOver | 37 | ok | needs carrier + adjacent teammate |
| dodge success | 277 | ok |  |
| dodge failure | 340 | ok |  |
| GFI rolls | 39 | ok |  |
| pickup success | 34 | ok |  |
| pickup failure | 25 | ok | turnover + scatter |
| catch success | 212 | ok |  |
| catch failure | 199 | ok |  |
| ball scatters | 326 | ok | failed pickup / dropped ball / bounces |
| throw-ins | 0 | **MISSING** | ball out of bounds |
| pass rolls | 907 | ok |  |
| pass deviates | 0 | absent (optional) | wildly inaccurate passes only |
| interceptions | 0 | absent (optional) | contract: agents decline voluntary interference |
| block 1 die | 423 | ok |  |
| block 2 dice | 95 | ok |  |
| block 2 dice against | 95 | ok | defender's choice |
| block 3 dice | 325 | ok | needs ST5+ differential via assists |
| block result Skull | 129 | ok |  |
| block result BothDown | 160 | ok |  |
| block result Pushback | 336 | ok |  |
| block result PowPushback | 154 | ok |  |
| block result Pow | 159 | ok |  |
| pushbacks | 653 | ok |  |
| crowd surfs | 1102 | ok | push off pitch — board-position dependent |
| players fell | 661 | ok |  |
| armor held | 1281 | ok |  |
| stunned | 240 | ok | injury 2-7 |
| KO | 226 | ok |  |
| casualty (d16) | 167 | ok |  |
| death | 14 | ok | d16 = 15-16 only |
| fouls | 145 | ok |  |
| argue the call | 21 | ok | referee spotted a foul (doubles) |
| argue success | 1 | ok | d6 = 6 only |
| players ejected | 21 | ok |  |
| touchdowns | 0 | **MISSING** | NEVER exercised: the random agent does not score — needs a scoring-biased tier |
| half starts | 200 | ok |  |
| weather changes | 27 | ok | kickoff event roll of 8 only |
| kickoff events | 200 | ok | per-result table below |

## Kickoff results

- Brilliant Coaching: 32
- Charge: 14
- Cheering Fans: 34
- Dodgy Snack: 10
- Get the Ref: 7
- High Kick: 27
- Pitch Invasion: 9
- Quick Snap: 19
- Solid Defence: 14
- Time-out: 7
- Weather Change: 27

## Hash-verified (not evented)

- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and
  banned-players-stay-off are not separate GameEvents; they are covered by
  the per-activation state hashes that must match Java exactly.

Result: REQUIRED ITEMS MISSING
