# -*- coding: utf-8 -*-
"""Per-agent statistics for a heuristic-agent run, from its event dumps.

Everything is attributed by the `home_`/`away_` prefix on the acting player's id, then reported
per AGENT (i.e. per team) rather than per game, because the question being asked is always "how
good is this policy", not "how eventful was the pair".

Derived quantities that no single event carries:
  * knockdown       - the defender of a block fell down before the next block or activation
  * forced fumble   - that same defender was holding the ball, so a scatter followed the fall
  * casualty / KO   - an injury event inside a block's window, credited to the ATTACKER
  * pass completed  - an accurate passRoll whose catch by a team-mate then succeeded
"""
import collections
import glob
import json
import os
import sys


def side(pid):
    if not pid:
        return None
    return 'home' if pid.startswith('home') else ('away' if pid.startswith('away') else None)


def load(path):
    ev = []
    for line in open(path, errors='ignore'):
        line = line.strip()
        if not line:
            continue
        try:
            ev.append(json.loads(line))
        except ValueError:
            pass
    return ev


def report(d, lab):
    st = {s: collections.Counter() for s in ('home', 'away')}
    games = 0
    events = 0
    wins = collections.Counter()

    for f in sorted(glob.glob(os.path.join(d, 'seed_*_%s_events.jsonl' % lab))):
        ev = load(f)
        if not ev:
            continue
        games += 1
        events += len(ev)
        gs = {'home': 0, 'away': 0}

        for i, e in enumerate(ev):
            t = e.get('type')

            if t == 'touchdown':
                s = side(e.get('player_id'))
                if s:
                    gs[s] += 1
                    st[s]['touchdowns'] += 1

            elif t == 'blockRoll':
                a = side(e.get('attacker_id'))
                dfd = e.get('defender_id')
                if not a:
                    continue
                st[a]['blocks'] += 1
                # Window: up to the next block or activation. What the block caused is inside it.
                fell = False
                scattered = False
                for x in ev[i + 1:i + 25]:
                    xt = x.get('type')
                    if xt in ('blockRoll', 'playerAction', 'turnEnd'):
                        break
                    if xt == 'playerFellDown' and x.get('player_id') == dfd:
                        fell = True
                    elif xt == 'scatterBall' and fell:
                        scattered = True
                    elif xt == 'injury' and x.get('player_id') == dfd:
                        if x.get('was_cas'):
                            st[a]['casualties_inflicted'] += 1
                        elif x.get('was_ko'):
                            st[a]['kos_inflicted'] += 1
                if fell:
                    st[a]['knockdowns'] += 1
                if scattered:
                    st[a]['forced_fumbles'] += 1

            elif t == 'pushback':
                a = side(e.get('attacker_id'))
                if a:
                    st[a]['pushbacks'] += 1

            elif t == 'handOver':
                s = side(e.get('from_id'))
                if s:
                    st[s]['handoffs'] += 1

            elif t == 'passRoll':
                s = side(e.get('player_id'))
                if not s:
                    continue
                st[s]['passes'] += 1
                # PassOutcome::Complete is the on-target throw; Inaccurate/WildlyInaccurate
                # scatter, Fumble drops at the thrower's feet.
                res = str(e.get('result'))
                if res == 'Complete':
                    st[s]['passes_accurate'] += 1
                elif res == 'Fumble':
                    st[s]['passes_fumbled'] += 1
                for x in ev[i + 1:i + 12]:
                    if x.get('type') == 'catchRoll':
                        if side(x.get('player_id')) == s and x.get('success'):
                            st[s]['passes_completed'] += 1
                        break

            elif t == 'interceptionRoll':
                s = side(e.get('player_id'))
                if s:
                    st[s]['interception_attempts'] += 1
                    if e.get('success'):
                        st[s]['interceptions'] += 1

            elif t == 'foul':
                s = side(e.get('attacker_id'))
                if s:
                    st[s]['fouls'] += 1

            elif t == 'playerMoved':
                s = side(e.get('player_id'))
                if s:
                    st[s]['squares_moved'] += 1

            elif t in ('dodgeRoll', 'pickupRoll', 'catchRoll', 'goForItRoll'):
                s = side(e.get('player_id'))
                if not s:
                    continue
                name = {'dodgeRoll': 'dodge', 'pickupRoll': 'pickup',
                        'catchRoll': 'catch', 'goForItRoll': 'rush'}[t]
                st[s][name + '_att'] += 1
                if e.get('success'):
                    st[s][name + '_ok'] += 1

            elif t == 'playerAction':
                s = side(e.get('player_id'))
                if s:
                    st[s]['activations'] += 1

        if gs['home'] > gs['away']:
            wins['home'] += 1
        elif gs['away'] > gs['home']:
            wins['away'] += 1
        else:
            wins['draw'] += 1

    return games, events, st, wins


ROWS = [
    ('touchdowns', 'touchdowns'),
    ('activations', 'activations'),
    ('squares moved', 'squares_moved'),
    ('', ''),
    ('blocks thrown', 'blocks'),
    ('knockdowns caused', 'knockdowns'),
    ('pushbacks', 'pushbacks'),
    ('forced fumbles', 'forced_fumbles'),
    ('casualties inflicted', 'casualties_inflicted'),
    ('KOs inflicted', 'kos_inflicted'),
    ('fouls', 'fouls'),
    ('', ''),
    ('hand-offs', 'handoffs'),
    ('passes thrown', 'passes'),
    ('  ...on target', 'passes_accurate'),
    ('  ...fumbled', 'passes_fumbled'),
    ('  ...completed', 'passes_completed'),
    ('interception attempts', 'interception_attempts'),
    ('interceptions made', 'interceptions'),
    ('', ''),
    ('dodges', 'dodge_att'),
    ('  ...succeeded', 'dodge_ok'),
    ('rushes (GFI)', 'rush_att'),
    ('  ...succeeded', 'rush_ok'),
    ('pickups', 'pickup_att'),
    ('  ...succeeded', 'pickup_ok'),
    ('catches', 'catch_att'),
    ('  ...succeeded', 'catch_ok'),
]

if __name__ == '__main__':
    d, lab = sys.argv[1], sys.argv[2]
    names = sys.argv[3].split(',') if len(sys.argv) > 3 else ['home', 'away']
    games, events, st, wins = report(d, lab)
    if not games:
        print('no games found in %s' % d)
        sys.exit(1)

    print('%d games, %.0f events/game' % (games, float(events) / games))
    print('record: %d-%d-%d  (%s wins - draws - %s wins)'
          % (wins['home'], wins['draw'], wins['away'], names[0], names[1]))
    print()
    same = names[0] == names[1]
    if same:
        # Identical policies on both sides: the per-AGENT figure is the mean of the two, and the
        # home/away split is an artefact of who receives the kickoff, not of the policy.
        print('%-24s %12s' % ('per agent, per game', names[0]))
        print('%-24s %12s' % ('-' * 24, '-' * 12))
        for label, key in ROWS:
            if not label:
                print()
                continue
            v = (st['home'][key] + st['away'][key]) / (2.0 * games)
            print('%-24s %12.3f' % (label, v))
    else:
        print('%-24s %12s %12s' % ('per agent, per game', names[0], names[1]))
        print('%-24s %12s %12s' % ('-' * 24, '-' * 12, '-' * 12))
        for label, key in ROWS:
            if not label:
                print()
                continue
            print('%-24s %12.3f %12.3f'
                  % (label, st['home'][key] / float(games), st['away'][key] / float(games)))
