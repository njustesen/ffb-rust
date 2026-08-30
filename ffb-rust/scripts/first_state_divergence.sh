#!/bin/sh
# first_state_divergence.sh <edition> <seed> [--no-rerun]
#
# The LEADING indicator: re-run one seed with the heuristic agent and report the first per-step
# state-hash divergence from the recorded jsonl, plus the activation whose RESOLUTION produced it
# (the step before) and whether that activation was the first one after a side/turn flip -- i.e.
# right after a non-REGULAR window (kickoff-return / pass-block) closed.
#
# classify.sh works on candidate-count differences, which only notice a divergence once it changes
# a LIST SIZE; a board that has already drifted can agree on counts for dozens of activations
# first (bb2025 seed 33: 16 activations late). Use THIS first, classify.sh only to bucket.
#
# --no-rerun reads the jsonl already on disk. Only safe when you KNOW the last writer of
# parity/<edition>/amazon_vs_amazon/seed_<n>_*.jsonl was the heuristic run you mean; the random
# control writes to FFB_PARITY_ROOT=parity_random, but any other run of the same matchup clobbers.
cd /c/Users/Admin/niels/ffb-rust/ffb-rust
E=$1; S=$2
if [ "$3" != "--no-rerun" ]; then
  ./target/release/ffb-parity --home amazon --away amazon --edition $E --tier 3 \
      --seeds $S-$S --no-abort --agent heuristic --heur-scale ${HEUR_SCALE:-1.0} --heur-classes all >/dev/null 2>&1
fi
python - "parity/$E/amazon_vs_amazon/seed_${S}_rust.jsonl" "parity/$E/amazon_vs_amazon/seed_${S}_java.jsonl" "$S" <<'PY'
import json,sys,re
def load(p): return [json.loads(l) for l in open(p,encoding='utf-8') if '"type":"step"' in l]
R=load(sys.argv[1]); J=load(sys.argv[2]); seed=sys.argv[3]
def short(c): return re.sub(r'teamAmazonParity2[05]','',c)
first=None
for i,(r,j) in enumerate(zip(R,J)):
    if r['state_hash']!=j['state_hash']: first=i; break
if first is None:
    if len(R)!=len(J): print('seed %-3s  no hash diff but LENGTH differs: rust %d java %d'%(seed,len(R),len(J)))
    else: print('seed %-3s  no state divergence in %d steps'%(seed,len(R)))
    raise SystemExit
if first==0:
    print('seed %-3s  diverges at the very first step'%seed); raise SystemExit
r=R[first-1]; j=J[first-1]
flip = first>=2 and (R[first-2]['active'],R[first-2]['turn'])!=(r['active'],r['turn'])
def norm(c):
    # Java `Activate(Home6,BLITZ_MOVE)` vs Rust `Activate(home_06,Blitz)`: same declaration.
    c = short(c).lower().replace('blitz_move','blitz').replace('hand_over','handoff').replace('_','')
    return re.sub(r'([a-z])0+(\d)', r'\1\2', c)
same_decl = norm(j['chosen']) == norm(r['chosen'])
print('seed %-3s  first hash diff idx %-4d  resolving idx %-4d %s t%s %-4s %-30s dice=%s%s%s'%(
    seed, first, first-1, 'R' , r['turn'], r['active'], r['chosen'][:30], r['dice'],
    '' if same_decl else '  [DECLARATION DIFFERS: J %s]'%short(j['chosen']),
    '  [after side/turn flip]' if flip else ''))
if len(sys.argv)>4 or True:
    for i in range(max(0,first-3), min(len(R),len(J),first+1)):
        rr,jj=R[i],J[i]; tag='  <-- DIFF' if rr['state_hash']!=jj['state_hash'] else ''
        print('   i=%-4s R t%-2s %-4s %-34s | J t%-2s %-4s %-34s%s'%(rr['i'],rr['turn'],rr['active'],rr['chosen'][:34],jj['turn'],jj['active'],short(jj['chosen'])[:34],tag))
PY
