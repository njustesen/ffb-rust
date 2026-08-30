#!/bin/sh
# firstdiff.sh <edition> <seed>: the FIRST per-step state-hash divergence, from the recorded
# jsonl -- the leading indicator. classify.sh works on activation candidate counts, which only
# notice a divergence once it changes a LIST SIZE; a board that has already drifted (a player on a
# different square) can agree on counts for dozens of activations first (bb2025 seed 33).
cd /c/Users/Admin/niels/ffb-rust/ffb-rust
./target/release/ffb-parity --home amazon --away amazon --edition $1 --tier 3 \
    --seeds $2-$2 --no-abort --agent heuristic --heur-scale 1.0 --heur-classes all >/dev/null 2>&1
python - "parity/$1/amazon_vs_amazon/seed_$2_rust.jsonl" "parity/$1/amazon_vs_amazon/seed_$2_java.jsonl" <<'PY'
import json,sys
def load(p): return [json.loads(l) for l in open(p,encoding='utf-8') if '"type":"step"' in l]
R=load(sys.argv[1]); J=load(sys.argv[2])
first=None
for i,(r,j) in enumerate(zip(R,J)):
    if r['state_hash']!=j['state_hash']: first=i; break
if first is None:
    print('no state divergence in %d/%d steps'%(len(R),len(J))); raise SystemExit
print('first state divergence at list index %d (rust i=%s)'%(first,R[first]['i']))
for i in range(max(0,first-4), min(len(R),len(J),first+2)):
    r,j=R[i],J[i]
    tag='  <-- DIFF' if r['state_hash']!=j['state_hash'] else ''
    print(' i=%-4s R t%-2s %-4s %-38s dice=%s' % (r['i'],r['turn'],r['active'],r['chosen'][:38],r['dice']))
    print('        J t%-2s %-4s %-38s dice=%s%s' % (j['turn'],j['active'],j['chosen'][:38],j['dice'],tag))
PY
