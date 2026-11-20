#!/usr/bin/env python
"""Call Claude Code in a loop until parity is proven on every skill (BACKLOG §M).

Each iteration:
  1. STOP file present (ffb-rust/STOP_LOOP)?  -> exit.
  2. Wait file present (ffb-rust/.loop_wait.json)? -> sleep until its PIDs have exited or its
     deadline passed, so a multi-hour cert costs no sessions that only poll. A session writes it
     when it hands a long background run to the next iteration:
        {"pids": [1234, 5678], "until": "2026-11-21T07:30:00", "reason": "cert7 star suite"}
  3. scripts/skill_parity_check.py -> exit 0 = DONE, the loop ends.
  4. Otherwise run ONE headless session, `claude -p <prompt>`, from the repo root (so it gets the
     project's CLAUDE.md and auto-memory), with the checker's open items in the prompt.

usage (from anywhere):
  python scripts/skill_parity_loop.py [--max-iters 200] [--permission-mode auto]
         [--model opus] [--effort high] [--dry-run]

Logs: ffb-rust/loop_logs/iter_NNN.json (the session's JSON result) + loop.log (one line each).
Stop it any time: Ctrl-C, or create ffb-rust/STOP_LOOP (it stops before the next session).
"""
import argparse
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)                     # ffb-rust/ffb-rust
OUTER = os.path.dirname(ROOT)                    # the git repo root (Claude's project dir)
STOP = os.path.join(ROOT, 'STOP_LOOP')
WAIT = os.path.join(ROOT, '.loop_wait.json')
LOGS = os.path.join(ROOT, 'loop_logs')

PROMPT = """\
You are one iteration of an unattended loop (scripts/skill_parity_loop.py) driving BACKLOG §M to
its goal: PARITY PROVEN ON EVERY SKILL. Nobody is watching this session; it ends when you answer,
and the loop then re-runs the done-check and starts a fresh session.

The goal is defined ONLY by scripts/skill_parity_check.py (read its docstring). Its current output:

{check}

Iteration {it}. {stall}

How to work:
- Start from the auto-memory index, docs/BACKLOG.md §M ("RESUME HERE" + the phase-2b status) and
  the plan at C:/Users/Admin/.claude/plans/figure-out-where-we-precious-dahl.md. All the house
  rules in memory apply (parity fixes are Rust-only and 1:1 against Java, the harness may be
  co-edited, no shell operators in Bash, freeze exe AND jar for gates/certs, the overnight
  resource budget, never touch C:/Users/Admin/niels/ffb, ...).
- Take the most valuable open item and finish ONE coherent unit of work: e.g. make a missing skill
  fire (agent rule in BOTH harnesses, squad data, or a 1:1 engine port), gate it (20 seeds, steps
  AND reports identical, every edition that has the skill), commit with explicit paths.
- A skill that genuinely cannot fire (no player can carry it, or Java itself cannot resolve it)
  goes into ffb-rust/data/agent/skill_exempt.json with a reason of >= 40 chars traced to the Java
  source; record the same in BACKLOG. Never exempt something merely hard.
- The proof at the end: freeze one exe + jar, run the full cert (scripts/cert_chain.ps1, all four
  suites, PARITY_CP pinned), fix every red 1:1 and re-run, run scripts/skill_ledger.py run+report
  with that SAME exe, then write ffb-rust/docs/cert_manifest.json. Never write a manifest that
  points at results you did not see green.
- Long runs: this session's own background tasks die with it. Launch them detached
  (powershell Start-Process ... -PassThru), then write ffb-rust/.loop_wait.json with their PIDs and
  a deadline so the loop sleeps instead of starting sessions that only poll; record what is running
  and how to read the result in BACKLOG "RESUME HERE".
- Before you answer: BACKLOG and memory must say exactly where things stand, and everything
  finished must be committed. Push after a green milestone (user, 2026-11-20: "push now and after").
- Do NOT edit scripts/skill_parity_check.py or scripts/skill_parity_loop.py, and do not weaken a
  gate to make it pass. If you believe the goal definition itself is wrong, say so in BACKLOG
  under "QUESTION FOR THE USER" and work on something else.
- If you are blocked on a decision only the user can make, write it under "QUESTION FOR THE USER"
  in BACKLOG §M, create ffb-rust/STOP_LOOP, and stop.
End your answer with one line: SUMMARY: <what changed this iteration>.
"""


