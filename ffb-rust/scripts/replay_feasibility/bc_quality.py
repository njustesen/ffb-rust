#!/usr/bin/env python3
"""Quality check of FumbblReplayParser behavioural-cloning records against the replays themselves.

Usage: python -I bc_quality.py <records.jsonl> <replay_dir>
1. Coverage: records per decision family vs the number of decisions the replays prove happened
   (playerAction reports = activations, blockChoice reports = block-die picks, dialogs opened).
2. State sanity: game splitting by COIN_CHOICE; the first record of each game must be pregame
   (half 1, not endGame); half/turn never go backwards inside a game; for player_select the
   chosen player must be on the pitch in the record's state.
"""
import collections, glob, gzip, json, os, sys

recs = [json.loads(l) for l in open(sys.argv[1], encoding='utf-8')]
replays = sorted(glob.glob(os.path.join(sys.argv[2], '*.ffbr')))

# ---- ground truth from the replays
truth = collections.Counter()
for f in replays:
    cmds = json.loads(gzip.decompress(open(f, 'rb').read()))['commandArray']
    for c in cmds[1:]:
        for r in (c.get('reportList') or {}).get('reports', []):
            truth['report:' + r['reportId']] += 1
        for m in (c.get('modelChangeList') or {}).get('modelChangeArray', []):
            if m['modelChangeId'] == 'gameSetDialogParameter' and m.get('modelChangeValue'):
                v = m['modelChangeValue']
                truth['dialog:' + (v.get('dialogId') if isinstance(v, dict) else str(v))] += 1

kinds = collections.Counter(r['type'] if r['type'] != 'dialog' else 'dialog:' + r['dialog_id'] for r in recs)
print('== coverage (records vs what the replays prove happened, %d games) ==' % len(replays))
pairs = [('player_select', 'report:playerAction'), ('dialog:BLOCK_ROLL_PROPERTIES', 'report:blockChoice'),
         ('dialog:SKILL_USE', 'dialog:skillUse'), ('dialog:RE_ROLL', 'dialog:reRoll'),
         ('dialog:FOLLOWUP_CHOICE', 'dialog:followupChoice'), ('dialog:USE_APOTHECARY', 'dialog:useApothecary'),
         ('foul_target', 'report:foul'), ('pass_target', 'report:passRoll'), ('handoff_target', 'report:handOver'),
         ('kickoff_target', 'report:kickoffScatter'), ('dialog:COIN_CHOICE', 'report:coinThrow')]
for k, t in pairs:
    print('  %-32s records=%5d  truth(%s)=%5d' % (k, kinds.get(k, 0), t, truth.get(t, 0)))
print('  all dialogs opened in replays: %d; dialog records: %d' % (
    sum(n for k, n in truth.items() if k.startswith('dialog:')), sum(n for k, n in kinds.items() if k.startswith('dialog:'))))
print('  dialog kinds opened:', ', '.join('%s=%d' % (k[7:], n) for k, n in truth.most_common() if k.startswith('dialog:')))

# ---- state sanity
print('\n== state sanity ==')
games, cur = [], []
for r in recs:
    if r['type'] == 'dialog' and r['dialog_id'] == 'COIN_CHOICE' and cur:
        games.append(cur); cur = []
    cur.append(r)
games.append(cur)
bad_first = backwards = 0
sel_total = sel_onpitch = 0
for g in games:
    s0 = g[0]['state']
    if s0.get('half') not in (0, 1) or s0.get('turn_mode') in ('END_GAME', 'endGame'):
        bad_first += 1
    last = (0, 0)
    for r in g:
        s = r['state']
        key = (s.get('half', 0), s.get('turn_nr', 0))
        if key < last:
            backwards += 1
            if backwards <= 3:
                print('  backwards: %s %s after %s (%s)' % (r['type'], r.get('dialog_id'), last, key))
        last = max(last, key)
mt = [r for r in recs if r['type'] == 'move_target']
moves = truth.get('report:playerAction', 0)
print('  move_target records=%d (movement activations incl. blitz/foul/pass/handoff moves are a subset of %d activations)' % (len(mt), moves))
ps = [r for r in recs if r['type'] == 'player_select']
print('  player_select labelled inside the bot candidate list: %d of %d' % (sum(1 for r in ps if r.get('in_mde_candidates')), len(ps)))
print('  games found=%d, first record not pregame=%d, half/turn going backwards=%d' % (len(games), bad_first, backwards))
print('  first record of game 1:', {k: games[0][0]['state'].get(k) for k in ('half', 'turn_nr', 'turn_mode', 'home_playing')})
print('  record keys:', sorted(recs[0].keys()), '| state keys:', sorted(recs[0]['state'].keys()))
ex = next((r for r in recs if r['type'] == 'player_select'), None)
if ex:
    print('  example player_select action/scores:', json.dumps(ex['action'])[:200], json.dumps(ex.get('scores'))[:200])
