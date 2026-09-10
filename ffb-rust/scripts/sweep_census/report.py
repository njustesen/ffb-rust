#!/usr/bin/env python
"""Master analysis of the 2026-09-10 full-matrix sweep (333 gates / 33,300 games)."""
import json
import glob
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(r"C:\Users\Admin\niels\ffb-rust\ffb-rust")

FOLD = {
    "Move": "Move", "HandOverMove": "Move", "PassMove": "Move", "FoulMove": "Move",
    "GazeMove": "Move", "KickTeamMateMove": "Move", "ThrowTeamMateMove": "Move",
    "PuntMove": "Move", "PutridRegurgitationMove": "Move",
    "Block": "Block", "PutridRegurgitationBlock": "Block", "KickEmBlock": "Block",
    "Blitz": "Blitz", "BlitzSelect": "Blitz", "BlitzMove": "Blitz",
    "PutridRegurgitationBlitz": "Blitz", "KickEmBlitz": "Blitz",
    "Gaze": "HypnoticGaze", "GazeSelect": "HypnoticGaze", "AutoGazeZoat": "HypnoticGaze",
}

gates = [json.load(open(f)) for f in glob.glob('out/*.json')]
gates.sort(key=lambda d: (d['race'], d['edition'], d['scale']))
p2 = {}
for f in glob.glob('out2/*.json'):
    d = json.load(open(f))
    p2['%s__%s__%s' % (d['race'], d['edition'], d['scale'])] = d


def folded(d):
    c = Counter()
    for k, v in d['actions'].items():
        c[FOLD.get(k, k)] += v
    return c


def checklist(d):
    """Reproduce t3_checklist::lineman_items -> {item: count}, required set."""
    a = folded(d)
    t = d['types']
    r = d['rolls']
    inj = d['injuries']
    g = lambda k: t.get(k, 0)
    roll = lambda k, f: r.get(k, {}).get(f, 0)
    stunned = inj.get('total', 0) - (inj.get('armor_only', 0) + inj.get('ko', 0) + inj.get('cas', 0))
    bd = d['block_dice']
    br = d['block_result']
    items = {
        'action Move': a['Move'], 'action Block': a['Block'], 'action Blitz': a['Blitz'],
        'action Foul': a['Foul'], 'action Pass': a['Pass'], 'action HandOver': a['HandOver'],
        'dodge success': roll('dodgeRoll', 'success'),
        'dodge failure': roll('dodgeRoll', 'total') - roll('dodgeRoll', 'success'),
        'GFI rolls': roll('goForItRoll', 'total'),
        'pickup success': roll('pickupRoll', 'success'),
        'pickup failure': roll('pickupRoll', 'total') - roll('pickupRoll', 'success'),
        'catch success': roll('catchRoll', 'success'),
        'catch failure': roll('catchRoll', 'total') - roll('catchRoll', 'success'),
        'ball scatters': g('scatterBall') + g('ballScattered'),
        'throw-ins': g('throwIn'),
        'pass rolls': roll('passRoll', 'total'),
        'pass deviates': g('passDeviate'),
        'interceptions': roll('interceptionRoll', 'total'),
        'block 1 die': bd.get('1', 0), 'block 2 dice': bd.get('2', 0),
        'block 2 dice against': bd.get('-2', 0),
        'block 3 dice': bd.get('3', 0) + bd.get('-3', 0),
        'block result Skull': br.get('Skull', 0), 'block result BothDown': br.get('BothDown', 0),
        'block result Pushback': br.get('Pushback', 0),
        'block result PowPushback': br.get('PowPushback', 0), 'block result Pow': br.get('Pow', 0),
        'pushbacks': g('pushback'), 'crowd surfs': g('scatterPlayer'),
        'players fell': g('playerFellDown'),
        'armor held': inj.get('armor_only', 0), 'stunned': stunned,
        'KO': inj.get('ko', 0), 'casualty (d16)': inj.get('cas', 0), 'death': inj.get('dead', 0),
        'fouls': g('foul'), 'argue the call': roll('argueTheCall', 'total'),
        'argue success': roll('argueTheCall', 'success'), 'players ejected': g('playerEjected'),
        'touchdowns': g('touchdown'), 'half starts': g('startHalf'),
        'weather changes': g('weatherChange'), 'kickoff events': sum(d['kickoff'].values()),
    }
    return items


REQUIRED = ['action Move', 'action Block', 'action Blitz', 'action Foul', 'action Pass',
            'action HandOver', 'dodge success', 'dodge failure', 'pickup success',
            'pickup failure', 'catch success', 'catch failure', 'ball scatters', 'throw-ins',
            'pass rolls', 'block 1 die', 'block 2 dice', 'block 2 dice against',
            'block result Skull', 'block result BothDown', 'block result Pushback',
            'block result PowPushback', 'block result Pow', 'pushbacks', 'players fell',
            'armor held', 'stunned', 'KO', 'casualty (d16)', 'fouls', 'argue the call',
            'players ejected', 'half starts', 'kickoff events']

OUT = []
w = OUT.append

# ── 1. scale ────────────────────────────────────────────────────────────────
tot_events = sum(sum(d['types'].values()) for d in gates)
w('# Sweep 2026-09-10 — event-level coverage analysis\n')
w('gates=%d  games=%d  events=%d\n' % (len(gates), sum(d['games'] for d in gates), tot_events))

# ── 2. per-gate required-item failures ──────────────────────────────────────
fails = {}
for d in gates:
    tag = '%s %s @%s' % (d['race'], d['edition'], d['scale'])
    items = checklist(d)
    miss = [k for k in REQUIRED if items[k] == 0]
    if miss:
        fails[tag] = miss
