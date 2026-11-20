#!/usr/bin/env python
"""Skill coverage ledger (BACKLOG §M phase 0).

Every skill Java exposes per edition x what our sweeps actually do with it:

    NOT FIELDED   no player in any gate carries it
    SILENT        fielded, but no engine skill check ever came back true for it
    THIN          touched fewer than --thin times over the whole schedule
    LIVE          touched at least --thin times

"Touched" = Rust's skill-touch telemetry (crates/ffb-model/src/skill_touch.rs): an ENGINE skill
check (has_skill / has_skill_property* / has_unused_skill_with_property / skill_id_with_property)
returned true for a player who has the skill, outside the agents. It is a lower bound: a site that
reads the skill list directly is not counted. Parity proves Java does the same thing, so the
Rust-only bench run (no JVM) measures both engines.

    python scripts/skill_ledger.py run    --out D:/ffb_ledger/base [--workers 3] [--kinds mirror,cross,star]
    python scripts/skill_ledger.py report --out D:/ffb_ledger/base [--thin 100]

`run` replays the certification schedules (same seeds, scales and stars) through
`ffb-parity --bench` and stores one JSON per job; it resumes, skipping jobs already stored.
`report` aggregates them against the Java skill catalogue and writes <out>/LEDGER.txt + ledger.json.
"""
import argparse
import glob
import json
import os
import re
import subprocess
import sys
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUTER = os.path.dirname(ROOT)
JAVA_SKILLS = os.path.join(OUTER, 'ffb-java', 'ffb', 'ffb-common', 'src', 'main', 'java',
                           'com', 'fumbbl', 'ffb', 'skill')
DEFAULT_EXE = os.path.join(OUTER, 'target-fix', 'release', 'ffb-parity.exe')

SCHEDULES = {
    # kind: (glob of job files, line parser -> (key, argv), seeds)
    'mirror': os.path.join(ROOT, 'sweepjobs', 'w*.txt'),
    'cross': os.path.join(OUTER, 'crossjobs_cert2', 'w*.txt'),
    'star': os.path.join(OUTER, 'starjobs_full', 'w*.txt'),
    # §M phase 2b: the skill-advancement squads (mirror games, as scripts/sksweep.py runs them).
    'skill': os.path.join(ROOT, 'data', 'teams_skills', '*', 'team_*_sk.json'),
    # §M phase 2b(h): the stat-increase squads (scripts/draft_stat_squads.py), same schedule.
    'skill_st': os.path.join(ROOT, 'data', 'teams_stats', '*', 'team_*_st.json'),
}


def jobs(kind):
    out = []
    if kind == 'skill':
        # §M 2b(p): home-side star hires of the skill suite (same table as scripts/sksweep.py).
        hires = json.load(open(os.path.join(ROOT, 'data', 'teams_skills', 'star_hires.json'),
                               encoding='utf-8'))
        for path in sorted(glob.glob(SCHEDULES[kind])) + sorted(glob.glob(SCHEDULES['skill_st'])):
            ed = os.path.basename(os.path.dirname(path))
            sq = os.path.basename(path)[len('team_'):-len('.json')]
            star = hires.get(ed, {}).get(sq)
            out.append(('%s_%s' % (ed, sq), ed,
                        ['--home', sq, '--away', sq, '--edition', ed, '--seeds', '1-20',
                         '--heur-scale', '1.0'] + (['--home-star', star] if star else [])))
        return out
    for path in sorted(glob.glob(SCHEDULES[kind])):
        for line in open(path, encoding='utf-8'):
            f = line.split()
            if not f:
                continue
            if kind == 'mirror':
                ed, race, sc = f
                out.append(('%s_%s_%s' % (ed, race, sc), ed,
                            ['--home', race, '--away', race, '--edition', ed, '--seeds', '1-100',
                             '--heur-scale', sc]))
            elif kind == 'cross':
                ed, hm, aw = f
                out.append(('%s_%s__%s' % (ed, hm, aw), ed,
                            ['--home', hm, '--away', aw, '--edition', ed, '--seeds', '1-3',
                             '--heur-scale', '1.0']))
            else:
                ed, hm, hs, aw, as_ = f
                out.append(('%s_%s_%s__%s_%s' % (ed, hm, hs, aw, as_), ed,
                            ['--home', hm, '--away', aw, '--home-star', hs, '--away-star', as_,
                             '--edition', ed, '--seeds', '1-2', '--heur-scale', '1.0']))
    return out


