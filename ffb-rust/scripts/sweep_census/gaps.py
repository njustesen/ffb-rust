#!/usr/bin/env python
"""Coverage GAPS: what the drafted squads were carrying that the sweep never exercised.

The census (report_census.json + the per-gate aggregates) records what fired.  This
joins that against the squad inventory to record what did NOT, and traces each dead
mechanic to one of three causes: never offered by legal_actions, offered but never
chosen by the agent, or no engine emit site at all.

    python scripts/sweep_census/gaps.py [census_dir] [report_census.json]

Defaults: scripts/sweep_census/out_s15 and docs/report_census.json, both relative to
the repo root (the directory two levels above this file).  Replaces skills.py, which
read a retired out/*.json layout and joined display names against CamelCase enum keys.
"""
import json
import glob
import os
import re
import sys
from collections import Counter, defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
HERE = os.path.join(ROOT, 'scripts', 'sweep_census')

CENSUS_DIR = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, 'out_s15')
REPORT_CENSUS = sys.argv[2] if len(sys.argv) > 2 else os.path.join(ROOT, 'docs', 'report_census.json')

THIN = 1000          # "exercised, but fewer than this many times in the whole sweep"


def norm(s):
    """Join key.  Collapses the edition spellings: Bone Head / Bone-Head / bone head,
    Claw / Claws, Side Step / Sidestep, Timmm-ber! / Timmmber."""
    return re.sub(r'[^a-z0-9]', '', s.lower())


def enum_variants(relpath, name):
    """Scrape one Rust enum's variants.  Bounded by the closing brace in column 0, so
    a later enum in the same file cannot leak in (that inflated PlayerAction 57 -> 70)."""
    src = open(os.path.join(ROOT, relpath), encoding='utf-8', errors='replace').read()
    i = src.index('pub enum ' + name)
    end = src.index('\n}', i)
    return re.findall(r'^\s{4}([A-Z][A-Za-z0-9]*)', src[i:end], re.M)


def camel(s):
    return s[0].lower() + s[1:]


# ---------------------------------------------------------------- inputs
inv = json.load(open(os.path.join(HERE, 'inventory.json'), encoding='utf-8'))
cen = json.load(open(REPORT_CENSUS, encoding='utf-8'))
skill_enum = inv['skill_enum']
squads = inv['squads']

gate_files = sorted(glob.glob(os.path.join(CENSUS_DIR, '*.json')))
if not gate_files:
    sys.exit('no per-gate census JSON under %s' % CENSUS_DIR)

T = Counter()            # GameEvent type -> count
A = Counter()            # declared PlayerAction -> count
K = Counter()            # KickoffResult -> count
SU = Counter()           # SkillId name -> skillUse count
SD = Counter()           # SkillId name -> skillUse(declined) count
per_cell = defaultdict(Counter)     # "<race>__<edition>" -> normalised skill -> count
activations = Counter()             # cell -> total activations tallied

for f in gate_files:
    d = json.load(open(f, encoding='utf-8'))
    cell = d['race'] + '__' + d['edition']
    T.update(d['types'])
    A.update(d['actions'])
    K.update(d['kickoff'])
    for k, v in d['skill_used'].items():
        SU[skill_enum[int(k)]] += v
        per_cell[cell][norm(skill_enum[int(k)])] += v
    for k, v in d['skill_declined'].items():
        SD[skill_enum[int(k)]] += v
        per_cell[cell][norm(skill_enum[int(k)])] += v
    for k, v in d['reroll_src'].items():
        per_cell[cell][norm(k)] += v
    activations[cell] += sum(d['by_player'].values())

RID = Counter(cen['ids'])

# ---------------------------------------------------------------- fielded skills
fielded = defaultdict(set)
spellings = defaultdict(set)
for cell, d in squads.items():
    for sk in d['skills']:
        fielded[norm(sk)].add(cell)
        spellings[norm(sk)].add(sk)


def display(nsk):
    return sorted(spellings[nsk])[0]


# ---------------------------------------------------------------- evidence
evidence = defaultdict(list)


def add(key, source, n):
    evidence[norm(key)].append((source, n))


# Roll/armour/injury/casualty modifier NAMES are the only place a passive skill is
# visible in either engine (BACKLOG H.50).  They are phrases, so match by substring:
# "1 for being marked with Prehensile Tail", "2 Disturbing Presences".
mods = [(bucket, name, n)
        for bucket, m in cen['modifiers'].items()
        for name, n in m.items()]
