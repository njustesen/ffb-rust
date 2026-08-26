# -*- coding: utf-8 -*-
"""Do passes and hand-offs actually produce touchdowns, and were they the only way to get one?

A ball-move "converts" if the throwing team scores before its turn ends — i.e. the receiver caught
it and ran it in, or the drive it rescued reached the endzone on the same turn.

"Running would not have scored" is taken from the AGENT's own classification (the note it recorded
when it chose the action), not re-derived here: SCORES NOW means the receiver was unactivated and
inside MA+2 of the line, RESCUE means the carrier could no longer reach the endzone in the turns
remaining but the receiver could.
"""
import collections
import glob
import json
import os
import re
import sys

B = 'C:/Users/Admin/AppData/Local/Temp/claude'


def side_of(pid):
    if not pid:
        return None
    return 'home' if pid.startswith('home') else ('away' if pid.startswith('away') else None)


def conversions(d, lab):
    """Per ball-move: (kind, turn, converted)."""
    out = []
    for f in sorted(glob.glob(os.path.join(B, d, 'seed_*_%s_events.jsonl' % lab))):
        ev = []
        for line in open(f, errors='ignore'):
            line = line.strip()
            if line:
                try:
                    ev.append(json.loads(line))
                except ValueError:
                    pass
        turn = 0
        last_actor = None
        for i, e in enumerate(ev):
            t = e.get('type')
            if t == 'turnEnd':
                turn = e.get('turn_nr', turn)
                continue
            if t == 'playerAction':
                last_actor = side_of(e.get('player_id'))
                continue
            if t not in ('passRoll', 'handOver'):
                continue
            who = side_of(e.get('player_id')) or last_actor
            kind = 'pass' if t == 'passRoll' else 'handoff'
            # did that team score before its turn ended?
            scored = False
            for x in ev[i + 1:]:
                xt = x.get('type')
                if xt == 'turnEnd':
                    break
                if xt == 'touchdown':
                    s = side_of(x.get('player_id'))
                    if s is None or s == who:
                        scored = True
                    break
            out.append((kind, turn, scored))
    return out


def notes(path):
    c = collections.Counter()
    for line in open(path, errors='ignore'):
        if not line.startswith('BM '):
            continue
        kind = 'pass' if 'Pass' in line.split()[1] else 'handoff'
        if 'SCORES NOW' in line:
            tag = 'scores now'
        elif 'RESCUE' in line:
            tag = 'rescue'
        elif 'drive lost' in line:
            tag = 'drive already lost'
        else:
            tag = 'could still run it in'
        c[(kind, tag)] += 1
    return c


if __name__ == '__main__':
    d, lab, games = sys.argv[1], sys.argv[2], int(sys.argv[3])
    rows = conversions(d, lab)
    print('%d games' % games)
    for kind in ('pass', 'handoff'):
        sel = [r for r in rows if r[0] == kind]
        conv = [r for r in sel if r[2]]
        late = [r for r in sel if r[1] >= 5]
        late_conv = [r for r in late if r[2]]
        print()
        print('%s: %d thrown (%.2f/game)' % (kind.upper(), len(sel), len(sel) / games))
        if sel:
            print('   led to a touchdown that turn : %d (%.0f%%)'
                  % (len(conv), 100.0 * len(conv) / len(sel)))
            print('   thrown in turn 5-8           : %d (%.0f%%)'
                  % (len(late), 100.0 * len(late) / len(sel)))
            if late:
                print('   ...of those, converted       : %d (%.0f%%)'
                      % (len(late_conv), 100.0 * len(late_conv) / len(late)))
    if len(sys.argv) > 4:
        print()
        print('why the agent chose them (its own classification):')
        for (k, tag), n in sorted(notes(sys.argv[4]).items(), key=lambda x: -x[1]):
            print('   %-8s %-24s %d' % (k, tag, n))
