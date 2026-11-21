"""Dice recovery with a source -> (report kind, field) table, the design the follower would use.

Pass 1 learns, from an unordered value match, which report field each die SOURCE (DiceRoller method
via step) lands in. Pass 2 replays every game strictly: each die takes the EARLIEST unconsumed
value, in report order, among the fields mapped to its source, within its decision window; it must
equal the die. Cross-validated: the table is learned on the other matchup and tested on this one.
"""
import collections, glob, json, re, sys

DIE = re.compile(r'JAVA_DIE rng=(\d+) d(\d+)=(\d+) from=(\S+)(.*)')
JSTEP = re.compile(r'JSTEP i=(\d+) rng_calls=(\d+)')
NOT_DICE = {'minimumRoll', 'reRolled'}

def entries(r, ridx):
    out = []
    rid = r.get('reportId')
    def walk(k, v, path):
        if isinstance(v, bool) or k in NOT_DICE:
            return
        if isinstance(v, int) and re.search(r'[Rr]oll', k):
            out.append((ridx, rid, path, v))
        elif isinstance(v, list):
            for x in v:
                if isinstance(x, int) and not isinstance(x, bool) and re.search(r'[Rr]oll', k):
                    out.append((ridx, rid, path, x))
                elif isinstance(x, dict):
                    for kk, vv in x.items():
                        walk(kk, vv, path + '.' + kk)
        elif isinstance(v, dict):
            for kk, vv in v.items():
                walk(kk, vv, path + '.' + kk)
    for k, v in r.items():
        walk(k, v, k)
    if rid == 'coinThrow':
        out.append((ridx, rid, 'coinThrowHeads', 'COIN'))
    return out

def source(frm, deep):
    method = frm.split('.')[-1].split(':')[0]
    roller = re.findall(r'<- DiceRoller\.(\w+):\d+', deep)
    if method == 'rollDice' and roller:
        method = roller[0]
    steps = re.findall(r'<- (\w+)\.(\w+):\d+', deep)
    step = next((s for s, m in steps if s.startswith('Step') or 'Mechanic' in s or 'Util' in s), steps[0][0] if steps else '?')
    return '%s via %s' % (method, step)

def load(base):
    windows, dice = [], []
    for line in open(base + '_stderr.txt', encoding='utf-8', errors='replace'):
        m = JSTEP.search(line)
        if m:
            windows.append((int(m.group(2)), int(m.group(1)))); continue
        m = DIE.search(line)
        if m:
            dice.append((int(m.group(1)), int(m.group(2)), int(m.group(3)), source(m.group(4), m.group(5))))
    pools = collections.defaultdict(list)
    for ridx, line in enumerate(open(base + '_reports.jsonl', encoding='utf-8')):
        r = json.loads(line)
        pools[r.get('i', 0)].extend(entries(r, ridx))
    bounds = sorted(windows)
    def w(pos):
        k = 0
        for rc, st in bounds:
            if pos > rc: k = st
            else: break
        return k
    return [(w(p), s, v, src) for p, s, v, src in dice], pools

def learn(games):
    table = collections.defaultdict(collections.Counter)
    for dice, pools in games:
        pools = {k: list(v) for k, v in pools.items()}
        for win, sides, val, src in dice:
            pool = pools.get(win, [])
            for e in pool:
                if e[3] == val or (sides == 2 and e[3] == 'COIN'):
                    table[src][(e[1], e[2])] += 1; pool.remove(e); break
    return table

def test(dice, pools, table):
    pools = {k: list(v) for k, v in pools.items()}
    tot = hid = 0; miss = collections.Counter()
    for win, sides, val, src in dice:
        counts = table.get(src, {})
        top = max(counts.values()) if counts else 0
        fields = {f for f, n in counts.items() if n >= 0.3 * top}
        look = int(LOOKAHEAD)
        cand = []
        for ww in range(win, win + look + 1):
            pool = pools.get(ww, [])
            cand = [(e, pool) for e in pool if (e[1], e[2]) in fields]
            if cand:
                break
        tot += 1
        if cand and (cand[0][0][3] == val or (sides == 2 and cand[0][0][3] == 'COIN')):
            cand[0][1].remove(cand[0][0])
        else:
            hid += 1; miss[src] += 1
            if DEBUG and 'Injury' in src and DEBUG[0] > 0:
                DEBUG[0] -= 1
                print('   MISS win=%d d%d=%d %s fields=%s cand=%s' % (win, sides, val, src, sorted(fields)[:4], [c[0] for c in cand[:3]]))
    return tot, hid, miss

D = sys.argv[1]
DEBUG = [6] if '--debug' in sys.argv else None
LOOKAHEAD =sys.argv[sys.argv.index('--look') + 1] if '--look' in sys.argv else 0
groups = collections.defaultdict(list)
for p in sorted(glob.glob(D + '/*_stderr.txt')):
    base = p[:-len('_stderr.txt')]
    tag = base.replace('\\', '/').split('/')[-1].rsplit('_s', 1)[0]
    groups[tag].append((base, load(base)))
T = H = 0; MISS = collections.Counter()
for tag, games in groups.items():
    train = [g for t, gs in groups.items() if t != tag for _, g in gs]
    table = learn(train + [g for _, g in games]) if '--same' in sys.argv else learn(train)
    for base, (dice, pools) in games:
        t, h, miss = test(dice, pools, table)
        T += t; H += h; MISS.update(miss)
        print('%-24s dice=%4d unrecovered=%3d (%.1f%%)' % (base.replace('\\', '/').split('/')[-1], t, h, 100.0 * h / t))
print('\nALL strict, field-mapped: dice=%d unrecovered=%d (%.1f%%)' % (T, H, 100.0 * H / T))
for k, n in MISS.most_common(15):
    print('  %4d %s' % (n, k))
