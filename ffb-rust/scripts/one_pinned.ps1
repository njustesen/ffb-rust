# Run ONE gate pinned to a core mask, print wall time + TIMING + PARITY. mask 255 = 8 of 16 cores.
param([string]$race,[string]$ed,[string]$sc,[string]$root,[int]$mask=255,[switch]$reuse)
$repo="C:\Users\Admin\niels\ffb-rust\ffb-rust"
$scr="C:\Users\Admin\AppData\Local\Temp\claude\C--Users-Admin-niels-ffb-rust\956518da-7f53-489e-b876-3f34f78d940b\scratchpad"
$out=Join-Path $scr ("op_" + $root + ".log")
$env:FFB_PARITY_ROOT=$root
$a=@("--home",$race,"--away",$race,"--edition",$ed,"--tier","3","--seeds","1-100","--no-abort",
     "--agent","heuristic","--heur-scale",$sc,"--heur-classes","all")
if ($reuse) { $a += "--reuse-java" }
$t0=Get-Date
$p=Start-Process -FilePath (Join-Path $repo "target\release\ffb-parity.exe") -WorkingDirectory $repo `
   -NoNewWindow -PassThru -ArgumentList $a -RedirectStandardOutput $out -RedirectStandardError "$out.err"
Start-Sleep -Milliseconds 300
try { $p.ProcessorAffinity=[IntPtr]$mask } catch {}
$p.WaitForExit()
$w=[math]::Round(((Get-Date)-$t0).TotalSeconds,1)
Write-Output ("WALL=" + $w + "s reuse=" + $reuse)
foreach ($f in @($out,"$out.err")) {
  if (Test-Path $f) {
    Get-Content $f | Where-Object {$_ -match '^PARITY: |^TIMING '} | Select-Object -Last 3 | ForEach-Object { Write-Output ("  " + $_) }
  }
}
