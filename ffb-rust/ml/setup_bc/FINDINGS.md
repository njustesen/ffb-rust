# Setup policy: training findings

Behavioural cloning of kick-off formations from FUMBBL replays. Status as of 2026-11-24.
Commands are in `README.md`; this file records what was tried and what came out.

## Summary

- A 785k-parameter model trained on 11,153 human formations places about twice as many players
  exactly as the human did as the hand-written heuristic does, on coaches it has never seen.
- The model overfits after about epoch 10 of 20. More data is the obvious lever; we train on
  2,211 replays out of roughly 1.97 million games on FUMBBL.
- Short-handed formations (fewer than 11 players) come out lopsided more often than full ones.
  The model is still more symmetric than humans at every player count.
- Conditioning on the game result or on coach rank does not improve prediction. Both move the
  formation in a small, consistent direction: stronger or winning coaches kick from a deeper setup.

## Data

| | Replays | Formations |
|---|---|---|
| With setups, total | 2,211 | |
| Train | 1,541 | 11,153 (11,144 legal) |
| Validation | 355 | 1,359 |
| Test | 283 | 1,091 |
| Dropped: game straddles the split | | 2,485 |
| Replays unused | 32 | |

- Replay ids 1,963,617 to 1,966,189; 98.9% BB2025. 1,214 coaches, 2,569 teams.
- Largest team seen has 23 players; `TEAM_MAX` is 24. Raise it to 32 before a bigger data pull.
- Results (`data/outcomes.json`): 867 home wins, 501 draws, 843 home losses, 129 concessions
  (a concession counts as a loss).

### Split

Split by coach, not by replay: `sha1(coach) % 1000`, below 75 is test, below 150 is validation.

- A formation belongs to the split of the coach who set it up.
- A training formation whose opponent is a held-out coach is dropped, because the opponent's
  formation or bench is part of its input.
- Validation-against-test games are dropped, so the two held-out sets share no game.

The first models used a replay-level split, where the same coach and team appear on both sides.
Moving to the coach split raised validation loss from 1.755 to 1.927. The earlier number was
flattered by leakage; only coach-split numbers should be quoted.

## Training recipe

- Teacher forcing with a set-valued target: the loss is minus the log of the probability mass on
  every action consistent with the human formation, so interchangeable players count as the same.
- AdamW, OneCycleLR (peak 2e-3, 10% warm-up), weight decay 1e-4, dropout 0.1, batch 48
  formations, 20 epochs, y-flip augmentation.
- `--random-subset 0.5`: half the formations are shown as random partial formations instead of
  bench-order prefixes.
- About 45 s per epoch on the GPU.

Operational notes:

- Run training in the foreground in chunks (`--resume --max-minutes 8`). Background and
  concurrent runs get killed by the low-memory reaper.
- Do not let an input file change between chunks. The rank model's last epoch crashed because
  `coach_ratings.json` grew mid-run, which changed the number of steps per epoch under OneCycleLR.

## Runs

| Run | What | Train formations | Best epoch | Val loss |
|---|---|---|---|---|
| `v3_subset50` | replay split (leaky) | | 14 | 1.755 |
| `v4_coach_split` | **baseline**, coach split | 11,153 | 10 | 1.927 |
| `v5_outcome` | + z = game result | 11,153 | 10 | 1.915 |
| `v6_winning` | teams that won or drew only | 6,852 | 11 | 2.013 |
| `v6_losing` | teams that lost only | 4,292 | 12 | 2.187 |
| `v6_ctrl_a` | random 60% control | 6,644 | 13 | 1.994 |
| `v6_ctrl_b` | random 40% control | 4,500 | 13 | 2.105 |
| `v7_rating` | + z = coach rank (83% of data) | 9,246 | 11 | 1.948 |

Validation loss of `v7_rating` is on the subset with a known rank and is not comparable with the
others.

## Baseline against the heuristic (`v4_coach_split`, test split)

Test loss 1.791. Rollouts compared with the human formation:

| | Same kind of player on the same square | Same squares | Mean displacement |
|---|---|---|---|
| **Kicking (defense head)** | | | |
| Model | 18.9% | 43.0% | 1.60 |
| Heuristic scorer | 8.9% | 29.2% | 1.88 |
| Most frequent formation | n/a | 43.9% | 1.92 |
| **Receiving (offense head)** | | | |
| Model | 22.6% | 56.4% | 1.10 |
| Heuristic scorer | 10.6% | 32.3% | 1.67 |
| Most frequent formation | n/a | 58.5% | 1.14 |

- The model beats the heuristic on every measure.
- On squares alone, always playing the most common formation is as good as the model. The
  model's gain is in putting the right kind of player on each square.
- Training loss keeps falling after epoch 10 while validation loss rises: the model is limited
  by data, not capacity.

## Symmetry when short-handed

Observation: with fewer than 11 players the model often builds its usual 11-player formation and
leaves one square empty.

