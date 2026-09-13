#!/usr/bin/env python
"""Compare two censuses produced by agg.py (one JSON per gate): baseline vs new.

usage: compare.py <baseline_dir> <new_dir> [--out report.md]

Reports the headline totals, touchdowns per game by scale, the GameEvent / PlayerAction variant
sets (what appeared, what vanished), skill-use tallies, re-roll rates, injuries, kickoff results and
the harness required-item checklist failures, so a policy change (e.g. the 2026-09-12 setup
heuristic) can be judged on coverage as well as on parity.
"""
import glob
import json
import sys
from collections import Counter, defaultdict

FOLD = {
    "Move": "Move", "HandOverMove": "Move", "PassMove": "Move", "FoulMove": "Move",
    "GazeMove": "Move", "KickTeamMateMove": "Move", "ThrowTeamMateMove": "Move",
    "PuntMove": "Move", "PutridRegurgitationMove": "Move",
    "Block": "Block", "PutridRegurgitationBlock": "Block", "KickEmBlock": "Block",
    "Blitz": "Blitz", "BlitzSelect": "Blitz", "BlitzMove": "Blitz",
    "PutridRegurgitationBlitz": "Blitz", "KickEmBlitz": "Blitz",
    "Gaze": "HypnoticGaze", "GazeSelect": "HypnoticGaze", "AutoGazeZoat": "HypnoticGaze",
}
REQUIRED = ['action Move', 'action Block', 'action Blitz', 'action Foul', 'action Pass',
            'action HandOver', 'dodge success', 'dodge failure', 'pickup success',
            'pickup failure', 'catch success', 'catch failure', 'ball scatters', 'throw-ins',
            'pass rolls', 'block 1 die', 'block 2 dice', 'block 2 dice against',
            'block result Skull', 'block result BothDown', 'block result Pushback',
            'block result PowPushback', 'block result Pow', 'pushbacks', 'players fell',
            'armor held', 'stunned', 'KO', 'casualty (d16)', 'fouls', 'argue the call',
            'players ejected', 'half starts', 'kickoff events']


def load(d):
    return {(g['race'], g['edition'], g['scale']): g for g in (json.load(open(f)) for f in glob.glob(d + '/*.json'))}


def folded(d):
    c = Counter()
    for k, v in d['actions'].items():
        c[FOLD.get(k, k)] += v
    return c


def checklist(d):
    a = folded(d)
    t = d['types']
    r = d['rolls']
    inj = d['injuries']
    g = lambda k: t.get(k, 0)
    roll = lambda k, f: r.get(k, {}).get(f, 0)
    stunned = inj.get('total', 0) - (inj.get('armor_only', 0) + inj.get('ko', 0) + inj.get('cas', 0))
    bd = d['block_dice']
    br = d['block_result']
    return {
        'action Move': a['Move'], 'action Block': a['Block'], 'action Blitz': a['Blitz'],
        'action Foul': a['Foul'], 'action Pass': a['Pass'], 'action HandOver': a['HandOver'],
        'dodge success': roll('dodgeRoll', 'success'),
        'dodge failure': roll('dodgeRoll', 'total') - roll('dodgeRoll', 'success'),
        'pickup success': roll('pickupRoll', 'success'),
        'pickup failure': roll('pickupRoll', 'total') - roll('pickupRoll', 'success'),
        'catch success': roll('catchRoll', 'success'),
        'catch failure': roll('catchRoll', 'total') - roll('catchRoll', 'success'),
        'ball scatters': g('scatterBall') + g('ballScattered'), 'throw-ins': g('throwIn'),
        'pass rolls': roll('passRoll', 'total'),
        'block 1 die': bd.get('1', 0), 'block 2 dice': bd.get('2', 0),
        'block 2 dice against': bd.get('-2', 0),
        'block result Skull': br.get('Skull', 0), 'block result BothDown': br.get('BothDown', 0),
        'block result Pushback': br.get('Pushback', 0),
        'block result PowPushback': br.get('PowPushback', 0), 'block result Pow': br.get('Pow', 0),
        'pushbacks': g('pushback'), 'players fell': g('playerFellDown'),
        'armor held': inj.get('armor_only', 0), 'stunned': stunned,
        'KO': inj.get('ko', 0), 'casualty (d16)': inj.get('cas', 0),
        'fouls': g('foul'), 'argue the call': roll('argueTheCall', 'total'),
        'players ejected': g('playerEjected'),
        'half starts': g('startHalf'), 'kickoff events': sum(d['kickoff'].values()),
    }


