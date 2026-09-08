# Drive ONE cell to green: re-gate a (race, edition) at all three scales, one gate at a time,
# pinned to 4 of 16 CPUs, per-gate FFB_PARITY_ROOT. Prints a verdict line per scale plus a
# CELL: GREEN / CELL: RED summary the caller can grep. Stop file honoured between gates.
param([string]$race, [string]$ed, [string]$tag)
$repo="C:\Users\Admin\niels\ffb-rust\ffb-rust"
$scr="C:\Users\Admin\AppData\Local\Temp\claude\C--Users-Admin-niels-ffb-rust\956518da-7f53-489e-b876-3f34f78d940b\scratchpad"
$log=Join-Path $scr "cell_loop.log"
$stop=Join-Path $scr "STOP_CELL"
$green=$true
Add-Content $log ("=== " + $race + " " + $ed + " [" + $tag + "] " + (Get-Date -Format s) + " ===")
foreach ($sc in @("1.0","0","1e6")) {
  if (Test-Path $stop) { Add-Content $log "STOPPED"; exit 0 }
  $out=Join-Path $scr ("cl_" + $race + "_" + $ed + "_" + $sc + "_" + $tag + ".log")
  $env:FFB_PARITY_ROOT = "parity_cl_" + $race + "_" + $ed + "_" + $sc + "_" + $tag
  $t0=Get-Date
  $p=Start-Process -FilePath (Join-Path $repo "target\release\ffb-parity.exe") -WorkingDirectory $repo `
     -NoNewWindow -PassThru -ArgumentList @("--home",$race,"--away",$race,"--edition",$ed,"--tier","3",
       "--seeds","1-100","--no-abort","--agent","heuristic","--heur-scale",$sc,"--heur-classes","all") `
     -RedirectStandardOutput $out -RedirectStandardError "$out.err"
  Start-Sleep -Milliseconds 400
  try { $p.ProcessorAffinity=[IntPtr]15 } catch {}
  $p.WaitForExit()
  $v=$null
  foreach ($f in @($out,"$out.err")) {
    if (Test-Path $f) {
      $h=(Get-Content $f -ErrorAction SilentlyContinue | Where-Object {$_ -match '^PARITY: '} | Select-Object -Last 1)
      if ($h) { if (-not $v -or $h -match 'games match') { $v=$h } }
    }
  }
  if (-not $v) { $v="NO VERDICT exit=" + $p.ExitCode; $green=$false }
  elseif ($v -notmatch '^PARITY: 100/100 games match') { $green=$false }
  Add-Content $log ("{0,-22} {1,5}m  {2}" -f ("$race $ed @$sc"), [math]::Round(((Get-Date)-$t0).TotalMinutes,1), $v)
}
if ($green) { Add-Content $log ("CELL: GREEN " + $race + " " + $ed) } else { Add-Content $log ("CELL: RED " + $race + " " + $ed) }
