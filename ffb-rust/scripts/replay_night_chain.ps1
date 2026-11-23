# BACKLOG §O: the whole-corpus proof run, chained for the 21:00-07:00 window (user budget: <= 70% of
# the machine, launch through a .cmd with `start /AFFINITY 7FF` so java and the exe inherit the mask).
#   0. optional: wait for a leftover day run (-StopPid) and stop it at -StopAt if still alive
#   1. sleep until -NotBefore (21:00)
#   2. Java: FumbblReplayParser --per-replay over the 5 corpus dirs -> $JavaOut (frozen -Jar)
#   3. Rust: <frozen -Exe> actions $RustOut $JavaOut <5 dirs> (states + EVERY record, MDE included)
#      -- only started when >= -RustHours remain before -NotAfter
# Every phase is stopped at -NotAfter (07:00). Log: <JavaOut>.chain.log; results: <JavaOut>.log/.err
# (Java), <RustOut>.log/.err (Rust; final line "actions (all records): N identical ...").
param(
    [string]$Jar = 'D:/ffb_replay_parity/bin/ffb-ai-replay-o3.jar',
    [string]$Exe = 'D:/ffb_replay_parity/bin/ffb-replay-o3b.exe',
    [string]$JavaOut = 'D:/ffb_replay_parity/run6_java',
    [string]$RustOut = 'D:/ffb_replay_parity/rust6',
    [int]$StopPid = 0,
    [string]$StopAt = '08:00',
    [string]$NotBefore = '21:00',
    [string]$NotAfter = '07:00',
    [double]$RustHours = 4.0,
    [string]$Xmx = '6g'
)
$corpus = @(
    'D:/ffb_replays/fumbbl',
    'D:/ffb_replays/partial',
    'D:/ffb_replays/window',
    'D:/ffb_replays/fumbbl_night_20261122',
    'D:/ffb_replays/fumbbl_night_20261122/partial'
)
$log = "$JavaOut.chain.log"
function Log([string]$m) { "$(Get-Date -Format s) $m" | Add-Content -Path $log }

function WaitOrStop($proc, [datetime]$deadline, [string]$what) {
    while (-not $proc.HasExited) {
        if ((Get-Date) -ge $deadline) {
            Stop-Process -Id $proc.Id -Force
            Log "$what STOPPED at the window end $deadline (partial outputs)"
            return $false
        }
        Start-Sleep -Seconds 60
    }
    Log "$what exited with code $($proc.ExitCode)"
    return $true
}

Log "chain start (pid $PID); jar $Jar exe $Exe"
if ($StopPid -gt 0) {
    $stop = [datetime]::Today.Add([timespan]::Parse($StopAt))
    while ((Get-Process -Id $StopPid -ErrorAction SilentlyContinue) -and (Get-Date) -lt $stop) { Start-Sleep -Seconds 60 }
    if (Get-Process -Id $StopPid -ErrorAction SilentlyContinue) {
        Stop-Process -Id $StopPid -Force
        Log "stopped leftover run $StopPid at $StopAt"
    } else {
        Log "leftover run $StopPid ended by itself"
    }
}
$start = [datetime]::Today.Add([timespan]::Parse($NotBefore))
if ((Get-Date) -lt $start) {
    Log "sleeping until $start"
    Start-Sleep -Seconds ([int][math]::Ceiling(($start - (Get-Date)).TotalSeconds))
}
$end = [datetime]::Today.Add([timespan]::Parse($NotAfter))
if ($end -le (Get-Date)) { $end = $end.AddDays(1) }
Log "window until $end"

$jargs = @("-Xmx$Xmx", '-cp', $Jar, 'com.fumbbl.ffb.ai.simulation.FumbblReplayParser',
    '--per-replay', $JavaOut, '--tool', $Jar) + $corpus
$j = Start-Process -FilePath java -ArgumentList $jargs -RedirectStandardOutput "$JavaOut.log" `
    -RedirectStandardError "$JavaOut.err" -WindowStyle Hidden -PassThru
$j.PriorityClass = 'BelowNormal'
Log "java pid $($j.Id) -> $JavaOut"
if (-not (WaitOrStop $j $end 'java')) { exit 1 }

$left = ($end - (Get-Date)).TotalHours
if ($left -lt $RustHours) {
    Log "rust NOT started: only $([math]::Round($left, 2)) h left before $end (needs $RustHours)"
    exit 2
}
$rargs = @('actions', $RustOut, $JavaOut) + $corpus
$r = Start-Process -FilePath $Exe -ArgumentList $rargs -RedirectStandardOutput "$RustOut.log" `
    -RedirectStandardError "$RustOut.err" -WindowStyle Hidden -PassThru
$r.PriorityClass = 'BelowNormal'
Log "rust pid $($r.Id) -> $RustOut"
if (-not (WaitOrStop $r $end 'rust')) { exit 1 }
Log "chain done"
