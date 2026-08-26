# -*- coding: utf-8 -*-
"""Render every decision of one game as a PNG, using the Java client's own graphics.

Geometry is taken from the Java client rather than guessed:
  LayoutSettings.BASE_SQUARE_SIZE = 30            -> one pitch square is 30px
  PlayerIconFactory: iconSize = sheet.width / 4   -> 28px icons, 4 columns
      column 0/1 = home standing/moving, 2/3 = away standing/moving
      row        = player.iconSetIndex
  pitch png is 782x452 = 26*30+2 by 15*30+2       -> a 1px border, so origin is (1,1)

Each square is tinted by the probability the agent gave to acting on it, normalised to the most
likely option in that decision so a near-certain choice and a wide-open one are both readable.
The chosen option gets a white outline.
"""
import io
import json
import os
import sys
import zipfile

from PIL import Image, ImageDraw, ImageFont

JAVA = os.environ.get('FFB_JAVA_ICONS', 'C:/Users/Admin/niels/ffb-rust/ffb-java/ffb-resources/src/main/resources/icons')
SQ = 30
ICON = 28
OX = OY = 1
COLS, ROWS = 26, 15
PANEL = 268
FOOT = 26

PS_PRONE, PS_STUNNED = 0x3, 0x4
DOWN = {PS_PRONE, PS_STUNNED}
OFF_PITCH = {0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xd}


def load_pitch(weather='nice'):
    z = zipfile.ZipFile(os.path.join(JAVA, 'cached/pitches/default.zip'))
    return Image.open(io.BytesIO(z.read('%s.png' % weather))).convert('RGBA')


_sheets = {}
substituted = set()


def _load(name):
    p = os.path.join(JAVA, 'cached/players/iconsets', name + '.png')
    return Image.open(p).convert('RGBA') if os.path.exists(p) else None


def sheet(pos):
    """'human.ogre' -> human_ogre.png.

    Three attempts, because the Rust roster's position keys and the Java icon filenames do not
    agree in every case: exact, then with the sub-name squashed
    ('human.halfling_hopeful' -> human_halflinghopeful), then the team's lineman as a stand-in.
    Star players added after this resource set was cut have no icon at all, so the stand-in keeps
    the board readable rather than dropping the player -- `substituted` reports which those are.
    """
    if pos in _sheets:
        return _sheets[pos]
    team, _, sub = pos.partition('.')
    team = team.lower()
    cands = [pos.replace('.', '_').lower(),
             '%s_%s' % (team, sub.replace('_', '').lower()),
             '%s_lineman' % team,
             '%s_%sman' % (team, team)]
    im = None
    for i, c in enumerate(cands):
        im = _load(c)
        if im is not None:
            if i >= 2:
                substituted.add(pos)
            break
    _sheets[pos] = im
    return im


def player_icon(pos, home, moving, icon_index=0):
    sh = sheet(pos)
    if sh is None:
        return None
    size = sh.width // 4
    col = (0 if not moving else 1) if home else (2 if not moving else 3)
    row = icon_index
    if (row + 1) * size > sh.height:
        row = 0
    return sh.crop((col * size, row * size, (col + 1) * size, (row + 1) * size))


def sq_px(x, y):
    return OX + x * SQ, OY + y * SQ


def anchors(rec):
    """Map each option to a board square. Returns (per_square {(x,y): [idx..]}, spatial idx set)."""
    ps = {p['id']: p for p in rec['snap']['ps']}
    out = {}
    for i, o in enumerate(rec['options']):
        a = o['action']
        t = a.get('type')
        c = None
        if t == 'move':
            path = a.get('path') or []
            if path:
                c = (path[-1]['x'], path[-1]['y'])
        elif t == 'activatePlayer':
            p = ps.get(a.get('player_id'))
            if p:
                c = (p['x'], p['y'])
        elif t in ('block', 'stab'):
            p = ps.get(a.get('defender_id'))
            if p:
                c = (p['x'], p['y'])
        elif t in ('selectPlayer', 'touchback'):
            p = ps.get(a.get('player_id'))
            if p:
                c = (p['x'], p['y'])
        elif t in ('pushTo', 'kickBall', 'pass', 'raidingPartyTarget', 'placePlayer'):
            k = a.get('coord')
            if k:
                c = (k['x'], k['y'])
        elif t == 'foul':
            p = ps.get(a.get('target_id'))
            if p:
                c = (p['x'], p['y'])
        elif t == 'handOff':
            p = ps.get(a.get('receiver_id'))
            if p:
                c = (p['x'], p['y'])
        if c and 0 <= c[0] < COLS and 0 <= c[1] < ROWS:
            out.setdefault(c, []).append(i)
    return out


def heat(p_norm):
    """Low probability -> cool blue, high -> hot amber. Alpha rises with probability."""
    p = max(0.0, min(1.0, p_norm))
    r = int(60 + 195 * p)
    g = int(90 + 110 * p)
    b = int(230 - 200 * p)
    a = int(60 + 150 * p)
    return (r, g, b, a)


