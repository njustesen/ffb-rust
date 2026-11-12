#!/usr/bin/env python
"""Rank the report-stream divergences of a whole sweep into fixable families (BACKLOG §J).

usage: report_first_diff.py <root> [<root> ...] [--out FILE] [--nproc N] [--examples K]

Walks every <dir>/seed_N_java_reports.jsonl that has a sibling seed_N_rust_reports.jsonl under the
given roots (a mirror sweep root, a cross sweep root, a single gate's FFB_PARITY_ROOT, ...). Each
game is normalised exactly as scripts/report_diff.py does (ids, coordinates, coach names,
null == absent == [], HashSet-ordered modifier lists) and its two reportId sequences are aligned
with difflib. Every difference becomes a FAMILY signature:

    only-java   <reportId>                    a report Java writes and Rust does not
    only-rust   <reportId>                    a report Rust writes and Java does not
    payload     <reportId> {key,key,...}      an aligned pair whose bodies differ in those keys

The table ranks families by the number of GAMES they occur in (a family that touches every game
outranks one that fires 50 times in a single game), and gives the edition split and an example
game with its first occurrence. A separate table ranks the FIRST divergence of each game, which is
what the per-gate `REPORTS DIFF` line shows; fixing a family moves those first divergences later.

Also listed: games whose report COUNTS differ by more than 10%, which point at whole missing report
families (BACKLOG §I.12: bb2016 amazon vs dark_elf_league_fumbbl seed 2, Java 859 / Rust 362).
"""
import difflib
import json
import os
import re
import sys
from collections import Counter, defaultdict
from multiprocessing import Pool

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from report_diff import load, norm  # noqa: E402

RE_JAVA = re.compile(r'^seed_(\d+)_java_reports\.jsonl$')
EDITIONS = ('bb2016', 'bb2020', 'bb2025')


def find_games(roots):
    games = []
    for root in roots:
        for dirpath, _dirs, files in os.walk(root):
            for f in files:
                m = RE_JAVA.match(f)
                if not m:
                    continue
                rust = os.path.join(dirpath, 'seed_%s_rust_reports.jsonl' % m.group(1))
                if os.path.exists(rust):
                    games.append((os.path.join(dirpath, f), rust))
    games.sort()
    return games


def edition_of(path):
    parts = re.split(r'[\\/]', path)
    for p in parts:
        if p in EDITIONS:
            return p
    for p in parts:  # mirror sweep roots: parity_<race>_<edition>_<scale>
        for ed in EDITIONS:
            if '_%s_' % ed in p:
                return ed
    return '?'


def diff_keys(a, b):
    keys = set(a) | set(b)
    return tuple(sorted(k for k in keys if a.get(k) != b.get(k)))


def classify(pair):
    jp, rp = pair
    try:
        J, R = load(jp), load(rp)
    except (OSError, ValueError) as e:
        return {'game': jp, 'error': str(e)}
    jn, rn = [norm(x) for x in J], [norm(x) for x in R]
    jid = [x.get('reportId', '?') for x in jn]
    rid = [x.get('reportId', '?') for x in rn]
    families = Counter()
    first = None  # (position, signature)
    sm = difflib.SequenceMatcher(None, jid, rid, autojunk=False)
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == 'equal':
            for a, b in zip(range(i1, i2), range(j1, j2)):
                if jn[a] != rn[b]:
                    sig = ('payload', jid[a], ','.join(diff_keys(jn[a], rn[b])))
                    families[sig] += 1
                    if first is None:
                        first = (a, sig)
            continue
        for a in range(i1, i2):
            sig = ('only-java', jid[a], '')
            families[sig] += 1
            if first is None:
                first = (a, sig)
        for b in range(j1, j2):
            sig = ('only-rust', rid[b], '')
            families[sig] += 1
            if first is None:
                first = (i1, sig)
    return {
        'game': jp,
        'edition': edition_of(jp),
        'java': len(J),
        'rust': len(R),
        'identical': not families,
        'families': families,
        'first': first,
    }


