"""Watch the trained policy set up two teams.

    python watch.py --home human --away orc --edition bb2025 --ckpt runs/v1/best.pt --out watch.html

The kicking team (--kicker, default away) sets up first with the defense head; the receiving team
then sets up against that formation with the offense head. The page steps through every placement
and shows the policy's probability for each square. Besides the greedy formation it holds a few
sampled ones (--samples).
"""
from __future__ import annotations

import argparse
import copy
import json
import os
import subprocess
import sys

import torch

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))          # the cargo workspace
sys.path.insert(0, os.path.join(ROOT, "scripts"))

from features import A_SKIP, N_SQ, W, build_proto, xy  # noqa: E402
from model import SetupPolicy  # noqa: E402
from rollout import rollout  # noqa: E402


def scenario(home: str, away: str, edition: str, exe: str) -> dict:
    out = subprocess.run([exe, "--setup-scenario", "--home", home, "--away", away, "--edition", edition],
                         cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return json.loads(out.strip().splitlines()[-1])


def set_up(model, vocab, rec: dict, side: str, device, temperature: float, seed: int, z: float = 0.5):
    """Rolls out one side's setup. Returns (steps for the page, the record with that side placed).
    `z` only matters to an outcome-conditioned model (1 = set up like a team that wins)."""
    proto, _ = build_proto(rec, vocab, side=side, check=False)
    proto.z = z
    episodes, traces = rollout(model, [proto], device, temperature, seed, keep_probs=True)
    real = (lambda x: x) if side == "home" else (lambda x: W - 1 - x)
    rec = copy.deepcopy(rec)
    by_id = {p["id"]: p for p in rec["players"]}
    steps = []
    for row, action, probs in traces[0]:
        heat = [[real(xy(s)[0]), xy(s)[1], round(float(probs[s]), 4)] for s in range(N_SQ) if probs[s] >= 0.002]
        step = {"side": side, "id": proto.ids[row], "heat": heat, "skip": round(float(probs[A_SKIP]), 4)}
        if action != A_SKIP:
            x, y = xy(action)
            step["to"] = [real(x), y]
            step["p"] = round(float(probs[action]), 4)
            by_id[proto.ids[row]].update(x=real(x), y=y, state="STANDING")
        steps.append(step)
    return steps, rec


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--home", required=True)
    ap.add_argument("--away", required=True)
    ap.add_argument("--edition", default="bb2025")
    ap.add_argument("--kicker", choices=["home", "away"], default="away")
    ap.add_argument("--ckpt", default=os.path.join(HERE, "runs", "v1", "best.pt"))
    ap.add_argument("--out", default=None)
    ap.add_argument("--samples", type=int, default=4, help="sampled formations besides the greedy one")
    ap.add_argument("--temperature", type=float, default=1.0)
    ap.add_argument("--exe", default=os.path.join(ROOT, "target", "release", "ffb-parity.exe"))
    args = ap.parse_args()

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    ck = torch.load(args.ckpt, map_location=device)
    model = SetupPolicy(**ck["hp"]).to(device)
    model.load_state_dict(ck["model"])

    rec = scenario(args.home, args.away, args.edition, args.exe)
    receiver = "home" if args.kicker == "away" else "away"
    variants = []
    for k in range(args.samples + 1):
        temp = 0.0 if k == 0 else args.temperature
        d_steps, placed = set_up(model, ck["vocab"], rec, args.kicker, device, temp, seed=k)
        o_steps, _ = set_up(model, ck["vocab"], placed, receiver, device, temp, seed=1000 + k)
        variants.append({"name": "most likely (greedy)" if k == 0 else f"sample {k}", "steps": d_steps + o_steps})

    from pack_decision_replay import icon_strip, pitch_uri
    pitch, (pw, ph) = pitch_uri()
    icons = {}
    for pos in sorted({p["position_id"] for p in rec["players"]}):
        uri, size, _ = icon_strip(pos)
        if uri:
            icons[pos] = {"d": uri, "s": size}
    payload = {
        "pitch": pitch, "pw": pw, "ph": ph, "icons": icons, "variants": variants,
        "players": rec["players"], "kicker": args.kicker, "edition": args.edition,
        "teams": {"home": f"{rec['home']['race']} ({args.home})", "away": f"{rec['away']['race']} ({args.away})"},
        "ckpt": f"epoch {ck['epoch']}, val nll {ck['val']['nll']:.3f}",
    }
    out = args.out or os.path.join(HERE, "runs", f"watch_{args.home}_vs_{args.away}_{args.edition}.html")
    with open(os.path.join(HERE, "watch_template.html"), encoding="utf-8") as f:
        html = f.read().replace("/*PAYLOAD*/null", json.dumps(payload, separators=(",", ":")))
    with open(out, "w", encoding="utf-8") as f:
        f.write(html)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
