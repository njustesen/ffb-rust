#!/usr/bin/env python3
"""ZW.0: rewrite ffb-client-logic tracker rows from false blanket ✓ to honest ○/—.

GUI/Swing-AWT subdirectories (dialog, ui, layer, overlay, sound) and a fixed list of
root-level GUI files are marked `—` (not translating, no headless equivalent).
Everything else in the ffb-client-logic module (animation, factory, handler, model,
net, report, state, util, plus the non-GUI root files) is marked `○` (not started —
the existing Rust files are ~10-line placeholder stubs, not real translations).
"""
import re
import sys

TRACKER = "TRANSLATION_TRACKER.md"

GUI_SUBDIRS = {"dialog", "ui", "layer", "overlay", "sound"}

# client/root/ files that are pure Swing/AWT rendering/layout/input glue.
ROOT_GUI_FILES = {
    "ActionKey.java", "ActionKeyAction.java", "ActionKeyBindings.java",
    "ActionKeyGroup.java", "ActionKeyMultiAction.java", "ClientLayout.java",
    "Component.java", "DimensionProvider.java", "DugoutDimensionProvider.java",
    "FantasyFootballClient.java", "FieldComponent.java", "FontCache.java",
    "GameTitle.java", "IconCache.java", "LayoutSettings.java",
    "ParagraphStyle.java", "PitchDimensionProvider.java", "RenderContext.java",
    "StyleProvider.java", "TextStyle.java", "UiDimensionProvider.java",
    "UtilStyle.java",
}

row_re = re.compile(
    r"^\| `client/([^`]+)\.java` \| `ffb-client` \| `(src/client/[^`]+)` \| ✓ \|\s*$"
)


def classify(java_path):
    """java_path is the part after 'client/' and before '.java', e.g. 'report/bb2016/Foo'."""
    parts = java_path.split("/")
    filename = parts[-1] + ".java"
    if len(parts) == 1:
        return "—" if filename in ROOT_GUI_FILES else "○"
    top_subdir = parts[0]
    return "—" if top_subdir in GUI_SUBDIRS else "○"


def main():
    with open(TRACKER, encoding="utf-8") as f:
        lines = f.readlines()

    in_client_module = False
    counts = {"—": 0, "○": 0, "unchanged": 0}
    out = []
    for line in lines:
        if line.startswith("## Module: ffb-client-logic"):
            in_client_module = True
        elif line.startswith("## Module: ") and not line.startswith("## Module: ffb-client-logic"):
            in_client_module = False

        if in_client_module:
            m = row_re.match(line.rstrip("\n"))
            if m:
                java_path, target = m.group(1), m.group(2)
                status = classify(java_path)
                counts[status] += 1
                out.append(f"| `client/{java_path}.java` | `ffb-client` | `{target}` | {status} |\n")
                continue
        out.append(line)

    with open(TRACKER, "w", encoding="utf-8") as f:
        f.writelines(out)

    print("Reclassified: {} -> pending, {} -> skip".format(counts["○"], counts["—"]))


if __name__ == "__main__":
    sys.exit(main())
