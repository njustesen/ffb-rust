import json, os, sys
from collections import Counter
from multiprocessing import Pool
from pathlib import Path
from agg import gate_dirs
OUT = Path('out3'); OUT.mkdir(exist_ok=True)

def do_gate(job):
    race, edition, scale, d = job
    tag = "%s__%s__%s" % (race, edition, scale)
    dst = OUT / (tag + '.json')
    if dst.exists():
        return tag + ' cached'
    apo = Counter(); added = Counter()
    for f in sorted(d.glob('seed_*_rust_events.jsonl')):
        with open(f, 'r', encoding='utf-8', errors='replace') as fh:
            for line in fh:
                if line.startswith('{"type":"apothecaryRoll"'):
                    apo['total'] += 1
                    if '"roll":null' not in line:
                        apo['rolled'] += 1
                    if '"new_state":null' not in line:
                        apo['healed'] += 1
                elif line.startswith('{"type":"playerAdded"'):
                    added[line.strip()[:200]] += 1
    dst.write_text(json.dumps({'race': race, 'edition': edition, 'scale': scale,
                               'apo': dict(apo), 'added': dict(added)}), encoding='utf-8')
    return '%s apo=%s added=%d' % (tag, dict(apo), sum(added.values()))

if __name__ == '__main__':
    jobs = list(gate_dirs())
    with Pool(10) as p:
        for i, m in enumerate(p.imap_unordered(do_gate, jobs), 1):
            print('[%3d] %s' % (i, m), flush=True)