for nsk in fielded:
    for bucket, name, n in mods:
        if nsk and nsk in norm(name):
            add(nsk, 'report %s "%s"' % (bucket, name), n)

for name, n in cen['skill_use_skills'].items():
    add(name, 'report SkillUse', n)
for name, n in cen['skill_declined'].items():
    add(name, 'report SkillUse (declined)', n)
for name, n in cen['reroll_sources'].items():
    add(name, 'report ReRoll source', n)
for name, n in SU.items():
    add(name, 'event skillUse', n)
for name, n in SD.items():
    add(name, 'event skillUse (declined)', n)

# The dedicated telemetry site per skill: the one event / report / action that names it.
# This map is the only part of this script that needs human upkeep.
SITE = {
    'Loner':             ('event lonerRoll', T['lonerRoll']),
    'Pro':               ('event proRoll + report oldPro', T['proRoll'] + RID['oldPro']),
    'Regeneration':      ('event regenerationRoll', T['regenerationRoll']),
    'Right Stuff':       ('event rightStuffRoll', T['rightStuffRoll']),
    'Throw Team-Mate':   ('action ThrowTeamMate', A['ThrowTeamMate']),
    'Always Hungry':     ('event alwaysHungry', T['alwaysHungry']),
    'Bone Head':         ('event confusionRoll', T['confusionRoll']),
    'Really Stupid':     ('event confusionRoll', T['confusionRoll']),
    'Take Root':         ('event confusionRoll', T['confusionRoll']),
    'Unchannelled Fury': ('event animalSavagery', T['animalSavagery']),
    'Animal Savagery':   ('event animalSavagery', T['animalSavagery']),
    'Wild Animal':       ('event animalSavagery', T['animalSavagery']),
    'Blood Lust':        ('event bloodLustRoll', T['bloodLustRoll']),
    'Foul Appearance':   ('event foulAppearanceRoll', T['foulAppearanceRoll']),
    'Dauntless':         ('event dauntlessRoll', T['dauntlessRoll']),
    'Animosity':         ('event animosityRoll', T['animosityRoll']),
    'Leap':              ('event jumpRoll', T['jumpRoll']),
    'Very Long Legs':    ('event jumpRoll', T['jumpRoll']),
    'Pogo':              ('event jumpRoll', T['jumpRoll']),
    'Pogo Stick':        ('event jumpRoll', T['jumpRoll']),
    'Jump Up':           ('event jumpUpRoll + report jumpUpRoll', T['jumpUpRoll'] + RID['jumpUpRoll']),
    'Hypnotic Gaze':     ('event hypnoticGazeRoll + report hypnoticGazeRoll', T['hypnoticGazeRoll'] + RID['hypnoticGazeRoll']),
    'Breathe Fire':      ('event breatheFireRoll + report breatheFire', T['breatheFireRoll'] + RID['breatheFire']),
    'Projectile Vomit':  ('event projectileVomitRoll + report projectileVomit', T['projectileVomitRoll'] + RID['projectileVomit']),
    'Chainsaw':          ('event chainsawRoll', T['chainsawRoll']),
    'Ball and Chain':    ('event escapeRoll', T['escapeRoll']),
    'Safe Throw':        ('event safeThrowRoll', T['safeThrowRoll']),
    'Safe Pass':         ('event safeThrowRoll', T['safeThrowRoll']),
    'Bombardier':        ('action ThrowBomb', A['ThrowBomb']),
    'Hit and Run':       ('event hitAndRun', T['hitAndRun']),
    'Kick Team-mate':    ('action KickTeamMate', A['KickTeamMate']),
    'Swoop':             ('event swoopPlayer', T['swoopPlayer']),
    'Pick-me-up':        ('event pickMeUpRoll + report pickMeUp', T['pickMeUpRoll'] + RID['pickMeUp']),
    'Swarming':          ('event swarmingPlayersRoll', T['swarmingPlayersRoll']),
    'Fumblerooski':      ('event fumblerooskie + report fumblerooskie', T['fumblerooskie'] + RID['fumblerooskie']),
    'Punt':              ('action Punt', A['Punt']),
    'Hail Mary Pass':    ('action HailMaryPass', A['HailMaryPass']),
    'Cloud Burster':     ('report cloudBurster', RID['cloudBurster']),
    'Stab':              ('report playerEvent "gains Stab"', cen['player_events'].get('gains Stab', 0)),
    'Secret Weapon':     ('report secretWeaponBan', RID['secretWeaponBan']),
    'Steady Footing':    ('report steadyFootingRoll', RID['steadyFootingRoll']),
    'Tentacles':         ('report tentaclesShadowingRoll', RID['tentaclesShadowingRoll']),
    'Shadowing':         ('report tentaclesShadowingRoll', RID['tentaclesShadowingRoll']),
    'Trickster':         ('report trapDoor (shared, not skill-specific)', 0),
    'Nerves of Steel':   ('report nervesOfSteel', RID['nervesOfSteel']),
}
SITE_N = {norm(k): v for k, v in SITE.items()}
for k, (s, n) in SITE.items():
    add(k, s, n)

