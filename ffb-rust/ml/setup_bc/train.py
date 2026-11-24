"""Train the setup policy by behavioural cloning.

    python train.py --data data/setups.jsonl --out runs/v1

Each epoch every human formation is replayed once along a freshly drawn human-consistent order
(teacher forcing); at every step the loss is -log of the probability mass the policy puts on the
actions that agree with that formation (features.Episode.consistent_mask).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import time

import numpy as np
import torch

from features import build_proto, stack, subset_steps, teacher_steps
from model import SetupPolicy, set_loss, to_torch
from rollout import evaluate_rollouts


def bucket(coach: str) -> str:
    h = int(hashlib.sha1(coach.encode()).hexdigest(), 16) % 1000
    return "test" if h < 75 else ("val" if h < 150 else "train")


def load_coaches(data_path: str) -> dict:
    """data/coaches.json (written by coaches.py), next to the dataset."""
    path = os.path.join(os.path.dirname(os.path.abspath(data_path)), "coaches.json")
    if not os.path.exists(path):
        raise SystemExit(f"{path} is missing: run coaches.py first")
    with open(path) as f:
        return json.load(f)


def split_of(rec: dict, coaches: dict) -> str | None:
    """Deterministic split by COACH (a formation is a coach's habit; a coach has several teams and
    every team several games). A formation belongs to the split of the coach who set it up.
    Held-out coaches never reach training in any role: a training formation whose opponent is a
    held-out coach is dropped (None), because that opponent's formation or bench is its input.
    Dropped as well: val against test, so the two held-out splits share no game."""
    c = coaches[rec["replay_id"]]
    me, them = ("home", "away") if rec["setting_home"] else ("away", "home")
    a, b = bucket(c[me]["coach"]), bucket(c[them]["coach"])
    if a == "train":
        return "train" if b == "train" else None
    return a if b in ("train", a) else None


def read(path: str, limit: int = 0):
    """Yields the records one at a time (the parsed file is several GB as Python objects)."""
    with open(path, encoding="utf-8") as f:
        for n, line in enumerate(f):
            if limit and n >= limit:
                break
            yield json.loads(line)


LADDER = ["Rookie", "Experienced", "Veteran", "Emerging Star", "Star", "Super Star", "Mega-Star", "Legend"]


def load_ratings(data_path: str) -> dict:
    """coach hash -> FUMBBL rank on a 0..1 scale (Rookie 0 ... Legend 1); data/coach_ratings.json."""
    path = os.path.join(os.path.dirname(os.path.abspath(data_path)), "coach_ratings.json")
    with open(path) as f:
        raw = json.load(f)
    return {h: LADDER.index(v["rating"]) / (len(LADDER) - 1) for h, v in raw.items() if v["rating"] in LADDER}


def load_protos(path: str, limit: int = 0, vocab=None, z_source: str = "outcome"):
    """Two streaming passes: the skill vocabulary from the training split (unless given), then
    the protos per split. Returns (vocab, {split: protos}, {split: drop reasons}).
    `z_source`: "outcome" = how the game ended for the team setting up; "rating" = the rank of
    the coach setting up (formations of coaches without a known rank are dropped)."""
    coaches = load_coaches(path)
    ratings = load_ratings(path) if z_source == "rating" else None
    outcomes = {}
    opath = os.path.join(os.path.dirname(os.path.abspath(path)), "outcomes.json")
    if os.path.exists(opath):
        from outcomes import z_of
        with open(opath) as f:
            outcomes = json.load(f)
    if vocab is None:
        names = set()
        for r in read(path, limit):
            if split_of(r, coaches) == "train":
                names.update(s for p in r["players"] for s in p["skills"])
        vocab = {n: i + 2 for i, n in enumerate(sorted(names))}
    data = {"train": [], "val": [], "test": []}
    dropped = {"train": {}, "val": {}, "test": {}}
    for r in read(path, limit):
        s = split_of(r, coaches)
        if s is None:
            dropped["train"]["straddles the split"] = dropped["train"].get("straddles the split", 0) + 1
            continue
        rating = None
        if ratings is not None:
            rating = ratings.get(coaches[r["replay_id"]]["home" if r["setting_home"] else "away"]["coach"])
            if rating is None:
                dropped[s]["coach rank unknown"] = dropped[s].get("coach rank unknown", 0) + 1
                continue
        p, why = build_proto(r, vocab)
        if p is None:
            why = "fielded too few" if why.startswith("fielded") else why
            dropped[s][why] = dropped[s].get(why, 0) + 1
        else:
            p.heuristic = r.get("heuristic")
            if rating is not None:
                p.z = rating
            elif r["replay_id"] in outcomes:
                p.z = z_of(outcomes[r["replay_id"]], "home" if r["setting_home"] else "away")
            data[s].append(p)
    return vocab, data, dropped


def batches(protos, rng, size: int, augment: bool, random_subset: float = 0.0):
    """`random_subset`: the share of formations shown as random partial formations
    (features.subset_steps) instead of bench-order prefixes."""
    order = rng.permutation(len(protos))
    for lo in range(0, len(order), size):
        steps = []
        for i in order[lo:lo + size]:
            make = subset_steps if rng.random() < random_subset else teacher_steps
            steps += make(protos[i], rng, flip_y=augment and rng.random() < 0.5)
        yield stack(steps)


@torch.no_grad()
def evaluate(model, protos, device, seed: int = 123) -> dict[str, float]:
    """Teacher-forced loss and accuracy on fixed trajectories."""
    model.eval()
    rng = np.random.default_rng(seed)
    tot = {0: [0.0, 0.0, 0], 1: [0.0, 0.0, 0]}
    for b in batches(protos, rng, 128, augment=False):
        t = to_torch(b, device)
        logits = model(t)
        loss = set_loss(logits, t["target"])
        hit = t["target"].gather(1, logits.argmax(-1, keepdim=True))[:, 0].float()
        for h in (0, 1):
            m = t["head"] == h
            tot[h][0] += loss[m].sum().item()
            tot[h][1] += hit[m].sum().item()
            tot[h][2] += int(m.sum())
    n = tot[0][2] + tot[1][2]
    return {
        "nll": (tot[0][0] + tot[1][0]) / max(n, 1),
        "acc": (tot[0][1] + tot[1][1]) / max(n, 1),
        "nll_def": tot[0][0] / max(tot[0][2], 1), "acc_def": tot[0][1] / max(tot[0][2], 1),
        "nll_off": tot[1][0] / max(tot[1][2], 1), "acc_off": tot[1][1] / max(tot[1][2], 1),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default="data/setups.jsonl")
    ap.add_argument("--out", default="runs/v1")
    ap.add_argument("--epochs", type=int, default=40)
    ap.add_argument("--batch", type=int, default=48, help="formations per batch (~13 steps each)")
    ap.add_argument("--lr", type=float, default=2e-3)
    ap.add_argument("--wd", type=float, default=1e-4)
    ap.add_argument("--channels", type=int, default=64)
    ap.add_argument("--blocks", type=int, default=6)
    ap.add_argument("--dropout", type=float, default=0.1)
    ap.add_argument("--limit", type=int, default=0, help="read only the first N records")
    ap.add_argument("--overfit", type=int, default=0, help="sanity check: train on N formations only")
    ap.add_argument("--random-subset", type=float, default=0.0,
                    help="share of training formations shown as random partial formations")
    ap.add_argument("--outcome", action="store_true",
                    help="extra input z: how the game ended for the team setting up (needs data/outcomes.json)")
    ap.add_argument("--rating", action="store_true",
                    help="extra input z: FUMBBL rank of the coach setting up, 0..1 (needs data/coach_ratings.json)")
    ap.add_argument("--subset", default=None, choices=["winning", "losing", "ctrl_a", "ctrl_b"],
                    help="train on part of the training split only (winning = won or drew)")
    ap.add_argument("--resume", action="store_true", help="continue from <out>/last.pt")
    ap.add_argument("--max-minutes", type=float, default=0, help="stop cleanly after this long (then --resume)")
    ap.add_argument("--seed", type=int, default=1)
    args = ap.parse_args()

    torch.manual_seed(args.seed)
    rng = np.random.default_rng(args.seed)
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    os.makedirs(args.out, exist_ok=True)

    z_source = "rating" if args.rating else "outcome"
    vocab, data, dropped = load_protos(args.data, args.limit, z_source=z_source)
    for name in data:
        print(f"{name}: {len(data[name])} formations; dropped {dropped[name]}")
    if args.subset:
        # winning / losing: by how the game ended for the team setting up (data/outcomes.json).
        # ctrl_a / ctrl_b: a random partition of the same proportions (60 / 40) by (game, side),
        # the yardstick for how much two models differ just from seeing different data.
        def part(p):
            h = int(hashlib.sha1(f"{p.rec_id[0]}:{p.setting_home}".encode()).hexdigest(), 16) % 1000
            return "ctrl_a" if h < 600 else "ctrl_b"
        keep = {"winning": lambda p: p.z >= 0.5, "losing": lambda p: p.z < 0.5,
                "ctrl_a": lambda p: part(p) == "ctrl_a", "ctrl_b": lambda p: part(p) == "ctrl_b"}[args.subset]
        data["train"] = [p for p in data["train"] if keep(p)]
        print(f"subset {args.subset}: {len(data['train'])} training formations")
    if args.overfit:
        data["train"] = data["train"][:args.overfit]
        data["val"] = data["train"]

    hp = {"n_skills": len(vocab) + 2, "ch": args.channels, "blocks": args.blocks, "drop": args.dropout}
    if args.outcome or args.rating:
        hp["z"] = True
    model = SetupPolicy(**hp).to(device)
    print(f"device {device}, {sum(p.numel() for p in model.parameters()):,} parameters, {len(vocab)} skills")
    opt = torch.optim.AdamW(model.parameters(), lr=args.lr, weight_decay=args.wd)
    steps_per_epoch = math.ceil(len(data["train"]) / args.batch)
    sched = torch.optim.lr_scheduler.OneCycleLR(opt, args.lr, total_steps=args.epochs * steps_per_epoch, pct_start=0.1)

    best, log, first = float("inf"), [], 1
    last = os.path.join(args.out, "last.pt")
    if args.resume and os.path.exists(last):
        st = torch.load(last, map_location=device, weights_only=False)
        model.load_state_dict(st["model"])
        opt.load_state_dict(st["opt"])
        sched.load_state_dict(st["sched"])
        rng.bit_generator.state = st["rng"]
        best, log, first = st["best"], st["log"], st["epoch"] + 1
        print(f"resumed after epoch {st['epoch']} (best val {best:.4f})")
    started = time.time()
    for epoch in range(first, args.epochs + 1):
        if args.max_minutes and (time.time() - started) / 60 > args.max_minutes:
            print(f"stopping before epoch {epoch}: --max-minutes reached; rerun with --resume")
            return
        model.train()
        t0, run, n = time.time(), 0.0, 0
        for b in batches(data["train"], rng, args.batch, augment=not args.overfit, random_subset=args.random_subset):
            t = to_torch(b, device)
            loss = set_loss(model(t), t["target"]).mean()
            opt.zero_grad(set_to_none=True)
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            opt.step()
            sched.step()
            run += loss.item() * len(b["cur"])
            n += len(b["cur"])
        val = evaluate(model, data["val"], device)
        row = {"epoch": epoch, "train_nll": run / n, **{f"val_{k}": v for k, v in val.items()}, "sec": time.time() - t0}
        log.append(row)
        print(f"epoch {epoch:3d}  train {row['train_nll']:.4f}  val {val['nll']:.4f}  acc {val['acc']:.3f}"
              f"  (def {val['acc_def']:.3f} off {val['acc_off']:.3f})  {row['sec']:.0f}s", flush=True)
        if val["nll"] < best:
            best = val["nll"]
            torch.save({"model": model.state_dict(), "hp": hp, "vocab": vocab, "epoch": epoch, "val": val,
                        "split": "coach", "z_source": z_source},
                       os.path.join(args.out, "best.pt"))
        torch.save({"model": model.state_dict(), "opt": opt.state_dict(), "sched": sched.state_dict(),
                    "rng": rng.bit_generator.state, "best": best, "log": log, "epoch": epoch}, last)

    ck = torch.load(os.path.join(args.out, "best.pt"), map_location=device, weights_only=False)
    model.load_state_dict(ck["model"])
    report = {"best_epoch": ck["epoch"], "log": log}
    for name in ("val", "test"):
        report[name] = {"teacher_forced": evaluate(model, data[name], device),
                        "rollout": evaluate_rollouts(model, data[name], device)}
        print(name, json.dumps(report[name], indent=1))
    with open(os.path.join(args.out, "report.json"), "w") as f:
        json.dump(report, f, indent=1)


if __name__ == "__main__":
    main()
