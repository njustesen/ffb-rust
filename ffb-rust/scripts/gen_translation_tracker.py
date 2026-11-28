"""Generate TRANSLATION_TRACKER.md from the Java sources AND the Rust tree (HARDENING_BACKLOG §Q6.1).

Run from anywhere:
    python -I scripts/gen_translation_tracker.py

For every Java class in ffb-common / ffb-server / ffb-client-logic (ffb-java/ffb/<module>) it finds the
Rust file ON DISK and derives the row from the filesystem, so the tracker can no longer claim a file
that does not exist:

* Rust target: the predicted path if it exists, else the file with the same snake_case name in the
  same crate (preferring the same edition folder: bb2016 / bb2020 / bb2025 / mixed), else none.
* Status: `—` for excluded layers; `○` when no Rust file exists (this OVERRIDES any older hand-set
  status -- the old tracker had 77 `✓` rows naming files that did not exist); otherwise the previous
  hand-set `✓`/`~` is kept, defaulting to `~`.
* Driver (step files only, i.e. files with `impl Step for`): `live` when `step/driver.rs` dispatches
  the module (named by path, or glob-imported folder + module name), else `dead twin` -- an edition
  copy the engine never runs because the driver serves that edition from a shared step.
"""
import os
import re
from collections import defaultdict
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent.parent  # repo root (holds ffb-java/ and ffb-rust/)
JAVA_BASE = REPO_ROOT / "ffb-java" / "ffb"
RUST_BASE = REPO_ROOT / "ffb-rust" / "crates"
OUT_PATH = REPO_ROOT / "ffb-rust" / "TRANSLATION_TRACKER.md"
DRIVER = RUST_BASE / "ffb-engine" / "src" / "step" / "driver.rs"
EDITIONS = ("bb2016", "bb2020", "bb2025", "mixed")


def java_to_snake(name: str) -> str:
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()


def get_ffb_rel(java_rel: str):
    """Strip com/fumbbl/ffb/ prefix, return list of path parts."""
    parts = list(Path(java_rel).parts)
    try:
        idx = parts.index('ffb') + 1
    except ValueError:
        return None
    return parts[idx:]


EXCLUDED_PREFIXES = [
    'server/db/', 'server/handler/', 'server/net/',
    'server/admin/', 'server/commandline/', 'server/request/',
    'ffb/json/', 'ffb/xml/', 'ffb/stats/', 'ffb/report/',
]
EXCLUDED_PACKAGES_COMMON = {'json', 'xml', 'stats', 'report'}


def get_rust_target(java_rel: str, module_name: str):
    """The predicted (crate, path) -- the convention the port was written to."""
    for ex in EXCLUDED_PREFIXES:
        if ex in java_rel:
            return "—", "—"
    rel = get_ffb_rel(java_rel)
    if rel is None:
        return "—", "—"
    filename = rel[-1].replace('.java', '')
    snake = java_to_snake(filename)
    pkg = rel[0] if rel else 'root'

    if module_name == 'ffb-common':
        if len(rel) == 1:
            return "ffb-model", f"src/model/{snake}.rs"
        if pkg in EXCLUDED_PACKAGES_COMMON:
            return "—", "—"
        subpath = '/'.join(rel[1:-1])
        sub = f"{subpath}/{snake}.rs" if subpath else f"{snake}.rs"
        mapping = {
            'model': ("ffb-model", f"src/model/{sub}"),
            'skill': ("ffb-model", f"src/skill/{sub}"),
            'mechanics': ("ffb-mechanics", f"src/{sub}"),
            'modifiers': ("ffb-mechanics", f"src/modifiers/{sub}"),
            'net': ("ffb-protocol", f"src/{sub}"),
            'injury': ("ffb-model", f"src/injury/{sub}"),
            'inducement': ("ffb-model", f"src/inducement/{sub}"),
            'dialog': ("ffb-model", f"src/dialog/{sub}"),
            'kickoff': ("ffb-model", f"src/kickoff/{sub}"),
            'marking': ("ffb-model", f"src/marking/{sub}"),
            'factory': ("ffb-model", f"src/factory/{sub}"),
            'util': ("ffb-model", f"src/util/{sub}"),
            'option': ("ffb-model", f"src/option/{sub}"),
        }
        if pkg in mapping:
            return mapping[pkg]
        return "ffb-model", f"src/{pkg}/{sub}"

    if module_name == 'ffb-server':
        server_parts = rel[1:] if rel[0] == 'server' else rel
        if not server_parts:
            return "ffb-engine", f"src/{snake}.rs"
        server_pkg = server_parts[0]
        subpath = '/'.join(server_parts[1:-1])
        sub = f"{subpath}/{snake}.rs" if subpath else f"{snake}.rs"
        if server_pkg == 'step':
            return "ffb-engine", f"src/step/{sub}"
        if server_pkg == 'skillbehaviour':
            return "ffb-engine", f"src/skill_behaviour/{sub}"
        mapping = {
            'mechanic': ("ffb-engine", f"src/mechanic/{sub}"),
            'util': ("ffb-engine", f"src/util/{sub}"),
            'model': ("ffb-engine", f"src/model/{sub}"),
            'inducements': ("ffb-engine", f"src/inducements/{sub}"),
            'injury': ("ffb-engine", f"src/injury/{sub}"),
            'marking': ("ffb-engine", f"src/marking/{sub}"),
            'factory': ("ffb-engine", f"src/factory/{sub}"),
        }
        if server_pkg in mapping:
            return mapping[server_pkg]
        if len(server_parts) == 1:
            return "ffb-engine", f"src/{snake}.rs"
        return "—", "—"

    if module_name == 'ffb-client-logic':
        return "ffb-client", "src/" + '/'.join(rel[:-1] + [snake + '.rs'])

    return "—", "—"


