#!/usr/bin/env python
"""Build the standalone census report page for one sweep.

usage: build_report.py <census_dir> <out.html>      (census_dir = agg.py output, e.g. out_st)

Reads the per-gate census JSONs, the roster-side inventory (inventory.py -> inventory.json,
fielded.json in this directory) and the Rust enum catalogs, and writes one self-contained HTML
page: aggregated and per-gate statistics on events, actions, mechanics, kick-off results,
skills, injuries, weather, prayers -- and, for every catalog, what the sweep never reached.
"""
import glob
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
CENSUS = Path(sys.argv[1] if len(sys.argv) > 1 else HERE / 'out_st')
OUT = Path(sys.argv[2] if len(sys.argv) > 2 else REPO / 'docs' / 'setup_census_report.html')


def norm(x):
    return re.sub(r'[^a-z0-9]', '', x.lower())


def enum_variants(path, enum_name):
    src = (REPO / path).read_text(encoding='utf-8')
    body = src.split('pub enum %s {' % enum_name, 1)[1].split('\n}', 1)[0]
    out = []
    for line in body.split('\n'):
        s = line.strip()
        if not s or s.startswith('//') or s.startswith('#'):
            continue
        m = re.match(r'([A-Za-z_][A-Za-z0-9_]*)\s*(?:=\s*\d+)?\s*(?:\{|\(|,)', s)
        if m:
            out.append(m.group(1))
    return out


def camel(n):
    return n[0].lower() + n[1:]


# ── catalogs ─────────────────────────────────────────────────────────────────
SKILLS = enum_variants('crates/ffb-model/src/enums/skill_id.rs', 'SkillId')
EVENTS = [camel(v) for v in enum_variants('crates/ffb-model/src/events/game_event.rs', 'GameEvent')]
ACTIONS = enum_variants('crates/ffb-model/src/enums/player.rs', 'PlayerAction')
KICKOFF = [v for v in enum_variants('crates/ffb-model/src/enums/kickoff_result.rs', 'KickoffResult')] \
    if (REPO / 'crates/ffb-model/src/enums/kickoff_result.rs').exists() else []
if not KICKOFF:
    for f in (REPO / 'crates/ffb-model/src/enums').glob('*.rs'):
        if 'pub enum KickoffResult {' in f.read_text(encoding='utf-8'):
            KICKOFF = enum_variants(f.relative_to(REPO), 'KickoffResult')
            break
WEATHER = []
for f in (REPO / 'crates/ffb-model/src/enums').glob('*.rs'):
    if 'pub enum Weather {' in f.read_text(encoding='utf-8'):
        WEATHER = enum_variants(f.relative_to(REPO), 'Weather')
        break
SI_KIND = enum_variants('crates/ffb-model/src/enums/injury.rs', 'SeriousInjuryKind')
SI_ED = {e: enum_variants('crates/ffb-model/src/%s/serious_injury.rs' % e, 'SeriousInjury')
         for e in ('bb2016', 'bb2020', 'bb2025')}
PRAYERS = {e: enum_variants('crates/ffb-model/src/inducement/%s/prayer.rs' % e, 'Prayer')
           for e in ('bb2020', 'bb2025')}

