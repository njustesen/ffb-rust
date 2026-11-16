# One STAR-tier sweep worker (BACKLOG §L): runs its shard's gates sequentially, pinned to its own
# core mask. Sibling of cross_worker.ps1; each job line carries both squads AND their hired star.
#
# Usage:
#   powershell -File scripts/star_worker.ps1 -jobfile <file> -mask <affinity> -tag w1 -scratch <dir>
#       [-root D:/ffb_star/sweep] [-seeds 1-2] [-scale 1.0] [-exe <ffb-parity.exe>]
#
# Job line: "<edition> <home> <home_star> <away> <away_star>" (scripts/star_jobs.py).
# FFB_PARITY_ROOT is PER GATE (<root>/<tag>/<n>): the logs and the private Java server dir hang
# off <root>/<edition>/<home>_vs_<away>/, and one ordered squad pair appears once per schedule,
# but a per-gate root also keeps a re-run of a single line from touching any other gate.
#
# sweep_<tag>.log line shape (scripts/sweep_verdicts.py --per-cell 1 after concatenation):
#   <home>_vs_<away> <edition> @<scale> [<home_star>|<away_star>]   <minutes>m  PARITY: ...
param(
  [Parameter(Mandatory=$true)][string]$jobfile,
  [Parameter(Mandatory=$true)][int]$mask,
  [Parameter(Mandatory=$true)][string]$tag,
  [Parameter(Mandatory=$true)][string]$scratch,
  [string]$root = "D:/ffb_star/sweep",
  [string]$seeds = "1-2",
  [string]$scale = "1.0",
  [string]$exe = ""
)
$repo = "C:\Users\Admin\niels\ffb-rust\ffb-rust"
if (-not $exe) { $exe = Join-Path $repo "target\release\ffb-parity.exe" }
$log  = Join-Path $scratch ("sweep_" + $tag + ".log")
$stop = Join-Path $scratch "STOP_SWEEP"
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
$env:PARITY_JVM_CORES = "2"
$n = 0
foreach ($line in (Get-Content $jobfile)) {
  $t = $line.Trim(); if (-not $t) { continue }
  $n++
  if (Test-Path $stop) { try { Add-Content $log ("[" + $tag + "] STOPPED") } catch {}; exit 0 }
  $f = $t -split '\s+'; $ed = $f[0]; $hm = $f[1]; $hs = $f[2]; $aw = $f[3]; $as = $f[4]
  $env:FFB_PARITY_ROOT = $root + "/" + $tag + "/" + $n
  $cell = $hm + "_vs_" + $aw
  $out = Join-Path $scratch ("sw_" + $tag + "_" + $n + ".log")
  $t0 = Get-Date
  $p = Start-Process -FilePath $exe -WorkingDirectory $repo `
     -NoNewWindow -PassThru -ArgumentList @("--home",$hm,"--away",$aw,"--home-star",$hs,"--away-star",$as,
       "--edition",$ed,"--tier","3","--seeds",$seeds,"--no-abort","--agent","heuristic",
       "--heur-scale",$scale,"--heur-classes","all") `
     -RedirectStandardOutput $out -RedirectStandardError "$out.err"
  Start-Sleep -Milliseconds 300
  try { $p.ProcessorAffinity = [IntPtr]$mask } catch {}
  $p.WaitForExit()
  $v = $null; $ff = $null; $rep = $null
  foreach ($g in @($out,"$out.err")) {
    if (Test-Path $g) {
      $c = Get-Content $g -ErrorAction SilentlyContinue
      $h = ($c | Where-Object {$_ -match '^PARITY: '} | Select-Object -Last 1)
      if ($h) { if (-not $v -or $h -match 'games match') { $v = $h } }
      if (-not $ff) { $ff = ($c | Where-Object {$_ -match '^PARITY FAIL seed='} | Select-Object -First 1) }
      if (-not $rep) { $rep = ($c | Where-Object {$_ -match '^REPORTS: '} | Select-Object -Last 1) }
    }
  }
  if (-not $v) { $v = "NO VERDICT exit=" + $p.ExitCode }
  if ($ff -and $ff -match '^PARITY FAIL seed=(\d+)[^,]*, step (\d+)') { $v = $v + "  first-fail seed=" + $matches[1] + " step=" + $matches[2] }
  if ($rep) { $v = $v + "  " + $rep }
  $line = ("{0,-50} {1,5}m  {2}" -f ("$cell $ed @$scale [$hs|$as] #$tag/$n"), [math]::Round(((Get-Date)-$t0).TotalMinutes,1), $v)
  $ok = $false
  for ($try = 0; $try -lt 20 -and -not $ok; $try++) {
    try { Add-Content $log $line -ErrorAction Stop; $ok = $true } catch { Start-Sleep -Milliseconds 250 }
  }
  if (-not $ok) { Add-Content (Join-Path $scratch ("sweep_" + $tag + ".lost")) $line }
}
try { Add-Content $log ("[" + $tag + "] shard done") } catch {}