# ── the Rust tree, indexed by crate and by file name ──────────────────────────────────────────────
by_name = defaultdict(list)  # (crate, "file.rs") -> ["src/...", ...]
for crate_dir in RUST_BASE.iterdir():
    if not crate_dir.is_dir():
        continue
    for f in crate_dir.rglob("*.rs"):
        rel = f.relative_to(crate_dir).as_posix()
        if rel.startswith("target/"):
            continue
        by_name[(crate_dir.name, f.name)].append(rel)


def pick(cands, predicted):
    """One candidate path: the only one, or the one in the predicted edition folder; else None."""
    if len(cands) == 1:
        return cands[0]
    wanted = [p for p in Path(predicted).parts if p in EDITIONS]
    hits = [c for c in cands if wanted and wanted[0] in Path(c[1]).parts]
    if not wanted:
        hits = [c for c in cands if not any(e in Path(c[1]).parts for e in EDITIONS)]
    return hits[0] if len(hits) == 1 else None


def locate(crate: str, predicted: str):
    """(crate, path) of the Rust file actually on disk for a predicted target, or None. Falls back to
    the same file name in the predicted crate, then in any crate (several ports moved, e.g. the
    modifier factories live in ffb-mechanics)."""
    if crate == "—":
        return None
    if (RUST_BASE / crate / predicted).exists():
        return crate, predicted
    name = Path(predicted).name
    same = [(crate, p) for p in by_name.get((crate, name), [])]
    if same:
        return pick(same, predicted)
    anywhere = [(c, p) for (c, n), ps in by_name.items() if n == name for p in ps]
    return pick(anywhere, predicted) if anywhere else None


# ── driver.rs liveness for step files ─────────────────────────────────────────────────────────────
driver = DRIVER.read_text(encoding="utf-8") if DRIVER.exists() else ""
globbed = set(re.findall(r"use crate::(step(?:::\w+)+)::\*;", driver))


def driver_verdict(crate: str, rust_path: str) -> str:
    if crate != "ffb-engine" or not rust_path.startswith("src/step/"):
        return ""
    full = RUST_BASE / crate / rust_path
    if "impl Step for" not in full.read_text(encoding="utf-8", errors="replace"):
        return ""
    module = rust_path[len("src/"):-len(".rs")].replace("/", "::")  # step::bb2025::block::step_x
    parent, name = module.rsplit("::", 1)
    if module in driver or (parent in globbed and re.search(rf"\b{name}::", driver)):
        return "live"
    return "dead twin"


# ── previous hand-set statuses (kept only where the Rust file exists) ─────────────────────────────
existing_status = {}
if OUT_PATH.exists():
    for line in OUT_PATH.read_text(encoding='utf-8').splitlines():
        m = re.match(r'\|\s*`([^`]+\.java)`\s*\|[^|]*\|[^|]*\|\s*([○~✓—])\s*\|', line)
        if m:
            existing_status[m.group(1)] = m.group(2)

