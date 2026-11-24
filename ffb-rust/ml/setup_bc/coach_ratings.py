"""FUMBBL coach rating for every coach in the dataset -> data/coach_ratings.json.

    python coach_ratings.py            # resumable; safe to stop and start again

Two API requests per coach (search by name -> id, then coach/get), one request every 11 seconds,
never in parallel: about 6.8 hours for 1,214 coaches. Coaches with the most formations go first,
so a partial file already covers most of the data. The output is keyed by the same coach hash as
data/coaches.json and holds the rating only; coach names are never written.
"""
from __future__ import annotations

import argparse
import collections
import glob
import gzip
import hashlib
import json
import os
import re
import time
import urllib.parse
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
TEAM = re.compile(r'"(teamAway|teamHome)"\s*:\s*\{\s*"teamId"\s*:\s*"[^"]*"\s*,\s*"teamName"\s*:\s*"(?:[^"\\]|\\.)*"\s*,'
                  r'\s*"coach"\s*:\s*"((?:[^"\\]|\\.)*)"')
GAP = 11.0
last = [0.0]


def api(path: str):
    wait = GAP - (time.time() - last[0])
    if wait > 0:
        time.sleep(wait)
    req = urllib.request.Request("https://fumbbl.com/api/" + path,
                                 headers={"User-Agent": "python-urllib", "Accept": "application/json"})
    try:
        return json.loads(urllib.request.urlopen(req, timeout=40).read().decode("utf-8", "replace"))
    finally:
        last[0] = time.time()


def names(path: str) -> list[str]:
    text, found = "", {}
    with gzip.open(path, "rt", encoding="utf-8") as f:
        while len(found) < 2 and len(text) < 8_000_000:
            chunk = f.read(262144)
            if not chunk:
                break
            text += chunk
            for m in TEAM.finditer(text):
                found.setdefault(m.group(1), json.loads('"' + m.group(2) + '"'))
    return list(found.values())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=os.path.join(HERE, "data", "setups_h.jsonl"))
    ap.add_argument("--replays", default="D:/ffb_replays")
    ap.add_argument("--limit", type=int, default=0, help="stop after this many coaches (0 = all)")
    args = ap.parse_args()
    out = os.path.join(os.path.dirname(os.path.abspath(args.data)), "coach_ratings.json")
    done = {}
    if os.path.exists(out):
        with open(out) as f:
            done = json.load(f)

    weight = collections.Counter()                         # formations per replay
    with open(args.data, encoding="utf-8") as f:
        for line in f:
            weight[re.search(r'"replay_id":\s*"([^"]+)"', line).group(1)] += 1
    files = {os.path.basename(p)[:-5]: p for p in glob.glob(os.path.join(args.replays, "**", "*.ffbr"), recursive=True)}
    coach = {}                                             # hash -> name (memory only)
    count = collections.Counter()
    for rid, n in weight.items():
        for name in names(files[rid]):
            h = hashlib.sha1(name.encode()).hexdigest()[:16]
            coach[h] = name
            count[h] += n
    todo = [h for h, _ in count.most_common() if h not in done]
    print(f"{len(coach)} coaches, {len(done)} already fetched, {len(todo)} to go", flush=True)

    errors = 0
    for k, h in enumerate(todo):
        if args.limit and k >= args.limit:
            break
        name = coach[h]
        try:
            hits = api("coach/search/" + urllib.parse.quote(name, safe=""))
            cid = next((x["id"] for x in hits if x["name"].lower() == name.lower()), None)
            rating = api(f"coach/get/{cid}").get("rating") if cid is not None else None
            done[h] = {"rating": rating}
            errors = 0
        except Exception as e:  # noqa: BLE001
            errors += 1
            print(f"error on coach {k}: {type(e).__name__} {getattr(e, 'code', '')}", flush=True)
            if errors >= 5:
                print("five errors in a row, stopping", flush=True)
                break
            continue
        with open(out + ".tmp", "w") as f:
            json.dump(done, f)
        os.replace(out + ".tmp", out)
        if (k + 1) % 25 == 0:
            print(f"{k + 1}/{len(todo)} fetched; ratings so far: "
                  f"{dict(collections.Counter(str(v['rating']) for v in done.values()))}", flush=True)
    print(f"done: {len(done)} coaches in {out}", flush=True)


if __name__ == "__main__":
    main()
