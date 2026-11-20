# Re-gate ONE edition on a frozen exe + jar: every mirror cell of sweepjobs/w1-6.txt for that
# edition (all races x 3 scales) at a short seed range, then that edition's skill squads
# (teams_skills + teams_stats, star hires, seed hunts). Written for the §M supervisor directive of
# 2026-11-20 (bb2020 team special rules changed every bb2020 base team).
# usage: powershell -NoProfile -File scripts/regate_edition.ps1 -edition bb2020 -exe <frozen exe>
#          -jar <frozen jar> -out <dir> [-seeds 1-20] [-workers 4]
# Results: <out>/mirror/sweep_m*.log (one verdict line per cell) and <out>/skill/results.txt;
# <out>/DONE is written at the end. Budget: pinned to cores 0-10 (0x7FF, <=70%), children inherit.
param(
  [Parameter(Mandatory=$true)][string]$edition,
  [Parameter(Mandatory=$true)][string]$exe,
  [Parameter(Mandatory=$true)][string]$jar,
  [Parameter(Mandatory=$true)][string]$out,
  [string]$seeds = "1-20",
  [int]$workers = 4
)
$repo = "C:\Users\Admin\niels\ffb-rust\ffb-rust"
$mask = 2047
try { (Get-Process -Id $PID).ProcessorAffinity = [IntPtr]$mask } catch {}
$env:JAVA_TOOL_OPTIONS = "-Xmx1g -XX:+UseSerialGC"
$env:PARITY_CP = $jar
$scratch = Join-Path $out "mirror"
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
# Shard by matchup (all three scales of a matchup on ONE worker: never two runs of one
# edition+matchup at once).
$lines = @()
1..6 | ForEach-Object { $lines += @(Get-Content "$repo\sweepjobs\w$_.txt" | Where-Object { $_ -match "^$edition\s" }) }
$races = @($lines | ForEach-Object { ($_ -split '\s+')[1] } | Select-Object -Unique)
$shards = @{}
for ($i = 0; $i -lt $races.Count; $i++) {
  $k = $i % $workers
  if (-not $shards.ContainsKey($k)) { $shards[$k] = @() }
  $shards[$k] += @($lines | Where-Object { ($_ -split '\s+')[1] -eq $races[$i] })
}
$procs = @()
foreach ($k in $shards.Keys) {
  $jf = Join-Path $out ("jobs_m" + ($k + 1) + ".txt")
  Set-Content $jf $shards[$k]
  $procs += Start-Process powershell -PassThru -WindowStyle Hidden -ArgumentList @(
    "-NoProfile","-File","$repo\scripts\sweep_worker.ps1","-jobfile",$jf,"-mask",$mask,
    "-tag",("m" + ($k + 1)),"-scratch",$scratch,"-root",(Join-Path $out "roots"),"-exe",$exe,"-seeds",$seeds)
}
"[" + (Get-Date) + "] mirror: " + $lines.Count + " cells, " + $races.Count + " races, " + $procs.Count + " workers"
$procs | ForEach-Object { $_.WaitForExit() }
"[" + (Get-Date) + "] mirror done"
$env:SKSWEEP_DIRS = "teams_skills,teams_stats"
$env:SKSWEEP_EDITIONS = $edition
$p = Start-Process python -PassThru -WindowStyle Hidden -ArgumentList @(
  "-I","$repo\scripts\sksweep.py",$exe,(Join-Path $out "skill"),$seeds,"1.0","0","$workers")
$p.WaitForExit()
"[" + (Get-Date) + "] skill done"
Set-Content (Join-Path $out "DONE") ("done " + (Get-Date))