modules = [
    ("ffb-common", JAVA_BASE / "ffb-common/src/main/java"),
    ("ffb-server", JAVA_BASE / "ffb-server/src/main/java"),
    ("ffb-client-logic", JAVA_BASE / "ffb-client-logic/src/main/java"),
]

sections = {}
demoted = 0
for module_name, java_src in modules:
    if not java_src.exists():
        raise SystemExit(f"missing Java sources: {java_src}")
    entries = []
    for f in sorted(java_src.rglob("*.java")):
        rel = f.relative_to(java_src).as_posix()
        crate, predicted = get_rust_target(rel, module_name)
        display = rel.replace('com/fumbbl/ffb/', '')
        if crate == "—":
            entries.append((display, "—", "—", "—", ""))
            continue
        found = locate(crate, predicted)
        if found is None:
            if existing_status.get(display) in ("✓", "~"):
                demoted += 1
            entries.append((display, crate, predicted, "○", ""))
            continue
        status = existing_status.get(display) if existing_status.get(display) in ("✓", "~") else "~"
        fcrate, fpath = found
        entries.append((display, fcrate, fpath, status, driver_verdict(fcrate, fpath)))
    sections[module_name] = entries
    print(f"{module_name}: {len(entries)} files")

counts = defaultdict(int)
for entries in sections.values():
    for e in entries:
        counts[e[3]] += 1
        if e[4]:
            counts[e[4]] += 1

lines = [
    "# FFB Java to Rust 1:1 Translation Tracker\n\n",
    "<!-- Generated from the Java sources and the Rust tree. Regenerate: python -I scripts/gen_translation_tracker.py -->\n\n",
    "Every Java class in ffb-common, ffb-server and ffb-client-logic (`ffb-java/ffb/`) against the Rust file\n",
    "that holds its port, located on disk. A row can only say `~`/`✓` when that file exists.\n\n",
    "## Legend\n\n",
    "- `○` no Rust file found\n",
    "- `~` Rust file exists (not marked done by hand)\n",
    "- `✓` Rust file exists and was marked 1:1 by hand (kept across regenerations)\n",
    "- `—` not translated (DB, network handlers, JSON/XML, stats, reports: other layers)\n",
    "- **Driver** (step files only): `live` = dispatched by `step/driver.rs`; `dead twin` = an edition copy\n",
    "  the engine never runs (the driver serves that edition from a shared step)\n\n",
    "## Summary\n\n",
    "| Metric | Count |\n|--------|-------|\n",
    f"| Rust file found, done (✓) | {counts['✓']} |\n",
    f"| Rust file found (~) | {counts['~']} |\n",
    f"| No Rust file (○) | {counts['○']} |\n",
    f"| Not translated (—) | {counts['—']} |\n",
    f"| Step files: live / dead twin | {counts['live']} / {counts['dead twin']} |\n\n",
    "---\n\n",
]
for module_name, entries in sections.items():
    lines.append(f"## Module: {module_name}\n\n")
    by_pkg = defaultdict(list)
    for e in entries:
        parts = e[0].split('/')
        if module_name == 'ffb-server' and len(parts) > 3 and parts[1] in ('step', 'skillbehaviour'):
            key = '/'.join(parts[:3])
        elif len(parts) > 2:
            key = '/'.join(parts[:2])
        else:
            key = parts[0] if len(parts) > 1 else 'root'
        by_pkg[key].append(e)
    for pkg in sorted(by_pkg):
        pe = by_pkg[pkg]
        lines.append(f"### {pkg}/ ({len(pe)} files)\n\n")
        lines.append("| Java File | Rust Crate | Rust Target | Status | Driver |\n")
        lines.append("|-----------|-----------|-------------|--------|--------|\n")
        for display, crate, path, status, drv in pe:
            lines.append(f"| `{display}` | `{crate}` | `{path}` | {status} | {drv} |\n")
        lines.append("\n")

OUT_PATH.write_text(''.join(lines), encoding='utf-8')
names = {"✓": "done", "~": "found", "○": "missing", "—": "excluded"}
print({names.get(k, k): v for k, v in counts.items()}, f"demoted {demoted} rows that claimed a missing Rust file")