def describe(a):
    t = a.get('type')
    if t == 'activatePlayer':
        d = a.get('block_defender_id')
        return '%s %s%s' % (a['player_id'], a['player_action'], (' -> ' + d) if d else '')
    if t == 'move':
        path = a.get('path') or []
        return 'move %d sq -> %d,%d' % (len(path), path[-1]['x'], path[-1]['y']) if path else 'move'
    if t == 'block':
        return 'block %s' % a.get('defender_id')
    if t == 'selectPlayer':
        return 'target %s' % a.get('player_id')
    if t == 'pushTo':
        c = a['coord']
        return 'push -> %d,%d' % (c['x'], c['y'])
    if t == 'blockChoice':
        return 'die #%d' % a.get('die_index', 0)
    if t == 'followUp':
        return 'follow up: %s' % ('yes' if a.get('follow_up') else 'no')
    if t == 'useReRoll':
        return 're-roll: %s' % ('yes' if a.get('use_reroll') else 'no')
    if t == 'useSkill':
        return 'skill %s: %s' % (a.get('skill_id'), 'yes' if a.get('use_skill') else 'no')
    if t == 'kickBall':
        c = a['coord']
        return 'kick -> %d,%d' % (c['x'], c['y'])
    if t == 'endTurn':
        return 'END TURN'
    if t == 'endPlayerAction':
        return 'end activation'
    return t


def team_key(im, dd, x0, y, sn, font):
    """Which colour is which team, cropped from the icons actually in use."""
    for k, (label, home) in enumerate([('HOME', True), ('AWAY', False)]):
        pos = next((p['pos'] for p in sn['ps'] if p['h'] == home), None)
        if pos:
            ic = player_icon(pos, home, False)
            if ic is not None:
                im.alpha_composite(ic.resize((18, 18)), (x0, y + k * 20))
        dd.text((x0 + 24, y + 3 + k * 20), label, font=font, fill=(200, 205, 215))


