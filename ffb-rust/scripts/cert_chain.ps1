# Certification chain (BACKLOG §M phase 5; cert6 = the frozen exe built from e0f63037e).
# usage: powershell -NoProfile -File scripts/cert_chain.ps1 -suites starrest,skill [-maxRun 4] [-anytime]
# Suites:
#   mirror   330 cells   (sweepjobs/w1-6.txt, sweep_worker.ps1)
#   cross    4012 gates  (../crossjobs_cert2/w1-6.txt, cross_worker.ps1)
#   star     4012 gates  (../starjobs_full/w1-3.txt, star_worker.ps1)
#   starrest the 3023 star gates cert6 did not reach (../starjobs_cert6/rest_r1-3.txt, tags r1-r3)
#   skill    121 gates = 110 _sk + 6 _st squads + 5 seed hunts (scripts/sksweep.py, SKSWEEP_DIRS,
#            star_hires.json, seed_hunts.json; resumable via results.txt)
# Low footprint: at most $maxRun workers at once (job files queued), JVM heap capped at 1 GB.
# Resource budget (user): <=70% of the machine (cores 0-10, mask 0x7FF), only 21:00-07:00 unless
# the user says otherwise (-anytime). Outside the window no new suite starts and workers get
# STOP_SWEEP (each finishes its current gate, then exits).
# -name certN selects the frozen pair D:/ffb_star/bin/ffb-parity-certN.exe + ffb-ai-certN.jar and
# the output dirs ../mirror_certN, ../cross_certN, ../starjobs_certN (scratch + sweep logs),
# D:/ffb_star/{mirror_certN,cross_certN,certN} (game roots) and D:/ffb_sksweep/certN (skill).
# (cert6 itself ran with the mirror dirs named mirror_cert6b; that run is complete.)
param([string[]]$suites = @("mirror","cross","star","skill"), [int]$maxRun = 4, [switch]$anytime,
      [string]$name = "cert6")
$suites = @($suites | ForEach-Object { $_ -split "," })
$exe   = "D:/ffb_star/bin/ffb-parity-$name.exe"
$repo  = "C:\Users\Admin\niels\ffb-rust\ffb-rust"
$outer = "C:\Users\Admin\niels\ffb-rust"
$mask  = 2047
$env:JAVA_TOOL_OPTIONS = "-Xmx1g -XX:+UseSerialGC"
# Freeze the Java harness too: runner.rs otherwise resolves the IN-REPO jar, so any `mvn package`
# during a cert silently swaps the oracle mid-run (happened 2026-11-20 14:40). Workers inherit this.
$env:PARITY_CP = "D:/ffb_star/bin/ffb-ai-$name.jar"
if (-not (Test-Path $exe) -or -not (Test-Path $env:PARITY_CP)) { throw "frozen pair for $name missing: $exe / $env:PARITY_CP" }
try { (Get-Process -Id $PID).ProcessorAffinity = [IntPtr]$mask } catch {}

function InWindow { if ($anytime) { return $true }; $h = (Get-Date).Hour; return ($h -ge 21 -or $h -lt 7) }
while (-not (InWindow)) { Start-Sleep -Seconds 60 }

foreach ($suite in $suites) {
  if (-not (InWindow)) { "[" + (Get-Date) + "] outside 21:00-07:00, not starting $suite"; break }
  $jobs = @(); $tagp = "w"
  switch ($suite) {
    "mirror" { $scratch = "$outer\mirror_$name\scratch"; $script = "sweep_worker.ps1"; $root = "D:/ffb_star/mirror_$name"
      1..6 | ForEach-Object { $jobs += "$repo\sweepjobs\w$_.txt" } }
    "cross"  { $scratch = "$outer\cross_$name\scratch"; $script = "cross_worker.ps1"; $root = "D:/ffb_star/cross_$name"
      1..6 | ForEach-Object { $jobs += "$outer\crossjobs_cert2\w$_.txt" } }
    "star"   { $scratch = "$outer\starjobs_$name\scratch"; $script = "star_worker.ps1"; $root = "D:/ffb_star/$name"
      1..3 | ForEach-Object { $jobs += "$outer\starjobs_full\w$_.txt" } }
    "starrest" { $scratch = "$outer\starjobs_$name\scratch"; $script = "star_worker.ps1"; $root = "D:/ffb_star/$name"; $tagp = "r"
      1..3 | ForEach-Object { $jobs += "$outer\starjobs_$name\rest_r$_.txt" } }
    "skill"  { $scratch = "D:\ffb_sksweep\$name" }
    default { throw "unknown suite $suite" }
  }
  $procs = @()
  # A resumed chain (next evening) must clear the previous window's 07:00 STOP_SWEEP first, or
  # every worker sees it before its first gate and exits -- the suite then "finishes" unjudged.
  if ($suite -ne "skill") { Remove-Item (Join-Path $scratch "STOP_SWEEP") -ErrorAction SilentlyContinue }
  if ($suite -eq "skill") {
    # §M 2b(h): the stat-increase squads (data/teams_stats) belong to the skill suite too.
    $env:SKSWEEP_DIRS = "teams_skills,teams_stats"
    $procs += Start-Process python -PassThru -WindowStyle Hidden -ArgumentList @(
      "-I","$repo\scripts\sksweep.py",$exe,"D:/ffb_sksweep/$name","1-20","1.0","0","$maxRun")
  }
  "[" + (Get-Date) + "] started $suite (" + $jobs.Count + " job files, max $maxRun at once)"
  $next = 0
  while ($true) {
    $running = @($procs | Where-Object { -not $_.HasExited })
    if (InWindow) {
      while ($next -lt $jobs.Count -and $running.Count -lt $maxRun) {
        $tag = $tagp + ($next + 1)
        $p = Start-Process powershell -PassThru -WindowStyle Hidden -ArgumentList @(
          "-NoProfile","-File","$repo\scripts\$script","-jobfile",$jobs[$next],
          "-mask",$mask,"-tag",$tag,"-scratch",$scratch,"-root",$root,"-exe",$exe)
        $procs += $p; $running += $p; $next++
        "[" + (Get-Date) + "]   $suite $tag started"
      }
    } elseif ($suite -ne "skill") {
      New-Item -ItemType Directory -Force -Path $scratch | Out-Null
      Set-Content (Join-Path $scratch "STOP_SWEEP") "07:00 budget window closed"
    } else {
      $running | ForEach-Object { Stop-Process -Id $_.Id -Force }
    }
    if ($running.Count -eq 0 -and ($next -ge $jobs.Count -or -not (InWindow))) { break }
    Start-Sleep -Seconds 60
  }
  "[" + (Get-Date) + "] finished $suite"
}
"cert chain done"
