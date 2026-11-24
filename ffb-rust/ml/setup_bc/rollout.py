"""Running the policy in its own loop, and comparing the result with a human formation."""
from __future__ import annotations

import numpy as np
import torch

from features import Episode, H, human_formation, stack, xy
from model import to_torch


@torch.no_grad()
def rollout(model, protos, device, temperature: float = 0.0, seed: int = 0, keep_probs: bool = False,
            batch: int = 512):
    """Sets up every proto with the model. temperature 0 = greedy, otherwise sampled.

    Returns the finished Episodes; with `keep_probs` also, per episode, the list of
    (table row, action, probabilities over the actions) of every step."""
    model.eval()
    gen = torch.Generator(device="cpu").manual_seed(seed)
    episodes = [Episode(p) for p in protos]
    trace = [[] for _ in protos]
    live = [i for i, e in enumerate(episodes) if not e.done]
    while live:
        for lo in range(0, len(live), batch):
            idx = live[lo:lo + batch]
            obs = stack([episodes[i].obs() for i in idx])
            probs = torch.softmax(model(to_torch(obs, device)).float(), -1).cpu()
            if temperature <= 0:
                acts = probs.argmax(-1)
            else:
                scaled = torch.softmax(torch.log(probs.clamp_min(1e-30)) / temperature, -1)
                acts = torch.multinomial(scaled, 1, generator=gen)[:, 0]
            for k, i in enumerate(idx):
                e = episodes[i]
                if keep_probs:
                    trace[i].append((e.cur_row, int(acts[k]), probs[k].numpy()))
                e.step(int(acts[k]))
        live = [i for i in live if not episodes[i].done]
    return (episodes, trace) if keep_probs else episodes


def _displacement(a: list[int], b: list[int]) -> float:
    """Mean king-move distance of the best one-to-one matching of two square sets."""
    n = min(len(a), len(b))
    if n == 0:
        return 0.0
    cost = np.array([[max(abs(xy(s)[0] - xy(t)[0]), abs(xy(s)[1] - xy(t)[1])) for t in b] for s in a], float)
    try:
        from scipy.optimize import linear_sum_assignment
        r, c = linear_sum_assignment(cost)
        return float(cost[r, c].sum() / n)
    except ImportError:  # greedy fallback: an upper bound on the optimum
        total, cost = 0.0, cost.copy()
        for _ in range(n):
            i, j = np.unravel_index(np.argmin(cost), cost.shape)
            total += cost[i, j]
            cost[i, :], cost[:, j] = np.inf, np.inf
        return total / n


def _mirror(formation):
    return sorted((c, (s // H) * H + (H - 1 - s % H)) for c, s in formation)


def compare(model_formation, human) -> dict[str, float]:
    """Formation difference, players of one class interchangeable. A formation and its top/bottom
    mirror image are the same formation, so the closer of the two is scored."""
    best = None
    for cand in (model_formation, _mirror(model_formation)):
        n = max(len(human), 1)
        common = len(set(cand) & set(human))
        shape = len({s for _, s in cand} & {s for _, s in human})
        m = {
            "exact": float(sorted(cand) == sorted(human)),
            "class_overlap": common / n,            # same class of player on the same square
            "shape_overlap": shape / n,             # some player on the same square
            "displacement": _displacement([s for _, s in cand], [s for _, s in human]),
        }
        if best is None or (m["class_overlap"], -m["displacement"]) > (best["class_overlap"], -best["displacement"]):
            best = m
    return best


def evaluate_rollouts(model, protos, device) -> dict[str, dict[str, float]]:
    """Greedy rollouts against the human formations, split by head."""
    episodes = rollout(model, protos, device)
    out = {}
    for name, flag in (("defense", False), ("offense", True)):
        rows = [compare(e.formation(), human_formation(p)) for e, p in zip(episodes, protos) if p.offense == flag]
        if rows:
            out[name] = {k: float(np.mean([r[k] for r in rows])) for k in rows[0]}
            out[name]["n"] = len(rows)
    return out
