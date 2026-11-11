# One CROSS-matchup sweep worker: runs its shard's non-mirror gates sequentially, pinned to its
# own core mask. Sibling of sweep_worker.ps1 (the mirror matrix); the job line carries BOTH
# squads and every gate runs a few seeds instead of 100.
#
# Usage:
#   powershell -File scripts/cross_worker.ps1 -jobfile <file> -mask <affinity> -tag w1 -scratch <dir>
#       [-root D:/ffb_cross/sweep] [-seeds 1-3] [-scale 1.0]
#
# The job file has one gate per line: "<edition> <home> <away>" (scripts/cross_jobs.py).
# One FFB_PARITY_ROOT per WORKER is enough: the harness keys its logs on
# <root>/<edition>/<home>_vs_<away>/, so distinct pairs never clobber each other, and a pair is
# on exactly one worker. No --reuse-java: every gate gets a fresh JVM.
#
# sweep_<tag>.log line shape (read by scripts/sweep_verdicts.py --per-cell 1 after concatenation):
#   <home>_vs_<away> <edition> @<scale>   <minutes>m  PARITY: ...
param(
  [Parameter(Mandatory=$true)][string]$jobfile,
  [Parameter(Mandatory=$true)][int]$mask,
  [Parameter(Mandatory=$true)][string]$tag,
  [Parameter(Mandatory=$true)][string]$scratch,
  [string]$root = "D:/ffb_cross/sweep",
  [string]$seeds = "1-3",
  [string]$scale = "1.0"
)
$repo = "C:\Users\Admin\niels\ffb-rust\ffb-rust"
# One verdict log PER WORKER: a `tail -f` monitor on a shared sweep.log locked it on Windows and
# every worker's Add-Content failed for 30 minutes (1,110 verdicts lost, recovered by
# scripts/cross_recover.py). Concatenate sweep_w*.log for scripts/sweep_verdicts.py.
$log  = Join-Path $scratch ("sweep_" + $tag + ".log")
$stop = Join-Path $scratch "STOP_SWEEP"
$env:FFB_PARITY_ROOT = $root + "/" + $tag
$env:PARITY_JVM_CORES = "2"
foreach ($line in (Get-Content $jobfile)) {
  $t = $line.Trim(); if (-not $t) { continue }
  if (Test-Path $stop) { try { Add-Content $log ("[" + $tag + "] STOPPED") } catch {}; exit 0 }
  $f = $t -split '\s+'; $ed = $f[0]; $hm = $f[1]; $aw = $f[2]
  $cell = $hm + "_vs_" + $aw
  $out = Join-Path $scratch ("cw_" + $cell + "_" + $ed + ".log")
  $t0 = Get-Date
  $p = Start-Process -FilePath (Join-Path $repo "target\release\ffb-parity.exe") -WorkingDirectory $repo `
     -NoNewWindow -PassThru -ArgumentList @("--home",$hm,"--away",$aw,"--edition",$ed,"--tier","3",
       "--seeds",$seeds,"--no-abort","--agent","heuristic","--heur-scale",$scale,"--heur-classes","all") `
     -RedirectStandardOutput $out -RedirectStandardError "$out.err"
  Start-Sleep -Milliseconds 300
  try { $p.ProcessorAffinity = [IntPtr]$mask } catch {}
  $p.WaitForExit()
  # Scrape the verdict from BOTH streams, preferring the "games match" line; keep the first
  # PARITY FAIL line too so the verdict table can say where each red diverged.
  $v = $null; $ff = $null
  foreach ($g in @($out,"$out.err")) {
    if (Test-Path $g) {
      $c = Get-Content $g -ErrorAction SilentlyContinue
      $h = ($c | Where-Object {$_ -match '^PARITY: '} | Select-Object -Last 1)
      if ($h) { if (-not $v -or $h -match 'games match') { $v = $h } }
      if (-not $ff) { $ff = ($c | Where-Object {$_ -match '^PARITY FAIL seed='} | Select-Object -First 1) }
    }
  }
  if (-not $v) { $v = "NO VERDICT exit=" + $p.ExitCode }
  if ($ff -and $ff -match '^PARITY FAIL seed=(\d+)[^,]*, step (\d+)') { $v = $v + "  first-fail seed=" + $matches[1] + " step=" + $matches[2] }
  $line = ("{0,-50} {1,5}m  {2}" -f ("$cell $ed @$scale"), [math]::Round(((Get-Date)-$t0).TotalMinutes,1), $v)
  $ok = $false
  for ($try = 0; $try -lt 20 -and -not $ok; $try++) {
    try { Add-Content $log $line -ErrorAction Stop; $ok = $true } catch { Start-Sleep -Milliseconds 250 }
  }
  if (-not $ok) { Add-Content (Join-Path $scratch ("sweep_" + $tag + ".lost")) $line }
}
try { Add-Content $log ("[" + $tag + "] shard done") } catch {}
