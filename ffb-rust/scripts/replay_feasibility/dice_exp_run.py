"""Run Java bot games with JAVA_DIE tracing + report sink, for the dice-recoverability experiment."""
import os, subprocess, sys
OUT = 'D:/ffb_replays/dice_exp'
os.makedirs(OUT, exist_ok=True)
JAR = 'D:/ffb_replays/ffb-ai-replay.jar'
SERVER = 'C:/Users/Admin/niels/ffb-rust/ffb-java/ffb/ffb-server'
MATCHUPS = [
    ('BB2025', 'teamOrcSkParity25Home', 'teamDwarfSkParity25Away', 'orc_dwarf_25'),
    ('BB2020', 'teamNurgleSkParity20Home', 'teamSkavenSkParity20Away', 'nurgle_skaven_20'),
]
for rules, home, away, tag in MATCHUPS:
    for seed in (1, 2, 3):
        base = '%s/%s_s%d' % (OUT, tag, seed)
        cmd = ['java', '-cp', JAR, '--add-opens', 'java.base/java.util=ALL-UNNAMED', '-Dffb.parityDebug=true', '-Dffb.parityDebugDeep=true',
               'com.fumbbl.ffb.ai.parity.ParityRunner', SERVER, home, away, str(seed), base + '_java.jsonl',
               '--tier', '3', '--seed-end', str(seed), '--reports', base + '_reports.jsonl', '--ruleset', rules,
               '--agent', 'heuristic', '--heur-scale', '1.0', '--heur-classes', 'all']
        with open(base + '_stderr.txt', 'w', encoding='utf-8') as err:
            rc = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=err, cwd=SERVER).returncode
        print(tag, seed, 'rc', rc, flush=True)