# Kick-off tables per edition (the enum is shared; each edition rolls 11 of them).
KICKOFF_ED = {
    'bb2016': ['GetTheRef', 'Riot', 'PerfectDefence', 'HighKick', 'CheeringFans', 'WeatherChange',
               'BrilliantCoaching', 'QuickSnap', 'Blitz', 'ThrowARock', 'PitchInvasion'],
    'bb2020': ['GetTheRef', 'TimeOut', 'SolidDefence', 'HighKick', 'CheeringFans', 'WeatherChange',
               'BrilliantCoaching', 'QuickSnap', 'Blitz', 'OficiousRef', 'PitchInvasion'],
    'bb2025': ['GetTheRef', 'TimeOut', 'SolidDefence', 'HighKick', 'CheeringFans', 'WeatherChange',
               'BrilliantCoaching', 'QuickSnap', 'Charge', 'DodgySnack', 'PitchInvasion'],
}
KICKOFF_LABEL = {
    'GetTheRef': 'Get the Ref', 'Riot': 'Riot', 'PerfectDefence': 'Perfect Defence', 'HighKick': 'High Kick',
    'CheeringFans': 'Cheering Fans', 'WeatherChange': 'Changing Weather', 'BrilliantCoaching': 'Brilliant Coaching',
    'QuickSnap': 'Quick Snap', 'Blitz': 'Blitz!', 'ThrowARock': 'Throw a Rock', 'PitchInvasion': 'Pitch Invasion',
    'TimeOut': 'Time-out', 'SolidDefence': 'Solid Defence', 'OficiousRef': 'Officious Ref', 'Charge': 'Charge!',
    'DodgySnack': 'Dodgy Snack',
}

# ── data ─────────────────────────────────────────────────────────────────────
G = [json.load(open(f, encoding='utf-8')) for f in sorted(glob.glob(str(CENSUS / '*.json')))]
G.sort(key=lambda d: (d['race'], d['edition'], float(d['scale'])))
INV = json.load(open(HERE / 'inventory.json', encoding='utf-8'))
FIELDED = json.load(open(HERE / 'fielded.json', encoding='utf-8'))['fielded']   # display name -> cells


def cnt(key, gates=G):
    c = Counter()
    for d in gates:
        v = d.get(key, {})
        if isinstance(v, dict):
            for k, x in v.items():
                if isinstance(x, int):
                    c[k] += x
    return c


def rolls(gates=G):
    r = defaultdict(Counter)
    for d in gates:
        for k, v in d['rolls'].items():
            for kk, x in v.items():
                r[k][kk] += x
    return r


FOLD = {
    "Move": "Move", "HandOverMove": "Move", "PassMove": "Move", "FoulMove": "Move",
    "GazeMove": "Move", "KickTeamMateMove": "Move", "ThrowTeamMateMove": "Move",
    "PuntMove": "Move", "PutridRegurgitationMove": "Move",
    "Block": "Block", "PutridRegurgitationBlock": "Block", "KickEmBlock": "Block",
    "Blitz": "Blitz", "BlitzSelect": "Blitz", "BlitzMove": "Blitz",
    "PutridRegurgitationBlitz": "Blitz", "KickEmBlitz": "Blitz",
    "Gaze": "HypnoticGaze", "GazeSelect": "HypnoticGaze", "AutoGazeZoat": "HypnoticGaze",
}
REQUIRED = ['action Move', 'action Block', 'action Blitz', 'action Foul', 'action Pass',
            'action HandOver', 'dodge success', 'dodge failure', 'pickup success',
            'pickup failure', 'catch success', 'catch failure', 'ball scatters', 'throw-ins',
            'pass rolls', 'block 1 die', 'block 2 dice', 'block 2 dice against',
            'block result Skull', 'block result BothDown', 'block result Pushback',
            'block result PowPushback', 'block result Pow', 'pushbacks', 'players fell',
            'armor held', 'stunned', 'KO', 'casualty (d16)', 'fouls', 'argue the call',
            'players ejected', 'half starts', 'kickoff events']