def log(msg):
    line = '[%s] %s' % (datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S'), msg)
    print(line, flush=True)
    os.makedirs(LOGS, exist_ok=True)
    with open(os.path.join(LOGS, 'loop.log'), 'a', encoding='utf-8') as fh:
        fh.write(line + '\n')


def pid_alive(pid):
    out = subprocess.run(['tasklist', '/FI', 'PID eq %d' % pid, '/NH'],
                         capture_output=True, text=True).stdout
    return str(pid) in out


def wait_if_asked():
    """Honour .loop_wait.json: sleep while any listed PID lives and the deadline is ahead."""
    if not os.path.exists(WAIT):
        return
    try:
        w = json.load(open(WAIT, encoding='utf-8'))
    except (OSError, ValueError) as e:
        log('unreadable %s (%s); ignoring it' % (WAIT, e))
        os.remove(WAIT)
        return
    until = datetime.datetime.fromisoformat(w['until']) if w.get('until') else None
    pids = [int(p) for p in w.get('pids', [])]
    log('waiting: %s (pids %s, until %s)' % (w.get('reason', '?'), pids, until))
    while True:
        if os.path.exists(STOP):
            return
        alive = [p for p in pids if pid_alive(p)]
        if until and datetime.datetime.now() >= until:
            log('wait deadline reached (still alive: %s)' % alive)
            break
        if pids and not alive:
            log('waited-for processes have exited')
            break
        if not pids and not until:
            break
        time.sleep(300)
    os.remove(WAIT)


def run_check():
    p = subprocess.run([sys.executable, '-I', os.path.join(HERE, 'skill_parity_check.py')],
                       capture_output=True, text=True, errors='replace')
    return p.returncode == 0, (p.stdout + p.stderr).strip()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--max-iters', type=int, default=200)
    ap.add_argument('--permission-mode', default='auto',
                    choices=['acceptEdits', 'auto', 'bypassPermissions', 'dontAsk'])
    ap.add_argument('--model', default=None)
    ap.add_argument('--effort', default=None, choices=['low', 'medium', 'high', 'xhigh', 'max'])
    ap.add_argument('--claude', default='claude', help='path of the claude CLI')
    ap.add_argument('--dry-run', action='store_true', help='print the first prompt and exit')
    a = ap.parse_args()

    history = []                                   # checker-output hashes, for stall notes
    for it in range(1, a.max_iters + 1):
        if os.path.exists(STOP):
            log('STOP_LOOP present -- stopping (delete it to resume)')
            return 2
        wait_if_asked()
        if os.path.exists(STOP):
            log('STOP_LOOP present -- stopping (delete it to resume)')
            return 2
        done, check = run_check()
        if done:
            log('DONE -- skill_parity_check.py passes:\n' + check)
            return 0
        h = hashlib.sha256(check.encode()).hexdigest()
        same = 0
        for old in reversed(history):
            if old != h:
                break
            same += 1
        history.append(h)
        stall = ''
        if same >= 2:
            stall = ('The done-check output has not changed for %d iterations. That is fine while '
                     'a long run is in flight; otherwise switch tactics.' % same)
        prompt = PROMPT.format(check=check[-6000:], it=it, stall=stall)
        if a.dry_run:
            print(prompt)
            return 0
        cmd = [a.claude, '-p', prompt, '--output-format', 'json',
               '--permission-mode', a.permission_mode, '--permission-prompts', 'none']
        if a.model:
            cmd += ['--model', a.model]
        if a.effort:
            cmd += ['--effort', a.effort]
        log('iteration %d: starting session (%s)' % (it, check.splitlines()[-1]))
        t0 = time.time()
        p = subprocess.run(cmd, cwd=OUTER, capture_output=True, text=True, errors='replace')
        out_path = os.path.join(LOGS, 'iter_%03d.json' % it)
        with open(out_path, 'w', encoding='utf-8') as fh:
            fh.write(p.stdout or '')
            if p.stderr:
                fh.write('\n--- stderr ---\n' + p.stderr)
        summary, turns = '', None
        try:
            res = json.loads(p.stdout)
            text = res.get('result') or ''
            turns = res.get('num_turns')
            summary = next((l for l in reversed(text.splitlines()) if l.startswith('SUMMARY:')),
                           text[-200:].replace('\n', ' '))
        except ValueError:
            summary = 'no JSON result (rc=%s): %s' % (p.returncode, (p.stderr or '')[-300:])
        log('iteration %d: rc=%s %.0f min turns=%s  %s' % (
            it, p.returncode, (time.time() - t0) / 60, turns, summary))
        if p.returncode != 0 and not p.stdout:
            log('session failed to start; pausing 10 min before retrying')
            time.sleep(600)
    log('max iterations (%d) reached without DONE' % a.max_iters)
    return 1


if __name__ == '__main__':
    sys.exit(main())