def render(rec, pitch, font, fontb, overlay=False):
    W = pitch.width + PANEL
    H = pitch.height + FOOT
    if overlay:
        # transparent over the pitch, opaque chrome everywhere else
        im = Image.new('RGBA', (W, H), (0, 0, 0, 0))
        ImageDraw.Draw(im).rectangle([pitch.width, 0, W, H], fill=(17, 19, 24, 255))
        ImageDraw.Draw(im).rectangle([0, pitch.height, W, H], fill=(17, 19, 24, 255))
    else:
        im = Image.new('RGBA', (W, H), (17, 19, 24, 255))
        im.paste(pitch, (0, 0))

    sn = rec['snap']
    ps = {p['id']: p for p in sn['ps']}
    acting = sn.get('act_id')

    # ---- probability overlay, under the players so icons stay readable ----
    per_sq = anchors(rec)
    pmax = max([o['p'] for o in rec['options']] or [1.0]) or 1.0
    chosen_sq = None
    ov = Image.new('RGBA', (pitch.width, pitch.height), (0, 0, 0, 0))
    d = ImageDraw.Draw(ov)
    for (x, y), idxs in per_sq.items():
        p = sum(rec['options'][i]['p'] for i in idxs)
        px, py = sq_px(x, y)
        d.rectangle([px, py, px + SQ - 1, py + SQ - 1], fill=heat(p / pmax))
        if rec['chosen'] in idxs:
            chosen_sq = (px, py)
    im.alpha_composite(ov)

    # ---- players ----
    for p in sn['ps']:
        base = p['bs'] & 0xff
        if base in OFF_PITCH or not (0 <= p['x'] < COLS and 0 <= p['y'] < ROWS):
            continue
        ic = player_icon(p['pos'], p['h'], p['id'] == acting)
        px, py = sq_px(p['x'], p['y'])
        if ic is None:
            dd = ImageDraw.Draw(im)
            col = (60, 130, 246, 255) if p['h'] else (239, 68, 68, 255)
            dd.ellipse([px + 4, py + 4, px + SQ - 5, py + SQ - 5], fill=col)
            continue
        if base in DOWN:
            ic = ic.rotate(90, expand=True)
        if not p['act']:
            ic = Image.blend(Image.new('RGBA', ic.size, (0, 0, 0, 0)), ic, 0.45)
        im.alpha_composite(ic, (px + (SQ - ic.width) // 2, py + (SQ - ic.height) // 2))
        if p['pos'] in substituted:
            ImageDraw.Draw(im).ellipse([px + SQ - 8, py + 2, px + SQ - 3, py + 7],
                                       fill=(255, 215, 90, 255))

    dd = ImageDraw.Draw(im)

    # ball
    if sn.get('bx') is not None and sn.get('bp'):
        bx, by = sq_px(sn['bx'], sn['by'])
        dd.ellipse([bx + 11, by + 11, bx + 19, by + 19], fill=(255, 240, 160, 255),
                   outline=(40, 30, 0, 255), width=2)

    # the option actually taken
    if chosen_sq:
        px, py = chosen_sq
        dd.rectangle([px, py, px + SQ - 1, py + SQ - 1], outline=(255, 255, 255, 255), width=3)

    # ---- side panel ----
    x0 = pitch.width + 10
    dd.text((x0, 8), 'step %d' % rec['i'], font=fontb, fill=(240, 240, 245))
    dd.text((x0, 26), '%s  %s' % (rec['side'].upper(), rec['prompt']), font=font, fill=(150, 200, 255))
    dd.text((x0, 44), 'half %d  turn %d   %d-%d' % (sn['hl'], sn['t'], sn['hs'], sn['aw']),
            font=font, fill=(170, 170, 180))
    dd.text((x0, 62), '%d options' % len(rec['options']), font=font, fill=(170, 170, 180))

    # A prompt with no options was not a decision: the activation plan answered it (§20.1/§20.2),
    # or a fixed rule did. Saying so is the honest rendering -- and it is most Move prompts.
    if not rec['options']:
        dd.text((x0, 86), 'no decision', font=fontb, fill=(255, 190, 110))
        dd.text((x0, 106), 'answered from the activation', font=font, fill=(170, 170, 180))
        dd.text((x0, 120), 'plan, or by a fixed rule.', font=font, fill=(170, 170, 180))
        dd.text((x0, 138), 'Nothing was scored, so there', font=font, fill=(120, 122, 130))
        dd.text((x0, 152), 'is no distribution to show.', font=font, fill=(120, 122, 130))
        team_key(im, dd, x0, pitch.height - 46, sn, font)
        dd.text((8, pitch.height + 6),
                'colour intensity = probability mass on that square   |   white outline = the '
                'option sampled   |   gold dot = icon substituted   |   graphics: FFB Java client',
                font=font, fill=(130, 135, 145))
        return im if overlay else im.convert('RGB')

    order = sorted(range(len(rec['options'])), key=lambda i: -rec['options'][i]['p'])
    y = 86
    for i in order[:20]:
        o = rec['options'][i]
        mark = '>' if i == rec['chosen'] else ' '
        col = (255, 255, 255) if i == rec['chosen'] else (185, 190, 200)
        bar = int(round(60 * (o['p'] / pmax)))
        dd.rectangle([x0, y + 4, x0 + bar, y + 10], fill=heat(o['p'] / pmax)[:3])
        dd.text((x0 + 64, y), '%s%4.1f%% %s' % (mark, 100 * o['p'], describe(o['action'])[:34]),
                font=font, fill=col)
        y += 15
        if y > pitch.height - 20:
            break
    if len(rec['options']) > 20:
        dd.text((x0, y), '  ... %d more' % (len(rec['options']) - 20), font=font,
                fill=(120, 120, 130))
    team_key(im, dd, x0, pitch.height - 46, sn, font)

    dd.text((8, pitch.height + 6),
            'colour intensity = probability mass on that square   |   white outline = the option '
            'sampled   |   gold dot = icon substituted   |   graphics: FFB Java client',
            font=font, fill=(130, 135, 145))
    return im if overlay else im.convert('RGB')


def main():
    src = sys.argv[1]
    outdir = sys.argv[2]
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    os.makedirs(outdir, exist_ok=True)
    recs = [json.loads(l) for l in open(src) if l.strip()]
    if limit:
        recs = recs[:limit]
    pitch = load_pitch('nice')
    try:
        font = ImageFont.truetype('consola.ttf', 11)
        fontb = ImageFont.truetype('consolab.ttf', 14)
    except Exception:
        font = ImageFont.load_default()
        fontb = font
    overlay = bool(os.environ.get('FFB_FRAME_OVERLAY'))
    if overlay:
        pitch.convert('RGB').save(os.path.join(outdir, 'pitch.jpg'), quality=78, optimize=True)
    ext = os.environ.get('FFB_FRAME_FMT', 'png')
    scale = float(os.environ.get('FFB_FRAME_SCALE', '1'))
    only_dec = os.environ.get('FFB_FRAME_DECISIONS_ONLY')
    n = 0
    for r in recs:
        if only_dec and not r['options']:
            continue
        im = render(r, pitch, font, fontb, overlay)
        if overlay:
            # Crop to the pitch: the panel is re-rendered as HTML from the same JSON, which is both
            # smaller and better (selectable text, a scrollable list) than baking it into a raster.
            im = im.crop((0, 0, pitch.width, pitch.height))
            im = im.quantize(colors=int(os.environ.get('FFB_FRAME_COLORS','96')), method=Image.FASTOCTREE, dither=Image.Dither.NONE).convert('RGBA')
            im.save(os.path.join(outdir, 'step_%04d.png' % r['i']), optimize=True)
            n += 1
            continue
        if scale != 1:
            im = im.resize((int(im.width * scale), int(im.height * scale)), Image.LANCZOS)
        if ext == 'jpg':
            im.save(os.path.join(outdir, 'step_%04d.jpg' % r['i']), quality=int(os.environ.get('FFB_FRAME_Q','72')), optimize=True, subsampling=0)
        else:
            im.save(os.path.join(outdir, 'step_%04d.png' % r['i']))
        n += 1
    print('wrote %d images to %s' % (n, outdir))


if __name__ == '__main__':
    main()