- Receiving formations are lopsided 13–15% of the time with 9–10 players, against 3% with 11.
  Humans are lopsided 30–38% of the time, so the model is more symmetric than its teachers at
  every player count.
- Counterfactual (remove one random player from an 11-player situation): 42–48% of the resulting
  10-player formations reuse 10 of the same 11 squares, and 19–30% of formations that were
  symmetric with 11 are no longer symmetric.
- Picking the most symmetric of 16 sampled formations halves the unpaired players on the kicking
  side and does little on the receiving side.

Not done: retraining with short-handed formations weighted about 3x plus a symmetry penalty.
Short-handed formations are a small share of the data, so more data may fix this by itself.

Scripts: `symmetry.py`, `shorthand_test.py`.

## Conditioning on the game result (`v5_outcome`)

Extra scalar input z: 1 won, 0.5 drew, 0 lost, for the team setting up.

- The model nearly ignores it. Held-out loss: true z 1.8563, z fixed at 0.5 1.8565, z flipped
  1.8572.
- z = 1 against z = 0 changes 21% of formations (0.71 squares kicking, 0.38 receiving).
- Direction: z = 1 is slightly deeper when kicking and slightly wider when receiving.

Script: `outcome_compare.py`.

## Separate models for winners and losers (`v6_*`)

Two models on disjoint data, with two control models on a random split of the same proportions.
The control pair is the yardstick: whatever it shows is what any two models trained on
different data show.

| Same 2,450 held-out situations | Winning vs losing | Control A vs B |
|---|---|---|
| Kicking: formations that differ | 91.3% | 89.0% |
| Kicking: squares that differ | 4.61 | 4.25 |
| Receiving: formations that differ | 96.8% | 92.7% |
| Receiving: squares that differ | 3.19 | 2.75 |

Nearly all of the visible difference is training noise. What exceeds the control:

- Receiving: winning model has 0.60 ± 0.09 more players in the wide zones (control −0.12).
- Kicking: winning model has 0.51 ± 0.13 more players deep and 0.43 ± 0.10 more in the wide
  zones (control about 0.3 in the opposite direction).
- Cross-fit: the winning model fits held-out winners better than losers by a wider margin than
  the losing model does; interaction +0.102 in loss, control −0.032.

Caveats: one control pair gives the scale of the noise, not an error bar. Losing teams are more
often short-handed and behind, so part of the difference is situation, not choice. Both split
models are worse than the model trained on everything.

Script: `subset_compare.py`.

## Conditioning on coach rank (`v7_rating`)

z is the FUMBBL coach rank from the public API (`coach/search/<name>` then `coach/get/<id>`,
field `rating`), mapped evenly to 0..1 in the order Rookie, Experienced, Veteran, Emerging Star,
Star, Super Star, Mega-Star, Legend.

- Coverage when trained: 561 of 1,214 coaches, 83% of formations. Formations without a rank
  were left out.
- Heavily skewed: of 2,047 held-out formations, 1,084 are from Super Star coaches, 191 from
  Rookies and 43 from Legends.
- Prediction does not improve. Held-out loss: true rank 1.8586, z fixed at 0.5 1.8593, rank
  flipped 1.8610.

Legend (z = 1) against Rookie (z = 0) on the same situations:

| | Kicking | Receiving |
|---|---|---|
| Formations that differ | 46.4% | 30.7% |
| Squares that differ | 1.93 | 0.65 |
| Players within 1 square of the line | −0.41 ± 0.09 | −0.12 ± 0.03 |
| Players deep (x ≤ 7) | +0.44 ± 0.11 | +0.01 ± 0.01 |
| Players in the wide zones | −0.13 ± 0.07 | +0.02 ± 0.04 |

Caveats:

- The rank is the coach's rank today, not at the time of the game.
- Eight coarse levels; the even spacing and the ladder order are assumptions, not taken from a
  FUMBBL page.
- "Legends set up deeper" is what the model learned. The raw human formations have not been
  compared by rank.
- Accuracy against the baseline on the same test formations has not been measured.

Scripts: `coach_ratings.py`, `outcome_compare.py`.

## What agrees across the three experiments

| | Kicking | Receiving |
|---|---|---|
| z = win | slightly deeper | slightly wider |
| Winning model | deeper and wider | wider |
| z = Legend | deeper, slightly narrower | no change |

A deeper kicking setup for better or winning coaches shows up in all three. Width does not
agree. None of the three improves prediction.

## Open items

- More data. Overfitting after epoch 10 and the short-handed symmetry problem both point there.
  A learning curve at 25 / 50 / 100% of the data would say how much more is worth fetching.
- Symmetry fix: reweight short-handed formations and add a symmetry penalty.
- Like-for-like test accuracy of the rank model against the baseline.
- Raise `TEAM_MAX` to 32 before the next data pull.
- Per-drive outcome (did this drive score) as a sharper signal than the game result.
- Finish the coach rank download and retrain if coverage changes materially.
- The example pages (`runs/examples_val*.html`) have been checked by script only, not by eye.
