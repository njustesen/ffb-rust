"""Summarise one per-replay output dir (BACKLOG §O): how many replays are complete, and why not.

Usage: python -I scripts/replay_java_summary.py <outDir> [--list KIND]
Groups incomplete replays by problem: exception class + first FFB frame, skipped syncs, record-error
kind, end-check mismatch, apply errors (first message, enum values stripped of the replay-specific
part). --list KIND prints the replay ids whose problem key contains KIND.
"""
import collections
import glob
import json
import os
import re
import sys


def problem_keys(st):
    keys = []
    exc = st.get('exception')
    if exc:
        if exc.startswith('model changes failed to apply'):
            m = re.search(r'first: sync \d+: (.*)', exc)
            keys.append('apply: ' + (m.group(1) if m else exc)[:160])
        else:
            frames = re.findall(r' at (com\.fumbbl[^ ]+)', exc)
            keys.append('exception: ' + exc.split(' at ')[0][:100] + (' @ ' + frames[0] if frames else ''))
    if st.get('skipped_syncs'):
        keys.append('skipped syncs')
    for k in (st.get('record_errors') or {}):
        keys.append('record error ' + k)
    end = st.get('end_check')
    if end != 'OK':
        keys.append('end check ' + (end or 'missing').split(':')[0])
    return keys


def main():
    out = sys.argv[1]
    want = sys.argv[sys.argv.index('--list') + 1] if '--list' in sys.argv else None
    files = sorted(glob.glob(os.path.join(out, '*.status.json')))
    by_key = collections.defaultdict(list)
    unknown = collections.Counter()
    clean = 0
    records = 0
    for f in files:
        rid = os.path.basename(f)[:-len('.status.json')]
        st = json.load(open(f))
        records += st.get('records', 0)
        unknown.update(st.get('unknown_model_changes') or {})
        keys = problem_keys(st)
        if not keys:
            clean += 1
        for k in keys:
            by_key[k].append(rid)
    if want:
        for k, ids in by_key.items():
            if want in k:
                print(k)
                print(' '.join(ids))
        return
    print('%d replays with status, %d complete, %d incomplete; %d action records'
          % (len(files), clean, len(files) - clean, records))
    for k, ids in sorted(by_key.items(), key=lambda kv: -len(kv[1])):
        print('%5d  %s   e.g. %s' % (len(ids), k, ' '.join(ids[:3])))
    print('unknown model changes dropped:', dict(unknown))


if __name__ == '__main__':
    main()