def fmt_sig(sig):
    kind, rid, keys = sig
    return '%-9s %-32s %s' % (kind, rid, ('{' + keys + '}') if keys else '')


def main():
    args = sys.argv[1:]
    out_path, nproc, n_examples = None, 6, 1
    roots = []
    i = 0
    while i < len(args):
        if args[i] == '--out':
            out_path = args[i + 1]; i += 2
        elif args[i] == '--nproc':
            nproc = int(args[i + 1]); i += 2
        elif args[i] == '--examples':
            n_examples = int(args[i + 1]); i += 2
        else:
            roots.append(args[i]); i += 1
    if not roots:
        sys.exit(__doc__)

    games = find_games(roots)
    with Pool(nproc) as pool:
        results = pool.map(classify, games, chunksize=16)

    lines = []
    emit = lines.append
    errors = [r for r in results if 'error' in r]
    ok = [r for r in results if 'error' not in r]
    emit('report-stream divergence families over %d games (%d unreadable) under: %s'
         % (len(ok), len(errors), ', '.join(roots)))
    emit('')
    emit('IDENTICAL GAMES per edition')
    per_ed = defaultdict(lambda: [0, 0])
    for r in ok:
        per_ed[r['edition']][1] += 1
        if r['identical']:
            per_ed[r['edition']][0] += 1
    for ed in sorted(per_ed):
        same, total = per_ed[ed]
        emit('  %-7s %6d / %-6d identical (%.1f%%)' % (ed, same, total, 100.0 * same / max(total, 1)))
    same = sum(v[0] for v in per_ed.values())
    total = sum(v[1] for v in per_ed.values())
    emit('  %-7s %6d / %-6d identical (%.1f%%)' % ('ALL', same, total, 100.0 * same / max(total, 1)))

    fam_games = Counter()
    fam_total = Counter()
    fam_eds = defaultdict(Counter)
    fam_example = defaultdict(list)
    first_games = Counter()
    first_example = {}
    for r in ok:
        for sig, n in r['families'].items():
            fam_games[sig] += 1
            fam_total[sig] += n
            fam_eds[sig][r['edition']] += 1
            if len(fam_example[sig]) < n_examples:
                fam_example[sig].append(r['game'])
        if r['first'] is not None:
            pos, sig = r['first']
            first_games[sig] += 1
            first_example.setdefault(sig, (r['game'], pos))

    def eds(sig):
        return ' '.join('%s:%d' % (ed[2:], fam_eds[sig][ed]) for ed in EDITIONS if fam_eds[sig][ed])

    emit('')
    emit('FAMILIES ranked by games affected (games / occurrences / editions)')
    for sig, g in fam_games.most_common():
        emit('  %6d %8d  %-74s %s' % (g, fam_total[sig], fmt_sig(sig), eds(sig)))
        for ex in fam_example[sig]:
            emit('           e.g. %s' % ex)

    emit('')
    emit('FIRST DIVERGENCE ranked by games (what each gate\'s REPORTS DIFF line shows)')
    for sig, g in first_games.most_common():
        game, pos = first_example[sig]
        emit('  %6d  %-74s e.g. %s @#%d' % (g, fmt_sig(sig), game, pos))

    emit('')
    emit('REPORT-COUNT GAPS > 10% (whole missing families)')
    gaps = [r for r in ok if max(r['java'], r['rust']) > 0
            and abs(r['java'] - r['rust']) > 0.10 * max(r['java'], r['rust'])]
    gaps.sort(key=lambda r: -abs(r['java'] - r['rust']))
    emit('  %d games' % len(gaps))
    for r in gaps[:40]:
        emit('  java %5d / rust %5d  %s' % (r['java'], r['rust'], r['game']))

    if errors:
        emit('')
        emit('UNREADABLE')
        for r in errors[:20]:
            emit('  %s: %s' % (r['game'], r['error']))

    text = '\n'.join(lines) + '\n'
    if out_path:
        with open(out_path, 'w', encoding='utf-8') as fh:
            fh.write(text)
    sys.stdout.write(text)


if __name__ == '__main__':
    main()
