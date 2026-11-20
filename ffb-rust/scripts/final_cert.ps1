# BACKLOG §M final proof driver: on ONE frozen pair D:/ffb_star/bin/ffb-parity-<name>.exe +
# ffb-ai-<name>.jar run, in order,
#   1. the skill ledger   (scripts/skill_ledger.py run, kinds skill,mirror,cross,star -> D:/ffb_ledger/<name>)
#   2. its report         (LEDGER.txt + ledger.json; scripts/skill_parity_check.py reads the newest ledger)
#   3. the full cert      (scripts/cert_chain.ps1 -name <name>: mirror, cross, star, skill)
# The ledger goes first because it is Rust-only and short (~3-4 h) and measures which skills still
# do not fire; a loop session can read it and stop this driver before the long cert if needed.
# Resource budget (user): 21:00-07:00 only, <=70% of the machine (mask 0x7FF). Launch it pinned at
# the root so every child inherits the mask, e.g. from bash:
#   cmd //c start "" //b //affinity 7FF powershell -NoProfile -File scripts/final_cert.ps1 -name cert7
# Logs: D:/ffb_ledger/<name>.run.log, D:/ffb_ledger/<name>.report.log, D:/ffb_star/<name>.chain.log
param([string]$name = "cert7", [int]$maxRun = 4, [switch]$anytime, [switch]$skipLedger)
$repo   = "C:\Users\Admin\niels\ffb-rust\ffb-rust"
$exe    = "D:/ffb_star/bin/ffb-parity-$name.exe"
$jar    = "D:/ffb_star/bin/ffb-ai-$name.jar"
$ledger = "D:/ffb_ledger/$name"
$mask   = 2047
try { (Get-Process -Id $PID).ProcessorAffinity = [IntPtr]$mask } catch {}
if (-not (Test-Path $exe) -or -not (Test-Path $jar)) { throw "frozen pair for $name missing: $exe / $jar" }
# ffb-parity must run with cwd = ffb-rust/ffb-rust (from the repo root it rewrites progress.html).
Set-Location $repo
function InWindow { if ($anytime) { return $true }; $h = (Get-Date).Hour; return ($h -ge 21 -or $h -lt 7) }
while (-not (InWindow)) { Start-Sleep -Seconds 60 }
if (-not $skipLedger) {
  "[" + (Get-Date) + "] ledger start" | Add-Content "D:/ffb_ledger/$name.run.log"
  & python -I "$repo\scripts\skill_ledger.py" run --out $ledger --kinds skill,mirror,cross,star `
      --exe $exe --workers $maxRun *>> "D:/ffb_ledger/$name.run.log"
  "[" + (Get-Date) + "] ledger end rc=$LASTEXITCODE" | Add-Content "D:/ffb_ledger/$name.run.log"
  & python -I "$repo\scripts\skill_ledger.py" report --out $ledger *> "D:/ffb_ledger/$name.report.log"
}
$chainArgs = @("-NoProfile", "-File", "$repo\scripts\cert_chain.ps1", "-name", $name, "-maxRun", $maxRun)
if ($anytime) { $chainArgs += "-anytime" }
& powershell @chainArgs *>> "D:/ffb_star/$name.chain.log"
"[" + (Get-Date) + "] final_cert done" | Add-Content "D:/ffb_star/$name.chain.log"
