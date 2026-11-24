"""Diagnostic: are the model's formations lopsided when the team is short-handed?

    python symmetry.py --ckpt runs/v4_coach_split/best.pt

For every held-out formation (val + test) it measures top/bottom symmetry of the human formation
and of the model's, grouped by how many players the team could field:

  unpaired   players whose mirror square (y -> 14 - y) is empty (a centre-row player pairs with itself)
  imbalance  |players above the centre row - players below it|
  mirror     mean distance (squares) between the formation and its own mirror image, best matching

Decoders compared: greedy (the most likely action at every step) and best-of-K (K sampled
rollouts plus the greedy one, keeping the formation with the highest total log-probability).
"""
from __future__ import annotations

import argparse
import math

import numpy as np
import torch

from features import A_SKIP, H, human_formation
from model import SetupPolicy
from rollout import _displacement, compare, rollout
from train import load_protos


def sym(squares: list[int]) -> dict[str, float]:
    mirror = [(s // H) * H + (H - 1 - s % H) for s in squares]
    ys = [s % H for s in squares]
    return {"unpaired": float(len(set(squares) - set(mirror))),
            "imbalance": float(abs(sum(y < 7 for y in ys) - sum(y > 7 for y in ys))),
            "mirror": _displacement(squares, mirror)}


def best_of_k(model, protos, device, k: int, margin: float | None = None):
    """Per proto the rollout with the highest joint log-probability among greedy + k samples.
    With `margin`: among the rollouts within `margin` nats of that best one, the most symmetric
    (fewest unpaired players), ties to the more likely."""
    cands = [[] for _ in protos]
    for j in range(k + 1):
        eps, traces = rollout(model, protos, device, temperature=0.0 if j == 0 else 1.0, seed=j, keep_probs=True)
        for i, (e, tr) in enumerate(zip(eps, traces)):
            lp = sum(math.log(max(float(p[a]), 1e-30)) for _, a, p in tr)
            cands[i].append((lp, e))
    out = []
    for cs in cands:
        top = max(lp for lp, _ in cs)
        if margin is None:
            out.append(max(cs, key=lambda c: c[0])[1])
        else:
            near = [c for c in cs if c[0] >= top - margin]
            out.append(min(near, key=lambda c: (sym([s for _, s in c[1].formation()])["unpaired"], -c[0]))[1])
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default="data/setups_h.jsonl")
    ap.add_argument("--ckpt", default="runs/v4_coach_split/best.pt")
    ap.add_argument("--k", type=int, default=16)
    args = ap.parse_args()

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    ck = torch.load(args.ckpt, map_location=device, weights_only=False)
    model = SetupPolicy(**ck["hp"]).to(device)
    model.load_state_dict(ck["model"])
    _, data, _ = load_protos(args.data, vocab=ck["vocab"])
    protos = data["val"] + data["test"]
    n_train = {}
    for p in data["train"]:
        n_train[min(p.n_field, 11)] = n_train.get(min(p.n_field, 11), 0) + 1
    print("training formations by players fielded:", dict(sorted(n_train.items())))

    decoders = {"greedy": rollout(model, protos, device), f"best of {args.k}": best_of_k(model, protos, device, args.k),
                "sym. of 16": best_of_k(model, protos, device, args.k, margin=2.0)}
    groups = [("11 players", lambda n: n == 11), ("10 players", lambda n: n == 10), ("9 or fewer", lambda n: n <= 9)]
    for head, flag in (("defense", False), ("offense", True)):
        print(f"\n{head}")
        print(f"  {'':12s} {'n':>5s} {'who':>12s} {'unpaired':>9s} {'imbalance':>10s} {'mirror':>7s} {'lopsided':>9s} {'vs human: class+sq':>19s} {'displ.':>7s}")
        for name, ok in groups:
            idx = [i for i, p in enumerate(protos) if p.offense == flag and ok(p.n_field)]
            if not idx:
                continue
            rows = {"human": [sym([s for _, s in human_formation(protos[i])]) for i in idx]}
            cmp = {}
            for dname, eps in decoders.items():
                rows[dname] = [sym([s for _, s in eps[i].formation()]) for i in idx]
                cmp[dname] = [compare(eps[i].formation(), human_formation(protos[i])) for i in idx]
            for who, r in rows.items():
                lop = np.mean([x["imbalance"] >= 2 for x in r])
                extra = ""
                if who in cmp:
                    extra = f" {np.mean([c['class_overlap'] for c in cmp[who]]):19.3f} {np.mean([c['displacement'] for c in cmp[who]]):7.2f}"
                print(f"  {name:12s} {len(idx):5d} {who:>12s} {np.mean([x['unpaired'] for x in r]):9.2f} "
                      f"{np.mean([x['imbalance'] for x in r]):10.2f} {np.mean([x['mirror'] for x in r]):7.2f} {lop:9.1%}{extra}")


if __name__ == "__main__":
    main()
