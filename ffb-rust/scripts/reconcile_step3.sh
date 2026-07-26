#!/usr/bin/env bash
# Step 3 reconciliation: Rust ffb-mechanics + ffb-engine(NON-step) -> Java ffb-server.
# Classifies each Rust test file COVERED/GAP/INVENTED by Java MAIN-class existence (ffb-server or
# ffb-common) and whether any ffb-server TEST body references the PascalCase type. Excludes step/
# (that's Step 4) and the ffb-java/ffb/ clone. NEVER use grep -v /ffb/ (package path contains /ffb/).
set -uo pipefail
cd "$(dirname "$0")/../.."
# Java MAIN classes (server + common, since server tests can target either)
find ffb-java/ffb-server/src/main ffb-java/ffb-common/src/main -name "*.java" -not -path "*ffb-java/ffb/*" -exec basename {} .java \; 2>/dev/null | sort -u > /tmp/jmain3.txt
# Java TEST corpus + tokens. Scan ffb-server AND ffb-common (+ client-logic): a Rust
# ffb-mechanics type may be ported to a Java test in ANY module that has the class on its
# classpath (modifier contexts/collections live in com.fumbbl.ffb.modifiers = ffb-common).
find ffb-java/ffb-server/src/test ffb-java/ffb-common/src/test ffb-java/ffb-client-logic/src/test \
  -name "*.java" -not -path "*ffb-java/ffb/*" -print0 2>/dev/null | xargs -0 cat > /tmp/alltest3.txt
grep -oE '\b[A-Z][A-Za-z0-9]+\b' /tmp/alltest3.txt | sort -u > /tmp/jtok3.txt
# Rust test-file counts: ffb-mechanics (all) + ffb-engine EXCLUDING step/
{ grep -rcE '^\s*#\[test\]' ffb-rust/crates/ffb-mechanics/src 2>/dev/null;
  grep -rcE '^\s*#\[test\]' ffb-rust/crates/ffb-engine/src 2>/dev/null | grep -vE '/src/step/'; } | awk -F: '$2>0' > /tmp/rustc3.txt
: > /tmp/gap3.txt
awk '
  FILENAME=="/tmp/jmain3.txt"{main[$0]=1;next}
  FILENAME=="/tmp/jtok3.txt"{tok[$0]=1;next}
  { n=split($0,p,":"); cnt=p[n]; m=split(p[1],q,"/"); base=q[m]; sub(/\.rs$/,"",base);
    if(base=="mod")next; dir="?"; for(i=1;i<=m;i++)if(q[i]=="src"){dir=q[i+1];break}
    z=split(base,pp,"_"); pas=""; for(i=1;i<=z;i++)pas=pas toupper(substr(pp[i],1,1)) substr(pp[i],2)
    if(!(pas in main)){invf++;invt+=cnt}
    else if(pas in tok){covf++;covt+=cnt}
    else {gapf++;gapt+=cnt; print cnt"|"base"|"dir > "/tmp/gap3.txt"} }
  END{printf "COVERED %d files/%d tests\nGAP %d files/%d tests\nINVENTED %d files/%d tests\n",covf,covt,gapf,gapt,invf,invt}
' /tmp/jmain3.txt /tmp/jtok3.txt /tmp/rustc3.txt
echo "--- GAP by dir ---"; awk -F'|' '{s[$3]+=$1}END{for(k in s)print s[k],k}' /tmp/gap3.txt | sort -rn
