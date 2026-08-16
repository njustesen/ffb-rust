#!/usr/bin/env python3
"""First-pass triage: does each Rust `step/generator/bb2020/*.rs` emit the same StepId sequence as
its Java `step/generator/bb2020/*.java` counterpart?

Why this exists
---------------
Closing the BB2020 structural gap means routing `make_step_for` / the shared generators AT the
BB2020 generator files instead of running the BB2025 ones (see
`docs/PARITY_BB2020_STRUCTURAL_GAP.md`). Those Rust files are currently DEAD CODE -- nothing
instantiates them (ITER117) -- so before delegating to one you have to know it has not drifted from
the Java it claims to mirror. This script is that check.

It is TRIAGE, not proof. The scanner is a regex over `sequence.add/jump` and `seq.add/add_labelled/
jump`, so it reports false positives in three known shapes. All three were hand-checked in ITER121
and are equivalent:

  * an `if/else` that emits one step on each branch is counted twice     (bb2020 select.rs)
  * steps emitted from a helper fn defined later in the file sort wrong  (bb2020 select_blitz_target.rs)
  * differing spellings of the same StepId                               (MULTI_BLOCK_FORK vs MultipleBlockFork)

So: treat a MATCH as reassuring and a DIFFER as "go read both files", never the reverse.

Usage:  python scripts/audit_bb2020_generators.py
"""
import difflib
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
JAVA_GEN = os.path.normpath(os.path.join(
    REPO, '..', 'ffb-java', 'ffb', 'ffb-server', 'src', 'main', 'java', 'com', 'fumbbl', 'ffb',
    'server', 'step', 'generator', 'bb2020'))
RUST_GEN = os.path.join(REPO, 'crates', 'ffb-engine', 'src', 'step', 'generator', 'bb2020')

# StepIds the two sources spell differently. Left = normalised Java, right = normalised Rust.
SPELLING = {'multiblockfork': 'multipleblockfork'}


def _normalise(name):
    return SPELLING.get(name.replace('_', '').lower(), name.replace('_', '').lower())


def java_steps(path):
    src = open(path, encoding='utf-8', errors='replace').read()
    steps = []
    for m in re.finditer(r'sequence\.(add|jump)\s*\(\s*(?:StepId\.(\w+))?', src):
        steps.append('JUMP' if m.group(1) == 'jump' else (m.group(2) or '?'))
    return [_normalise(s) for s in steps]


def rust_steps(path):
    src = open(path, encoding='utf-8', errors='replace').read()
    cut = src.find('#[cfg(test)]')
    if cut != -1:
        src = src[:cut]
    steps = []
    for m in re.finditer(r'seq\.(add_labelled|add|jump)\s*\(\s*(?:StepId::(\w+))?', src):
        steps.append('JUMP' if m.group(1) == 'jump' else (m.group(2) or '?'))
    return [_normalise(s) for s in steps]


def rust_filename(java_filename):
    base = java_filename[:-len('.java')]
    out = []
    for i, ch in enumerate(base):
        if ch.isupper() and i:
            out.append('_')
        out.append(ch.lower())
    name = ''.join(out)
    return ('move_' if name == 'move' else name) + '.rs'


def main():
    if not os.path.isdir(JAVA_GEN):
        sys.exit('Java generator dir not found: %s' % JAVA_GEN)

    matched, differing, missing = [], [], []
    for java_file in sorted(f for f in os.listdir(JAVA_GEN) if f.endswith('.java')):
        rust_file = rust_filename(java_file)
        rust_path = os.path.join(RUST_GEN, rust_file)
        if not os.path.exists(rust_path):
            missing.append((java_file, rust_file))
            print('%-34s %-30s NO RUST FILE' % (java_file, rust_file))
            continue
        js = java_steps(os.path.join(JAVA_GEN, java_file))
        rs = rust_steps(rust_path)
        if js == rs:
            matched.append(java_file)
            print('%-34s %-30s MATCH' % (java_file, rust_file))
        else:
            differing.append((java_file, rust_file, js, rs))
            print('%-34s %-30s DIFFER (java %d, rust %d)'
                  % (java_file, rust_file, len(js), len(rs)))

    print('\nMATCH %d   DIFFER %d   MISSING %d' % (len(matched), len(differing), len(missing)))
    for java_file, _rust_file, js, rs in differing:
        print('\n=== %s ===' % java_file)
        for line in difflib.unified_diff(js, rs, 'java', 'rust', lineterm='', n=1):
            print('   ' + line)
    if differing:
        print('\nDIFFER is a prompt to read both files, not a verdict -- see the module docstring '
              'for the three known false-positive shapes.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
