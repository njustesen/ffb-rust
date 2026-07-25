#!/usr/bin/env bash
# Reliable per-file reconciliation for the test-equalization campaign (ffb-model/protocol -> Java).
# Classifies each Rust test file as COVERED / GAP / INVENTED by checking whether the Java class it
# tests (PascalCase of the basename) (a) exists as a Java MAIN class and (b) is referenced by ANY
# Java TEST body across BOTH ffb-common and ffb-server test trees. Excludes the ffb-java/ffb/ clone.
# NOTE: exclude the clone with -path '*ffb-java/ffb/*'  — NEVER grep -v '/ffb/' (the Java package
# path com/fumbbl/ffb/ contains /ffb/ and would hide every file).
set -uo pipefail
cd "$(dirname "$0")/../.."   # repo root (contains ffb-java/ and ffb-rust/)
find ffb-java/ffb-common/src/main ffb-java/ffb-server/src/main -name "*.java" -not -path "*ffb-java/ffb/*" -exec basename {} .java \; 2>/dev/null | sort -u > /tmp/jmain.txt
find ffb-java/ffb-common/src/test ffb-java/ffb-server/src/test -name "*.java" -not -path "*ffb-java/ffb/*" -print0 2>/dev/null | xargs -0 cat > /tmp/alltest.txt
grep -oE '\b[A-Z][A-Za-z0-9]+\b' /tmp/alltest.txt | sort -u > /tmp/jtesttokens.txt
grep -rcE '^\s*#\[test\]' ffb-rust/crates/ffb-model/src ffb-rust/crates/ffb-protocol/src 2>/dev/null | awk -F: '$2>0' > /tmp/rustcounts.txt
: > /tmp/gaplist.txt
awk '
  FILENAME=="/tmp/jmain.txt"{main[$0]=1;next}
  FILENAME=="/tmp/jtesttokens.txt"{tok[$0]=1;next}
  { n=split($0,p,":"); cnt=p[n]; m=split(p[1],q,"/"); base=q[m]; sub(/\.rs$/,"",base);
    if(base=="mod")next; dir="?"; for(i=1;i<=m;i++)if(q[i]=="src"){dir=q[i+1];break}
    z=split(base,pp,"_"); pas=""; for(i=1;i<=z;i++)pas=pas toupper(substr(pp[i],1,1)) substr(pp[i],2)
    if(!(pas in main)){invf++;invt+=cnt}
    else if(pas in tok){covf++;covt+=cnt}
    else {gapf++;gapt+=cnt; print cnt"|"base"|"dir > "/tmp/gaplist.txt"} }
  END{printf "COVERED %d files/%d tests\nGAP %d files/%d tests\nINVENTED %d files/%d tests\n",covf,covt,gapf,gapt,invf,invt}
' /tmp/jmain.txt /tmp/jtesttokens.txt /tmp/rustcounts.txt
echo "--- gap files (cnt|base|dir) ---"; sort -t'|' -k1 -rn /tmp/gaplist.txt