def checklist(d):
    a = Counter()
    for k, v in d['actions'].items():
        a[FOLD.get(k, k)] += v
    t = d['types']
    r = d['rolls']
    inj = d['injuries']
    g = lambda k: t.get(k, 0)
    roll = lambda k, f: r.get(k, {}).get(f, 0)
    stunned = inj.get('total', 0) - (inj.get('armor_only', 0) + inj.get('ko', 0) + inj.get('cas', 0))
    bd, br = d['block_dice'], d['block_result']
    return {
        'action Move': a['Move'], 'action Block': a['Block'], 'action Blitz': a['Blitz'],
        'action Foul': a['Foul'], 'action Pass': a['Pass'], 'action HandOver': a['HandOver'],
        'dodge success': roll('dodgeRoll', 'success'),
        'dodge failure': roll('dodgeRoll', 'total') - roll('dodgeRoll', 'success'),
        'pickup success': roll('pickupRoll', 'success'),
        'pickup failure': roll('pickupRoll', 'total') - roll('pickupRoll', 'success'),
        'catch success': roll('catchRoll', 'success'),
        'catch failure': roll('catchRoll', 'total') - roll('catchRoll', 'success'),
        'ball scatters': g('scatterBall') + g('ballScattered'), 'throw-ins': g('throwIn'),
        'pass rolls': roll('passRoll', 'total'),
        'block 1 die': bd.get('1', 0), 'block 2 dice': bd.get('2', 0),
        'block 2 dice against': bd.get('-2', 0),
        'block result Skull': br.get('Skull', 0), 'block result BothDown': br.get('BothDown', 0),
        'block result Pushback': br.get('Pushback', 0),
        'block result PowPushback': br.get('PowPushback', 0), 'block result Pow': br.get('Pow', 0),
        'pushbacks': g('pushback'), 'players fell': g('playerFellDown'),
        'armor held': inj.get('armor_only', 0), 'stunned': stunned,
        'KO': inj.get('ko', 0), 'casualty (d16)': inj.get('cas', 0),
        'fouls': g('foul'), 'argue the call': roll('argueTheCall', 'total'),
        'players ejected': g('playerEjected'),
        'half starts': g('startHalf'), 'kickoff events': sum(d['kickoff'].values()),
    }


# ── skills: what the stream can prove ────────────────────────────────────────
T = cnt('types')
A = cnt('actions')
SU = Counter({SKILLS[int(k)]: v for k, v in cnt('skill_used').items()})
SD = Counter({SKILLS[int(k)]: v for k, v in cnt('skill_declined').items()})
SUn = Counter()
for k, v in SU.items():
    SUn[norm(k)] += v
SDn = Counter()
for k, v in SD.items():
    SDn[norm(k)] += v

