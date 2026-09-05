#!/usr/bin/env python
"""Resolve skillUse events to skill names for harvest_coverage.sh.

GameEvent::SkillUse serialises skill_id as a NUMBER (`"skill_id":127`). The
harvest script's original grep required a quoted value, so it matched nothing
and the "Skill uses / re-rolls seen" section came out empty for every race --
including ones with hundreds of skillUse events. Read the ids and map them
through the SkillId enum's declaration order (its discriminants are positional).

usage: skill_use_report.py <events.txt>
"""
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


def main():
    names = skill_names()
    text = Path(sys.argv[1]).read_text(encoding='utf-8', errors='replace')

    used = {}
    for ev in re.findall(r'"type":"skillUse"[^}]*}', text):
        sid = re.search(r'"skill_id":(\d+)', ev)
        if not sid:
            continue
        i = int(sid.group(1))
        name = names[i] if i < len(names) else 'skill_%d' % i
        flag = 'used=true' if '"used":true' in ev else 'used=false'
        used['%s %s' % (name, flag)] = used.get('%s %s' % (name, flag), 0) + 1

    if not used:
        print('(no skillUse events in this run)')
        print()
        print('Note: GameEvent::SkillUse is emitted by only five sites --')
        print('block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle. A roster')
        print('with none of those legitimately produces zero. Every other skill is')
        print('used silently (BACKLOG E6); GameEvent::ReRoll has no emit site at all.')
        return

    for key, n in sorted(used.items(), key=lambda kv: -kv[1]):
        print('%7d %s' % (n, key))


if __name__ == '__main__':
    main()
