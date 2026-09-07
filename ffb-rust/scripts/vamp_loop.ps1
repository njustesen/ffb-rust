# Vampire nine-gate loop. Deliberate constraints, learned from 2026-09-07:
#   * ONE ffb-parity at a time, never two gates concurrently.
#   * Pinned to 4 of 16 logical CPUs via ProcessorAffinity, so a 16-thread gate
#     cannot take the machine. Capping threads is not possible (no --jobs flag).
#   * A stop file halts it between gates: no PID hunting, no respawn race.
#   * A PID file so whatever is running is always identifiable.
$repo  = "C:\Users\Admin\niels\ffb-rust\ffb-rust"
$scr   = "C:\Users\Admin\AppData\Local\Temp\claude\C--Users-Admin-niels-ffb-rust\956518da-7f53-489e-b876-3f34f78d940b\scratchpad"
$stop  = Join-Path $scr "STOP_VAMP"
$log   = Join-Path $scr "vamp_gates.log"
$pidf  = Join-Path $scr "vamp_loop.pid"
$exe   = Join-Path $repo "target\release\ffb-parity.exe"
$AFFINITY = 15          # cores 0-3 of 16

Set-Content -Path $pidf -Value $PID
if (Test-Path $stop) { Remove-Item $stop -Force }
Add-Content $log ("=== vamp loop start " + (Get-Date -Format s) + " affinity=" + $AFFINITY + " ===")

foreach ($ed in @("bb2016","bb2020","bb2025")) {
  foreach ($sc in @("1.0","0","1e6")) {
    if (Test-Path $stop) { Add-Content $log "STOPPED by stop file"; exit 0 }
    $tag = "vampire_${ed}_${sc}"
    $out = Join-Path $scr "g_$tag.log"
    # A per-GATE root. All three scales of one edition share a matchup dir, so a single
    # "parity_vloop" root lets @0 and @1e6 overwrite @1.0's jsonl -- which silently invalidates
    # any per-seed classification done afterwards. Learned the hard way on 2026-09-07.
    $env:FFB_PARITY_ROOT = "parity_vloop_${ed}_${sc}"
    $t0 = Get-Date
    $p = Start-Process -FilePath $exe -WorkingDirectory $repo -NoNewWindow -PassThru `
         -ArgumentList @("--home","vampire","--away","vampire","--edition",$ed,"--tier","3",
                         "--seeds","1-100","--no-abort","--agent","heuristic",
                         "--heur-scale",$sc,"--heur-classes","all") `
         -RedirectStandardOutput $out -RedirectStandardError "$out.err"
    Start-Sleep -Milliseconds 400
    try { $p.ProcessorAffinity = [IntPtr]$AFFINITY } catch { Add-Content $log "affinity set FAILED for $tag" }
    $p.WaitForExit()
    $mins = [math]::Round(((Get-Date) - $t0).TotalMinutes,1)
    # The PARITY verdict goes to STDERR, not stdout, and the stream carries NUL bytes that make
    # Select-String treat the file as binary -- read the .err file and filter in PowerShell.
    # "PARITY: 100/100 games match." goes to STDOUT; "PARITY: N/100 passed, M FAILED." goes to
    # STDERR. Read BOTH or a green gate reads as "no verdict" and a red one as missing.
    $verdict = $null
    foreach ($f in @($out, "$out.err")) {
      if (-not (Test-Path $f)) { continue }
      $hit = (Get-Content $f -ErrorAction SilentlyContinue |
              Where-Object { $_ -match '^PARITY: ' } | Select-Object -Last 1)
      if ($hit) { if (-not $verdict -or $hit -match 'games match') { $verdict = $hit } }
    }
    if (-not $verdict) { $verdict = "NO PARITY LINE (crash?) exit=" + $p.ExitCode }
    Add-Content $log ("{0,-24} {1,6}m  {2}" -f $tag, $mins, $verdict)
  }
}
Add-Content $log ("=== vamp loop done " + (Get-Date -Format s) + " ===")