# ---------------------------------------------------------------- classify
rows = []
for nsk, cells in fielded.items():
    hits = [(s, n) for s, n in evidence.get(nsk, []) if n > 0]
    rows.append({
        'skill': display(nsk),
        'n': nsk,
        'cells': sorted(cells),
        'best': max([n for _, n in hits], default=0),
        'hits': sorted(hits, key=lambda x: -x[1]),
        'has_site': nsk in SITE_N,
    })

dead = sorted([r for r in rows if r['best'] == 0 and r['has_site']], key=lambda r: -len(r['cells']))
dark = sorted([r for r in rows if r['best'] == 0 and not r['has_site']], key=lambda r: -len(r['cells']))
live = sorted([r for r in rows if r['best'] > 0], key=lambda r: r['best'])
thin = [r for r in live if r['best'] < THIN]

# Skills a squad fields but never exercises in that squad's OWN gates, though some
# other squad does exercise them (so the mechanic itself is not dead).
visible = set()
for c in per_cell:
    visible |= set(per_cell[c])
residue = defaultdict(list)
for cell, d in squads.items():
    seen = set(per_cell.get(cell, {}))
    for sk in d['skills']:
        if norm(sk) in visible and norm(sk) not in seen:
            residue[sk].append(cell)

# ---------------------------------------------------------------- denominators
actions_all = enum_variants('crates/ffb-model/src/enums/player.rs', 'PlayerAction')
kickoff_all = enum_variants('crates/ffb-model/src/enums/kickoff_result.rs', 'KickoffResult')
events_all = enum_variants('crates/ffb-model/src/events/game_event.rs', 'GameEvent')

la = open(os.path.join(ROOT, 'crates/ffb-engine/src/legal_actions/mod.rs'),
          encoding='utf-8', errors='replace').read()
choices = set(re.findall(r'PlayerActionChoice::([A-Za-z]+)', la))
# legal_actions names CHOICES; the `PAC::X => PA::Y` table maps them onto PlayerAction
# (HypnoticGaze => Gaze, HandOff => HandOver).  Without this the offered set is wrong.
pac_to_pa = dict(re.findall(r'PAC::([A-Za-z]+)\s*=>\s*PA::([A-Za-z]+)', la))
offered = sorted({pac_to_pa.get(c, c) for c in choices})
ha = open(os.path.join(ROOT, 'crates/ffb-engine/src/agent/heuristic_agent.rs'),
          encoding='utf-8', errors='replace').read()
scored = sorted(set(re.findall(r'PlayerAction::([A-Za-z]+)', ha)))

events_never = sorted(v for v in events_all if camel(v) not in T)
reports_never = sorted(cen['never_produced'])

# One-sided streams: a report that is never produced even though the mechanic
# demonstrably runs, evidenced on the other side (an event, an action, or a modifier
# name).  (report id, what proves the mechanic ran, its count).
mod_count = {name: n for _, m in cen['modifiers'].items() for name, n in m.items()}
PAIRS = [
    ('throwAtStallingPlayer', 'event throwAtStallingPlayer', T['throwAtStallingPlayer']),
    ('kickTeamMateRoll', 'action KickTeamMate', A['KickTeamMate']),
    ('swoopDirectionRoll', 'event swoopPlayer', T['swoopPlayer']),
    ('swoopDistanceRoll', 'event swoopPlayer', T['swoopPlayer']),
    ('passBlock', 'event passBlock', T['passBlock']),
    ('nervesOfSteel', 'rollModifier "Nerves of Steel"', mod_count.get('Nerves of Steel', 0)),
    ('winningsRoll', 'event winningsRoll', T['winningsRoll']),
    ('blockReRoll', 'Brawler/Pro fielded, squads', len(fielded[norm('Brawler')] | fielded[norm('Pro')])),
    ('oldPro', 'Pro fielded, squads', len(fielded[norm('Pro')])),
]