def run_one(exe, outdir, kind, key, ed, argv):
    dst = os.path.join(outdir, kind, key + '.json')
    if os.path.exists(dst):
        return 'skip'
    cmd = [exe, '--bench', '--tier', '3', '--agent', 'heuristic', '--heur-classes', 'all'] + argv
    p = subprocess.run(cmd, capture_output=True, text=True)
    touches, fielded, reports, games = None, None, {}, 0
    for line in p.stdout.splitlines():
        if line.startswith('BENCH skills '):
            touches = json.loads(line[len('BENCH skills '):])
        elif line.startswith('BENCH fielded '):
            fielded = json.loads(line[len('BENCH fielded '):])
        elif line.startswith('BENCH reports '):
            reports = json.loads(line[len('BENCH reports '):])
        m = re.search(r' games=(\d+)', line)
        if m and line.startswith('BENCH agent='):
            games = int(m.group(1))
    if touches is None:
        return 'FAIL %s rc=%s %s' % (key, p.returncode, p.stderr[-300:].replace('\n', ' | '))
    tmp = dst + '.tmp'
    json.dump({'kind': kind, 'key': key, 'edition': ed, 'argv': argv, 'games': games,
               'touches': touches, 'fielded': fielded or [], 'reports': reports}, open(tmp, 'w'))
    os.replace(tmp, dst)
    return 'ok'


def exe_sha256(path):
    import hashlib
    h = hashlib.sha256()
    with open(path, 'rb') as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def cmd_run(a):
    exe = a.exe
    # STAMP: which build produced this ledger. scripts/skill_parity_check.py only accepts a ledger
    # whose exe is byte-identical to the certified one, and a resumed run must not mix builds.
    os.makedirs(a.out, exist_ok=True)
    stamp_path = os.path.join(a.out, 'STAMP.json')
    sha = exe_sha256(exe)
    if os.path.exists(stamp_path):
        old = json.load(open(stamp_path))
        if old.get('sha256') != sha:
            sys.exit('%s was produced by a different exe (%s); use a fresh --out' % (a.out, old.get('exe')))
    else:
        json.dump({'exe': exe, 'sha256': sha}, open(stamp_path, 'w'), indent=1)
    todo = []
    for kind in a.kinds.split(','):
        os.makedirs(os.path.join(a.out, kind), exist_ok=True)
        todo += [(kind,) + j for j in jobs(kind)]
    print('jobs: %d' % len(todo), flush=True)
    done = Counter()
    with ThreadPoolExecutor(max_workers=a.workers) as ex:
        futs = [ex.submit(run_one, exe, a.out, *t) for t in todo]
        for i, f in enumerate(futs, 1):
            r = f.result()
            done[r.split()[0]] += 1
            if r.startswith('FAIL'):
                print(r, flush=True)
            if i % 100 == 0:
                print('%d/%d %s' % (i, len(todo), dict(done)), flush=True)
    print('done %s' % dict(done), flush=True)


# ---------------------------------------------------------------- catalogue
def java_catalogue():
    """{edition: {class_name: category}} from the Java skill packages: common applies to every
    edition, mixed by its @RulesCollection annotations, bbXXXX to its own edition."""
    cat = defaultdict(dict)
    eds = ['bb2016', 'bb2020', 'bb2025']
    for pkg in ['common', 'mixed'] + eds:
        for path in glob.glob(os.path.join(JAVA_SKILLS, pkg, '**', '*.java'), recursive=True):
            src = open(path, encoding='utf-8', errors='replace').read()
            if not re.search(r'extends\s+Skill\b', src):
                continue
            cls = os.path.splitext(os.path.basename(path))[0]
            m = re.search(r'SkillCategory\.([A-Z_]+)', src)
            category = m.group(1) if m else '?'
            m = re.search(r'super\(\s*"([^"]+)"', src)
            display = m.group(1) if m else cls
            if pkg == 'common':
                targets = eds
            elif pkg == 'mixed':
                targets = [r.lower() for r in re.findall(r'Rules\.(BB20\d\d)', src)]
            else:
                targets = [pkg]
            for ed in targets:
                cat[ed][cls] = (category, display)
    return cat


MODIFIER_RE = re.compile(r'registerModifier\(\s*new\s+\w+(?:<[^>]*>)?\(\s*"([^"]+)"')


