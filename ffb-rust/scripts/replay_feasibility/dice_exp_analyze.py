"""Ground truth: how many of the engine's die calls can be recovered from its REPORTS alone?

For every die call (JAVA_DIE, with deep caller) find a dice-like field value in a report emitted in
the same decision window (multiset match). Unmatched dice are 'hidden' -- a FUMBBL replay would
not carry them. Windows: die rng positions in (rng_calls[JSTEP k], rng_calls[JSTEP k+1]] map to
reports stamped i=k.
"""
import collections, glob, json, re, sys

DIE = re.compile(r'JAVA_DIE rng=(\d+) d(\d+)=(\d+) from=(\S+)(.*)')
JSTEP = re.compile(r'JSTEP i=(\d+) rng_calls=(\d+)')
NOT_DICE = {'minimumRoll', 'reRolled', 'minimumRollDodge', 'minimumRollRegular'}

def report_dice(r):
    vals = []
    def walk(k, v):
        if isinstance(v, bool) or k in NOT_DICE:
            return
        if isinstance(v, int) and re.search(r'[Rr]oll', k):
            vals.append(v)
        elif isinstance(v, list):
            for x in v:
                if isinstance(x, int) and not isinstance(x, bool) and re.search(r'[Rr]oll', k):
                    vals.append(x)
                elif isinstance(x, (dict, list)):
                    walk(k, x)
        elif isinstance(v, dict):
            for kk, vv in v.items():
                walk(kk, vv)
    for k, v in r.items():
        walk(k, v)
    if r.get('reportId') == 'coinThrow':
        vals.append(-999)  # outcome-coded coin: matched by any d2 below
    return vals

def caller_key(frm, deep):
    method = frm.split('.')[-1].split(':')[0]
    steps = re.findall(r'<- (\w+)\.(\w+):\d+', deep)
    step = next((s for s, m in steps if s.startswith('Step') or 'Mechanic' in s or 'Util' in s), steps[0][0] if steps else '?')
    return '%s via %s' % (method, step)

ORDERED = '--ordered' in sys.argv
tot = collections.Counter(); hid = collections.Counter(); calls_by = collections.Counter()
games = sorted(glob.glob(sys.argv[1] + '/*_stderr.txt'))
for path in games:
    base = path[:-len('_stderr.txt')]
    windows = []  # (rng_calls, step)
    dice = []
    for line in open(path, encoding='utf-8', errors='replace'):
        m = JSTEP.search(line)
        if m:
            windows.append((int(m.group(2)), int(m.group(1))))
            continue
        m = DIE.search(line)
        if m:
            dice.append((int(m.group(1)), int(m.group(2)), int(m.group(3)), caller_key(m.group(4), m.group(5))))
    pools = collections.defaultdict(list)
    for line in open(base + '_reports.jsonl', encoding='utf-8'):
        r = json.loads(line)
        pools[r.get('i', 0)].extend(report_dice(r))
    bounds = sorted(windows)
    def window_of(pos):
        k = 0
        for rc, step in bounds:
            if pos > rc:
                k = step
            else:
                break
        return k
    g_tot = g_hid = 0
    for pos, sides, val, key in dice:
        w = window_of(pos)
        pool = pools[w]
        ok = False
        if ORDERED:
            # strict: the die must equal the NEXT unconsumed report die (skipping only coin marks)
            while pool and pool[0] == -999 and sides != 2:
                pool.pop(0)
            if pool and (pool[0] == val or (sides == 2 and pool[0] == -999)):
                pool.pop(0); ok = True
        elif val in pool:
            pool.remove(val); ok = True
        elif sides == 2 and -999 in pool:
            pool.remove(-999); ok = True
        tot[key] += 1; g_tot += 1
        if not ok:
            hid[key] += 1; g_hid += 1
    print('%-40s dice=%4d hidden=%4d (%.1f%%)' % (base.split('/')[-1].split('\\')[-1], g_tot, g_hid, 100.0 * g_hid / max(1, g_tot)))
T = sum(tot.values()); H = sum(hid.values())
print('\nALL: dice=%d recoverable=%d hidden=%d (%.1f%%)' % (T, T - H, H, 100.0 * H / T))
print('\nhidden by source (hidden/total):')
for k, n in hid.most_common():
    print('  %4d/%-4d %s' % (n, tot[k], k))
print('\nfully recovered sources:', len([k for k in tot if not hid[k]]), 'of', len(tot))