# dedicated telemetry (other than SkillUse) per fielded skill, keyed by normalised name
EV = {
    'loner': ('event lonerRoll', T['lonerRoll']), 'pro': ('event proRoll', T['proRoll']),
    'oldpro': ('event proRoll', T['proRoll']),
    'regeneration': ('event regenerationRoll', T['regenerationRoll']),
    'rightstuff': ('event rightStuffRoll', T['rightStuffRoll']),
    'throwteammate': ('action ThrowTeamMate', A['ThrowTeamMate']),
    'alwayshungry': ('event alwaysHungry', T['alwaysHungry']),
    'bonehead': ('event confusionRoll', T['confusionRoll']),
    'reallystupid': ('event confusionRoll', T['confusionRoll']),
    'takeroot': ('event confusionRoll', T['confusionRoll']),
    'animalsavagery': ('event animalSavagery', T['animalSavagery']),
    'unchannelledfury': ('event animalSavagery', T['animalSavagery']),
    'wildanimal': ('event animalSavagery', T['animalSavagery']),
    'bloodlust': ('event bloodLustRoll', T['bloodLustRoll']),
    'foulappearance': ('event foulAppearanceRoll', T['foulAppearanceRoll']),
    'dauntless': ('event dauntlessRoll', T['dauntlessRoll']),
    'animosity': ('event animosityRoll', T['animosityRoll']),
    'leap': ('event jumpRoll', T['jumpRoll']), 'verylonglegs': ('event jumpRoll', T['jumpRoll']),
    'pogo': ('event jumpRoll', T['jumpRoll']), 'pogostick': ('event jumpRoll', T['jumpRoll']),
    'jumpup': ('event jumpUpRoll', T['jumpUpRoll']),
    'hypnoticgaze': ('event hypnoticGazeRoll', T['hypnoticGazeRoll']),
    'balefulhex': ('event balefulHexRoll', T['balefulHexRoll']),
    'lookintomyeyes': ('event lookIntoMyEyesRoll', T['lookIntoMyEyesRoll']),
    'weepingdagger': ('event weepingDaggerRoll', T['weepingDaggerRoll']),
    'breathefire': ('event breatheFireRoll', T['breatheFireRoll']),
    'projectilevomit': ('event projectileVomitRoll', T['projectileVomitRoll']),
    'chainsaw': ('event chainsawRoll', T['chainsawRoll']),
    'ballandchain': ('event escapeRoll', T['escapeRoll']),
    'safethrow': ('event safeThrowRoll', T['safeThrowRoll']),
    'bombardier': ('action ThrowBomb', A['ThrowBomb']),
    'hitandrun': ('event hitAndRun', T['hitAndRun']),
    'kickteammate': ('action KickTeamMate', A['KickTeamMate']),
    'allyoucaneat': ('event allYouCanEatRoll', T['allYouCanEatRoll']),
    'swoop': ('event swoopPlayer', T['swoopPlayer']),
    'pickmeup': ('event pickMeUpRoll', T['pickMeUpRoll']),
    'swarming': ('event swarmingPlayersRoll', T['swarmingPlayersRoll']),
    'pilingon': ('event pilingOn', T['pilingOn']), 'piledriver': ('event pilingOn', T['pilingOn']),
    'fumblerooskie': ('event fumblerooskie', T['fumblerooskie']),
    'fumblerooski': ('event fumblerooskie', T['fumblerooskie']),
    'leader': ('event leader', T['leader']),
    'teamcaptain': ('event teamCaptainRoll', T['teamCaptainRoll']),
    'pumpupthecrowd': ('event pumpUpTheCrowdReRoll', T['pumpUpTheCrowdReRoll']),
    'passblock': ('event passBlock', T['passBlock']),
    'thenistartedblastin': ('event thenIStartedBlastin', T['thenIStartedBlastin']),
    'beerbarrelbash': ('event kegThrow', T['kegThrow']),
    'wisdomofthewhitedwarf': ('action WisdomOfTheWhiteDwarf', A['WisdomOfTheWhiteDwarf']),
    'treacherous': ('action Treacherous', A['Treacherous']),
    'raidingparty': ('action RaidingParty', A['RaidingParty']),
    'blackink': ('action BlackInk', A['BlackInk']),
    'catchoftheday': ('action CatchOfTheDay', A['CatchOfTheDay']),
    'furiousoutburst': ('action FuriousOutburst', A['FuriousOutburst']),
    'punt': ('action Punt', A['Punt']),
    'multipleblock': ('action MultipleBlock', A['MultipleBlock']),
    'hailmarypass': ('action HailMaryPass', A['HailMaryPass']),
    'excusemeareyouazoat': ('action AutoGazeZoat', A['AutoGazeZoat']),
    'stab': ('action Stab', A['Stab']),
    'secretweapon': ('event secretWeaponBan', T['secretWeaponBan']),
    'timmmber': ('event standUpRoll', T['standUpRoll']),
    'sprint': ('event goForItRoll (3rd rush)', 0),
    'kick': ('event kickoffScatter', 0),
}

skill_rows = []
for name, cells in FIELDED.items():
    n = norm(name)
    su = SUn.get(n, 0)
    sd = SDn.get(n, 0)
    ev = EV.get(n)
    if su or sd:
        cls = 'used'
        evidence = 'SkillUse'
        count = su + sd
    elif ev and ev[1] > 0:
        cls = 'proven'
        evidence = ev[0]
        count = ev[1]
    elif ev:
        cls = 'silent'
        evidence = ev[0]
        count = 0
    else:
        cls = 'invisible'
        evidence = ''
        count = 0
    eds = sorted({c.rsplit('__', 1)[1] for c in cells})
    skill_rows.append({'skill': name, 'cells': len(cells), 'editions': eds, 'cls': cls,
                       'evidence': evidence, 'count': count, 'used': su, 'declined': sd})
