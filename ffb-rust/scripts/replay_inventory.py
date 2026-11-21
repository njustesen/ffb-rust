#!/usr/bin/env python3
"""Offline inventory of downloaded FUMBBL replays (.ffbr = gzip JSON from FumbblReplayDownloader).

Answers the scoping questions of BACKLOG §N before any engine work: which rulesets / rosters /
skills / game options / inducements the batch carries, and which report kinds (the only place a
replay records dice) occur, with one sample of each so the dice-extraction table can be written.

Usage: python -I scripts/replay_inventory.py D:/ffb_replays/fumbbl [--samples out.json]
Reads only; never touches the network.
"""
import collections
import glob
import gzip
import json
import os
import sys


def load(path):
    with open(path, 'rb') as f:
        return json.loads(gzip.decompress(f.read()))


def rules_of(game):
    opts = json.dumps(game.get('gameOptions'))
    for r in ('BB2025', 'BB2020', 'BB2016'):
        if r in opts:
            return r
    return '?'


def main():
    src = sys.argv[1]
    samples_out = sys.argv[sys.argv.index('--samples') + 1] if '--samples' in sys.argv else None
    files = sorted(glob.glob(os.path.join(src, '*.ffbr')))
    rep_total, rep_files, samples = collections.Counter(), collections.Counter(), {}
    mc_total = collections.Counter()
    skills = collections.Counter()
    opts_seen = collections.defaultdict(collections.Counter)
    for path in files:
        root = load(path)
        cmds = root['commandArray']
        g = cmds[0]['game']
        teams = [g['teamHome'], g['teamAway']]
        kinds = collections.Counter()
        for c in cmds[1:]:
            for m in (c.get('modelChangeList') or {}).get('modelChangeArray', []):
                mc_total[m.get('modelChangeId')] += 1
            for r in (c.get('reportList') or {}).get('reports', []):
                k = r.get('reportId')
                kinds[k] += 1
                samples.setdefault(k, r)
        rep_total.update(kinds)
        rep_files.update(set(kinds))
        for o in g.get('gameOptions', {}).get('gameOptionArray', []):
            opts_seen[o['gameOptionId']][str(o['gameOptionValue'])] += 1
        for t in teams:
            for p in t['playerArray']:
                skills.update(p.get('skillArray', []))
        finished = any('fumbblResultUpload' == r.get('reportId') for c in cmds
                       for r in (c.get('reportList') or {}).get('reports', []))
        inducements = kinds.get('cardsAndInducementsBought', 0)
        print('%s %s %s(%s, %d pl) v %s(%s, %d pl) cmds=%d complete=%s inducement_reports=%d' % (
            os.path.basename(path), rules_of(g),
            teams[0]['race'], teams[0]['roster'].get('rosterId'), len(teams[0]['playerArray']),
            teams[1]['race'], teams[1]['roster'].get('rosterId'), len(teams[1]['playerArray']),
            len(cmds), finished, inducements))
    print('\nreport kinds (total / replays):')
    for k, n in rep_total.most_common():
        print('  %-32s %6d %3d' % (k, n, rep_files[k]))
    print('\nmodel change kinds:', len(mc_total))
    print('\nroster skills (%d distinct):' % len(skills), ', '.join(
        '%s:%d' % kv for kv in skills.most_common()))
    print('\ngame options with more than one value across the batch:')
    for k, vals in sorted(opts_seen.items()):
        if len(vals) > 1:
            print('  %s %s' % (k, dict(vals)))
    if samples_out:
        with open(samples_out, 'w', encoding='utf-8') as f:
            json.dump(samples, f, indent=1, sort_keys=True)
        print('\nwrote one sample per report kind to', samples_out)


if __name__ == '__main__':
    main()
