param([string]$race,[string]$ed,[string]$root,[int]$mask=255)
$repo="C:\Users\Admin\niels\ffb-rust\ffb-rust"
$scr="C:\Users\Admin\AppData\Local\Temp\claude\C--Users-Admin-niels-ffb-rust\956518da-7f53-489e-b876-3f34f78d940b\scratchpad"
$out=Join-Path $scr ("op_" + $root + ".log")
$env:FFB_PARITY_ROOT=$root
$a=@("--home",$race,"--away",$race,"--edition",$ed,"--tier","3","--seeds","1-100","--no-abort","--agent","random")
$t0=Get-Date
$p=Start-Process -FilePath (Join-Path $repo "target\release\ffb-parity.exe") -WorkingDirectory $repo `
   -NoNewWindow -PassThru -ArgumentList $a -RedirectStandardOutput $out -RedirectStandardError "$out.err"
Start-Sleep -Milliseconds 300
try { $p.ProcessorAffinity=[IntPtr]$mask } catch {}
$p.WaitForExit()
Write-Output ("WALL=" + [math]::Round(((Get-Date)-$t0).TotalSeconds,1) + "s")
foreach ($f in @($out,"$out.err")) {
  if (Test-Path $f) { Get-Content $f | Where-Object {$_ -match '^PARITY: |^TIMING '} | Select-Object -Last 2 | ForEach-Object { Write-Output ("  " + $_) } }
}
