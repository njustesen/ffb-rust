# -*- coding: utf-8 -*-
"""Build a decision-replay viewer for one agent mode.

  python scripts/build_decision_replay.py <payload.json> <mode> <out.html>

The template is shared; only the framing text differs, because the two modes need different things
explained. In wide mode almost every Move prompt is a replay of a plan decided earlier, so the
viewer has to point back at the step that decided it. In deep mode nearly every prompt IS a
decision, and the thing worth showing is how small each one is.
"""
import io
import json
import statistics
import sys

TEMPLATE = 'docs/decision_replay_template.html'


def main():
    payload_path, mode, out = sys.argv[1], sys.argv[2].lower(), sys.argv[3]
    pay = io.open(payload_path, encoding='utf-8').read()
    D = json.loads(pay)
    steps = D['steps']
    dec = [s for s in steps if s['o']]
    ns = [s.get('nopt', len(s['o'])) for s in dec]
    mean_opts = sum(ns) / len(ns) if ns else 0
    med_opts = statistics.median(ns) if ns else 0

    if mode == 'deep':
        title = 'Deep Decision Replay'
        shape = ('a chain — pick the player, then that player’s action-and-target, then the destination as a <b>full path</b>, from the one search this mode pays for')

        note = (
            '<b>The economy is one search, not a smaller move.</b> Of this game’s %d prompts, '
            '<b>%d</b> were scored, at a mean of <b>%.1f</b> options each (median %d) against wide '
            'mode’s several hundred. Deep still moves a full path chosen from every reachable '
            'square: the difference is that it pathfinds ONCE, for the player it has already '
            'picked, where wide pathfinds up to sixteen of them to build one joint enumeration.'
            % (len(steps), len(dec), mean_opts, med_opts))
    else:
        title = 'Wide Decision Replay'
        shape = ('one draw from the whole joint action space &mdash; every player &times; action '
                 '&times; target &times; destination')
        note = (
            '<b>Most prompts are not decisions.</b> Of this game\'s %d prompts, only <b>%d</b> were '
            'scored; the other <b>%d</b> were answered straight from the activation plan or by a '
            'fixed rule. That is &sect;20.1 and &sect;20.2 working: once a plain move has been '
            'delivered, moving again reaches the same square, so the activation ends without '
            'scoring anything. Where a step shows no decision, the panel names the step that made '
            'it. Mean options per real decision: <b>%.0f</b> (median %d).'
            % (len(steps), len(dec), len(steps) - len(dec), mean_opts, med_opts))

    t = io.open(TEMPLATE, encoding='utf-8').read()
    t = t.replace('<title>Decision Replay</title>', '<title>%s</title>' % title)
    t = t.replace('<h1>Decision Replay</h1>', '<h1>%s</h1>' % title)
    t = t.replace(
        'seed 1 &middot; sampled vs sampled &middot; human v human &middot; bb2025',
        'seed 1 &middot; sampled vs sampled &middot; <b>%s mode</b> &middot; human v human &middot; bb2025'
        % mode)
    t = t.replace(
        "not just\n    what it picked.",
        'not just what it picked. One decision here is %s.' % shape)

    # swap the standing explanatory note for the mode-specific one
    start = t.index('<div class="note" id="note-replay">')
    end = t.index('</div>', t.index('</code>', start)) + len('</div>')
    t = t[:start] + '<div class="note" id="note-replay">\n    ' + note + '\n  </div>' + t[end:]

    io.open(out, 'w', encoding='utf-8').write(t.replace('__PAYLOAD__', pay))
    print('%s -> %s  (%d steps, %d decisions, mean %.1f options)'
          % (mode, out, len(steps), len(dec), mean_opts))


if __name__ == '__main__':
    main()
