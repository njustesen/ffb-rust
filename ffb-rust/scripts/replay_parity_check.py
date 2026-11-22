"""Goal check for BACKLOG §O (replay processing parity). Exit 0 + 'DONE' only when it is met.

User goal (2026-11-22): "check all the downloaded replays and make sure we can process all of them;
get all states and actions. Make sure Rust engine does the same as Java in that respect."

Meaning, as checked here:
  * The corpus is EVERY downloaded FUMBBL replay (*.ffbr) in the CORPUS directories below, finished
    and unfinished alike, deduplicated by replay id.
  * Java (the harness's replay processor, on the in-repo engine) processes every replay completely:
    no exception, 0 skipped model syncs, 0 record-building errors, and the replayed end state equals
    the replay's own end state (end check OK).
  * Rust processes every replay the same way and produces, for every replay,
      - the SAME state sequence: one line per model sync, the sha256 of the canonical full Game JSON
        (Java Game.toJsonValue(), keys sorted) after that sync is applied, and
      - the SAME action records (the decision records the Java parser extracts), byte for byte.
  * Model-change ids that the in-repo engine does not know (a newer FUMBBL server) are dropped on
    both sides and only reported; they carry state this engine version has no field for.

Where the outputs are: docs/replay_manifest.json
  {"java": {"tool": <jar path>, "out": <dir>}, "rust": {"tool": <exe path>, "out": <dir>}}
Each out dir holds run_info.json {"tool_sha256": ...} and, per replay id:
  <id>.states        one line per applied model sync: "<sync index> <sha256>"
  <id>.actions.jsonl the action records, one JSON object per line
  <id>.status.json   {"exception": null|str, "skipped_syncs": int, "record_errors": {kind: n},
                      "end_check": "OK"|<text>, "unknown_model_changes": {id: n}}
The run_info tool_sha256 must equal the current sha256 of the manifest's tool, so an output can
only count if it came from the tool the manifest names.

Usage (from ffb-rust/): python -I scripts/replay_parity_check.py [--verbose]
"""
import glob
import hashlib
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
MANIFEST = os.path.join(ROOT, 'docs', 'replay_manifest.json')
CORPUS = [
    'D:/ffb_replays/fumbbl',
    'D:/ffb_replays/partial',
    'D:/ffb_replays/window',
    'D:/ffb_replays/fumbbl_night_20261122',
    'D:/ffb_replays/fumbbl_night_20261122/partial',
]
SHOW = 8


