#!/usr/bin/env python
"""Expected-side inventory: which skills each drafted squad actually FIELDS,
per (race, edition), plus the SkillId enum order and star-player skills."""
import json
import re
from pathlib import Path

REPO = Path(r"C:\Users\Admin\niels\ffb-rust\ffb-rust")


def skill_names():
    src = (REPO / 'crates/ffb-model/src/enums/skill_id.rs').read_text(encoding='utf-8')
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


def roster_positions(edition):
    out = {}
    for f in (REPO / 'data/rosters' / edition).glob('roster_*.json'):
        d = json.loads(f.read_text(encoding='utf-8'))
        for p in d.get('positions', []):
            out[p['id']] = p
    return out


def stars(edition):
    out = {}
    d = REPO / 'data/star_players'
    for f in d.glob('*.json'):
        try:
            j = json.loads(f.read_text(encoding='utf-8'))
        except Exception:
            continue
        items = j if isinstance(j, list) else j.get('star_players', j.get('stars', []))
        if isinstance(items, dict):
            items = list(items.values())
        for s in items:
            if isinstance(s, dict) and 'id' in s:
                out[s['id']] = s
    return out


def squad_skills(edition, team_file):
    d = json.loads(team_file.read_text(encoding='utf-8'))
    pos = roster_positions(edition)
    st = stars(edition)
    skills = {}
    missing_pos = []
    for pl in d.get('players', []):
        pid = pl.get('position_id')
        p = pos.get(pid)
        if not p:
            missing_pos.append(pid)
            continue
        for s in p.get('skills', []):
            if isinstance(s, dict):
                s = s.get('name')
            skills.setdefault(s, []).append(pid)
    for ent in d.get('stars', []) or []:
        sid = ent.get('star_id') if isinstance(ent, dict) else ent
        s = st.get(sid)
        if s:
            for sk in s.get('skills', []):
                if isinstance(sk, dict):
                    sk = sk.get('name')
                skills.setdefault(sk, []).append('star:' + sid)
        else:
            missing_pos.append('star:' + sid)
    return d, skills, missing_pos


if __name__ == '__main__':
    names = skill_names()
    result = {'skill_enum': names, 'squads': {}}
    for edition in ('bb2016', 'bb2020', 'bb2025'):
        tdir = REPO / 'data/teams' / edition
        if not tdir.is_dir():
            continue
        for f in sorted(tdir.glob('team_*.json')):
            race = f.stem[len('team_'):]
            d, skills, miss = squad_skills(edition, f)
            result['squads']['%s__%s' % (race, edition)] = {
                'race': race, 'edition': edition,
                'players': len(d.get('players', [])),
                'stars': d.get('stars') or [],
                'rerolls': d.get('rerolls'),
                'apothecaries': d.get('apothecaries'),
                'special_rules': d.get('special_rules'),
                'skills': {k: sorted(set(v)) for k, v in skills.items()},
                'unresolved': miss,
            }
    Path('inventory.json').write_text(json.dumps(result, indent=1), encoding='utf-8')

    # fielded.json: skill name -> the cells whose squad fields a player carrying it.
    # skills.py and build_page.py both READ this and nothing in the pipeline used to WRITE it,
    # so the skills half of the census could not be reproduced from the committed scripts --
    # `skills.py` died on FileNotFoundError. It is derived here because this is the step that
    # already resolves every squad's positions against the roster data.
    fielded = {}
    for cell, sq in result['squads'].items():
        for skill in sq['skills']:
            fielded.setdefault(skill, []).append(cell)
    Path('fielded.json').write_text(
        json.dumps({'fielded': {k: sorted(v) for k, v in sorted(fielded.items())}}, indent=1),
        encoding='utf-8')

    print('skills in enum:', len(names))
    print('skills fielded by at least one squad:', len(fielded))
    print('squads:', len(result['squads']))
    bad = {k: v['unresolved'] for k, v in result['squads'].items() if v['unresolved']}
    print('squads with unresolved positions/stars:', len(bad))
    for k, v in list(bad.items())[:10]:
        print(' ', k, v[:4])