def summarise(gates):
    s = {}
    s['gates'] = len(gates)
    s['games'] = sum(g['games'] for g in gates.values())
    types = Counter()
    actions = Counter()
    skills = Counter()
    kick = Counter()
    inj = Counter()
    rolls = defaultdict(lambda: [0, 0, 0])
    rr_src = Counter()
    td_scale = defaultdict(lambda: [0, 0])
    for g in gates.values():
        types.update(g['types'])
        actions.update(g['actions'])
        skills.update(g.get('skill_used', {}))
        kick.update(g['kickoff'])
        inj.update(g['injuries'])
        rr_src.update(g.get('reroll_src', {}))
        for k, v in g['rolls'].items():
            rolls[k][0] += v['total']
            rolls[k][1] += v['success']
            rolls[k][2] += v['rerolled']
        rolls['blockRoll'][0] += g['types'].get('blockRoll', 0)
        rolls['blockRoll'][2] += g['block_rerolled']
        td_scale[g['scale']][0] += g['games']
        td_scale[g['scale']][1] += g['types'].get('touchdown', 0)
    s.update(types=types, actions=actions, skills=skills, kick=kick, inj=inj, rolls=rolls,
             rr_src=rr_src, td_scale=td_scale, events=sum(types.values()))
    fails = {}
    for k, g in gates.items():
        miss = [i for i in REQUIRED if checklist(g)[i] == 0]
        if miss:
            fails[k] = miss
    s['fails'] = fails
    return s


def pct(a, b):
    return ('%+.1f%%' % (100.0 * (a - b) / b)) if b else 'n/a'


