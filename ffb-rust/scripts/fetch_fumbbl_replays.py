#!/usr/bin/env python3
"""Fetch a small, polite batch of FUMBBL replays for the replay-parity tier (BACKLOG §N).

Drives the harness's `FumbblReplayDownloader` (ffb-ai, ws://fumbbl.com:22223 -- the WebSocket the
FFB client itself uses; fumbbl.com's HTTP pages sit behind a bot filter, this does not) straight
from a jar with `java -cp`, never through Maven, so it can never rebuild or lock a cert jar.

Rules of the road (user, 2026-11-21): **wait >= 10 s between every contact with fumbbl.com**
(downloads, probes and filtered attempts alike), one connection at a time, never parallel.

Replays hold FUMBBL coach and team names, so they stay OUT of git: default output is
D:/ffb_replays/fumbbl/<id>.ffbr plus manifest.jsonl (one metadata line per kept replay).

Usage:
  python -I scripts/fetch_fumbbl_replays.py --count 10 [--want BB2020=5,BB2025=5]
         [--start-id N | --find-max-from 1832851] [--out D:/ffb_replays/fumbbl]
         [--jar D:/ffb_replays/ffb-ai-replay.jar] [--delay 10] [--max-attempts 200]

Walks replay ids DOWNWARD from the start id (newest first: the closest to the in-repo engine).
Each id is downloaded with a --rules filter for the ruleset that still has the largest deficit;
a filtered id (exit 2) is retried for the other wanted ruleset, after the delay.
"""
import argparse
import json
import os
import subprocess
import sys
import time

MAIN = 'com.fumbbl.ffb.ai.simulation.FumbblReplayDownloader'
_last_contact = [0.0]


def polite_wait(delay):
    gap = time.time() - _last_contact[0]
    if gap < delay:
        time.sleep(delay - gap)
    _last_contact[0] = time.time()


def run(jar, args, delay, timeout):
    polite_wait(delay)
    try:
        p = subprocess.run(['java', '-cp', jar, MAIN] + args, capture_output=True, text=True,
                           timeout=timeout)
    except subprocess.TimeoutExpired:
        return 1, None
    meta = None
    for line in p.stdout.splitlines():
        line = line.strip()
        if line.startswith('{'):
            try:
                meta = json.loads(line)
            except ValueError:
                pass
    return p.returncode, meta


def probe(jar, rid, delay):
    code, _ = run(jar, ['--probe-only', str(rid)], delay, 30)
    return code == 0


def find_max(jar, seed, delay):
    """Exponential probe upward from a known-valid id, then binary search."""
    low, step = seed, 4096
    while probe(jar, low + step, delay):
        low += step
        step *= 2
        print('  valid at', low, flush=True)
    high = low + step
    while high - low > 1:
        mid = (low + high) // 2
        if probe(jar, mid, delay):
            low = mid
        else:
            high = mid
        print('  bracket', low, high, flush=True)
    return low


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--count', type=int, default=10)
    ap.add_argument('--want', default='BB2020=5,BB2025=5')
    ap.add_argument('--start-id', type=int)
    ap.add_argument('--find-max-from', type=int, default=1832851)
    ap.add_argument('--out', default='D:/ffb_replays/fumbbl')
    ap.add_argument('--jar', default='D:/ffb_replays/ffb-ai-replay.jar')
    ap.add_argument('--delay', type=float, default=10.0)
    ap.add_argument('--max-attempts', type=int, default=200)
    a = ap.parse_args()
    if a.delay < 10:
        sys.exit('refusing: the delay between fumbbl.com contacts must be >= 10 s')
    want = {k: int(v) for k, v in (x.split('=') for x in a.want.split(','))}
    os.makedirs(a.out, exist_ok=True)
    manifest = os.path.join(a.out, 'manifest.jsonl')
    have = {k: 0 for k in want}
    done_ids = set()
    if os.path.exists(manifest):
        for line in open(manifest, encoding='utf-8'):
            m = json.loads(line)
            done_ids.add(m['id'])
            if m.get('rules') in have:
                have[m['rules']] += 1

    start = a.start_id or find_max(a.jar, a.find_max_from, a.delay)
    print('start id', start, 'have', have, flush=True)
    rid, attempts = start, 0
    while sum(have.values()) < a.count and attempts < a.max_attempts and rid > 0:
        if rid in done_ids:
            rid -= 1
            continue
        order = sorted((r for r in want if have[r] < want[r]), key=lambda r: have[r] - want[r])
        for rules in order:
            attempts += 1
            path = os.path.join(a.out, '%d.ffbr' % rid)
            code, meta = run(a.jar, [str(rid), path, '--rules', rules], a.delay, 600)
            print('id', rid, rules, 'exit', code, meta or '', flush=True)
            if code == 0 and meta:
                have[rules] += 1
                with open(manifest, 'a', encoding='utf-8') as f:
                    f.write(json.dumps(meta) + '\n')
                break
            if code == 1:
                break  # not found / error: no point retrying with another filter
        rid -= 1
    print('done: have', have, 'attempts', attempts, flush=True)
    return 0 if sum(have.values()) >= a.count else 1


if __name__ == '__main__':
    sys.exit(main())
