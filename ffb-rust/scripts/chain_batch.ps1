# Wait for any capped ffb-parity run to finish, then start the next job file. Keeps CPU at ONE
# gate (4 of 16 cores) instead of doubling it by running two batches at once.
param([string]$jobfile)
$scr="C:\Users\Admin\AppData\Local\Temp\claude\C--Users-Admin-niels-ffb-rust\956518da-7f53-489e-b876-3f34f78d940b\scratchpad"
$log=Join-Path $scr "gate_batch.log"
$stop=Join-Path $scr "STOP_GATES"
Add-Content $log ("=== chained batch waiting for the current run " + (Get-Date -Format s) + " ===")
$quiet=0
while ($quiet -lt 3) {
  if (Test-Path $stop) { Add-Content $log "chain STOPPED before starting"; exit 0 }
  Start-Sleep -Seconds 20
  if (@(Get-Process ffb-parity -ErrorAction SilentlyContinue).Count -eq 0) { $quiet++ } else { $quiet = 0 }
}
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path "C:\Users\Admin\niels\ffb-rust\ffb-rust\scripts" "gate_batch.ps1") $jobfile
