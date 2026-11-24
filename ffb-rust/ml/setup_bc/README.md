# setup_bc — behavioural cloning of kick-off formations

One model sets up both the kicking side (defense head) and the receiving side (offense head),
one player at a time, learned from human formations in FUMBBL replays.

## Pipeline

All commands from the cargo workspace root (`ffb-rust/ffb-rust`).

```bash
# 1. dataset: one JSON record per finished human setup (plus the heuristic agent's formation
#    for the same situation, as a baseline)
cargo build --release -p ffb-parity
target/release/ffb-replay setups ml/setup_bc/data/setups_h.jsonl D:/ffb_replays/fumbbl D:/ffb_replays/partial \
    D:/ffb_replays/window D:/ffb_replays/fumbbl_night_20261122 D:/ffb_replays/fumbbl_night_20261122/partial

# 2. train (GPU if there is one; ~80 s per epoch on 13.6k formations)
# who coached which side (hashed) -> data/coaches.json. The split is by COACH: a held-out coach
# never reaches training in any role (train.split_of); rerun after re-extracting the dataset.
python ml/setup_bc/coaches.py --data ml/setup_bc/data/setups_h.jsonl --replays D:/ffb_replays

# resumable: --max-minutes N stops cleanly, --resume continues from <out>/last.pt
python ml/setup_bc/train.py --data ml/setup_bc/data/setups_h.jsonl --out ml/setup_bc/runs/v4_coach_split --epochs 20 --random-subset 0.5

# held-out kick-offs, model next to the human formations, client graphics, hover for stats
python ml/setup_bc/examples.py --split val --n 12

python ml/setup_bc/train.py --data ml/setup_bc/data/setups_h.jsonl --out ml/setup_bc/runs/v1

# 3. score the model's own formations against the human ones and the baselines
python ml/setup_bc/eval.py --data ml/setup_bc/data/setups_h.jsonl --ckpt ml/setup_bc/runs/v1/best.pt

# 4. watch it set up two teams (any race in data/teams/<edition>/team_<race>.json)
python ml/setup_bc/watch.py --home human --away orc --edition bb2025 --kicker away
#    -> ml/setup_bc/runs/watch_human_vs_orc_bb2025.html (self-contained; open in a browser)

python -m pytest ml/setup_bc
```

## Design

- **Canonical view.** The side setting up is always on the left (x 0..12, line of scrimmage
  x = 12); the board is mirrored when the away team sets up.
- **Inputs.** A 26×15 board of 16-float player embeddings (zeros where empty), the embedding of
  the player being placed, fixed position planes, the legal-square plane; game variables, team
  variables and 16 bench embeddings per team go through an MLP that conditions every residual
  block (FiLM). The player encoder (stats + skill embeddings → 16 floats) is trained end to end.
- **Loop.** The bench is ordered by value, ST, AG, PA, AV, number of skills. Each step places the
  head of the bench on a square or skips it; the setup ends by rule (eleven fielded, or the bench
  is empty). A legality mask keeps every rollout rule-legal (own half, free square, at most two
  per wide zone, three on the line of scrimmage).
- **Loss.** Teacher forcing with a set-valued target: at each step the loss is `-log` of the
  probability on *every* action that agrees with the human formation — any still-empty square
  where the human put a player with identical stats and skills, plus "skip" when the human left
  one of those on the bench. Names and numbers never matter.
- **Evaluation.** The model's own greedy formation vs the human one: exact match, same class on
  the same square, same square, and mean displacement of the best one-to-one matching. A formation
  and its top/bottom mirror image are scored as the same formation.

## Files

| File | Purpose |
|---|---|
| `crates/ffb-parity/src/setup_record.rs` | the record (shared by the dataset and the viewer input) |
| `crates/ffb-parity/src/replay/setups.rs` | `ffb-replay setups`: human formations from replays |
| `features.py` | record → inputs, bench order, the placement loop, masks, targets |
| `model.py` | player encoder, FiLM ResNet, two heads, the set loss |
| `train.py`, `eval.py`, `rollout.py` | training, baselines, rollouts and formation metrics |
| `watch.py`, `watch_template.html` | the viewer |
