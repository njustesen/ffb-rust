#!/bin/sh
# classify.sh <edition> <seed>: print the first activation-level divergence signature.
#
# SIDE/TURN are checked FIRST and deliberately: the two amazon teams are mirrors, so when the
# engines disagree about whose turn it is, both still enumerate ~11 players and the candidate
# totals differ by a handful -- which the size test happily mislabels as LIST. Seed 2 was read
# that way for two iterations before the RELIG/JELIG player prefixes gave it away.
cd /c/Users/Admin/niels/ffb-rust/ffb-rust
S=/c/Users/Admin/AppData/Local/Temp/claude/C--Users-Admin-niels-ffb-rust/09776488-5c36-4b65-a667-eeb487670a51/scratchpad
FFB_CANDSUM=1 ./target/release/ffb-parity --home amazon --away amazon --edition $1 --tier 3 \
    --seeds $2-$2 --no-abort --agent heuristic --heur-scale 1.0 --heur-classes all \
    > /dev/null 2> $S/cls_$1_$2.err
python - "$S/cls_$1_$2.err" "$2" <<'PY'
import re, sys
R={};J={};RE={};JE={}
for line in open(sys.argv[1],encoding='utf-8',errors='replace'):
    m=re.match(r'([RJ])SUM k=(\d+) n=(\d+) draws=(\d+)',line)
    if m: (R if m.group(1)=='R' else J)[int(m.group(2))]=(int(m.group(3)),int(m.group(4)))
    m=re.match(r'RELIG k=(\d+) turn=(\d+) .*?(home_|away_)',line)
    if m: RE[int(m.group(1))]=(int(m.group(2)), 'home' if m.group(3)=='home_' else 'away')
    m=re.match(r'JELIG k=(\d+) turn=(\d+) \S*?(Home|Away)',line)
    if m: JE[int(m.group(1))]=(int(m.group(2)), m.group(3).lower())
for k in sorted(set(R)&set(J)):
    if R[k]!=J[k]:
        dn=R[k][0]-J[k][0]; dd=R[k][1]-J[k][1]
        rs=RE.get(k); js=JE.get(k)
        if rs and js and rs[1]!=js[1]:
            kind='SIDE(rust=%s java=%s)'%(rs[1],js[1])
        elif rs and js and rs[0]!=js[0]:
            kind='TURN(rust=%d java=%d)'%(rs[0],js[0])
        elif abs(dn)>300:
            kind='WINDOW(list reset)'
        elif dd!=0:
            kind='DRAWS'
        else:
            kind='LIST'
        print('seed %-4s k=%-4d dn=%-6d dd=%-4d  %s' % (sys.argv[2],k,dn,dd,kind))
        break
else:
    print('seed %s: activations agree' % sys.argv[2])
PY
