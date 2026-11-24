"""Do teams that win set up differently from teams that lose?

    python subset_compare.py

Compares the model trained on formations of teams that won or drew (runs/v6_winning) with the one
trained on teams that lost (runs/v6_losing), against two control models trained on a random
60 / 40 partition of the same data (runs/v6_ctrl_a, v6_ctrl_b). Whatever the win / loss pair shows
only means something where it exceeds what the control pair shows.

1. Cross-fit: each model's teacher-forced loss on held-out formations of winners and of losers.
   If setups differ by outcome, the winning model should fit winners relatively better.
2. How far apart the two models' own formations are on the same held-out situations.
"""
from __future__ import annotations

import argparse
import os

import numpy as np
import torch

from model import SetupPolicy
from outcome_compare import shape
from rollout import rollout
from train import evaluate, load_protos

HERE = os.path.dirname(os.path.abspath(__file__))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=os.path.join(HERE, "data", "setups_h.jsonl"))
    ap.add_argument("--runs", default=os.path.join(HERE, "runs"))
    args = ap.parse_args()
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    models, vocab = {}, None
    for name in ("winning", "losing", "ctrl_a", "ctrl_b"):
        ck = torch.load(os.path.join(args.runs, f"v6_{name}", "best.pt"), map_location=device, weights_only=False)
        m = SetupPolicy(**ck["hp"]).to(device)
        m.load_state_dict(ck["model"])
        models[name], vocab = m, ck["vocab"]
        print(f"{name:8s} best epoch {ck['epoch']:2d}, val nll {ck['val']['nll']:.4f}")
    _, data, _ = load_protos(args.data, vocab=vocab)
    protos = data["val"] + data["test"]
    groups = {"winners + drawers": [p for p in protos if p.z >= 0.5], "losers": [p for p in protos if p.z < 0.5]}

    print("\n1. teacher-forced loss on held-out formations (lower = fits better)")
    print(f"  {'model':10s}" + "".join(f"{g + ' (n=' + str(len(ps)) + ')':>28s}" for g, ps in groups.items()) + f"{'losers - winners':>20s}")
    gap = {}
    for name, m in models.items():
        nll = [evaluate(m, ps, device)["nll"] for ps in groups.values()]
        gap[name] = nll[1] - nll[0]
        print(f"  {name:10s}" + "".join(f"{v:28.4f}" for v in nll) + f"{gap[name]:+20.4f}")
    print(f"  interaction, winning vs losing model: {gap['winning'] - gap['losing']:+.4f}   "
          f"(control pair: {gap['ctrl_a'] - gap['ctrl_b']:+.4f})")

    print("\n2. the two models' own formations on the same held-out situations")
    eps = {name: rollout(m, protos, device) for name, m in models.items()}
    for head, flag in (("defense (kicking)", False), ("offense (receiving)", True)):
        idx = [i for i, p in enumerate(protos) if p.offense == flag]
        print(f"\n  {head}, n={len(idx)}")
        print(f"  {'':42s} {'winning vs losing':>18s} {'control a vs b':>16s}")
        rows = {}
        for label, (a, b) in (("wl", ("winning", "losing")), ("ctrl", ("ctrl_a", "ctrl_b"))):
            moved = [len({s for _, s in eps[a][i].formation()} - {s for _, s in eps[b][i].formation()}) for i in idx]
            sa, sb = [shape(eps[a][i]) for i in idx], [shape(eps[b][i]) for i in idx]
            rows[label] = {"formations that differ": f"{np.mean([m > 0 for m in moved]):.1%}",
                           "squares that differ, mean": f"{np.mean(moved):.2f}"}
            for k in sa[0]:
                d = np.array([x[k] - y[k] for x, y in zip(sa, sb)])
                rows[label][k] = f"{d.mean():+.3f} ±{2 * d.std() / np.sqrt(len(d)):.3f}"
        for k in rows["wl"]:
            print(f"  {k:42s} {rows['wl'][k]:>18s} {rows['ctrl'][k]:>16s}")


if __name__ == "__main__":
    main()
