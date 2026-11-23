"""BACKLOG §P: does every Rust report go out in Java's wire shape?

The Java client decodes each report of a `serverModelSync` with `ReportFactory` +
`IReport.initFrom`; one report in the wrong shape drops the whole sync. The parity report
comparison normalises shapes away (null == absent, `{x,y}` == `[x,y]`, sorted modifier lists),
so it cannot answer this. This script compares, per reportId, the JSON *schema* (key -> value
type, recursively; array elements merged) of the Java harness's `toJsonValue()` stream with the
Rust `to_json` stream over parity output directories that hold both.

usage: python -I scripts/report_wire_schema.py <root> [<root> ...] [--max-files N]
Prints one line per (reportId, path) difference; exit 0 when there is none.
"""
import json
import os
import sys
from collections import defaultdict


def typ(v):
    if v is None:
        return "null"
    if isinstance(v, bool):
        return "bool"
    if isinstance(v, int):
        return "int"
    if isinstance(v, float):
        return "num"
    if isinstance(v, str):
        return "str"
    if isinstance(v, list):
        return "arr"
    return "obj"


def walk(v, path, out):
    t = typ(v)
    out[path].add(t)
    if t == "obj":
        for k, x in v.items():
            walk(x, path + "." + k, out)
    elif t == "arr":
        for x in v:
            walk(x, path + "[]", out)


def collect(paths, schemas, counts):
    for p in paths:
        with open(p, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    o = json.loads(line)
                except ValueError:
                    continue
                o.pop("i", None)
                rid = o.get("reportId", "?")
                counts[rid] += 1
                walk(o, "", schemas[rid])


def main():
    args = sys.argv[1:]
    max_files = 400
    if "--max-files" in args:
        i = args.index("--max-files")
        max_files = int(args[i + 1])
        del args[i:i + 2]
    java, rust = [], []
    for root in args:
        for d, _, files in os.walk(root):
            for f in files:
                if f.endswith("_java_reports.jsonl"):
                    r = os.path.join(d, f.replace("_java_reports", "_rust_reports"))
                    if os.path.exists(r) and len(java) < max_files:
                        java.append(os.path.join(d, f))
                        rust.append(r)
    js, rs = defaultdict(lambda: defaultdict(set)), defaultdict(lambda: defaultdict(set))
    jc, rc = defaultdict(int), defaultdict(int)
    collect(java, js, jc)
    collect(rust, rs, rc)
    print(f"files {len(java)} pairs; java ids {len(js)}, rust ids {len(rs)}")
    bad = 0
    for rid in sorted(set(js) | set(rs)):
        if rid not in js:
            print(f"{rid}: rust only ({rc[rid]})")
            continue
        if rid not in rs:
            continue
        j, r = js[rid], rs[rid]
        for path in sorted(set(j) | set(r)):
            jt, rt = j.get(path, set()), r.get(path, set())
            # null is a value Java writes too; only types the other side never shows matter.
            jn, rn = jt - {"null"}, rt - {"null"}
            if path not in r:
                if jn:
                    print(f"{rid} {path}: java {sorted(jt)} rust ABSENT")
                    bad += 1
            elif path not in j:
                print(f"{rid} {path}: java ABSENT rust {sorted(rt)}")
                bad += 1
            elif (rn - jn and jn) or ("null" in rt and "null" not in jt):
                # A null Java never writes is a key Java omits: minimal-json hands initFrom a
                # JSON null literal instead of Java null, which e.g. toJsonObject rejects.
                print(f"{rid} {path}: java {sorted(jt)} rust {sorted(rt)}")
                bad += 1
    print(f"DIFFERENCES {bad}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
