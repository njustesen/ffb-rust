"""§M phase 2 skill sweep: every skilled squad mirror x edition, parity-gated against Java.
usage: python sksweep.py <frozen-exe> <out-dir> <seeds> <scale> <coverage-bias> [workers]
Resumable: a squad whose result line is already in <out>/results.txt is skipped."""
import concurrent.futures, glob, os, re, subprocess, sys, threading

ROOT = "C:/Users/Admin/niels/ffb-rust/ffb-rust"
exe, out, seeds, scale, cb = sys.argv[1:6]
workers = int(sys.argv[6]) if len(sys.argv) > 6 else 2
os.makedirs(out, exist_ok=True)
res_path = os.path.join(out, "results.txt")
done = set()
if os.path.exists(res_path):
    done = {" ".join(l.split()[:2]) for l in open(res_path, encoding="utf-8") if l.strip()}
# Squad directories under data/ (comma list). Default = the 110 Matched Play `_sk` squads only, so
# a sweep that was already scheduled (cert6) is unchanged; a full skill cert must name both:
# SKSWEEP_DIRS=teams_skills,teams_stats (cert_chain.ps1 does; skill_ledger.jobs('skill') = both).
DIRS = os.environ.get("SKSWEEP_DIRS", "teams_skills").split(",")
jobs = [(ed, os.path.basename(f)[5:-5]) for ed in ["bb2016", "bb2020", "bb2025"] for d in DIRS
        for f in sorted(glob.glob(f"{ROOT}/data/{d}/{ed}/team_*.json"))]
jobs = [j for j in jobs if f"{j[0]} {j[1]}" not in done]
lock = threading.Lock()
# §M 2b(p): stars the skill suite hires on the HOME side (data/teams_skills/star_hires.json).
import json
_hires = json.load(open(f"{ROOT}/data/teams_skills/star_hires.json", encoding="utf-8"))
STAR = {(ed, sq): st for ed, m in _hires.items() if not ed.startswith("_") for sq, st in m.items()}


def run(job):
    ed, sq = job
    env = dict(os.environ, FFB_PARITY_ROOT=f"{out}/root_{ed}_{sq}", PARITY_JVM_CORES="2")
    star = ["--home-star", STAR[(ed, sq)]] if (ed, sq) in STAR else []
    p = subprocess.run([exe, "--home", sq, "--away", sq, "--edition", ed, "--tier", "3", "--seeds", seeds,
                        "--no-abort", "--agent", "heuristic", "--heur-scale", scale, "--heur-classes", "all",
                        "--coverage-bias", cb, "--quiet"] + star, cwd=ROOT, env=env, capture_output=True,
                       text=True, errors="replace")
    txt = p.stdout + p.stderr
    par = re.search(r"PARITY: (\d+)/(\d+)", txt)
    rep = re.search(r"REPORTS: (\d+)/(\d+) games identical", txt)
    line = f"{ed} {sq} parity={par.group(1)+'/'+par.group(2) if par else 'NONE'} " \
           f"reports={rep.group(1)+'/'+rep.group(2) if rep else 'NONE'} rc={p.returncode}"
    with lock:
        with open(res_path, "a", encoding="utf-8") as fh:
            fh.write(line + "\n")
        with open(os.path.join(out, f"log_{ed}_{sq}.txt"), "w", encoding="utf-8") as fh:
            fh.write(txt[-20000:])
        print(line, flush=True)


with concurrent.futures.ThreadPoolExecutor(workers) as ex:
    list(ex.map(run, jobs))
print("sweep done", flush=True)
