"""Show the policy's formations on real teams from a held-out split, next to what the coaches did.

    python examples.py --ckpt runs/v3_subset50/best.pt --split val --n 12

For every picked kick-off the page holds two pitches drawn with the client's graphics: the model's
(kicking team by the defense head, then the receiving team by the offense head against the model's
own kicking formation) and the human one. Player names, position names and icons come from the
replay file itself (--replays); hovering or clicking a player shows its stat line and skills.
"""
from __future__ import annotations

import argparse
import base64
import copy
import difflib
import glob
import gzip
import io
import json
import os
import sys

import numpy as np
import torch

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(ROOT, "scripts"))

from features import H, W, build_proto, class_key, sq  # noqa: E402
from model import SetupPolicy  # noqa: E402
from rollout import compare  # noqa: E402
from train import bucket, load_coaches, read, split_of  # noqa: E402
from watch import set_up  # noqa: E402


def norm(s: str) -> str:
    return "".join(c for c in s.lower() if c.isalnum())


class Icons:
    """The client's cached icon sheets, found by race and position name."""

    def __init__(self, folder: str):
        self.folder = folder
        self.stems = [f[:-4] for f in os.listdir(folder) if f.endswith(".png")]
        self.by_race: dict[str, list[str]] = {}
        for s in self.stems:
            if "_" in s:
                self.by_race.setdefault(s.split("_", 1)[0].lower(), []).append(s)
        self.stars = {norm(s): s for s in self.stems if "_" not in s}
        self.cache: dict[str, dict] = {}

    def find(self, race: str, position: str) -> str | None:
        pos = norm(position)
        if pos in self.stars:
            return self.stars[pos]
        stems = self.by_race.get(norm(race), [])
        if not stems:
            return None
        subs = {s.split("_", 1)[1].lower(): s for s in stems}
        if pos in subs:
            return subs[pos]
        near = difflib.get_close_matches(pos, list(subs), n=1, cutoff=0.5)
        if near:
            return subs[near[0]]
        part = [k for k in subs if k in pos or pos in k]
        if part:
            return subs[part[0]]
        line = [k for k in subs if "line" in k]
        return subs[line[0]] if line else None

    def strip(self, stem: str) -> dict:
        """Row 0 of the sheet: [home, home-moving, away, away-moving]."""
        if stem not in self.cache:
            from PIL import Image
            sheet = Image.open(os.path.join(self.folder, stem + ".png")).convert("RGBA")
            size = sheet.width // 4
            buf = io.BytesIO()
            sheet.crop((0, 0, 4 * size, size)).save(buf, "PNG", optimize=True)
            self.cache[stem] = {"d": "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode(), "s": size}
        return self.cache[stem]


def replay_names(folder: str, replay_id: str) -> dict:
    """player id -> (player name, position name), position id -> position name, from the replay."""
    files = glob.glob(os.path.join(folder, "**", f"{replay_id}.ffbr"), recursive=True)
    if not files:
        return {"players": {}, "positions": {}, "teams": {}}
    with gzip.open(files[0], "rt", encoding="utf-8") as f:
        doc = json.load(f)
    game = next(c["game"] for c in doc["commandArray"] if isinstance(c, dict) and c.get("game"))
    out = {"players": {}, "positions": {}, "teams": {}}
    for side, key in (("home", "teamHome"), ("away", "teamAway")):
        team = game[key]
        out["teams"][side] = team.get("teamName") or ""
        for pos in team["roster"]["positionArray"]:
            out["positions"][pos["positionId"]] = pos.get("displayName") or pos["positionName"]
        for p in team["playerArray"]:
            out["players"][p["playerId"]] = p.get("playerName") or ""
    return out


def clear(rec: dict, side: str) -> None:
    for p in rec["players"]:
        if p["side"] == side and p["x"] is not None:
            p.update(x=None, y=None, state="RESERVE")


def agreement(rec_model: dict, rec_human: dict, side: str) -> dict:
    """compare() of one side's two formations, in that side's canonical frame."""
    cx = (lambda x: x) if side == "home" else (lambda x: W - 1 - x)
    ids: dict[tuple, int] = {}

    def formation(rec):
        return sorted((ids.setdefault(class_key(p), len(ids)), sq(cx(p["x"]), p["y"]))
                      for p in rec["players"] if p["side"] == side and p["x"] is not None)
    m, h = formation(rec_model), formation(rec_human)
    if not m or len(m) != len(h):
        return {}
    r = compare(m, h)
    return {"n": len(h), "same": round(r["class_overlap"] * len(h)), "squares": round(r["shape_overlap"] * len(h)),
            "disp": round(r["displacement"], 2)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=os.path.join(HERE, "data", "setups_h.jsonl"))
    ap.add_argument("--ckpt", default=os.path.join(HERE, "runs", "v4_coach_split", "best.pt"))
    ap.add_argument("--split", default="val", choices=["val", "test", "train"])
    ap.add_argument("--n", type=int, default=12, help="kick-offs to show")
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--replays", default="D:/ffb_replays")
    ap.add_argument("--out", default=None)
    ap.add_argument("--compare-z", action="store_true",
                    help="outcome-conditioned model: show z = 1 next to z = 0 instead of model next to human")
    ap.add_argument("--z-slider", action="store_true",
                    help="model with a z input: a slider on the page sets z (formations precomputed per value)")
    args = ap.parse_args()

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    ck = torch.load(args.ckpt, map_location=device, weights_only=False)
    model = SetupPolicy(**ck["hp"]).to(device)
    model.load_state_dict(ck["model"])
    vocab = ck["vocab"]
    zgrid, znames = None, None
    if args.z_slider:
        if ck.get("z_source") == "rating":
            from train import LADDER
            zgrid, znames = [i / (len(LADDER) - 1) for i in range(len(LADDER))], LADDER
        else:
            zgrid = [i / 10 for i in range(11)]
            znames = ["loses"] + [""] * 4 + ["draws"] + [""] * 4 + ["wins"]

    # kick-offs of the split that have both formations and pass the legality check
    # A formation is held out when the coach who set it up is in the split (train.split_of); the
    # other coach of that game may be a training coach (whose formations of this game are dropped
    # from training). Kick-offs where both coaches are held out come first.
    coaches = load_coaches(args.data)
    drives: dict[tuple, dict] = {}
    held: dict[tuple, int] = {}
    for r in read(args.data):
        if build_proto(r, vocab)[0] is None:
            continue
        key = (r["replay_id"], r["drive"])
        if split_of(r, coaches) == args.split:
            held[key] = held.get(key, 0) + 1
        r.pop("heuristic", None)
        drives.setdefault(key, {})[r["kind"]] = r
    keys = sorted(k for k, v in drives.items() if len(v) == 2 and k in held)
    rng = np.random.default_rng(args.seed)
    rng.shuffle(keys)
    keys.sort(key=lambda k: -held[k])                     # stable: both-held-out kick-offs first
    picked, seen = [], set()
    for k in keys:                                        # one kick-off per pairing of races first
        races = frozenset((drives[k]["offense"]["home"]["race"], drives[k]["offense"]["away"]["race"]))
        if races not in seen:
            seen.add(races)
            picked.append(k)
        if len(picked) == args.n:
            break

    from pack_decision_replay import JAVA, pitch_uri
    pitch, (pw, ph) = pitch_uri()
    icons = Icons(os.path.join(JAVA, "cached", "players", "iconsets"))
    used_icons, examples, missing = {}, [], set()
    for i, k in enumerate(picked):
        human = drives[k]["offense"]                      # both human formations are on this board
        kicker = "away" if human["setting_home"] else "home"
        receiver = "home" if kicker == "away" else "away"

        def run(z):
            base = copy.deepcopy(human)
            clear(base, "home")
            clear(base, "away")
            base["kind"] = "defense"
            d_steps, placed = set_up(model, vocab, base, kicker, device, 0.0, seed=i, z=z)
            placed["kind"] = "offense"
            o_steps, done = set_up(model, vocab, placed, receiver, device, 0.0, seed=i, z=z)
            return done, {s["id"]: s.get("p") for s in d_steps + o_steps if "to" in s}

        sweep = [run(z) for z in zgrid] if zgrid else []    # one formation per z, for the slider
        if zgrid:
            done, conf = sweep[len(zgrid) // 2]
            other = human
        elif args.compare_z:                              # left: z = 1 (wins), right: z = 0 (loses)
            done, conf = run(1.0)
            other, _ = run(0.0)
        else:
            done, conf = run(0.5)
            other = human

        names = replay_names(args.replays, k[0])
        players = []
        for j, (p, m, o) in enumerate(zip(human["players"], done["players"], other["players"])):
            pos = names["positions"].get(p["position_id"], p["position_id"])
            stem = icons.find(human[p["side"]]["race"], pos)
            if stem:
                used_icons[stem] = icons.strip(stem)
            else:
                missing.add((human[p["side"]]["race"], pos))
            players.append({
                "id": p["id"], "side": p["side"], "nr": p["nr"], "name": names["players"].get(p["id"], ""),
                "pos": pos, "icon": stem, "ma": p["ma"], "st": p["st"], "ag": p["ag"], "pa": p["pa"], "av": p["av"],
                "skills": p["skills"], "cost": p["cost"], "nig": p["niggling"], "inj": p["injuries"],
                "state": "RESERVE" if p["x"] is not None else p["state"],
                "h": None if o["x"] is None else [o["x"], o["y"]],
                "m": None if m["x"] is None else [m["x"], m["y"]],
                "conf": conf.get(p["id"]),
            })
            if zgrid:
                at = [d["players"][j] for d, _ in sweep]
                players[-1]["mz"] = [None if q["x"] is None else [q["x"], q["y"]] for q in at]
                players[-1]["confz"] = [c.get(p["id"]) for _, c in sweep]
        examples.append({
            "replay": k[0], "drive": k[1], "edition": human["edition"], "half": human["half"],
            "weather": human["weather"], "kicker": kicker,
            "unseen": {s: bucket(coaches[k[0]][s]["coach"]) != "train" for s in ("home", "away")},
            "teams": {s: {**{f: human[s][f] for f in ("race", "team_value", "rerolls", "apothecaries", "score", "turn")},
                          "name": names["teams"].get(s, "")} for s in ("home", "away")},
            "agree": {"kick": agreement(done, other, kicker), "recv": agreement(done, other, receiver)},
            "players": players,
        })
        if zgrid:
            examples[-1]["agreez"] = [{"kick": agreement(d, human, kicker), "recv": agreement(d, human, receiver)}
                                      for d, _ in sweep]
        print(f"{k[0]} drive {k[1]}: {human['home']['race']} v {human['away']['race']}  "
              f"kick {examples[-1]['agree']['kick']}  recv {examples[-1]['agree']['recv']}")
    if missing:
        print("no icon sheet for:", sorted(missing))

    payload = {"pitch": pitch, "pw": pw, "ph": ph, "icons": used_icons, "examples": examples, "split": args.split,
               "zmode": bool(args.compare_z) and not zgrid, "zgrid": zgrid, "znames": znames,
               "zlabels": ["set up like a Legend-ranked coach", "set up like a Rookie-ranked coach"]
               if ck.get("z_source") == "rating" else None,
               "ckpt": f"{os.path.basename(os.path.dirname(os.path.abspath(args.ckpt)))}, epoch {ck['epoch']}, "
                       f"val nll {ck['val']['nll']:.3f}"}
    out = args.out or os.path.join(HERE, "runs", f"examples_{args.split}{'_zslider' if zgrid else '_z' if args.compare_z else ''}.html")
    with open(os.path.join(HERE, "examples_template.html"), encoding="utf-8") as f:
        html = f.read().replace("/*PAYLOAD*/null", json.dumps(payload, separators=(",", ":")))
    with open(out, "w", encoding="utf-8") as f:
        f.write(html)
    print(f"wrote {out} ({os.path.getsize(out) // 1024} KB)")


if __name__ == "__main__":
    main()
