# Replay-parity feasibility scripts (2026-11-21)

Evidence behind the "Feasibility analysis" section of `docs/REPLAY_PARITY_PLAN.md`. All read-only
against the repo. Run with `python -I`. Inputs and outputs live under `D:/ffb_replays/` (outside git).

| Script | What it measures |
|---|---|
| `replay_facts.py OUT.json` | Option ids/values, skills, model-change and report kinds across the downloaded replays |
| `id_compat.py facts.json` | Replay report/model-change/option ids unknown to our Java (`ffb-java/ffb/ffb-common`) |
| `date_base.py` | Which upstream (christerk/ffb) revision our engine files match (uses `gh api`) |
| `dice_exp_run.py` | Runs 6 Java bot games with `-Dffb.parityDebug(Deep)` + `--reports` on a private jar copy |
| `dice_exp_analyze.py DIR [--ordered]` | Die values found in same-window reports: unordered (upper bound) or naive flat order |
| `dice_exp_fields.py DIR` | Strict recovery with a learned source→report-field table, cross-validated across matchups |

Results (2026-11-21, 1,398 dice): unordered 98.8%; naive ordered 69.4%; field-mapped strict 94.8%.
