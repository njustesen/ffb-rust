# Sequential, CPU-capped gate runner. ONE ffb-parity at a time pinned to 4 of 16 logical CPUs.
# Per-GATE FFB_PARITY_ROOT so a later gate cannot overwrite an earlier one's jsonl.
# Verdict is read from BOTH streams: "100/100 games match" -> stdout, "N/100 passed" -> stderr.
param([string]$jobfile)
$repo="C:\Users\Admin\niels\ffb-rust\ffb-rust"
$scr="C:\Users\Admin\AppData\Local\Temp\claude\C--Users-Admin-niels-ffb-rust\956518da-7f53-489e-b876-3f34f78d940b\scratchpad"
$log=Join-Path $scr "gate_batch.log"
$stop=Join-Path $scr "STOP_GATES"
if (Test-Path $stop) { Remove-Item $stop -Force }
Add-Content $log ("=== batch " + (Get-Date -Format s) + " ===")
foreach ($line in (Get-Content $jobfile)) {
  $t = $line.Trim()
  if (-not $t) { continue }
  if (Test-Path $stop) { Add-Content $log "STOPPED"; exit 0 }
  $p2 = $t -split '\s+'
  $ed=$p2[0]; $race=$p2[1]; $sc=$p2[2]
  $out=Join-Path $scr ("gb_" + $race + "_" + $ed + "_" + $sc + ".log")
  $env:FFB_PARITY_ROOT = "parity_gb_" + $race + "_" + $ed + "_" + $sc
  $t0=Get-Date
  $proc=Start-Process -FilePath (Join-Path $repo "target\release\ffb-parity.exe") -WorkingDirectory $repo `
    -NoNewWindow -PassThru -ArgumentList @("--home",$race,"--away",$race,"--edition",$ed,"--tier","3",
      "--seeds","1-100","--no-abort","--agent","heuristic","--heur-scale",$sc,"--heur-classes","all") `
    -RedirectStandardOutput $out -RedirectStandardError "$out.err"
  Start-Sleep -Milliseconds 400
  try { $proc.ProcessorAffinity=[IntPtr]15 } catch { Add-Content $log "affinity FAILED $race $ed $sc" }
  $proc.WaitForExit()
  $v=$null
  foreach ($f in @($out,"$out.err")) {
    if (Test-Path $f) {
      $h=(Get-Content $f -ErrorAction SilentlyContinue | Where-Object {$_ -match '^PARITY: '} | Select-Object -Last 1)
      if ($h) { if (-not $v -or $h -match 'games match') { $v=$h } }
    }
  }
  if (-not $v) { $v = "NO VERDICT exit=" + $proc.ExitCode }
  Add-Content $log ("{0,-26} {1,5}m  {2}" -f ("$race $ed @$sc"), [math]::Round(((Get-Date)-$t0).TotalMinutes,1), $v)
}
Add-Content $log ("=== batch done " + (Get-Date -Format s) + " ===")