def sha256(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def corpus_ids():
    ids = {}
    for d in CORPUS:
        for f in glob.glob(os.path.join(d, '*.ffbr')):
            ids.setdefault(os.path.basename(f)[:-len('.ffbr')], f)
    return ids


def read(path, mode='rb'):
    try:
        with open(path, mode) as f:
            return f.read()
    except OSError:
        return None


def status_problems(st):
    out = []
    if st is None:
        return ['no status']
    if st.get('exception'):
        out.append('exception: ' + str(st['exception'])[:120])
    if st.get('skipped_syncs', 1):
        out.append('skipped syncs %s' % st.get('skipped_syncs'))
    if st.get('record_errors', {'?': 1}):
        out.append('record errors %s' % st.get('record_errors'))
    if st.get('end_check') != 'OK':
        out.append('end check: %s' % st.get('end_check'))
    return out


def side(name, cfg, ids, lines):
    """Validate one engine's outputs; return {id: (states bytes, actions bytes)} for the good ones."""
    tool, out = cfg.get('tool'), cfg.get('out')
    if not tool or not os.path.isfile(tool):
        lines.append('TODO %s tool missing: %s' % (name, tool))
        return None
    info = read(os.path.join(out or '', 'run_info.json'), 'r')
    if info is None:
        lines.append('TODO %s run_info.json missing in %s' % (name, out))
        return None
    want = sha256(tool)
    if json.loads(info).get('tool_sha256') != want:
        lines.append('TODO %s outputs in %s were not made by %s (sha256 %s...)' % (name, out, tool, want[:12]))
        return None
    good, missing, bad, unknown = {}, [], [], {}
    for rid in sorted(ids, key=lambda s: int(s) if s.isdigit() else 0):
        states = read(os.path.join(out, rid + '.states'))
        actions = read(os.path.join(out, rid + '.actions.jsonl'))
        st_raw = read(os.path.join(out, rid + '.status.json'), 'r')
        if states is None or actions is None or st_raw is None:
            missing.append(rid)
            continue
        st = json.loads(st_raw)
        for k, v in (st.get('unknown_model_changes') or {}).items():
            unknown[k] = unknown.get(k, 0) + v
        probs = status_problems(st)
        if probs:
            bad.append((rid, probs))
        good[rid] = (states, actions)
    clean = len(ids) - len(missing) - len(bad)
    tag = 'ok  ' if clean == len(ids) else 'TODO'
    lines.append('%s %s processes %d/%d replays completely (missing outputs %d, incomplete %d)'
                 % (tag, name, clean, len(ids), len(missing), len(bad)))
    for rid in missing[:SHOW]:
        lines.append('       %s %s: no outputs' % (name, rid))
    for rid, probs in bad[:SHOW]:
        lines.append('       %s %s: %s' % (name, rid, '; '.join(probs)))
    if bad:
        kinds = {}
        for _, probs in bad:
            for p in probs:
                key = p.split(':')[0].split(' {')[0]
                kinds[key] = kinds.get(key, 0) + 1
        lines.append('       %s incomplete by kind: %s' % (name, kinds))
    if unknown:
        lines.append('info %s dropped model changes unknown to the in-repo engine: %s' % (name, unknown))
    return good if clean == len(ids) else {k: v for k, v in good.items() if k not in dict(bad)}


def first_diff(a, b):
    al, bl = a.splitlines(), b.splitlines()
    for i in range(min(len(al), len(bl))):
        if al[i] != bl[i]:
            return 'line %d' % (i + 1)
    return 'length %d vs %d lines' % (len(al), len(bl))


def main():
    verbose = '--verbose' in sys.argv
    lines = []
    ids = corpus_ids()
    lines.append('info corpus: %d replays in %d directories' % (len(ids), len(CORPUS)))
    raw = read(MANIFEST, 'r')
    if raw is None:
        lines.append('TODO docs/replay_manifest.json missing (no certified Java + Rust run yet)')
        print('\n'.join(lines))
        print('NOT DONE')
        return 1
    man = json.loads(raw)
    java = side('java', man.get('java', {}), ids, lines)
    rust = side('rust', man.get('rust', {}), ids, lines)
    done = java is not None and rust is not None and len(java) == len(ids) and len(rust) == len(ids)
    if java is not None and rust is not None:
        both = sorted(set(java) & set(rust))
        sdiff = [r for r in both if java[r][0] != rust[r][0]]
        adiff = [r for r in both if java[r][1] != rust[r][1]]
        n = len(ids)
        lines.append('%s states identical: %d/%d replays' % ('ok  ' if not sdiff and len(both) == n else 'TODO',
                                                             len(both) - len(sdiff), n))
        for r in sdiff[:SHOW if not verbose else None]:
            lines.append('       states %s differ at %s' % (r, first_diff(java[r][0].decode(), rust[r][0].decode())))
        lines.append('%s actions identical: %d/%d replays' % ('ok  ' if not adiff and len(both) == n else 'TODO',
                                                              len(both) - len(adiff), n))
        for r in adiff[:SHOW if not verbose else None]:
            lines.append('       actions %s differ at %s' % (r, first_diff(java[r][1].decode(), rust[r][1].decode())))
        done = done and not sdiff and not adiff
    print('\n'.join(lines))
    print('DONE' if done else 'NOT DONE')
    return 0 if done else 1


if __name__ == '__main__':
    sys.exit(main())
