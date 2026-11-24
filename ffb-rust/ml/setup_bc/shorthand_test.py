"""Counterfactual: does the policy re-plan when a player is missing, or play its 11-player
formation with a hole?

    python shorthand_test.py --ckpt runs/v4_coach_split/best.pt

Takes held-out teams with exactly 11 available players, knocks one out (the cheapest, or a random
one), and compares the model's 10-player formation with its own 11-player formation for the same
team in the same situation.
"""
from __future__ import annotations

import argparse
import copy

import numpy as np
import torch

from model import SetupPolicy
from rollout import rollout
from symmetry import sym
from train import load_protos

AVAIL, KO = 20, 21      # own team variables inside Proto.globals (features._team_vars)


def without(proto, j: int):
    """The proto with bench position j knocked out."""
    p = copy.copy(proto)
    p.bench = proto.bench[:j] + proto.bench[j + 1:]
    p.classes = proto.classes[:j] + proto.classes[j + 1:]
    p.n_field = len(p.bench)
    p.target = {}
    p.globals = proto.globals.copy()
    p.globals[AVAIL] -= 1 / 16.0
    p.globals[KO] += 1 / 5.0
    return p


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default="data/setups_h.jsonl")
    ap.add_argument("--ckpt", default="runs/v4_coach_split/best.pt")
    args = ap.parse_args()
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    ck = torch.load(args.ckpt, map_location=device, weights_only=False)
    model = SetupPolicy(**ck["hp"]).to(device)
    model.load_state_dict(ck["model"])
    _, data, _ = load_protos(args.data, vocab=ck["vocab"])
    full = [p for p in data["val"] + data["test"] if len(p.bench) == 11]
    rng = np.random.default_rng(0)
    base = rollout(model, full, device)

    for label, pick in (("cheapest player out", lambda p: 10), ("random player out", lambda p: int(rng.integers(11)))):
        js = [pick(p) for p in full]
        short = rollout(model, [without(p, j) for p, j in zip(full, js)], device)
        for head, flag in (("defense", False), ("offense", True)):
            rows = []
            for p, j, e11, e10 in zip(full, js, base, short):
                if p.offense != flag:
                    continue
                s11 = {s for _, s in e11.formation()}
                s10 = {s for _, s in e10.formation()}
                gone = dict(e11.placed).get(p.bench[j])            # where the 11-formation had that player
                a, b = sym(sorted(s11)), sym(sorted(s10))
                rows.append({
                    "same 10 squares": len(s10 & s11) == 10,
                    "hole exactly where the missing player stood": s10 == s11 - {gone},
                    "11 symmetric": a["unpaired"] == 0,
                    "10 symmetric": b["unpaired"] == 0,
                    "10 has one unpaired player": b["unpaired"] == 1,
                    "broke a symmetric 11": a["unpaired"] == 0 and b["unpaired"] > 0,
                    "squares moved": 10 - len(s10 & s11),
                })
            print(f"\n{label}, {head} (n={len(rows)})")
            for k in rows[0]:
                v = np.mean([r[k] for r in rows])
                print(f"  {k:46s} {v:6.2f}" if k == "squares moved" else f"  {k:46s} {v:6.1%}")


if __name__ == "__main__":
    main()
