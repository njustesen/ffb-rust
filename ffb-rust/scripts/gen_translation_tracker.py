"""Generate TRANSLATION_TRACKER.md from Java source file listing.

Run from the ffb-rust/ directory:
    python scripts/gen_translation_tracker.py

Updates TRANSLATION_TRACKER.md in place, preserving any manually-set status
values (○/~/✓) by reading the existing file before overwriting.
"""
import os
import re
from pathlib import Path

# Paths — adjust if repo layout changes
SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent.parent  # ffb-rust repo root
JAVA_BASE = REPO_ROOT / "ffb-java"
RUST_BASE = REPO_ROOT / "ffb-rust" / "crates"
OUT_PATH = REPO_ROOT / "ffb-rust" / "TRANSLATION_TRACKER.md"


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
        if pkg in ('bb2016', 'bb2020', 'bb2025'):
            return "ffb-model", f"src/{pkg}/{sub}"
        return "ffb-model", f"src/{pkg}/{sub}"

    if module_name == 'ffb-server':
        server_parts = rel[1:] if rel[0] == 'server' else rel
        if not server_parts:
            return "ffb-engine", f"src/{snake}.rs"
        server_pkg = server_parts[0]
        subpath = '/'.join(server_parts[1:-1])
        sub = f"{subpath}/{snake}.rs" if subpath else f"{snake}.rs"

        if server_pkg == 'step':
            step_dirs = '/'.join(server_parts[1:-1])
            step_file = java_to_snake(server_parts[-1].replace('.java', '')) + '.rs'
            full_step = f"{step_dirs}/{step_file}" if step_dirs else step_file
            return "ffb-engine", f"src/step/{full_step}"
        if server_pkg == 'skillbehaviour':
            sb_dirs = '/'.join(server_parts[1:-1])
            sb_file = java_to_snake(server_parts[-1].replace('.java', '')) + '.rs'
            full_sb = f"{sb_dirs}/{sb_file}" if sb_dirs else sb_file
            return "ffb-engine", f"src/skill_behaviour/{full_sb}"
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
        cl_rel = '/'.join(rel)
        return "ffb-client", f"src/{cl_rel.replace('.java', '.rs')}"

    return "—", "—"


def check_rust_exists(crate: str, rust_path: str) -> str:
    if crate == "—":
        return "—"
    full_path = RUST_BASE / crate / rust_path
    return "~" if full_path.exists() else "○"


def get_group_key(display: str, module_name: str) -> str:
    parts = display.split('/')
    if module_name == 'ffb-server':
        if len(parts) < 2 or parts[1].endswith('.java'):
            return f"{parts[0]}/root"
        if parts[1] in ('step', 'skillbehaviour') and len(parts) >= 3 and not parts[2].endswith('.java'):
            return f"{parts[0]}/{parts[1]}/{parts[2]}"
        return f"{parts[0]}/{parts[1]}"
    elif module_name == 'ffb-client-logic':
        if len(parts) < 2 or parts[1].endswith('.java'):
            return f"{parts[0]}/root"
        return f"{parts[0]}/{parts[1]}"
    else:
        return parts[0] if len(parts) > 1 else 'root'


# Load existing status overrides from the current tracker file
existing_status = {}
if OUT_PATH.exists():
    for line in OUT_PATH.read_text(encoding='utf-8').splitlines():
        m = re.match(r'\|\s*`([^`]+\.java)`\s*\|[^|]*\|[^|]*\|\s*([○~✓—])\s*\|', line)
        if m:
            java_file, status = m.group(1), m.group(2)
            existing_status[java_file] = status

modules = [
    ("ffb-common", JAVA_BASE / "ffb-common/src/main/java"),
    ("ffb-server", JAVA_BASE / "ffb-server/src/main/java"),
    ("ffb-client-logic", JAVA_BASE / "ffb-client-logic/src/main/java"),
]

