# -*- coding: utf-8 -*-
"""Did the give CREATE the touchdown, or just decorate one that was already available?

Reads an `FFB_GIVE_TRACE` stderr stream. Each give carries the thrower's distance to the endzone
AT THE START OF HIS TEAM'S TURN, and his own reach (movement + 2 rushes). The split that matters:

  in_range   - the thrower could have walked it in himself from where he began the turn, so the
               give was a stylistic choice and the touchdown is not evidence the give earned it
  OUT OF RANGE - no amount of running by that player could have reached the endzone this turn, so
               a touchdown after the give is one the give MADE POSSIBLE

A give "converted" if a touchdown follows it before that team's turn ends.
"""
import collections
import sys

rows = []          # (kind, in_range, converted)
pending = []       # gives seen since the last turn end

for line in open(sys.argv[1], errors='ignore'):
    if line.startswith('GIVE '):
        d = dict(p.split('=', 1) for p in line.split()[1:] if '=' in p)
        pending.append([d.get('kind'), d.get('in_range') == 'true', False,
                        int(d.get('d_turn_start', -1)), int(d.get('reach', -1))])
    elif line.startswith('GTD '):
        # Every give still open this turn is credited: the touchdown came after all of them.
        for p in pending:
            p[2] = True
    elif line.startswith('GEND '):
        rows.extend(pending)
        pending = []
rows.extend(pending)

if not rows:
    print('no GIVE records found')
    sys.exit(1)

print('%d gives traced' % len(rows))
print()
hdr = '%-10s %8s %8s %8s   %s' % ('kind', 'gives', 'TDs', 'conv%', 'thrower could have run it in?')
print(hdr)
print('-' * len(hdr))

total_created = 0
for kind in ('handoff', 'pass'):
    for in_range, tag in ((True, 'YES - running was available'),
                          (False, 'NO  - give made it possible')):
        sel = [r for r in rows if r[0] == kind and r[1] == in_range]
        if not sel:
            continue
        conv = sum(1 for r in sel if r[2])
        if not in_range:
            total_created += conv
        print('%-10s %8d %8d %7.1f%%   %s'
              % (kind, len(sel), conv, 100.0 * conv / len(sel), tag))
    print()

allc = sum(1 for r in rows if r[2])
print('touchdowns after a give, total                : %d' % allc)
print('...of those, the thrower was OUT of range     : %d (%.0f%%)'
      % (total_created, 100.0 * total_created / max(allc, 1)))
print()
print('These are touchdowns that running could not have produced: the player holding the ball at')
print('the start of the turn could not reach the endzone, and the give is what put it there.')

# distance profile of the out-of-range gives, to show how far beyond reach they were
far = [r for r in rows if not r[1] and r[3] >= 0]
if far:
    over = sorted(r[3] - r[4] for r in far)
    print()
    print('how far BEYOND his own reach the thrower was (squares), out-of-range gives:')
    print('   min %d   median %d   p90 %d   max %d'
          % (over[0], over[len(over) // 2], over[int(len(over) * 0.9)], over[-1]))
