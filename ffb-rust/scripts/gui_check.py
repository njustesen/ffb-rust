"""BACKLOG §P done-check: the unmodified Java client plays whole games on the Rust ffb-server.

Plays N games, each with a fresh `ffb-server` (working dir = the Java `ffb-server` tree, so it
loads `teams/`, `rosters/` and `setups/` exactly as Java standalone does) and two headless Java
clients (`AiMain`, the real Swing client with a hidden window) for coaches Kalimar (home) and
BattleLore (away), cycling through their team pairs.

A game passes when the server logs `GAME FINISHED` and none of these happened:
  - a server panic, an engine panic or an undecodable / undecoded client command,
  - an exception in either Java client,
  - the engine waiting for a decision with no Java dialog set (the client could not answer).
Reported but allowed: whole-game fallbacks (a model change the diff does not express yet), commands
the engine ignored (Java's UNHANDLED_COMMAND -- AiMain re-sends while it waits).

Prints DONE when every game passes. Logs go to --out (outside git: they carry coach names).

Usage: python -I scripts/gui_check.py [--games N] [--timeout S] [--out DIR] [--jar JAR] [--exe EXE]
"""
import argparse
import glob
import os
import re
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
JAVA_SERVER = os.path.normpath(os.path.join(ROOT, '..', 'ffb-java', 'ffb', 'ffb-server'))
DEFAULT_EXE = os.path.join(ROOT, 'target', 'debug', 'ffb-server.exe')
DEFAULT_JAR = os.path.normpath(os.path.join(ROOT, '..', 'ffb-java', 'ffb', 'ffb-ai', 'target',
                                            'ffb-ai-jar-with-dependencies.jar'))
PORT = 22227


def setup_team_ids():
    """Team ids with a saved setup in `setups/*.xml` -- AiMain can only set up by loading one
    (`sendTeamSetupLoad`), so a team without one loops on the setup screen, on the Java server too."""
    ids = set()
    for path in glob.glob(os.path.join(JAVA_SERVER, 'setups', '*.xml')):
        with open(path, encoding='utf-8', errors='replace') as f:
            m = re.search(r'teamId="([^"]*)"', f.read())
        if m:
            ids.add(m.group(1))
    return ids


def teams_of(coach):
    """(team id, team name) of every `teams/*.xml` whose <coach> is `coach` and that has a saved
    setup, sorted by id."""
    with_setup = setup_team_ids()
    out = []
    for path in sorted(glob.glob(os.path.join(JAVA_SERVER, 'teams', '*.xml'))):
        with open(path, encoding='utf-8', errors='replace') as f:
            text = f.read()
        c = re.search(r'<coach>([^<]*)</coach>', text)
        i = re.search(r'<team id="([^"]*)"', text)
        n = re.search(r'<name>([^<]*)</name>', text)
        if c and i and n and c.group(1).strip() == coach and i.group(1) in with_setup:
            out.append((i.group(1), n.group(1).strip()))
    return out


def count(path, pattern):
    if not os.path.exists(path):
        return 0
    rx = re.compile(pattern)
    with open(path, encoding='utf-8', errors='replace') as f:
        return sum(1 for line in f if rx.search(line))


def play(n, home, away, args):
    """One game; returns the verdict dict."""
    tag = f'game{n:02d}'
    log = os.path.join(args.out, f'{tag}_server.log')
    env = dict(os.environ, RUST_LOG='info,ffb_server=debug')
    with open(log, 'w') as server_err:
        server = subprocess.Popen([args.exe, f'127.0.0.1:{PORT}'], cwd=JAVA_SERVER, stderr=server_err,
                                  stdout=subprocess.DEVNULL, env=env)
    deadline = time.time() + 60
    while time.time() < deadline and count(log, 'listening') == 0:
        time.sleep(0.5)
    clients = []
    for coach, (team_id, team_name), extra in (('Kalimar', home, ['-home']), ('BattleLore', away, [])):
        out = open(os.path.join(args.out, f'{tag}_{coach}.log'), 'w')
        err = open(os.path.join(args.out, f'{tag}_{coach}.err'), 'w')
        cmd = ['java', '-cp', args.jar, 'com.fumbbl.ffb.ai.AiMain', '-coach', coach, '-password', 'test',
               '-server', 'localhost', '-port', str(PORT), '-teamId', team_id, '-teamName', team_name] + extra
        clients.append((subprocess.Popen(cmd, stdout=out, stderr=err), out, err))
        time.sleep(1)
    start = time.time()
    finished = False
    while time.time() - start < args.timeout:
        time.sleep(2)
        if count(log, 'GAME FINISHED') > 0 or server.poll() is not None:
            finished = count(log, 'GAME FINISHED') > 0
            time.sleep(3)  # let the end-of-game dialogs settle
            break
    for proc, out, err in clients:
        proc.kill()
        out.close()
        err.close()
    server.kill()
    server.wait()
    client_exceptions = sum(count(os.path.join(args.out, f'{tag}_{c}.err'), r'Exception')
                            for c in ('Kalimar', 'BattleLore'))
    result = {
        'game': tag, 'home': home[0], 'away': away[0], 'finished': finished,
        'seconds': round(time.time() - start),
        'panics': count(log, r'panicked|engine panicked'),
        'client_exceptions': client_exceptions,
        'undecoded': count(log, r'decode error|undecodable'),
        'missing_dialogs': count(log, r'P2 worklist'),
        'whole_game_fallbacks': count(log, r'sending the whole game'),
        'ignored_commands': count(log, r'unhandled while|engine rejected'),
    }
    score = None
    with open(log, encoding='utf-8', errors='replace') as f:
        for line in f:
            if 'GAME FINISHED' in line:
                score = line.split('GAME FINISHED', 1)[1].strip()
                break
    result['score'] = score
    result['pass'] = (finished and result['panics'] == 0 and client_exceptions == 0
                      and result['undecoded'] == 0 and result['missing_dialogs'] == 0)
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--games', type=int, default=5)
    ap.add_argument('--start', type=int, default=0, help='first team-pair index')
    ap.add_argument('--timeout', type=int, default=600)
    ap.add_argument('--out', default=os.path.join('D:' + os.sep, 'ffb_gui_check', time.strftime('%Y%m%d_%H%M%S')))
    ap.add_argument('--jar', default=DEFAULT_JAR)
    ap.add_argument('--exe', default=DEFAULT_EXE)
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)
    homes, aways = teams_of('Kalimar'), teams_of('BattleLore')
    if not homes or not aways:
        print('no Kalimar / BattleLore teams in', JAVA_SERVER)
        return 2
    results = []
    for n in range(args.start, args.start + args.games):
        r = play(n, homes[n % len(homes)], aways[n % len(aways)], args)
        results.append(r)
        print(('PASS ' if r['pass'] else 'FAIL ') + ' '.join(f'{k}={v}' for k, v in r.items() if k != 'pass'),
              flush=True)
    passed = sum(r['pass'] for r in results)
    print(f'{passed}/{len(results)} games passed; logs in {args.out}')
    if passed == len(results):
        print('DONE')
        return 0
    return 1


if __name__ == '__main__':
    sys.exit(main())
