#!/usr/bin/env python
"""Which skills on the drafted rosters ever raise a SkillUse event?

Empirical by necessity. A static grep for `SkillId::X` inside `ReportSkillUse::new`
CANNOT answer this: many reports pass a skill resolved at runtime
(`getSkillWithProperty(...) -> skill`), so the literal never appears. Eye Gouge is the
worked counter-example -- no literal site, yet the single most-evented skill in a probe.

So: read every `seed_*_rust_events.jsonl` a sweep produced, collect the distinct
`skill_id`s actually evented, and diff against the skills the rosters actually field.

usage: skill_coverage_report.py [<sweep-root-glob>]     (default: parity_rd_*)
"""
import glob
import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def skill_names():
    src = (ROOT / 'crates/ffb-model/src/enums/skill_id.rs').read_text(encoding='utf-8')
    body = src.split('pub enum SkillId {', 1)[1].split('\n}', 1)[0]
    names = []
    for line in body.split('\n'):
        line = line.strip()
        if not line or line.startswith('//') or line.startswith('#'):
            continue
        m = re.match(r'([A-Za-z_][A-Za-z0-9_]*)\s*(=\s*\d+)?\s*,', line)
        if m:
            names.append(m.group(1))
    return names


def norm(x):
    return re.sub(r'[^a-z0-9]', '', x.lower())


def roster_skills():
    """Distinct skills actually fielded, keyed normalised -> display name."""
    out = {}
    for f in glob.glob(str(ROOT / 'data/rosters/*/*.json')):
        d = json.load(open(f, encoding='utf-8'))
        for p in d.get('positions', []):
            for s in (p.get('skills') or []):
                nm = s if isinstance(s, str) else s.get('name', str(s))
                out.setdefault(norm(nm), nm)
    return out


def evented(pattern):
    """normalised skill name -> times evented, across every events log found."""
    names = skill_names()
    seen = {}
    files = 0
    for root in glob.glob(str(ROOT / pattern)):
        for dirpath, _dirnames, filenames in os.walk(root):
            for fn in filenames:
                if not fn.endswith('_rust_events.jsonl'):
                    continue
                files += 1
                text = Path(dirpath, fn).read_text(encoding='utf-8', errors='replace')
                for ev in re.findall(r'"type":"skillUse"[^}]*}', text):
                    m = re.search(r'"skill_id":(\d+)', ev)
                    if not m:
                        continue
                    i = int(m.group(1))
                    nm = names[i] if i < len(names) else 'skill_%d' % i
                    seen[norm(nm)] = seen.get(norm(nm), 0) + 1
    return seen, files


def main():
    pattern = sys.argv[1] if len(sys.argv) > 1 else 'parity_rd_*'
    rosters = roster_skills()
    seen, files = evented(pattern)

    covered = sorted((v, rosters[k]) for k, v in seen.items() if k in rosters)
    uncovered = sorted(v for k, v in rosters.items() if k not in seen)
    off_roster = sorted(k for k in seen if k not in rosters)

    print('games scanned: %d' % files)
    print('distinct skills fielded by the drafted rosters: %d' % len(rosters))
    print('  COVERED   (raised >=1 SkillUse event): %d' % len(covered))
    print('  UNCOVERED (never raised one):          %d' % len(uncovered))
    print()
    print('COVERED, by event count:')
    for n, name in sorted(covered, reverse=True):
        print('   %8d  %s' % (n, name))
    print()
    print('UNCOVERED (%d):' % len(uncovered))
    for i in range(0, len(uncovered), 6):
        print('   ' + ', '.join(uncovered[i:i + 6]))
    if off_roster:
        print()
        print('evented but not on any drafted roster (star/granted skills): %s'
              % ', '.join(sorted(off_roster)))


if __name__ == '__main__':
    main()
