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
  [Parameter(Mandatory=$true)][string]$scratch,
  [string]$root = "D:/ffb_sweep",
  # A frozen copy of the binary, so a certification never runs whatever target/release holds.
  [string]$exe = ""
)
$repo = "C:\Users\Admin\niels\ffb-rust\ffb-rust"
if (-not $exe) { $exe = Join-Path $repo "target\release\ffb-parity.exe" }
# One verdict log PER WORKER (see cross_worker.ps1: a shared log got locked by a tail -f).
$log  = Join-Path $scratch ("sweep_" + $tag + ".log")
$stop = Join-Path $scratch "STOP_SWEEP"
New-Item -ItemType Directory -Force -Path $scratch | Out-Null  # a missing dir silently loses verdicts
foreach ($line in (Get-Content $jobfile)) {
  $t = $line.Trim(); if (-not $t) { continue }
  if (Test-Path $stop) { Add-Content $log ("[" + $tag + "] STOPPED"); exit 0 }
  $f = $t -split '\s+'; $ed = $f[0]; $race = $f[1]; $sc = $f[2]
  $out = Join-Path $scratch ("sw_" + $race + "_" + $ed + "_" + $sc + ".log")
  # Sweep logs land on the DATA volume: a full 330-gate sweep writes ~8 GB of per-seed step,
  # event and report logs, which does not fit next to the repo. FFB_PARITY_ROOT takes an absolute
  # path and both engines create the directory tree under it.
  $env:FFB_PARITY_ROOT = $root + "/parity_" + $race + "_" + $ed + "_" + $sc
  $t0 = Get-Date
  $p = Start-Process -FilePath $exe -WorkingDirectory $repo `
     -NoNewWindow -PassThru -ArgumentList @("--home",$race,"--away",$race,"--edition",$ed,"--tier","3",
       "--seeds","1-100","--no-abort","--agent","heuristic","--heur-scale",$sc,"--heur-classes","all") `
     -RedirectStandardOutput $out -RedirectStandardError "$out.err"
  Start-Sleep -Milliseconds 300
  try { $p.ProcessorAffinity = [IntPtr]$mask } catch {}
  $p.WaitForExit()
  # Scrape the verdict from BOTH streams, preferring the "games match" line; the REPORTS line
  # too, because report identity is the bar (§J) and a steps-only verdict hid 120 report diffs.
  $v = $null; $rep = $null
  foreach ($g in @($out,"$out.err")) {
    if (Test-Path $g) {
      $c = Get-Content $g -ErrorAction SilentlyContinue
      $h = ($c | Where-Object {$_ -match '^PARITY: '} | Select-Object -Last 1)
      if ($h) { if (-not $v -or $h -match 'games match') { $v = $h } }
      if (-not $rep) { $rep = ($c | Where-Object {$_ -match '^REPORTS: '} | Select-Object -Last 1) }
    }
  }
  if (-not $v) { $v = "NO VERDICT exit=" + $p.ExitCode }
  if ($rep) { $v = $v + "  " + $rep }
  Add-Content $log ("{0,-34} {1,5}m  {2}" -f ("$race $ed @$sc"), [math]::Round(((Get-Date)-$t0).TotalMinutes,1), $v)
}
Add-Content $log ("[" + $tag + "] shard done")
