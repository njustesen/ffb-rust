# -*- coding: utf-8 -*-
"""Pack one game's decisions into a self-contained viewer payload.

The pitch and the icon strips are identical in every frame, so they ship once; only the per-step
state and the option distribution vary. Everything is drawn client-side on a canvas, which is what
makes 451 steps fit in an artifact at all -- a rasterised frame per step is 11-46MB.
"""
import base64
import io
import json
import os
import sys
import zipfile

from PIL import Image

JAVA = 'C:/Users/Admin/niels/ffb-rust/ffb-java/ffb-resources/src/main/resources/icons'
SQ = 30


def b64(data, mime):
    return 'data:%s;base64,%s' % (mime, base64.b64encode(data).decode())


def pitch_uri(weather='nice', q=70):
    z = zipfile.ZipFile(os.path.join(JAVA, 'cached/pitches/default.zip'))
    im = Image.open(io.BytesIO(z.read('%s.png' % weather))).convert('RGB')
    buf = io.BytesIO()
    im.save(buf, 'JPEG', quality=q, optimize=True)
    return b64(buf.getvalue(), 'image/jpeg'), im.size


def icon_strip(pos):
    """Row 0 of the sheet: [home, home-moving, away, away-moving], each iconSize square."""
    team, _, sub = pos.partition('.')
    team = team.lower()
    for c in [pos.replace('.', '_').lower(),
              '%s_%s' % (team, sub.replace('_', '').lower()),
              '%s_lineman' % team]:
        p = os.path.join(JAVA, 'cached/players/iconsets', c + '.png')
        if os.path.exists(p):
            sh = Image.open(p).convert('RGBA')
            size = sh.width // 4
            strip = sh.crop((0, 0, 4 * size, size))
            buf = io.BytesIO()
            strip.save(buf, 'PNG', optimize=True)
            return b64(buf.getvalue(), 'image/png'), size, (c != pos.replace('.', '_').lower())
    return None, 0, True


def anchor(a, ps):
    t = a.get('type')
    if t == 'move':
        p = a.get('path') or []
        return [p[-1]['x'], p[-1]['y']] if p else None
    if t == 'activatePlayer':
        q = ps.get(a.get('player_id'))
    elif t in ('block', 'stab'):
        q = ps.get(a.get('defender_id'))
    elif t in ('selectPlayer', 'touchback'):
        q = ps.get(a.get('player_id'))
    elif t == 'foul':
        q = ps.get(a.get('target_id'))
    elif t == 'handOff':
        q = ps.get(a.get('receiver_id'))
    elif t in ('pushTo', 'kickBall', 'pass', 'raidingPartyTarget', 'placePlayer'):
        c = a.get('coord')
        return [c['x'], c['y']] if c else None
    else:
        return None
    return [q['x'], q['y']] if q else None


def describe(a):
    t = a.get('type')
    if t == 'activatePlayer':
        d = a.get('block_defender_id')
        return '%s %s%s' % (a['player_id'], a['player_action'], (' \u2192 ' + d) if d else '')
    if t == 'move':
        p = a.get('path') or []
        return 'move %d sq \u2192 %d,%d' % (len(p), p[-1]['x'], p[-1]['y']) if p else 'move'
    if t == 'block':
        return 'block %s' % a.get('defender_id')
    if t == 'selectPlayer':
        return 'target %s' % a.get('player_id')
    if t == 'pushTo':
        c = a['coord']
        return 'push \u2192 %d,%d' % (c['x'], c['y'])
    if t == 'blockChoice':
        return 'take die #%d' % a.get('die_index', 0)
    if t == 'followUp':
        return 'follow up: %s' % ('yes' if a.get('follow_up') else 'no')
    if t == 'useReRoll':
        return 're-roll: %s' % ('yes' if a.get('use_reroll') else 'no')
    if t == 'useSkill':
        return 'skill %s: %s' % (a.get('skill_id'), 'yes' if a.get('use_skill') else 'no')
    if t == 'intercept':
        return 'intercept: %s' % ('yes' if a.get('attempt') else 'no')
    if t == 'kickBall':
        c = a['coord']
        return 'kick \u2192 %d,%d' % (c['x'], c['y'])
    if t == 'coinChoice':
        return 'heads' if a.get('heads') else 'tails'
    if t == 'receiveChoice':
        return 'receive' if a.get('receive') else 'kick'
    if t == 'endTurn':
        return 'END TURN'
    if t == 'endPlayerAction':
        return 'end activation'
    if t == 'touchback':
        return 'give ball to %s' % a.get('player_id')
    return t


def main():
    src, out = sys.argv[1], sys.argv[2]
    recs = [json.loads(l) for l in open(src) if l.strip()]

    positions = sorted({p['pos'] for r in recs for p in r['snap']['ps']})
    icons, subs = {}, []
    for pos in positions:
        uri, size, was_sub = icon_strip(pos)
        if uri:
            icons[pos] = {'d': uri, 's': size}
        if was_sub:
            subs.append(pos)

    pitch, (pw, ph) = pitch_uri()

    steps = []
    for r in recs:
        sn = r['snap']
        ps = {p['id']: p for p in sn['ps']}
        opts = []
        for i, o in enumerate(r['options']):
            opts.append({
                'l': describe(o['action']),
                'p': round(o['p'], 5),
                'w': round(o['w'], 4),
                'y': o['why'],
                'a': anchor(o['action'], ps),
            })
        steps.append({
            'i': r['i'],
            'side': r['side'],
            'pr': r['prompt'],
            'ch': r['chosen'],
            'hl': sn['hl'], 't': sn['t'], 'hs': sn['hs'], 'aw': sn['aw'],
            'act': sn.get('act_id'),
            'b': [sn['bx'], sn['by']] if (sn.get('bx') is not None and sn.get('bp')) else None,
            'ps': [[p['id'], p['x'], p['y'], 1 if p['h'] else 0, p['pos'],
                    p['bs'] & 0xff, 1 if p['act'] else 0, p['nm']] for p in sn['ps']],
            'o': opts,
        })

    payload = {'pitch': pitch, 'pw': pw, 'ph': ph, 'sq': SQ,
               'icons': icons, 'subs': subs, 'steps': steps}
    js = json.dumps(payload, separators=(',', ':'))
    open(out, 'w', encoding='utf-8').write(js)
    print('payload %.2f MB  (%d steps, %d icon strips, %d substituted)'
          % (len(js) / 1e6, len(steps), len(icons), len(subs)))
    dec = sum(1 for s in steps if s['o'])
    print('  %d real decisions, %d replayed/fixed' % (dec, len(steps) - dec))


if __name__ == '__main__':
    main()
