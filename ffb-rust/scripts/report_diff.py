#!/usr/bin/env python
"""Align the Java and Rust report streams of one parity game and say what differs.

usage: report_diff.py <seed_N_java_reports.jsonl> <seed_N_rust_reports.jsonl> [--full]

Normalises the harness-specific shapes (Java team/player ids -> home_NN / home, coordinates
{x,y} -> [x,y], coach names, null == absent), aligns the two reportId sequences with difflib and
prints (1) a per-reportId count of "only in Java" / "only in Rust", (2) the first payload
differences of aligned pairs, and with --full the whole aligned sequence.
"""
import difflib
import json
import re
import sys
from collections import Counter

RE_JAVA_PID = re.compile(r'^team[A-Za-z0-9_]*?Parity(?:16|20|25)?(Home|Away)(\d+)$')
RE_JAVA_TID = re.compile(r'^team[A-Za-z0-9_]*?Parity(?:16|20|25)?(Home|Away)$')
RE_RUST_PID = re.compile(r'^(home|away)_(\d+)$')
RE_RUST_TID = re.compile(r'^(home|away)_[a-z0-9_.]+$')


def norm_str(s):
    m = RE_JAVA_PID.match(s)
    if m:
        return '%s_%02d' % (m.group(1).lower(), int(m.group(2)))
    m = RE_JAVA_TID.match(s)
    if m:
        return m.group(1).lower()
    m = RE_RUST_PID.match(s)
    if m:
        return '%s_%02d' % (m.group(1), int(m.group(2)))
    if RE_RUST_TID.match(s):
        return s.split('_', 1)[0]
    low = s.lower()
    if low in ('home', 'coach_home'):
        return 'home'
    if low in ('away', 'coach_away'):
        return 'away'
    return s


# Java builds modifier collections with Collectors.toSet(), so their order is a HashSet artefact.
MODIFIER_KEYS = {'armorModifiers', 'injuryModifiers', 'casualtyModifiers', 'rollModifiers', 'passModifiers'}


def norm(v):
    if isinstance(v, dict):
        if set(v.keys()) == {'x', 'y'}:
            return [v['x'], v['y']]
        # Java omits null fields and (through IJsonOption) some empty arrays; Rust writes them.
        # Neither is a report difference, so absent == null == [] here.
        out = {}
        for k, x in v.items():
            if k == 'i' or x is None or x == []:
                continue
            nx = norm(x)
            if k in MODIFIER_KEYS and isinstance(nx, list):
                nx = sorted(nx, key=repr)
            out[k] = nx
        return out
    if isinstance(v, list):
        return [norm(x) for x in v]
    if isinstance(v, str):
        return norm_str(v)
    return v


def load(p):
    out = []
    for line in open(p, encoding='utf-8', errors='replace'):
        line = line.strip()
        if line:
            out.append(json.loads(line))
    return out


def main():
    jp, rp = sys.argv[1], sys.argv[2]
    full = '--full' in sys.argv
    J, R = load(jp), load(rp)
    jn, rn = [norm(x) for x in J], [norm(x) for x in R]
    jid = [x.get('reportId', '?') for x in jn]
    rid = [x.get('reportId', '?') for x in rn]
    print('java %d reports, rust %d reports' % (len(J), len(R)))
    sm = difflib.SequenceMatcher(None, jid, rid, autojunk=False)
    only_j, only_r = Counter(), Counter()
    payload = []
    aligned = 0
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == 'equal':
            for a, b in zip(range(i1, i2), range(j1, j2)):
                aligned += 1
                if jn[a] != rn[b]:
                    payload.append((a, b, J[a], R[b]))
            if full:
                for a, b in zip(range(i1, i2), range(j1, j2)):
                    flag = '' if jn[a] == rn[b] else '   <-- payload differs'
                    print('  = %-28s j#%-4d i=%-3s r#%-4d i=%-3s%s' % (jid[a], a, J[a].get('i'), b, R[b].get('i'), flag))
        else:
            for a in range(i1, i2):
                only_j[jid[a]] += 1
                if full:
                    print('  J %-28s j#%-4d i=%-3s %s' % (jid[a], a, J[a].get('i'), json.dumps(jn[a])[:140]))
            for b in range(j1, j2):
                only_r[rid[b]] += 1
                if full:
                    print('  R %-28s r#%-4d i=%-3s %s' % (rid[b], b, R[b].get('i'), json.dumps(rn[b])[:140]))
    print('\naligned %d; only-in-Java %d; only-in-Rust %d; aligned-with-payload-diff %d' % (
        aligned, sum(only_j.values()), sum(only_r.values()), len(payload)))
    print('\nONLY IN JAVA (reportId: count)')
    for k, v in only_j.most_common():
        print('  %-30s %d' % (k, v))
    print('\nONLY IN RUST (reportId: count)')
    for k, v in only_r.most_common():
        print('  %-30s %d' % (k, v))
    print('\nPAYLOAD DIFFS (first 8 aligned pairs that differ)')
    for a, b, ja, rb in payload[:8]:
        print('  j#%d r#%d %s' % (a, b, ja.get('reportId')))
        print('    java=%s' % json.dumps(norm(ja), sort_keys=True)[:400])
        print('    rust=%s' % json.dumps(norm(rb), sort_keys=True)[:400])
    pk = Counter(ja.get('reportId') for _, _, ja, _ in payload)
    if pk:
        print('\nPAYLOAD DIFFS by reportId')
        for k, v in pk.most_common():
            print('  %-30s %d' % (k, v))


if __name__ == '__main__':
    main()