def java_modifier_names(cat):
    """{edition: {norm(modifier name): class}} for the roll/armour/injury modifiers a Java skill
    class registers in postConstruct() under a name that is NOT its display name -- bb2016
    Claw.java registers StaticArmourModifier("Claws"), bb2025 DwarvenScourge.java registers
    "DwarfenScourge". A report naming such a modifier names the skill that registered it (the
    modifier exists only on that skill's players). Names registered by two classes of one edition
    are ambiguous and dropped; a name equal to some display name keeps the display match."""
    seen = defaultdict(lambda: defaultdict(set))   # ed -> norm(name) -> classes
    for pkg in ['common', 'mixed', 'bb2016', 'bb2020', 'bb2025']:
        for path in glob.glob(os.path.join(JAVA_SKILLS, pkg, '**', '*.java'), recursive=True):
            src = open(path, encoding='utf-8', errors='replace').read()
            if not re.search(r'extends\s+Skill\b', src):
                continue
            cls = os.path.splitext(os.path.basename(path))[0]
            if pkg == 'common':
                targets = list(cat)
            elif pkg == 'mixed':
                targets = [r.lower() for r in re.findall(r'Rules\.(BB20\d\d)', src)]
            else:
                targets = [pkg]
            for ed in targets:
                for name in MODIFIER_RE.findall(src):
                    seen[ed][norm(name)].add(cls)
    out = {}
    for ed in cat:
        disp = {norm(d) for _, d in cat[ed].values()}
        out[ed] = {n: next(iter(c)) for n, c in seen[ed].items() if len(c) == 1 and n not in disp}
    return out


# Report keys whose string values are never skill names (they would collide: playerAction
# "block" vs the skill Block, kickoff "Blitz", weather, directions, team/coach names).
NON_SKILL_KEYS = {'playerAction', 'direction', 'kickoffResult', 'weather', 'teamName',
                  'coach', 'raceName', 'blockResult', 'injuryType', 'passingDistance',
                  'seriousInjury', 'playerState', 'kickoffEvent', 'blockDieResult',
                  'apothecaryMode', 'name', 'reason', 'result', 'sendToBoxReason',
                  'dodgeDirection', 'pushbackMode', 'passResult', 'statType'}


ID_GENERIC = {'catch', 'kick', 'pass', 'block', 'dodge', 'leap', 'accurate', 'foul', 'grab',
              'guard', 'stab', 'swoop'}


def named_counts(reports, catalogue_ed, modifiers_ed=None):
    """skill class -> times a report NAMED it: a free-text value equal to the display name or
    starting with it plus a space ("Break Tackle ST 5+"), longest display name winning, then a
    value equal to a modifier name the class registers (java_modifier_names), plus reportIds
    that start with the class name (jumpUpRoll -> JumpUp)."""
    modifiers_ed = modifiers_ed or {}
    by_disp = sorted(((norm(d), cls) for cls, (_, d) in catalogue_ed.items()), key=lambda t: -len(t[0]))
    by_cls = sorted(((norm(cls), cls) for cls in catalogue_ed), key=lambda t: -len(t[0]))
    out = Counter()
    for k, n in reports.items():
        key, _, val = k.partition(':')
        if key == 'id':
            nv = norm(val)
            for nc, cls in by_cls:
                # Generic roll ids (catchRoll, kickoffResult, passRoll, blockRoll, dodgeRoll,
                # foul...) are not skill uses.
                if nc in ID_GENERIC:
                    continue
                if len(nc) >= 4 and nv.startswith(nc):
                    out[cls] += n
                    break
            continue
        if key in NON_SKILL_KEYS or val.isupper():
            continue    # ALL-CAPS values are enum results (ACCURATE, POW), never skill names
        low = val.lower()
        for nd, cls in by_disp:
            d = catalogue_ed[cls][1].lower()
            if low == d or low.startswith(d + ' ') or norm(val) == nd:
                out[cls] += n
                break
        else:
            cls = modifiers_ed.get(norm(val))
            if cls:
                out[cls] += n
    return out


def norm(s):
    return re.sub(r'[^a-z0-9]', '', s.lower())


