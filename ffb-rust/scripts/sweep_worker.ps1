# One sweep worker: runs its shard's gates sequentially, pinned to its own core mask.
# Sharded by (edition, matchup) so all three scales of a matchup stay on ONE worker -- the
# campaign rule is that two runs of the same edition+matchup must never overlap.
#
# Usage:
#   powershell -File scripts/sweep_worker.ps1 -jobfile <file> -mask <affinity> -tag w1 -scratch <dir>
#
# The job file has one gate per line: "<edition> <matchup> <scale>".
# Each gate gets its own FFB_PARITY_ROOT so concurrent gates cannot clobber each other's
# seed_N_*.jsonl. No --reuse-java: a stale Java cache once turned a 100/100 gate into 30/100,
# so every gate gets a fresh JVM.
param(
  [Parameter(Mandatory=$true)][string]$jobfile,
  [Parameter(Mandatory=$true)][int]$mask,
  [Parameter(Mandatory=$true)][string]$tag,
  [Parameter(Mandatory=$true)][string]$scratch
)
$repo = "C:\Users\Admin\niels\ffb-rust\ffb-rust"
$log  = Join-Path $scratch "sweep.log"
$stop = Join-Path $scratch "STOP_SWEEP"
foreach ($line in (Get-Content $jobfile)) {
  $t = $line.Trim(); if (-not $t) { continue }
  if (Test-Path $stop) { Add-Content $log ("[" + $tag + "] STOPPED"); exit 0 }
  $f = $t -split '\s+'; $ed = $f[0]; $race = $f[1]; $sc = $f[2]
  $out = Join-Path $scratch ("sw_" + $race + "_" + $ed + "_" + $sc + ".log")
  $env:FFB_PARITY_ROOT = "parity_rd_" + $race + "_" + $ed + "_" + $sc
  $t0 = Get-Date
  $p = Start-Process -FilePath (Join-Path $repo "target\release\ffb-parity.exe") -WorkingDirectory $repo `
     -NoNewWindow -PassThru -ArgumentList @("--home",$race,"--away",$race,"--edition",$ed,"--tier","3",
       "--seeds","1-100","--no-abort","--agent","heuristic","--heur-scale",$sc,"--heur-classes","all") `
     -RedirectStandardOutput $out -RedirectStandardError "$out.err"
  Start-Sleep -Milliseconds 300
  try { $p.ProcessorAffinity = [IntPtr]$mask } catch {}
  $p.WaitForExit()
  # Scrape the verdict from BOTH streams, preferring the "games match" line.
  $v = $null
  foreach ($g in @($out,"$out.err")) {
    if (Test-Path $g) {
      $h = (Get-Content $g -ErrorAction SilentlyContinue | Where-Object {$_ -match '^PARITY: '} | Select-Object -Last 1)
      if ($h) { if (-not $v -or $h -match 'games match') { $v = $h } }
    }
  }
  if (-not $v) { $v = "NO VERDICT exit=" + $p.ExitCode }
  Add-Content $log ("{0,-34} {1,5}m  {2}" -f ("$race $ed @$sc"), [math]::Round(((Get-Date)-$t0).TotalMinutes,1), $v)
}
Add-Content $log ("[" + $tag + "] shard done")
