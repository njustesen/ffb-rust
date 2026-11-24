"""What does the outcome input z change? (model trained with train.py --outcome)

    python outcome_compare.py --ckpt runs/v5_outcome/best.pt

1. Does the model use z at all: teacher-forced loss on held-out formations with the true z,
   with z fixed at 0.5, and with z flipped (1 - z).
2. What z = 1 ("a team that goes on to win") does differently from z = 0, on the same held-out
   situations: how many players stand elsewhere, and simple shape statistics.
"""
from __future__ import annotations

import argparse
import copy

import numpy as np
import torch

from features import H, LOS_X, is_los, wide_zone
from model import SetupPolicy
from rollout import rollout
from train import evaluate, load_protos


def with_z(protos, f):
    out = []
    for p in protos:
        q = copy.copy(p)
        q.z = f(p.z)
        out.append(q)
    return out


def shape(e) -> dict[str, float]:
    sq = [s for _, s in e.formation()]
    xs, ys = [s // H for s in sq], [s % H for s in sq]
    return {
        "on the line of scrimmage": float(sum(map(is_los, sq))),
        "within 1 square of the line (x >= 11)": float(sum(x >= LOS_X - 1 for x in xs)),
        "deep (x <= 7)": float(sum(x <= 7 for x in xs)),
        "mean distance behind the line": float(np.mean([LOS_X - x for x in xs])),
        "in the wide zones": float(sum(wide_zone(s) >= 0 for s in sq)),
        "mean distance from the centre row": float(np.mean([abs(y - 7) for y in ys])),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default="data/setups_h.jsonl")
    ap.add_argument("--ckpt", default="runs/v5_outcome/best.pt")
    args = ap.parse_args()
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    ck = torch.load(args.ckpt, map_location=device, weights_only=False)
    model = SetupPolicy(**ck["hp"]).to(device)
    model.load_state_dict(ck["model"])
    source = ck.get("z_source", "outcome")
    _, data, _ = load_protos(args.data, vocab=ck["vocab"], z_source=source)
    protos = data["val"] + data["test"]
    zs = [p.z for p in protos]
    if source == "rating":
        print(f"z = coach rank. {len(protos)} held-out formations by rank: "
              + ", ".join(f"{z:.2f}: {zs.count(z)}" for z in sorted(set(zs))))
    else:
        print(f"{len(protos)} held-out formations: won {zs.count(1.0)}, drew {zs.count(0.5)}, lost {zs.count(0.0)}")

    print("\nteacher-forced loss (lower = the human formation is more likely)")
    for name, f in (("true z", lambda z: z), ("z = 0.5 for all", lambda z: 0.5), ("z flipped", lambda z: 1 - z)):
        m = evaluate(model, with_z(protos, f), device)
        print(f"  {name:16s} nll {m['nll']:.4f}  (defense {m['nll_def']:.4f}, offense {m['nll_off']:.4f})")

    lose = rollout(model, with_z(protos, lambda z: 0.0), device)
    win = rollout(model, with_z(protos, lambda z: 1.0), device)
    for head, flag in (("defense (kicking)", False), ("offense (receiving)", True)):
        idx = [i for i, p in enumerate(protos) if p.offense == flag]
        moved = [len({s for _, s in win[i].formation()} - {s for _, s in lose[i].formation()}) for i in idx]
        print(f"\n{head}, n={len(idx)}")
        print(f"  formations that differ between z=1 and z=0   {np.mean([m > 0 for m in moved]):6.1%}")
        print(f"  squares used at z=1 but not at z=0, mean     {np.mean(moved):6.2f}")
        a = [shape(win[i]) for i in idx]
        b = [shape(lose[i]) for i in idx]
        print(f"  {'':40s} {'z=1':>7s} {'z=0':>7s} {'diff':>7s} {'±2se':>7s}")
        for k in a[0]:
            d = np.array([x[k] - y[k] for x, y in zip(a, b)])
            print(f"  {k:40s} {np.mean([x[k] for x in a]):7.2f} {np.mean([y[k] for y in b]):7.2f} "
                  f"{d.mean():+7.3f} {2 * d.std() / np.sqrt(len(d)):7.3f}")


if __name__ == "__main__":
    main()
