#!/usr/bin/env python
"""Is BACKLOG §M done? -- "parity proven on every skill", checked mechanically.

DONE (exit 0) requires ALL of:
  1. A cert manifest (default docs/cert_manifest.json) naming ONE frozen build:
       {"exe": "D:/ffb_star/bin/ffb-parity-certN.exe",
        "jar": "D:/ffb_star/bin/ffb-ai-certN.jar",
        "ledger": "D:/ffb_ledger/certN",
        "suites": {"mirror": "<glob of sweep_*.log>", "cross": "<glob>",
                   "star": "<glob>", "skill": "<path of sksweep results.txt>"}}
  2. Every suite fully GREEN on that build: one verdict per gate of the schedule (the expected
     count comes from the schedule files via skill_ledger.jobs(), never from the manifest), the
     LAST verdict of each gate green on steps AND reports, every verdict file newer than the exe.
  3. A complete skill ledger (scripts/skill_ledger.py run + report) produced by the SAME exe
     (STAMP.json sha256 == sha256(manifest exe)), every job present, ledger.json newer than them.
  4. Every skill OF EACH RULESET fires in that ledger (status LIVE, THIN or UNNAMED: an engine skill
     check came back true or a report named it). There is NO exemption list (user, 2026-11-21:
     "there should be no exempt list -- rosters that match each ruleset and have skill coverage
     for all skills within that ruleset"). A ruleset's skills (ruleset_universe()) are the Java
     skill classes of that edition that a player of the ruleset can actually hold:
       - carried by a position, an Infamous Staff member or a star of the edition's own data
         (data/rosters/<ed>/*.json positions + infamous_staff, data/star_players/<ed>.json),
       - granted by a roster special rule of the edition (e.g. bb2025 "Team Captain"),
       - or learnable by advancement: a skill category some position of the edition may take
         (skill_categories normal/double), or a STAT_INCREASE, unless Java's eligible() says no.
     bb2016 is the project's CRP/LRB6 set; its FUMBBL-legacy rosters (BB2016_DRAFTING_AND_ROSTERS
     §2.2) are not part of the ruleset.
     A skill whose ONLY rule is a team-draft restriction has nothing to fire in a game; it counts
     as covered when the edition's drafted teams obey it and at least one fields it (DRAFT_RULES).

Because the ledger replays exactly the certified schedules (same seeds, scales, stars) on exactly
the certified exe, a skill that fires in the ledger fires in green parity games.

usage: python scripts/skill_parity_check.py [--manifest docs/cert_manifest.json] [--json]
Prints a summary of everything still missing; the loop driver feeds it to the next session.
"""
import argparse
import glob
import json
import os
import re
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)
import skill_ledger  # noqa: E402  (jobs(), exe_sha256)

FIRES = {'LIVE', 'THIN', 'UNNAMED'}
SUITES = ['mirror', 'cross', 'star', 'skill']
SWEEP_RE = re.compile(r'^(?P<key>.*?)\s+[\d.]+m\s+(?P<rest>(PARITY|NO VERDICT).*)$')
PAR_RE = re.compile(r'PARITY: (\d+)/(\d+) games match')
REP_RE = re.compile(r'REPORTS: (\d+)/(\d+) games identical, 0 differ')
SK_RE = re.compile(r'^(\S+) (\S+) parity=(\d+)/(\d+) reports=(\d+)/(\d+) rc=')


DATA = os.path.join(ROOT, 'data')
# BB2016_DRAFTING_AND_ROSTERS.md §2.2: rosters with no BB2016-official definition.
BB2016_LEGACY = {'nippon', 'renegades', 'dark_elf_league_fumbbl', 'khemri_fumbbl',
                 'slann_fumbbl', 'lineman'}


def _norm(s):
    return re.sub(r'[^a-z0-9]', '', s.lower())


def _skill_name(s):
    return re.sub(r'\(.*\)$', '', s['name'] if isinstance(s, dict) else s).strip()


