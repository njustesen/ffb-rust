"""Client-view suite runner (§Q1.4): parity + reports + client view per job, resumable.

    python -I scripts/view_suite.py <root> <seeds> <exe> <workers> [--minutes=N] [--scale=S]
                                    [--jar=PATH] jobfile [jobfile ...]

Job lines: "<edition> <home> <away>" (mirror / cross) or
           "<edition> <home> <home_star> <away> <away_star>" (star).
Each job runs ffb-parity with FFB_CLIENT_VIEW_DIR set, then scripts/client_view_check.py on the
dumps. A job is OK only when PARITY n/n, REPORTS 0 differ and CLIENT VIEW n steps, n identical.
One line per job is appended to <root>/summary.txt (`OK ...` / `BAD ...`); a job with a
`<name>.check.txt` is skipped, so the run resumes after an interruption. Clean jobs' game and
view dirs are deleted (they are large). Never two runs of one edition+matchup at once.
--minutes=N stops taking new jobs after N minutes (foreground chunks); the last line is
`DONE left=0 bad=N` or `CHUNK END left=N bad=N`. Create <root>/STOP to stop early.
cert_tally.py reads <root>/summary.txt as the suite's client-view verdicts.
"""
import os
import queue
import re
import shutil
import subprocess
import sys
import threading
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

args = sys.argv[1:]
opts = {a.split("=", 1)[0]: a.split("=", 1)[1] for a in args if a.startswith("--")}
pos = [a for a in args if not a.startswith("--")]
root, seeds, exe, nworkers = pos[0], pos[1], pos[2], int(pos[3])
jobfiles = pos[4:]
deadline = time.time() + 60 * float(opts["--minutes"]) if "--minutes" in opts else None
scale = opts.get("--scale", "1.0")
jar = opts.get("--jar")

jobs = []
for jf in jobfiles:
    for line in open(jf):
        f = line.split()
        if len(f) == 3:
            jobs.append((f[0], f[1], f[2], None, None))
        elif len(f) == 5:
            jobs.append((f[0], f[1], f[3], f[2], f[4]))
busy = set()
os.makedirs(root, exist_ok=True)
lock = threading.Lock()
q = queue.Queue()
for j in jobs:
    q.put(j)


def job_name(ed, home, away, hs, aws):
    name = f"{ed}_{home}_vs_{away}"
    if hs:
        strip = lambda s: s[len("star."):] if s.startswith("star.") else s
        name += f"__{strip(hs)}__{strip(aws)}"
    return name


def clean(verdict, out):
    par = any((m := re.search(r"PARITY: (\d+)/(\d+) games match", v)) and m.group(1) == m.group(2)
              for v in verdict)
    rep = any(re.search(r"REPORTS: .* 0 differ, 0 skipped", v) for v in verdict)
    m = re.search(r"(\d+) steps, (\d+) identical", out)
    return par and rep and m is not None and m.group(1) == m.group(2) and m.group(1) != "0"


def run(ed, home, away, hs, aws):
    name = job_name(ed, home, away, hs, aws)
    d = os.path.join(root, name)
    vd = d + "_view"
    chk = os.path.join(root, name + ".check.txt")
    if os.path.exists(chk):
        return
    shutil.rmtree(vd, ignore_errors=True)
    os.makedirs(vd, exist_ok=True)
    stars = ["--home-star", hs, "--away-star", aws] if hs else []
    env = dict(os.environ, FFB_PARITY_ROOT=d, FFB_CLIENT_VIEW_DIR=vd, PARITY_JVM_CORES="2",
               FFB_COVERAGE_JSON=os.path.join(root, name + ".coverage.json"))
    if jar:
        env["PARITY_CP"] = jar
    logp = os.path.join(root, name + ".log")
    with open(logp, "w") as log:
        subprocess.run([exe, "--home", home, "--away", away] + stars + [
            "--edition", ed, "--tier", "3", "--seeds", seeds, "--no-abort", "--agent", "heuristic",
            "--heur-scale", scale, "--heur-classes", "all", "--quiet", "--out", d],
            cwd=REPO, env=env, stdout=log, stderr=subprocess.STDOUT)
    verdict = [l.strip() for l in open(logp, errors="replace") if "PARITY:" in l or "REPORTS:" in l]
    out = subprocess.run([sys.executable, "-I", os.path.join(REPO, "scripts", "client_view_check.py"), vd],
                         capture_output=True, text=True).stdout
    with open(chk, "w") as f:
        f.write("\n".join(verdict) + "\n" + out)
    ok = clean(verdict, out)
    last = out.strip().splitlines()[-1] if out.strip() else "no output"
    if ok:
        shutil.rmtree(vd, ignore_errors=True)
        shutil.rmtree(d, ignore_errors=True)
    line = f"{'OK ' if ok else 'BAD'} {name} | {' | '.join(verdict)} | {last}"
    with lock:
        with open(os.path.join(root, "summary.txt"), "a") as s:
            s.write(line + "\n")
        if not ok:
            print(line, flush=True)


def worker():
    while True:
        try:
            j = q.get_nowait()
        except queue.Empty:
            return
        if os.path.exists(os.path.join(root, "STOP")) or (deadline and time.time() > deadline):
            return
        key = j[:3]
        with lock:
            clash = key in busy
            if not clash:
                busy.add(key)
        if clash:
            q.put(j)
            time.sleep(1)
            continue
        try:
            run(*j)
        except Exception as e:  # keep the worker alive; no check.txt, so a rerun retries the job
            with lock:
                print("ERR", j, e, flush=True)
        finally:
            with lock:
                busy.discard(key)


ts = [threading.Thread(target=worker) for _ in range(nworkers)]
for t in ts:
    t.start()
for t in ts:
    t.join()
left = sum(1 for j in jobs if not os.path.exists(os.path.join(root, job_name(*j) + ".check.txt")))
summary = os.path.join(root, "summary.txt")
bad = sum(1 for l in open(summary) if l.startswith("BAD")) if os.path.exists(summary) else 0
if left == 0:
    # §Q1.5: enforce the tier-3 coverage checklist on the whole suite (every job's
    # <name>.coverage.json summed); the verdict lands in <root>/coverage.txt for cert_tally.py.
    cov = subprocess.run([exe, "--coverage-merge", root], cwd=REPO, capture_output=True, text=True,
                         errors="replace").stdout
    with open(os.path.join(root, "coverage.txt"), "w") as f:
        f.write(cov)
    print(cov.strip().splitlines()[-1] if cov.strip() else "COVERAGE: no output", flush=True)
print(("DONE" if left == 0 else "CHUNK END") + f" left={left} bad={bad}", flush=True)
