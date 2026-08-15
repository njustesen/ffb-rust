"""Field-by-field diff of the Java and Rust state strings at one step index.

Both engines print a `state=` blob under FFB_TRACE=1 — Java as `JSTEP i=N ... state=...`,
Rust as `RUST_STEP i=N ... state=...`. The blob is `h<half>t<turnH><turnA>a<active>s<h>,<a>`
followed by ` b<x>,<y>,<inplay> p` and then `|`-separated `id:x,y,State` entries.

This prints only the parts that DIFFER, which is usually one player.

Usage: python scripts/statediff.py <trace-file> <step-index>
"""
import re
import sys


def grab(path, prefix, want):
    # NOTE: the state blob CONTAINS SPACES (`h1t67aaways0,0 b25,8,true p<players>`), so this must
    # capture to end of line. `(\S+)` silently captured only the header and made every step look
    # identical.
    pat = re.compile(rf"^{prefix} i={want} .*?state=(.*)$")
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            m = pat.match(line)
            if m:
                return m.group(1)
    return None


def split_state(s):
    """Return (header, {player_id: 'x,y,State'})."""
    if s is None:
        return None, {}
    # players start at the first '|'-joined run of `id:` entries after ' p'
    idx = s.find(" p")
    head, rest = (s[:idx], s[idx + 2:]) if idx >= 0 else (s, "")
    players = {}
    for part in rest.split("|"):
        if ":" in part:
            pid, val = part.split(":", 1)
            players[pid] = val
    return head, players


def fnv1a64(data: bytes) -> str:
    """Same hash the parity log uses (ffb_model::util::state_hash)."""
    h = 0xcbf29ce484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def main() -> int:
    path, want = sys.argv[1], sys.argv[2]
    j = grab(path, "JSTEP", want)
    r = grab(path, "RUST_STEP", want)
    if j is not None:
        print(f"JAVA state hashes to {fnv1a64(j.encode())}")
    if r is not None:
        print(f"RUST state hashes to {fnv1a64(r.encode())}")
    if j is None or r is None:
        print(f"missing state for i={want}: java={j is not None} rust={r is not None}")
        return 1
    if j == r:
        print(f"i={want}: state strings are IDENTICAL")
        return 0

    jh, jp = split_state(j)
    rh, rp = split_state(r)
    if jh != rh:
        print(f"header differs:\n  JAVA {jh}\n  RUST {rh}")
    for pid in sorted(set(jp) | set(rp)):
        a, b = jp.get(pid), rp.get(pid)
        if a != b:
            print(f"  {pid}:  JAVA {a}   RUST {b}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