def ruleset_universe():
    """{edition: {skill class: [where it comes from, ...]}} -- see the docstring, item 4."""
    cat = skill_ledger.java_catalogue()
    uni = {}
    for ed in sorted(cat):
        lookup = defaultdict(set)
        for cls, (_, disp) in cat[ed].items():
            lookup[_norm(disp)].add(cls)
            lookup[_norm(cls)].add(cls)
        src, cats = defaultdict(list), set()

        def add(name, where):
            for c in lookup.get(_norm(name), ()):
                src[c].append(where)

        for f in sorted(glob.glob(os.path.join(DATA, 'rosters', ed, 'roster_*.json'))):
            key = os.path.basename(f)[len('roster_'):-len('.json')]
            if ed == 'bb2016' and key in BB2016_LEGACY:
                continue
            r = json.load(open(f, encoding='utf-8'))
            for rule in r.get('special_rules') or []:
                add(rule, 'rule:' + key)
            for p in r['positions'] + (r.get('infamous_staff') or []):
                if not p.get('quantity'):
                    continue
                for s in p.get('skills', []):
                    add(_skill_name(s), key + ':' + p.get('name', '?'))
                for k in ('normal', 'double'):
                    cats.update(c.upper() for c in (p.get('skill_categories') or {}).get(k, []))
        stars = os.path.join(DATA, 'star_players', ed + '.json')
        if os.path.exists(stars):
            for s in json.load(open(stars, encoding='utf-8')).get('stars', []):
                for pl in s.get('players', []):
                    for sk in pl.get('position', {}).get('skills', []):
                        add(_skill_name(sk), 'star:' + s.get('name', '?'))
        for cls, (category, _) in cat[ed].items():
            if category in cats or category == 'STAT_INCREASE':
                if not skill_ledger_ineligible(ed, cls):
                    src[cls].append('advancement:' + category)
        uni[ed] = dict(src)
    return uni


def skill_ledger_ineligible(ed, cls):
    """Java Skill.eligible() overridden to return false (bb2020 PilingOn): never learnable."""
    for path in glob.glob(os.path.join(skill_ledger.JAVA_SKILLS, '**', cls + '.java'), recursive=True):
        src = open(path, encoding='utf-8', errors='replace').read()
        if ('Rules.' + ed.upper()) not in src and os.sep + ed + os.sep not in path:
            continue
        if re.search(r'boolean\s+eligible\(\)\s*\{\s*return\s+false', src):
            return True
    return False


def _insignificant_ok():
    """bb2025 Insignificant: "you may not include more players with this Trait than players
    without this Trait" (skill/bb2025/Insignificant.java javadoc). Every drafted bb2025 team obeys
    it and at least one fields an Insignificant player."""
    pos = {}
    for f in glob.glob(os.path.join(DATA, 'rosters', 'bb2025', 'roster_*.json')):
        for p in json.load(open(f, encoding='utf-8'))['positions']:
            pos[p['id']] = {_skill_name(s) for s in p.get('skills', [])}
    fielded, bad = 0, []
    for f in glob.glob(os.path.join(DATA, 'teams', 'bb2025', '*.json')) + \
            glob.glob(os.path.join(DATA, 'teams_skills', 'bb2025', '*.json')):
        players = json.load(open(f, encoding='utf-8')).get('players', [])
        n = sum(1 for p in players if 'Insignificant' in pos.get(p.get('position_id'), ()))
        fielded += n
        if n > len(players) - n:
            bad.append('%s (%d of %d)' % (os.path.basename(f), n, len(players)))
    if bad:
        return False, 'teams break the Insignificant limit: ' + ', '.join(bad)
    if not fielded:
        return False, 'no bb2025 team fields an Insignificant player'
    return True, 'every bb2025 team obeys the draft limit; %d Insignificant players fielded' % fielded


DRAFT_RULES = {('bb2025', 'Insignificant'): _insignificant_ok}


def suite_verdicts(kind, pattern):
    """{gate key: green?} using the LAST verdict per gate, plus the files read."""
    files = sorted(glob.glob(pattern))
    last = {}
    for path in files:
        for line in open(path, encoding='utf-8', errors='replace'):
            line = line.rstrip('\n')
            if kind == 'skill':
                m = SK_RE.match(line)
                if not m:
                    continue
                ok = m.group(3) == m.group(4) != '0' and m.group(5) == m.group(6) != '0'
                last[(m.group(1), m.group(2))] = ok
                continue
            m = SWEEP_RE.match(line)
            if not m:
                continue
            key = re.sub(r'\s+#\S+', '', m.group('key')).strip()
            p, r = PAR_RE.search(m.group('rest')), REP_RE.search(m.group('rest'))
            ok = bool(p and r and p.group(1) == p.group(2) != '0' and r.group(1) == r.group(2))
            last[key] = ok
    return last, files


