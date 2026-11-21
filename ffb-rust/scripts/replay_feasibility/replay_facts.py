import gzip, json, glob, sys, collections
files = sorted(glob.glob('D:/ffb_replays/fumbbl/*.ffbr') + glob.glob('D:/ffb_replays/partial/*.ffbr'))
opts = collections.defaultdict(set)
skills = collections.defaultdict(set)
mcs = collections.Counter()
reps = collections.Counter()
samples_mc = {}
for f in files:
    cmds = json.loads(gzip.decompress(open(f, 'rb').read()))['commandArray']
    g = cmds[0]['game']
    rules = [o['gameOptionValue'] for o in g['gameOptions']['gameOptionArray'] if o['gameOptionId'] == 'rulesVersion']
    rules = rules[0] if rules else '?'
    for o in g['gameOptions']['gameOptionArray']:
        opts[(rules, o['gameOptionId'])].add(str(o['gameOptionValue']))
    for t in (g['teamHome'], g['teamAway']):
        for p in t['playerArray']:
            for s in p.get('skillArray', []):
                skills[rules].add(s)
        for pos in t['roster']['positionArray']:
            for s in pos.get('skillArray', []) if isinstance(pos.get('skillArray'), list) else []:
                skills[rules].add(s)
    for c in cmds[1:]:
        for m in (c.get('modelChangeList') or {}).get('modelChangeArray', []):
            mcs[m['modelChangeId']] += 1
            samples_mc.setdefault(m['modelChangeId'], m)
        for r in (c.get('reportList') or {}).get('reports', []):
            reps[r['reportId']] += 1
out = {
    'gameOptions': {'%s %s' % k: sorted(v) for k, v in sorted(opts.items())},
    'skills': {k: sorted(v) for k, v in skills.items()},
    'modelChangeCounts': dict(mcs.most_common()),
    'modelChangeSamples': samples_mc,
    'reportCounts': dict(reps.most_common()),
    'teamSample': cmds[0]['game']['teamHome'],
}
json.dump(out, open(sys.argv[1], 'w'), indent=1)
print(len(files), 'files;', len(opts), 'option rows;', {k: len(v) for k, v in skills.items()}, 'skills;', len(mcs), 'model change kinds;', len(reps), 'report kinds')