w('\n## Gates failing the harness required-item checklist: %d/%d\n' % (len(fails), len(gates)))
for tag, miss in sorted(fails.items()):
    w('  %-34s %s' % (tag, ', '.join(miss)))
missing_hist = Counter(m for v in fails.values() for m in v)
w('\nmissing-item frequency: %s\n' % dict(missing_hist))

# ── 3. optional/blocked items that are zero somewhere ───────────────────────
zero_any = defaultdict(list)
for d in gates:
    items = checklist(d)
    for k, v in items.items():
        if v == 0:
            zero_any[k].append('%s %s @%s' % (d['race'], d['edition'], d['scale']))
w('\n## Checklist items that are zero in at least one gate (of 333)\n')
for k, v in sorted(zero_any.items(), key=lambda kv: -len(kv[1])):
    w('  %-26s %3d gates' % (k, len(v)))

# ── 4. per-cell (race x edition) rollup of interesting counters ─────────────
cells = defaultdict(list)
for d in gates:
    cells[(d['race'], d['edition'])].append(d)
w('\n## Per-cell headline counts (summed over the 3 sampling scales, 300 games)\n')
w('%-32s %8s %7s %7s %7s %7s %7s %7s %6s %6s' % (
    'cell', 'events', 'blocks', 'inj', 'cas', 'TD', 'passes', 'handovr', 'fouls', 'kicks'))
cellrows = []
for (race, ed), ds in sorted(cells.items()):
    t = Counter()
    for d in ds:
        t.update(d['types'])
        t['cas'] += d['injuries'].get('cas', 0)
    row = (race + ' ' + ed, sum(v for k, v in t.items() if k != 'cas'), t['blockRoll'],
           t['injury'], t['cas'], t['touchdown'], t['passRoll'], t['handOver'],
           t['foul'], t['kickoffResultEvent'])
    cellrows.append(row)
    w('%-32s %8d %7d %7d %7d %7d %7d %7d %6d %6d' % row)

# ── 5. re-roll usage (the only reroll telemetry that exists) ───────────────
w('\n## Re-roll usage — GameEvent::ReRoll has NO producer; these are per-roll `rerolled` flags\n')
rr = Counter()
tt = Counter()
for d in gates:
    for k, v in d['rolls'].items():
        rr[k] += v['rerolled']
        tt[k] += v['total']
    rr['blockRoll'] += d['block_rerolled']
    tt['blockRoll'] += d['types'].get('blockRoll', 0)
for k in sorted(tt, key=lambda k: -tt[k]):
    w('  %-24s %9d rolls  %8d rerolled (%.2f%%)' % (k, tt[k], rr[k], 100.0 * rr[k] / max(tt[k], 1)))

# ── 6. weather ─────────────────────────────────────────────────────────────
wx = Counter()
wx_ed = defaultdict(Counter)
for d in gates:
    wx.update(d.get('weather', {}))
    wx_ed[d['edition']].update(d.get('weather', {}))
w('\n## Weather results (from weatherChange)\n  all: %s' % dict(wx))
for e in sorted(wx_ed):
    w('  %s: %s' % (e, dict(wx_ed[e])))

# ── 7. prayers ─────────────────────────────────────────────────────────────
pr = Counter()
pr_ed = defaultdict(Counter)
for d in gates:
    pr.update(d['prayers'])
    pr_ed[d['edition']].update(d['prayers'])
w('\n## Prayers to Nuffle\n')
for e in ('bb2016', 'bb2020', 'bb2025'):
    cat = REPO / ('data/prayers/%s_prayers.json' % e)
    names = []
    if cat.exists():
        j = json.loads(cat.read_text(encoding='utf-8'))
        pl = j.get('prayers', j)
        names = [p.get('id', p.get('name')) for p in pl] if isinstance(pl, list) else list(pl)
    seen = pr_ed[e]
    w('  %s: catalog=%d  rolled=%d  events=%d' % (e, len(names), len(seen), sum(seen.values())))
    if names:
        missing = [n for n in names if n not in seen]
        w('    never rolled: %s' % (missing or 'none'))

# ── 8. positional coverage ─────────────────────────────────────────────────
w('\n## Positional coverage — did every drafted player ever take an action?\n')
never_act = []
for (race, ed), ds in sorted(cells.items()):
    tf = REPO / ('data/teams/%s/team_%s.json' % (ed, race))
    if not tf.exists():
        continue
    team = json.loads(tf.read_text(encoding='utf-8'))
    nrs = {}
    for pl in team.get('players', []):
        nrs[pl['nr']] = pl['position_id']
    for st in team.get('stars', []) or []:
        nrs[st['nr']] = 'star:' + st['star_id']
    acted = Counter()
    for d in ds:
        for pid, n in d.get('by_player', {}).items():
            try:
                acted[int(pid.split('_')[1])] += n
            except Exception:
                pass
    silent = sorted(nr for nr in nrs if acted.get(nr, 0) == 0)
    if silent:
        never_act.append((race, ed, [(nr, nrs[nr]) for nr in silent]))
if never_act:
    for race, ed, s in never_act:
        w('  %-28s %s' % (race + ' ' + ed, ', '.join('#%d %s' % x for x in s)))
else:
    w('  every drafted player number took at least one action in every cell')

Path('report.txt').write_text('\n'.join(OUT), encoding='utf-8')
print('\n'.join(OUT[:40]))
print('...\nwrote report.txt (%d lines)' % len(OUT))