def main():
    base_dir, new_dir = sys.argv[1], sys.argv[2]
    out = None
    if '--out' in sys.argv:
        out = sys.argv[sys.argv.index('--out') + 1]
    B = summarise(load(base_dir))
    N = summarise(load(new_dir))
    L = []
    w = L.append
    w('# Census comparison — %s (baseline) vs %s (new)\n' % (base_dir, new_dir))
    w('| | baseline | new | change |')
    w('|---|---:|---:|---:|')
    w('| gates | %d | %d | |' % (B['gates'], N['gates']))
    w('| games | %d | %d | |' % (B['games'], N['games']))
    w('| events | %d | %d | %s |' % (B['events'], N['events'], pct(N['events'], B['events'])))
    w('| GameEvent variants | %d | %d | |' % (len(B['types']), len(N['types'])))
    w('| PlayerAction variants | %d | %d | |' % (len(B['actions']), len(N['actions'])))
    w('| skills raising SkillUse | %d | %d | |' % (len(B['skills']), len(N['skills'])))
    for key, label in [('touchdown', 'touchdowns'), ('handOver', 'hand-offs'), ('passRoll', 'pass rolls'),
                       ('blockRoll', 'block rolls'), ('foul', 'fouls'), ('kickoffResultEvent', 'kick-offs'),
                       ('throwIn', 'throw-ins'), ('interceptionRoll', 'interception rolls'),
                       ('catchRoll', 'catch rolls'), ('pickupRoll', 'pickup rolls'), ('dodgeRoll', 'dodge rolls'),
                       ('goForItRoll', 'rush rolls'), ('scatterBall', 'ball scatters'), ('touchback', 'touchbacks')]:
        w('| %s | %d | %d | %s |' % (label, B['types'].get(key, 0), N['types'].get(key, 0),
                                    pct(N['types'].get(key, 0), B['types'].get(key, 0))))
    for key in ['ko', 'cas', 'dead']:
        w('| injuries: %s | %d | %d | %s |' % (key, B['inj'].get(key, 0), N['inj'].get(key, 0),
                                              pct(N['inj'].get(key, 0), B['inj'].get(key, 0))))
    w('\n## Touchdowns per game, by sampling scale\n')
    w('| scale | baseline games | baseline TD/game | new games | new TD/game | change |')
    w('|---|---:|---:|---:|---:|---:|')
    for sc in sorted(set(B['td_scale']) | set(N['td_scale']), key=lambda s: float(s)):
        bg, bt = B['td_scale'].get(sc, [0, 0])
        ng, nt = N['td_scale'].get(sc, [0, 0])
        bpg = bt / bg if bg else 0
        npg = nt / ng if ng else 0
        w('| %s | %d | %.3f | %d | %.3f | %s |' % (sc, bg, bpg, ng, npg, pct(npg, bpg)))
    w('\n## Required-item checklist failures\n')
    w('baseline: %d gates short; new: %d gates short' % (len(B['fails']), len(N['fails'])))
    w('\nnew-only short gates:')
    for k in sorted(set(N['fails']) - set(B['fails'])):
        w('  %s %s @%s: %s' % (k[0], k[1], k[2], ', '.join(N['fails'][k])))
    w('\nfixed (short in baseline, complete now):')
    for k in sorted(set(B['fails']) - set(N['fails'])):
        w('  %s %s @%s: %s' % (k[0], k[1], k[2], ', '.join(B['fails'][k])))
    w('\nstill short in both:')
    for k in sorted(set(B['fails']) & set(N['fails'])):
        w('  %s %s @%s: %s' % (k[0], k[1], k[2], ', '.join(N['fails'][k])))
    w('\n## GameEvent variants\n')
    w('new only: %s' % (sorted(set(N['types']) - set(B['types'])) or 'none'))
    w('gone: %s' % (sorted(set(B['types']) - set(N['types'])) or 'none'))
    w('\n| event | baseline | new | change |')
    w('|---|---:|---:|---:|')
    for k in sorted(set(B['types']) | set(N['types']), key=lambda k: -N['types'].get(k, 0)):
        w('| %s | %d | %d | %s |' % (k, B['types'].get(k, 0), N['types'].get(k, 0), pct(N['types'].get(k, 0), B['types'].get(k, 0))))
    w('\n## PlayerAction variants\n')
    w('new only: %s' % (sorted(set(N['actions']) - set(B['actions'])) or 'none'))
    w('gone: %s' % (sorted(set(B['actions']) - set(N['actions'])) or 'none'))
    w('\n| action | baseline | new | change |')
    w('|---|---:|---:|---:|')
    for k in sorted(set(B['actions']) | set(N['actions']), key=lambda k: -N['actions'].get(k, 0)):
        w('| %s | %d | %d | %s |' % (k, B['actions'].get(k, 0), N['actions'].get(k, 0), pct(N['actions'].get(k, 0), B['actions'].get(k, 0))))
    w('\n## Skill uses (SkillUse events, used=true)\n')
    w('new only: %s' % (sorted(set(N['skills']) - set(B['skills'])) or 'none'))
    w('gone: %s' % (sorted(set(B['skills']) - set(N['skills'])) or 'none'))
    w('\n| skill | baseline | new | change |')
    w('|---|---:|---:|---:|')
    for k in sorted(set(B['skills']) | set(N['skills']), key=lambda k: -N['skills'].get(k, 0)):
        w('| %s | %d | %d | %s |' % (k, B['skills'].get(k, 0), N['skills'].get(k, 0), pct(N['skills'].get(k, 0), B['skills'].get(k, 0))))
    w('\n## Re-roll rates\n')
    w('| roll | baseline rolls | baseline rerolled | new rolls | new rerolled |')
    w('|---|---:|---:|---:|---:|')
    for k in sorted(set(B['rolls']) | set(N['rolls']), key=lambda k: -N['rolls'][k][0] if k in N['rolls'] else 0):
        bt, _, brr = B['rolls'].get(k, [0, 0, 0])
        nt, _, nrr = N['rolls'].get(k, [0, 0, 0])
        w('| %s | %d | %d (%.1f%%) | %d | %d (%.1f%%) |' % (k, bt, brr, 100.0 * brr / max(bt, 1), nt, nrr, 100.0 * nrr / max(nt, 1)))
    w('\n## Kick-off results\n')
    w('| result | baseline | new |')
    w('|---|---:|---:|')
    for k in sorted(set(B['kick']) | set(N['kick'])):
        w('| %s | %d | %d |' % (k, B['kick'].get(k, 0), N['kick'].get(k, 0)))
    text = '\n'.join(L) + '\n'
    if out:
        open(out, 'w', encoding='utf-8').write(text)
        print('wrote', out)
    else:
        print(text)


if __name__ == '__main__':
    main()
