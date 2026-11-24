"""BACKLOG §P P2 gate: the game a client holds, Java vs Rust, at every parity step.

Both harnesses dump `Game.toJsonValue()` before each recorded step when FFB_CLIENT_VIEW_DIR is
set (ParityRunner.recordStep / ffb-parity runner.rs): `seed_<n>_java_view.jsonl` and
`seed_<n>_rust_view.jsonl`, one `{"i": step, "game": {...}}` per line. The parity state hash
already proves the two games play identically; this compares everything a client is sent.

The engines name players and teams differently (Java `teamHumanParity25Home4`, Rust
`home_04`), so Java ids are mapped to Rust ids by team side and shirt number (`playerNr`;
raised players `<dead>R<n>` follow their base id) before comparing. Lists of objects keyed
by `playerId` are compared as maps. Paths that are not game state -- names and the roster/team
data the two harnesses load from different files -- are listed in IGNORED with the reason.

Usage: python -I scripts/client_view_check.py <view dir> [<view dir> ...] [--examples N]
Prints, per differing path pattern, the number of steps it differs in and examples; the last
line is `CLIENT VIEW: <steps> steps, <identical> identical` (identical == steps is the gate).
"""
import json
import os
import re
import sys
from collections import defaultdict

# path patterns (fnmatch-like: '*' = one segment) that are not client-visible game state.
IGNORED = {
}


def load(path):
    out = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                o = json.loads(line)
                out[o["i"]] = o["game"]
    return out


def id_map(java, rust):
    m = {}
    for side in ("teamHome", "teamAway"):
        jt, rt = java.get(side) or {}, rust.get(side) or {}
        if jt.get("teamId") and rt.get("teamId"):
            m[jt["teamId"]] = rt["teamId"]
        rust_by_nr = {}
        for p in rt.get("playerArray") or []:
            rust_by_nr.setdefault(p.get("playerNr"), p.get("playerId"))
        for p in jt.get("playerArray") or []:
            rid = rust_by_nr.get(p.get("playerNr"))
            if rid and p.get("playerId"):
                m[p["playerId"]] = rid
    # raised players: <java dead id>R<n> -> <rust dead id>R<n>
    for side in ("teamHome", "teamAway"):
        for p in (java.get(side) or {}).get("playerArray") or []:
            pid = p.get("playerId") or ""
            mm = re.match(r"^(.*)(R\d+)$", pid)
            if pid not in m and mm and mm.group(1) in m:
                m[pid] = m[mm.group(1)] + mm.group(2)
    return m


def remap(v, m):
    if isinstance(v, str):
        return m.get(v, v)
    if isinstance(v, list):
        return [remap(x, m) for x in v]
    if isinstance(v, dict):
        return {m.get(k, k): remap(x, m) for k, x in v.items()}
    return v


def keyed(v):
    """A list of objects that all carry a playerId -> a dict by playerId."""
    if isinstance(v, list) and v and all(isinstance(x, dict) and "playerId" in x for x in v):
        return {x["playerId"]: x for x in v}
    return v


def walk(j, r, path, out):
    j, r = keyed(j), keyed(r)
    if isinstance(j, dict) and isinstance(r, dict):
        for k in sorted(set(j) | set(r)):
            walk(j.get(k, "<absent>"), r.get(k, "<absent>"), path + [k], out)
        return
    if isinstance(j, list) and isinstance(r, list) and len(j) == len(r) and not all(
            isinstance(x, (str, int, float, bool)) or x is None for x in j + r):
        for n, (a, b) in enumerate(zip(j, r)):
            walk(a, b, path + [str(n)], out)
        return
    if isinstance(j, list) and isinstance(r, list) and sorted(map(json.dumps, j)) == sorted(map(json.dumps, r)):
        return  # same members, order only (sets serialized from Java/Rust hash sets)
    if j != r:
        out.append((path, j, r))


def pattern(path, ids):
    return ".".join("<p>" if p in ids else ("<n>" if p.isdigit() else p) for p in path)


def ignored(pat):
    return any(re.fullmatch(k.replace(".", r"\.").replace("*", r"[^.]+"), pat) for k in IGNORED)


def main(argv):
    examples = 2
    dirs = []
    i = 0
    while i < len(argv):
        if argv[i] == "--examples":
            examples = int(argv[i + 1])
            i += 2
        else:
            dirs.append(argv[i])
            i += 1
    stats = defaultdict(lambda: [0, []])
    steps = identical = 0
    for d in dirs:
        for name in sorted(os.listdir(d)):
            mm = re.match(r"seed_(\d+)_java_view\.jsonl$", name)
            if not mm:
                continue
            rust_path = os.path.join(d, f"seed_{mm.group(1)}_rust_view.jsonl")
            if not os.path.exists(rust_path):
                continue
            java, rust = load(os.path.join(d, name)), load(rust_path)
            for idx in sorted(set(java) & set(rust)):
                m = id_map(java[idx], rust[idx])
                rust_ids = set(m.values())
                diffs = []
                walk(remap(java[idx], m), rust[idx], [], diffs)
                seen = set()
                for path, jv, rv in diffs:
                    pat = pattern(path, rust_ids)
                    if ignored(pat) or pat in seen:
                        continue
                    seen.add(pat)
                    s = stats[pat]
                    s[0] += 1
                    if len(s[1]) < examples:
                        s[1].append((f"{os.path.basename(d)} seed {mm.group(1)} i={idx}", jv, rv))
                steps += 1
                identical += not seen
    for pat, (n, ex) in sorted(stats.items(), key=lambda kv: -kv[1][0]):
        print(f"{n:6d}  {pat}")
        for where, jv, rv in ex:
            print(f"          {where}: java={json.dumps(jv)[:120]} rust={json.dumps(rv)[:120]}")
    print(f"CLIENT VIEW: {steps} steps, {identical} identical")


if __name__ == "__main__":
    main(sys.argv[1:])
