"""Tally a cert's suite verdicts: judged / green / red per suite, reds listed.

A gate line is green when it matches
    PARITY: n/n games match ... REPORTS: n/n games identical, 0 differ, 0 skipped.
(the "required coverage items are MISSING" note on some mirror gates is not a parity red).

Usage:
  python -I scripts/cert_tally.py cert9
    -> mirror ../mirror_<name>, cross ../cross_<name>, star ../starjobs_<name> (scratch/sweep_w*.log,
       expected 330 / 4012 / 4012) and skill D:/ffb_sksweep/<name>/results.txt (expected 121,
       lines 'parity=n/n reports=n/n rc=0').
"""
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
GREEN = re.compile(r"PARITY: (\d+)/\1 games match.*REPORTS: \1/\1 games identical, 0 differ, 0 skipped\.")
SKILL_GREEN = re.compile(r"parity=(\d+)/\1 reports=(\d+)/\2 rc=0")
VIEW_GREEN = re.compile(r"^OK .*CLIENT VIEW: (\d+) steps, \1 identical")


def tally(label, files, marker, green_re, expect):
    judged = green = 0
    reds = []
    for f in files:
        for line in open(f, encoding="utf-8", errors="replace"):
            if marker not in line:
                continue
            judged += 1
            if green_re.search(line):
                green += 1
            else:
                reds.append("%s: %s" % (f, line.strip()[:300]))
    status = "GREEN" if judged == expect and green == expect else "open"
    print("%-6s judged %5d/%d  green %5d  red %d  %s" % (label, judged, expect, green, len(reds), status))
    for r in reds[:30]:
        print("   RED", r)
    return status == "GREEN"


def main():
    name = sys.argv[1]
    ok = True
    for label, d, expect in (("mirror", "mirror_" + name, 330), ("cross", "cross_" + name, 4012),
                             ("star", "starjobs_" + name, 4012)):
        files = sorted(glob.glob(os.path.join(ROOT, d, "scratch", "sweep_w*.log")))
        ok &= tally(label, files, "PARITY:", GREEN, expect)
    skill = os.path.join("D:/ffb_sksweep", name, "results.txt")
    ok &= tally("skill", [skill] if os.path.exists(skill) else [], "parity=", SKILL_GREEN, 121)
    # §Q1.4: the client view is part of the certification. scripts/view_suite.py writes one
    # `OK`/`BAD` line per job to <root>/summary.txt; roots D:/ffb_view_<name>_<suite>
    # (mirror = one root per heur scale: _mirror0, _mirror1.0, _mirror1e6, 110 jobs each).
    for label, suffixes, expect in (("view-m", ("mirror0", "mirror1.0", "mirror1e6"), 330),
                                    ("view-c", ("cross",), 4012), ("view-s", ("star",), 4012)):
        files = [os.path.join("D:/", "ffb_view_%s_%s" % (name, s), "summary.txt") for s in suffixes]
        ok &= tally(label, [f for f in files if os.path.exists(f)], " ", VIEW_GREEN, expect)
        # §Q1.5: suite-level coverage (view_suite.py writes coverage.txt when a root is DONE).
        for s in suffixes:
            cov = os.path.join("D:/", "ffb_view_%s_%s" % (name, s), "coverage.txt")
            text = open(cov, errors="replace").read() if os.path.exists(cov) else ""
            cov_ok = "all required items present" in text
            print("%-6s coverage %-10s %s" % (label, s, "OK" if cov_ok else ("MISSING items" if text else "not run")))
            ok &= cov_ok
    print("ALL SUITES GREEN (incl. client view)" if ok else "NOT ALL GREEN")


if __name__ == "__main__":
    main()