skill_rows.sort(key=lambda r: (-r['count'], -r['cells'], r['skill']))

# skills that raised SkillUse but are not on any drafted roster (granted skills, e.g. by prayers)
fielded_norm = {norm(k) for k in FIELDED}
extra_used = [{'skill': k, 'count': v} for k, v in SU.items() if norm(k) not in fielded_norm]

# ── per-edition and per-gate ────────────────────────────────────────────────
def ed_gates(e):
    return [d for d in G if d['edition'] == e]


editions = ['bb2016', 'bb2020', 'bb2025']
by_ed = {}
for e in editions:
    gs = ed_gates(e)
    t = cnt('types', gs)
    inj = cnt('injuries', gs)
    r = rolls(gs)
    by_ed[e] = {
        'gates': len(gs), 'games': sum(d['games'] for d in gs), 'events': sum(t.values()),
        'td': t['touchdown'], 'blocks': t['blockRoll'], 'dodges': r['dodgeRoll']['total'],
        'fouls': t['foul'], 'passes': r['passRoll']['total'], 'handoffs': t['handOver'],
        'ko': inj['ko'], 'cas': inj['cas'], 'dead': inj['dead'], 'skillUse': t['skillUse'],
        'kickoff': dict(cnt('kickoff', gs)), 'weather': dict(cnt('weather', gs)),
        'serious': dict(cnt('serious_injury', gs)), 'prayers': dict(cnt('prayers', gs)),
        'types': dict(t), 'actions': dict(cnt('actions', gs)),
        'rolls': {k: dict(v) for k, v in r.items()},
    }

gate_rows = []
for d in G:
    t, inj, r = d['types'], d['injuries'], d['rolls']
    ck = checklist(d)
    short = [k for k in REQUIRED if ck[k] == 0]
    gate_rows.append({
        'race': d['race'], 'edition': d['edition'], 'scale': d['scale'], 'games': d['games'],
        'events': sum(t.values()), 'td': t.get('touchdown', 0),
        'blocks': t.get('blockRoll', 0), 'dodges': r.get('dodgeRoll', {}).get('total', 0),
        'fouls': t.get('foul', 0), 'passes': r.get('passRoll', {}).get('total', 0),
        'handoffs': t.get('handOver', 0), 'ko': inj.get('ko', 0), 'cas': inj.get('cas', 0),
        'dead': inj.get('dead', 0), 'skillUse': t.get('skillUse', 0),
        'kickoffs': sum(d['kickoff'].values()), 'variants': len(t), 'short': short,
    })

# per race x edition summary (all three scales folded)
race_ed = defaultdict(lambda: Counter())
for d in G:
    k = (d['race'], d['edition'])
    race_ed[k]['games'] += d['games']
    race_ed[k]['td'] += d['types'].get('touchdown', 0)
    race_ed[k]['gates'] += 1
    race_ed[k]['td0'] += d['types'].get('touchdown', 0) if d['scale'] == '0' else 0
    race_ed[k]['g0'] += d['games'] if d['scale'] == '0' else 0
race_rows = []
for race in sorted({d['race'] for d in G}):
    row = {'race': race}
    for e in editions:
        c = race_ed.get((race, e))
        row[e] = None if not c else {'gates': c['gates'], 'td_pg': round(c['td'] / c['games'], 2),
                                     'td0_pg': round(c['td0'] / c['g0'], 2) if c['g0'] else None}
    race_rows.append(row)

