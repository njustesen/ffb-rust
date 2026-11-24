"""Score a trained policy's own formations against the human ones, next to two baselines.

    python eval.py --data data/setups_h.jsonl --ckpt runs/v1/best.pt --split test

model      the policy run in its own loop, greedy (most likely action at every step)
heuristic  the heuristic agent's setup scorer in the same situation, best option at every step
           (the "heuristic" field `ffb-replay setups` attaches to every record)
frequent   the squares humans use most often for that head, ignoring who stands there
"""
from __future__ import annotations

import argparse
import json
from collections import Counter

import numpy as np
import torch

from features import H, W, human_formation, sq
from model import SetupPolicy
from rollout import compare, rollout
from train import load_protos


def heuristic_formation(proto):
    cls = {proto.ids[row]: c for row, c in zip(proto.bench, proto.classes)}
    cx = (lambda x: x) if proto.setting_home else (lambda x: W - 1 - x)
    return sorted((cls.get(pid, -1), sq(cx(x), y)) for pid, x, y in proto.heuristic)


def summarise(rows):
    return {k: round(float(np.mean([r[k] for r in rows])), 4) for k in rows[0]} | {"n": len(rows)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default="data/setups_h.jsonl")
    ap.add_argument("--ckpt", default="runs/v1/best.pt")
    ap.add_argument("--split", default="test", choices=["val", "test"])
    ap.add_argument("--out", default=None)
    args = ap.parse_args()

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    ck = torch.load(args.ckpt, map_location=device, weights_only=False)
    model = SetupPolicy(**ck["hp"]).to(device)
    model.load_state_dict(ck["model"])

    _, data, _ = load_protos(args.data, vocab=ck["vocab"])

    # the most used squares per head, top/bottom symmetric (a formation and its mirror count once each)
    freq = {False: Counter(), True: Counter()}
    for p in data["train"]:
        for _, s in human_formation(p):
            freq[p.offense][s] += 1
            freq[p.offense][(s // H) * H + (H - 1 - s % H)] += 1

    protos = data[args.split]
    episodes = rollout(model, protos, device)
    result = {}
    for head, flag in (("defense", False), ("offense", True)):
        rows = {"model": [], "heuristic": [], "frequent": []}
        for p, e in zip(protos, episodes):
            if p.offense != flag:
                continue
            human = human_formation(p)
            rows["model"].append(compare(e.formation(), human))
            if p.heuristic is not None:
                rows["heuristic"].append(compare(heuristic_formation(p), human))
            top = [(-1, s) for s, _ in freq[flag].most_common(len(human))]
            rows["frequent"].append(compare(top, human))
        result[head] = {k: summarise(v) for k, v in rows.items() if v}

    print(f"{args.split} split, checkpoint epoch {ck['epoch']}")
    for head, by in result.items():
        print(f"\n{head}  (n={by['model']['n']})")
        print(f"  {'':10s} {'exact':>7s} {'same class+square':>18s} {'same square':>12s} {'displacement':>13s}")
        for name, m in by.items():
            cls = "      n/a" if name == "frequent" else f"{m['class_overlap']:9.3f}"
            print(f"  {name:10s} {m['exact']:7.3f} {cls:>18s} {m['shape_overlap']:12.3f} {m['displacement']:13.2f}")
    if args.out:
        with open(args.out, "w") as f:
            json.dump(result, f, indent=1)


if __name__ == "__main__":
    main()