def cmd_report(a):
    cat = java_catalogue()
    mods = java_modifier_names(cat)
    touches = defaultdict(Counter)      # ed -> class -> n
    fielded = defaultdict(set)          # ed -> class
    where = defaultdict(Counter)        # (ed, class) -> kind -> jobs touching
    games = Counter()
    reports = defaultdict(Counter)      # ed -> "key:value" -> n
    for path in glob.glob(os.path.join(a.out, '*', '*.json')):
        d = json.load(open(path))
        ed = d['edition']
        games[(ed, d['kind'])] += d['games']
        for s in d['fielded']:
            fielded[ed].add(s)
        for s, n in d['touches'].items():
            touches[ed][s] += n
            where[(ed, s)][d['kind']] += 1
        reports[ed].update(d.get('reports', {}))

    rows = []
    for ed in sorted(cat):
        named = named_counts(reports[ed], cat[ed], mods[ed])
        # Rust class names may differ in spelling from the Java file name; join on norm().
        rust_by_norm = {norm(k): k for k in set(touches[ed]) | fielded[ed]}
        for cls, (category, display) in sorted(cat[ed].items()):
            rk = rust_by_norm.get(norm(cls))
            n = touches[ed].get(rk, 0) if rk else 0
            nm = named.get(cls, 0)
            f = bool(rk and rk in fielded[ed])
            if not f and n == 0 and nm == 0:
                status = 'NOT FIELDED'
            elif n == 0 and nm == 0:
                status = 'SILENT'
            elif nm == 0:
                status = 'UNNAMED'     # the engine consults it; no report ever names it
            elif nm < a.thin:
                status = 'THIN'
            else:
                status = 'LIVE'
            rows.append({'edition': ed, 'skill': cls, 'display': display, 'category': category,
                         'fielded': f, 'touches': n, 'named': nm, 'status': status,
                         'jobs': dict(where[(ed, rk)]) if rk else {}})

    # Actions: Java PlayerAction names vs the playerAction values the reports carry.
    src = open(os.path.join(OUTER, 'ffb-java', 'ffb', 'ffb-common', 'src', 'main', 'java', 'com',
                            'fumbbl', 'ffb', 'PlayerAction.java'), encoding='utf-8').read()
    actions = re.findall(r'\b[A-Z_]+\("([A-Za-z]+)",\s*\d+', src)
    act_counts = {ed: Counter({k.partition(':')[2]: n for k, n in reports[ed].items()
                               if k.startswith('playerAction:')}) for ed in cat}

    order = ['NOT FIELDED', 'SILENT', 'UNNAMED', 'THIN', 'LIVE']
    lines = ['SKILL LEDGER  touches = engine skill check true (outside the agents, includes offer',
             'checks); named = report-stream mentions (Java-identical); thin < %d named' % a.thin,
             'games: ' + ', '.join('%s/%s=%d' % (e, k, n) for (e, k), n in sorted(games.items())), '']
    for ed in sorted(cat):
        rs = [r for r in rows if r['edition'] == ed]
        c = Counter(r['status'] for r in rs)
        lines.append('== %s: %d skills  ' % (ed, len(rs)) + '  '.join('%s %d' % (s, c[s]) for s in order))
        for s in order[:4]:
            sel = sorted((r for r in rs if r['status'] == s), key=lambda r: (r['category'], r['skill']))
            if not sel:
                continue
            lines.append('  -- %s' % s)
            for r in sel:
                lines.append('     %-28s %-14s touch %8d  named %7d  %s' % (
                    r['skill'], r['category'], r['touches'], r['named'],
                    ' '.join('%s:%d' % kv for kv in sorted(r['jobs'].items()))))
        lines.append('')
    lines.append('== ACTIONS (playerAction in reports; Java PlayerAction has %d)' % len(actions))
    for act in actions:
        lines.append('     %-26s ' % act + '  '.join('%s %8d' % (ed, act_counts[ed][act]) for ed in sorted(cat)))
    lines.append('')
    text = '\n'.join(lines)
    os.makedirs(a.out, exist_ok=True)
    open(os.path.join(a.out, 'LEDGER.txt'), 'w', encoding='utf-8').write(text)
    json.dump(rows, open(os.path.join(a.out, 'ledger.json'), 'w'), indent=1)
    print(text)


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='cmd', required=True)
    r = sub.add_parser('run')
    r.add_argument('--out', required=True)
    r.add_argument('--workers', type=int, default=3)
    r.add_argument('--kinds', default='mirror,cross,star')
    r.add_argument('--exe', default=DEFAULT_EXE)
    p = sub.add_parser('report')
    p.add_argument('--out', required=True)
    p.add_argument('--thin', type=int, default=100)
    a = ap.parse_args()
    {'run': cmd_run, 'report': cmd_report}[a.cmd](a)


if __name__ == '__main__':
    main()
