# -*- coding: utf-8 -*-
"""A/B one agent variant against the same agent with passing and hand-offs switched off.

Head-to-head, both colours, because self-play touchdown rate measures the pair and not the policy
(§23.3). Reports the result in standard errors so "it improved" is a claim with a number behind it.
"""
import collections
import glob
import json
import math
import os
import sys

B = 'C:/Users/Admin/AppData/Local/Temp/claude'


def rec(d, lab):
    C = {'home': collections.Counter(), 'away': collections.Counter()}
    n = 0
    for f in sorted(glob.glob(os.path.join(B, d, 'seed_*_%s_events.jsonl' % lab))):
        n += 1
        td = {'home': 0, 'away': 0}
        last = None
        for line in open(f, errors='ignore'):
            line = line.strip()
            if not line:
                continue
            try:
                e = json.loads(line)
            except ValueError:
                continue
            t = e.get('type')
            if t == 'playerAction':
                p = e.get('player_id') or ''
                last = 'home' if p.startswith('home') else ('away' if p.startswith('away') else last)
            elif t == 'touchdown':
                p = e.get('player_id') or ''
                s = 'home' if p.startswith('home') else ('away' if p.startswith('away') else last)
                if s:
                    td[s] += 1
            elif t == 'passRoll':
                s = last
                if s:
                    C[s]['pass'] += 1
            elif t == 'handOver':
                s = last
                if s:
                    C[s]['handoff'] += 1
        for s in ('home', 'away'):
            C[s]['td'] += td[s]
        if td['home'] > td['away']:
            C['home']['w'] += 1
            C['away']['l'] += 1
        elif td['away'] > td['home']:
            C['away']['w'] += 1
            C['home']['l'] += 1
        else:
            C['home']['d'] += 1
            C['away']['d'] += 1
    return C, n


def main():
    a_dir, b_dir, label = sys.argv[1], sys.argv[2], sys.argv[3]
    a, na = rec(a_dir, 'argmax')   # variant is HOME
    b, nb = rec(b_dir, 'argmax')   # variant is AWAY
    on = a['home'] + b['away']
    off = a['away'] + b['home']
    N = na + nb
    dec = on['w'] + on['l']
    se = math.sqrt(dec * 0.25) if dec else 0.0
    z = (on['w'] - dec / 2) / se if se else 0.0
    print('%s  —  %d games, both colours' % (label, N))
    print('   ON   %3d-%3d-%3d  TD/game %.2f   passes %.2f  hand-offs %.2f'
          % (on['w'], on['d'], on['l'], on['td'] / N, on['pass'] / N, on['handoff'] / N))
    print('   OFF  %3d-%3d-%3d  TD/game %.2f'
          % (off['w'], off['d'], off['l'], off['td'] / N))
    print('   decisive %d, ON won %d vs %.0f expected  ->  %+.2f SE' % (dec, on['w'], dec / 2, z))
    return z


if __name__ == '__main__':
    main()