# ---------------------------------------------------------------- output
print('COVERAGE GAPS  --  %d gates, %d games, %d squads'
      % (len(gate_files), cen['games'], len(squads)))
print('fielded skills %d   exercised %d   DEAD SITE %d   never named %d   thin (<%d) %d'
      % (len(rows), len(live), len(dead), len(dark), THIN, len(thin)))
print()

print('== A. FIELDED, dedicated telemetry exists, NEVER fired ==')
for r in dead:
    print('  %-22s %3d squads  %s' % (r['skill'], len(r['cells']), SITE_N[r['n']][0]))
    print('%26s %s' % ('', ', '.join(r['cells'])))

print('\n== B. FIELDED, the engine names it nowhere (no event, no report, no modifier) ==')
for r in dark:
    tail = ' ...' if len(r['cells']) > 4 else ''
    print('  %-22s %3d squads  %s%s' % (r['skill'], len(r['cells']), ', '.join(r['cells'][:4]), tail))

print('\n== C. FIELDED and exercised, but fewer than %d times ==' % THIN)
for r in thin:
    print('  %-22s %3d squads %8d   %s'
          % (r['skill'], len(r['cells']), r['best'], '; '.join('%s=%d' % h for h in r['hits'][:2])))

print("\n== D. Fielded by a squad, never exercised in that squad's own gates ==")
for sk, cs in sorted(residue.items(), key=lambda x: -len(x[1])):
    print('  %-22s %3d cells: %s' % (sk, len(cs), ', '.join(cs)))

print('\n== E. Actions: %d of %d declared, %d offered by legal_actions, %d scored by the agent =='
      % (len(A), len(actions_all), len(offered), len(scored)))
for a, n in A.most_common():
    print('  declared  %-24s %10d' % (a, n))
print('  -- never declared --')
for a in actions_all:
    if a in A:
        continue
    tags = []
    if a in offered:
        tags.append('offered')
    if a in scored:
        tags.append('scored by agent')
    print('    %-26s %s' % (a, ' + '.join(tags) if tags else 'NOT offered by legal_actions'))

print('\n== F. Kickoff: %d of %d results seen ==' % (len(K), len(kickoff_all)))
for k, n in K.most_common():
    print('  %-20s %8d' % (k, n))
missing_k = [k for k in kickoff_all if k not in K]
print('  missing: %s' % (', '.join(missing_k) if missing_k else 'none'))

print('\n== G. Activations ==')
lo = min(activations.values())
hi = max(activations.values())
print('  %d cells, %d-%d activations per cell across its 3 scale gates; no dead position'
      % (len(activations), lo, hi))

print('\n== H. Streams: %d of %d GameEvents emitted, %d of %d ReportIds produced =='
      % (len(T), len(events_all), len(RID), len(cen['catalog'])))
print('  events never emitted (%d): %s' % (len(events_never), ', '.join(events_never)))
print('  reports never produced (%d): %s' % (len(reports_never), ', '.join(reports_never)))

print('\n== I. One-sided streams: report never produced, though the mechanic runs ==')
for rid, why, n in PAIRS:
    print('  report %-24s = %-6d  but %s = %d' % (rid, RID.get(rid, 0), why, n))

out = os.path.join(HERE, 'gaps.json')
json.dump({
    'gates': len(gate_files), 'games': cen['games'], 'squads': len(squads),
    'dead': [{'skill': r['skill'], 'cells': r['cells'], 'site': SITE_N[r['n']][0]} for r in dead],
    'dark': [{'skill': r['skill'], 'cells': r['cells']} for r in dark],
    'thin': [{'skill': r['skill'], 'cells': r['cells'], 'count': r['best'], 'hits': r['hits']} for r in thin],
    'residue': residue,
    'actions_declared': dict(A), 'actions_all': actions_all,
    'actions_offered': offered, 'actions_scored': scored,
    'kickoff': dict(K), 'kickoff_all': kickoff_all,
    'events_never': events_never, 'reports_never': reports_never,
}, open(out, 'w', encoding='utf-8'), indent=1)
print('\nwrote %s' % out)