def check(manifest_path):
    problems, info = [], []
    if not os.path.exists(manifest_path):
        problems.append('NO CERT MANIFEST at %s -- the final step is a full cert (mirror, cross, '
                        'star, skill) on ONE frozen exe+jar, a ledger run on that same exe, then '
                        'this manifest (schema in the docstring).' % manifest_path)
        man = None
    else:
        man = json.load(open(manifest_path))

    exe_sha = None
    if man:
        exe = man.get('exe', '')
        if not os.path.isfile(exe):
            problems.append('manifest exe missing: %s' % exe)
        else:
            exe_sha = skill_ledger.exe_sha256(exe)
            exe_mtime = os.path.getmtime(exe)
        if not os.path.isfile(man.get('jar', '')):
            problems.append('manifest jar missing (freeze the Java harness too): %s' % man.get('jar'))
        for kind in SUITES:
            pattern = (man.get('suites') or {}).get(kind)
            if not pattern:
                problems.append('suite %s: not in the manifest' % kind)
                continue
            expected = len(skill_ledger.jobs(kind))
            last, files = suite_verdicts(kind, pattern)
            red = sorted(str(k) for k, ok in last.items() if not ok)
            if exe_sha and any(os.path.getmtime(f) < exe_mtime for f in files):
                problems.append('suite %s: verdict files older than the exe (stale cert)' % kind)
            if len(last) != expected:
                problems.append('suite %s: %d/%d gates have a verdict' % (kind, len(last), expected))
            if red:
                problems.append('suite %s: %d red gate(s): %s%s' % (
                    kind, len(red), ', '.join(red[:15]), ' ...' if len(red) > 15 else ''))
            if len(last) == expected and not red:
                info.append('suite %s: %d/%d green' % (kind, expected, expected))

    rows = None
    ledger = man.get('ledger') if man else None
    if man and not ledger:
        problems.append('manifest has no "ledger" directory')
    if not man:
        # PROGRESS ONLY (never proof): list the gaps from the newest ledger report so the work
        # can be steered before the final cert exists.
        reps = sorted(glob.glob('D:/ffb_ledger/*/ledger.json'), key=os.path.getmtime)
        if reps:
            rows = json.load(open(reps[-1]))
            info.append('progress view from %s (newest ledger; NOT proof)' % os.path.dirname(reps[-1]))
        else:
            info.append('no ledger report yet (scripts/skill_ledger.py run + report)')
    if ledger:
        stamp = os.path.join(ledger, 'STAMP.json')
        if not os.path.exists(stamp):
            problems.append('ledger %s has no STAMP.json (run skill_ledger.py run --exe <cert exe>)' % ledger)
        elif exe_sha and json.load(open(stamp)).get('sha256') != exe_sha:
            problems.append('ledger %s was produced by a different exe than the cert' % ledger)
        job_files = glob.glob(os.path.join(ledger, '*', '*.json'))
        for kind in SUITES:
            want = len(skill_ledger.jobs(kind))
            have = len(glob.glob(os.path.join(ledger, kind, '*.json')))
            if have != want:
                problems.append('ledger %s: %d/%d jobs' % (kind, have, want))
        lj = os.path.join(ledger, 'ledger.json')
        if not os.path.exists(lj):
            problems.append('ledger.json missing (run skill_ledger.py report --out %s)' % ledger)
        elif job_files and os.path.getmtime(lj) < max(os.path.getmtime(f) for f in job_files):
            problems.append('ledger.json is older than its jobs (re-run report)')
        else:
            rows = json.load(open(lj))

    missing = []
    if rows is not None:
        uni = ruleset_universe()
        fired, total, outside = 0, 0, []
        seen = set()
        for r in rows:
            if r['skill'] not in uni.get(r['edition'], {}):
                outside.append('%s %s' % (r['edition'], r['skill']))
                continue
            total += 1
            seen.add((r['edition'], r['skill']))
            if r['status'] in FIRES:
                fired += 1
                continue
            rule = DRAFT_RULES.get((r['edition'], r['skill']))
            if rule:
                ok, why = rule()
                if ok:
                    fired += 1
                    info.append('draft rule %s %s: %s' % (r['edition'], r['skill'], why))
                    continue
                missing.append('%s %s (draft rule: %s)' % (r['edition'], r['skill'], why))
                continue
            missing.append('%s %s (%s, %s; held via %s)' % (
                r['edition'], r['skill'], r['category'], r['status'],
                ', '.join(uni[r['edition']][r['skill']][:3])))
        for ed, skills in sorted(uni.items()):
            for cls in sorted(skills):
                if (ed, cls) not in seen:
                    missing.append('%s %s (in the ruleset but not in the ledger)' % (ed, cls))
        info.append('ruleset skills firing: %d/%d; missing: %d' % (fired, total, len(missing)))
        info.append('not part of any ruleset (no roster/star/staff/special rule carries them, not '
                    'learnable; not counted): %s' % ', '.join(outside))
        if missing:
            problems.append('%d skill x edition of the rulesets not firing:\n    %s' % (
                len(missing), '\n    '.join(missing)))
    return problems, info


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--manifest', default=os.path.join(ROOT, 'docs', 'cert_manifest.json'))
    ap.add_argument('--json', action='store_true')
    a = ap.parse_args()
    problems, info = check(a.manifest)
    if a.json:
        print(json.dumps({'done': not problems, 'problems': problems, 'info': info}, indent=1))
    else:
        for line in info:
            print('ok   ' + line)
        for line in problems:
            print('TODO ' + line)
        print('DONE' if not problems else 'NOT DONE (%d open item(s))' % len(problems))
    sys.exit(0 if not problems else 1)


if __name__ == '__main__':
    main()