# ── assemble ────────────────────────────────────────────────────────────────
R = rolls()
data = {
    'meta': {
        'sweep': '2026-09-13 heuristic-setup sweep', 'gates': len(G),
        'games': sum(d['games'] for d in G), 'events': sum(T.values()),
        'squads': len(INV['squads']), 'races': len({d['race'] for d in G}),
        'parity': '330/330',
    },
    'by_ed': by_ed,
    'types': dict(T), 'event_catalog': EVENTS,
    'actions': dict(A), 'action_catalog': ACTIONS,
    'kickoff': dict(cnt('kickoff')), 'kickoff_ed': KICKOFF_ED, 'kickoff_label': KICKOFF_LABEL,
    'kickoff_catalog': KICKOFF,
    'weather': dict(cnt('weather')), 'weather_catalog': WEATHER,
    'prayers': dict(cnt('prayers')), 'prayer_catalog': PRAYERS,
    'block_dice': dict(cnt('block_dice')), 'block_result': dict(cnt('block_result')),
    'block_own_choice': dict(cnt('block_own_choice')),
    'block_rerolled': sum(d['block_rerolled'] for d in G),
    'push_len': dict(cnt('push_len')),
    'pass_dist': dict(cnt('pass_dist')), 'pass_res': dict(cnt('pass_res')),
    'rolls': {k: dict(v) for k, v in R.items()},
    'injuries': dict(cnt('injuries')), 'serious': dict(cnt('serious_injury')),
    'serious_kind_catalog': SI_KIND, 'serious_ed_catalog': SI_ED,
    'reroll_src': dict(cnt('reroll_src')), 'notes': dict(cnt('notes')),
    'inducements': dict(cnt('inducements')), 'cards': dict(cnt('cards')), 'spells': dict(cnt('spells')),
    'skills': skill_rows, 'skills_extra_used': extra_used,
    'skill_enum_size': len(SKILLS), 'skill_declined': dict(SD),
    'squads': {k: {'edition': v['edition'], 'race': v['race'], 'players': v['players'],
                   'rerolls': v['rerolls'], 'apothecaries': v['apothecaries'],
                   'special_rules': v['special_rules'] or [], 'skills': sorted(v['skills'])}
               for k, v in INV['squads'].items()},
    'gates': gate_rows, 'race_rows': race_rows, 'required': REQUIRED,
}

# ── the REPORT census (report_census.py), when one has been built ────────────
REPORT_CENSUS = REPO / 'docs' / 'report_census.json'
if REPORT_CENSUS.exists():
    rc = json.load(open(REPORT_CENSUS, encoding='utf-8'))
    data['reports'] = rc

    # Re-class the skills the EVENT stream cannot see: a skill whose name appears in a report's
    # modifier list IS observable, just not as an event. Modifier names carry a count prefix
    # ("2 Tacklezones", "1 Prehensile Tail") and sometimes a qualifier ("Break Tackle ST 5+").
    def mod_key(name):
        t = re.sub(r'^\d+\s+', '', name)
        t = norm(t)
        return t[:-1] if t.endswith('s') else t

    seen_mods = {}
    for group, counts in rc['modifiers'].items():
        for name, n in counts.items():
            k = mod_key(name)
            seen_mods.setdefault(k, [0, name])
            seen_mods[k][0] += n
    for row in data['skills']:
        if row['cls'] not in ('invisible', 'silent'):
            continue
        k = mod_key(row['skill'])
        hit = seen_mods.get(k)
        if hit is None:
            hit = next((v for kk, v in seen_mods.items() if kk.startswith(k) and len(k) > 4), None)
        if hit:
            row['cls'] = 'modifier'
            row['evidence'] = 'modifier "%s"' % hit[1]
            row['count'] = hit[0]
    data['skills'].sort(key=lambda r: (-r['count'], -r['cells'], r['skill']))

TEMPLATE = (HERE / 'report_template.html').read_text(encoding='utf-8')
html = TEMPLATE.replace('/*__DATA__*/', 'const DATA = ' + json.dumps(data, separators=(',', ':')) + ';')
OUT.write_text(html, encoding='utf-8', newline='\n')
print('wrote', OUT, len(html) // 1024, 'KB;', 'skills fielded', len(data['skills']),
      Counter(r['cls'] for r in data['skills']))
print('events seen %d/%d, actions %d/%d' % (len(T), len(EVENTS), len(A), len(ACTIONS)))
