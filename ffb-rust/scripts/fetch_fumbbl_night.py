#!/usr/bin/env python3
"""Overnight FUMBBL replay fetch (BACKLOG §N): as many replays as possible inside a clock window,
newest first, walking replay ids back in time.

Rules of the road (user, 2026-11-21):
  * >= 10 s between every contact with fumbbl.com, measured from the END of one call to the START
    of the next; one connection at a time, never parallel;
  * STOP only when the server rejects us: connect failure/refusal (downloader exit 3), any server
    status other than "Replay Unavailable"/"Unknown Game Id" (exit 5), no answer at all (exit 6),
    a game state followed by silence (exit 1), or a hung call (timeout). An id the server answers
    with "Replay Unavailable" (exit 4), or with a complete but empty replay (exit 7, a game that was
    never played), is a normal gap (~1 in 8 ids) and never stops the walk.
  * replays hold coach names -> they stay out of git (default D:/ffb_replays/fumbbl_night_<date>).

All rulesets are kept (--rules ALL). Games still being played (the newest ids) end up in partial/.
Resumable: ids already in manifest.jsonl are skipped.

Usage:
  python -I scripts/fetch_fumbbl_night.py --begin 2026-11-22T00:00 --end 2026-11-22T10:00
         [--out D:/ffb_replays/fumbbl_night_20261122] [--find-max-from 1966013] [--delay 10]
         [--start-id N   # resume the walk at N, skipping the newest-id search]
"""
import argparse
import datetime as dt
import gzip
import json
import os
import subprocess
import sys
import time

MAIN = 'com.fumbbl.ffb.ai.simulation.FumbblReplayDownloader'
CP = 'D:/ffb_replays/classes;D:/ffb_replays/ffb-ai-replay.jar'  # private copies, never the cert jar
_last_end = [time.time()]  # also wait 10 s after any manual call made just before starting


class Refused(Exception):
    pass


def log(*a):
    print(dt.datetime.now().strftime('%H:%M:%S'), *a, flush=True)


def call(args, delay, timeout):
    gap = time.time() - _last_end[0]
    if gap < delay:
        time.sleep(delay - gap)
    try:
        p = subprocess.run(['java', '-cp', CP, MAIN] + args, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        _last_end[0] = time.time()
        raise Refused('call timed out after %d s: %s' % (timeout, ' '.join(args)))
    _last_end[0] = time.time()
    if p.returncode in (1, 3, 5, 6):
        why = {1: 'game state then silence', 3: 'connection refused/failed', 5: 'server error status',
               6: 'server never answered'}[p.returncode]
        raise Refused('%s (exit %d): %s' % (why, p.returncode, ' | '.join(p.stderr.strip().splitlines()[-3:])))
    meta = None
    for line in p.stdout.splitlines():
        if line.strip().startswith('{'):
            try:
                meta = json.loads(line)
            except ValueError:
                pass
    return p.returncode, meta


def is_complete(path):
    with open(path, 'rb') as f:
        cmds = json.loads(gzip.decompress(f.read()))['commandArray']
    for c in cmds[-40:]:
        for r in (c.get('reportList') or {}).get('reports', []):
            if r.get('reportId') == 'fumbblResultUpload':
                return True
        for m in (c.get('modelChangeList') or {}).get('modelChangeArray', []):
            if m.get('modelChangeId') == 'gameSetFinished' and m.get('modelChangeValue'):
                return True
    return False


def started_of(path):
    with open(path, 'rb') as f:
        return json.loads(gzip.decompress(f.read()))['commandArray'][0]['game'].get('started') or ''


def find_max(seed, delay):
    probe = lambda rid: call(['--probe-only', str(rid)], delay, 120)[0] == 0  # 4 = unavailable
    if not probe(seed):
        raise Refused('seed id %d is not valid (server refusing?)' % seed)
    low, step = seed, 256
    while probe(low + step):
        low += step
        step *= 2
        log('  valid at', low)
    high = low + step
    while high - low > 1:
        mid = (low + high) // 2
        if probe(mid):
            low = mid
        else:
            high = mid
        log('  bracket', low, high)
    return low


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--begin', required=True, help='local clock, e.g. 2026-11-22T00:00')
    ap.add_argument('--end', required=True)
    ap.add_argument('--out', default='D:/ffb_replays/fumbbl_night_20261122')
    ap.add_argument('--find-max-from', type=int, default=1966013)
    ap.add_argument('--delay', type=float, default=10.0)
    ap.add_argument('--start-id', type=int)
    a = ap.parse_args()
    if a.delay < 10:
        sys.exit('refusing: the delay between fumbbl.com contacts must be >= 10 s')
    begin, end = dt.datetime.fromisoformat(a.begin), dt.datetime.fromisoformat(a.end)
    for d in ('', 'partial'):
        os.makedirs(os.path.join(a.out, d), exist_ok=True)
    manifest = os.path.join(a.out, 'manifest.jsonl')
    done = set()
    if os.path.exists(manifest):
        done = {json.loads(l)['id'] for l in open(manifest, encoding='utf-8')}

    wait = (begin - dt.datetime.now()).total_seconds()
    if wait > 0:
        log('sleeping %.0f s until %s' % (wait, a.begin))
        time.sleep(wait)
    kept = partial = missing_total = 0
    try:
        top = a.start_id or find_max(a.find_max_from, a.delay)
        log('walking down from', top)
        rid, missing = top, 0
        while rid > 0 and dt.datetime.now() < end:
            if rid in done:
                rid -= 1
                continue
            path = os.path.join(a.out, '%d.ffbr' % rid)
            code, meta = call([str(rid), path, '--rules', 'ALL'], a.delay, 600)
            if code == 0 and meta and os.path.exists(path):
                missing = 0
                if not is_complete(path):
                    os.replace(path, os.path.join(a.out, 'partial', '%d.ffbr' % rid))
                    partial += 1
                    log('id', rid, 'unfinished -> partial/')
                else:
                    meta['started'] = started_of(path)
                    with open(manifest, 'a', encoding='utf-8') as f:
                        f.write(json.dumps(meta) + '\n')
                    done.add(rid)
                    kept += 1
                    log('id', rid, meta['started'], meta.get('rules'), meta.get('home'), 'v', meta.get('away'),
                        'KEPT', kept)
            else:
                missing += 1
                missing_total += 1
                why = {4: 'replay unavailable', 7: 'replay empty (game never played)'}.get(code, 'no replay')
                log('id', rid, '%s (exit %d), %d in a row' % (why, code, missing))
            rid -= 1
        log('end of window reached')
    except Refused as e:
        log('STOPPED:', e)
        print('done: kept %d, partial %d, missing %d -> %s' % (kept, partial, missing_total, a.out), flush=True)
        return 3
    print('done: kept %d, partial %d, missing %d -> %s' % (kept, partial, missing_total, a.out), flush=True)
    return 0


if __name__ == '__main__':
    sys.exit(main())