sections = {}
for module_name, java_src in modules:
    if not java_src.exists():
        print(f"WARNING: {java_src} does not exist, skipping")
        continue
    files = sorted(java_src.rglob("*.java"))
    entries = []
    for f in files:
        rel = str(f.relative_to(java_src)).replace(os.sep, '/')
        crate, rust_path = get_rust_target(rel, module_name)
        display = rel.replace('com/fumbbl/ffb/', '')
        # Preserve manually-set status, fall back to auto-detect
        if display in existing_status:
            status = existing_status[display]
        else:
            status = check_rust_exists(crate, rust_path)
        entries.append((display, crate, rust_path, status))
    sections[module_name] = entries
    print(f"{module_name}: {len(entries)} files")

lines = []
lines.append("# FFB Java to Rust 1:1 Translation Tracker\n\n")
lines.append("<!-- Auto-generated skeleton from Java source. Update status cells manually as files are translated. -->\n")
lines.append("<!-- To regenerate: python scripts/gen_translation_tracker.py -->\n\n")
lines.append("## How to Use\n\n")
lines.append("This file tracks every Java class in ffb-common, ffb-server, and ffb-client-logic against its target Rust file.\n\n")
lines.append("1. When you start translating a file: change its status to `~`\n")
lines.append("2. When it matches the Java source 1:1 and parity is confirmed: change to `✓`\n")
lines.append("3. When a race passes T3b 100/100, all files exercised by that race should be `✓`\n\n")
lines.append("**Workflow per Java file:**\n")
lines.append("- Read the Java source in `ffb-java/<module>/src/main/java/<path>.java`\n")
lines.append("- Find or create the corresponding Rust file at the listed Rust Target path\n")
lines.append("- Translate method by method, matching dice consumption order, conditions, and state transitions exactly\n")
lines.append("- Run `cargo test` after each file\n")
lines.append("- Run `python scripts/parity_run.py --home amazon --away amazon --seeds 1-10` to catch regressions\n\n")
lines.append("## Status Legend\n\n")
lines.append("- `○` Not started -- no Rust equivalent exists\n")
lines.append("- `~` Partial -- Rust equivalent exists but not yet 1:1 with Java\n")
lines.append("- `✓` Done -- Rust matches Java line-by-line, parity confirmed\n")
lines.append("- `—` Not translating (GUI, DB, WebSocket layer, serialization utils)\n\n")
lines.append("---\n\n")
lines.append("## Progress Summary\n\n")

total_o = total_tilde = total_done = total_skip = 0
for entries in sections.values():
    for _, _, _, status in entries:
        if status == "○": total_o += 1
        elif status == "~": total_tilde += 1
        elif status == "✓": total_done += 1
        elif status == "—": total_skip += 1

total = total_o + total_tilde + total_done
lines.append("| Metric | Count |\n")
lines.append("|--------|-------|\n")
lines.append(f"| Total Java files in scope | {total} |\n")
lines.append(f"| Not started (○) | {total_o} |\n")
lines.append(f"| Partial (~) | {total_tilde} |\n")
lines.append(f"| Done (✓) | {total_done} |\n")
lines.append(f"| Not translating (—) | {total_skip} |\n\n")
lines.append("---\n\n")

for module_name, entries in sections.items():
    lines.append(f"## Module: {module_name}\n\n")
    by_pkg = {}
    for entry in entries:
        display = entry[0]
        key = get_group_key(display, module_name)
        by_pkg.setdefault(key, []).append(entry)
    for pkg in sorted(by_pkg.keys()):
        pkg_entries = by_pkg[pkg]
        lines.append(f"### {pkg}/ ({len(pkg_entries)} files)\n\n")
        lines.append("| Java File | Rust Crate | Rust Target | Status |\n")
        lines.append("|-----------|-----------|-------------|--------|\n")
        for display, crate, rust_path, status in pkg_entries:
            lines.append(f"| `{display}` | `{crate}` | `{rust_path}` | {status} |\n")
        lines.append("\n")

content = ''.join(lines)
OUT_PATH.write_text(content, encoding='utf-8')
print(f"Written {len(lines)} lines ({len(content)} chars) to {OUT_PATH}")
