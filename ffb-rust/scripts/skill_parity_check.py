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
  4. Every Java skill x edition in that ledger FIRES (status LIVE, THIN or UNNAMED: an engine skill
     check came back true or a report named it), or is exempted in data/agent/skill_exempt.json
     ({"bb2025": {"SkillClass": "reason >= 40 chars, traced to Java"}}).

Because the ledger replays exactly the certified schedules (same seeds, scales, stars) on exactly
the certified exe, a skill that fires in the ledger fires in green parity games.

usage: python scripts/skill_parity_check.py [--manifest docs/cert_manifest.json]
                                            [--exempt data/agent/skill_exempt.json] [--json]
Prints a summary of everything still missing; the loop driver feeds it to the next session.
"""
import argparse
import glob
import json
import os
import re
import sys

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


def check(manifest_path, exempt_path):
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

    exempt = json.load(open(exempt_path)) if os.path.exists(exempt_path) else {}
    missing = []
    if rows is not None:
        fired = 0
        for r in rows:
            if r['status'] in FIRES:
                fired += 1
                if r['skill'] in exempt.get(r['edition'], {}):
                    info.append('stale exemption: %s %s now fires' % (r['edition'], r['skill']))
                continue
            reason = exempt.get(r['edition'], {}).get(r['skill'], '')
            if len(reason.strip()) >= 40:
                continue
            missing.append('%s %s (%s, %s)' % (r['edition'], r['skill'], r['category'], r['status']))
        info.append('skills firing: %d/%d; exempt: %d; missing: %d' % (
            fired, len(rows), sum(len(v) for v in exempt.values()), len(missing)))
        if missing:
            problems.append('%d skill x edition not firing and not exempt:\n    %s' % (
                len(missing), '\n    '.join(missing)))
    return problems, info


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--manifest', default=os.path.join(ROOT, 'docs', 'cert_manifest.json'))
    ap.add_argument('--exempt', default=os.path.join(ROOT, 'data', 'agent', 'skill_exempt.json'))
    ap.add_argument('--json', action='store_true')
    a = ap.parse_args()
    problems, info = check(a.manifest, a.exempt)
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
