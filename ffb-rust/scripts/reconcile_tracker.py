"""Reconcile TRANSLATION_TRACKER.md: upgrade ~ entries to ✓ where Rust file is complete.

For each entry in ffb-model or ffb-mechanics that is currently ~ or ○:
  - If the Rust file exists AND has no todo!/unimplemented! → mark ✓
  - If the Rust file exists but has stubs → keep ~
  - If the Rust file doesn't exist → keep ○

Run from the ffb-rust/ directory:
    python scripts/reconcile_tracker.py
"""
import re
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent.parent
RUST_BASE = REPO_ROOT / "ffb-rust" / "crates"
TRACKER_PATH = REPO_ROOT / "ffb-rust" / "TRANSLATION_TRACKER.md"

STUB_PATTERNS = re.compile(r'\btodo!\s*\(|\bunimplemented!\s*\(')

# Crates where we do the audit
AUDIT_CRATES = {"ffb-model", "ffb-mechanics"}


def is_clean(rust_file: Path) -> bool:
    """Return True if the file exists and has no todo!/unimplemented! markers."""
    if not rust_file.exists():
        return False
    text = rust_file.read_text(encoding="utf-8")
    return not STUB_PATTERNS.search(text)


ROW_RE = re.compile(
    r'^(\|\s*`[^`]+\.java`\s*\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|\s*)([○~✓—])(\s*\|)$'
)

upgraded = 0
kept_partial = 0
kept_missing = 0
lines_out = []

with open(TRACKER_PATH, encoding="utf-8") as f:
    lines = f.readlines()

for line in lines:
    m = ROW_RE.match(line.rstrip("\n"))
    if m:
        prefix, crate, rust_path, status, suffix = m.groups()
        if crate in AUDIT_CRATES and status in ("~", "○"):
            rust_file = RUST_BASE / crate / rust_path
            if rust_file.exists():
                if is_clean(rust_file):
                    line = prefix + "✓" + suffix + "\n"
                    upgraded += 1
                else:
                    # file exists but has stubs — keep ~ (set if was ○)
                    if status == "○":
                        line = prefix + "~" + suffix + "\n"
                    kept_partial += 1
            else:
                kept_missing += 1
    lines_out.append(line)

# Recompute summary counts from updated content
total_o = total_tilde = total_done = total_skip = 0
for line in lines_out:
    m2 = ROW_RE.match(line.rstrip("\n"))
    if m2:
        s = m2.group(4)
        if s == "○": total_o += 1
        elif s == "~": total_tilde += 1
        elif s == "✓": total_done += 1
        elif s == "—": total_skip += 1

total = total_o + total_tilde + total_done

# Update the progress summary table in place
summary_re = re.compile(r'\| Total Java files in scope \| \d+ \|')
new_lines = []
i = 0
while i < len(lines_out):
    line = lines_out[i]
    if "| Total Java files in scope |" in line:
        new_lines.append(f"| Total Java files in scope | {total} |\n")
        i += 1
        new_lines.append(f"| Not started (○) | {total_o} |\n"); i += 1
        new_lines.append(f"| Partial (~) | {total_tilde} |\n"); i += 1
        new_lines.append(f"| Done (✓) | {total_done} |\n"); i += 1
        new_lines.append(f"| Not translating (—) | {total_skip} |\n"); i += 1
    else:
        new_lines.append(line)
        i += 1

with open(TRACKER_PATH, "w", encoding="utf-8") as f:
    f.writelines(new_lines)

print(f"Upgraded to done: {upgraded}")
print(f"Kept partial: {kept_partial}")
print(f"Kept missing: {kept_missing}")
print(f"New totals: done={total_done}  partial={total_tilde}  not-started={total_o}  skip={total_skip}  scope={total}")
print(f"Progress: {total_done/total*100:.1f}% ({total_done}/{total})")
