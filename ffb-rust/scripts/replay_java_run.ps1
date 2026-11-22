# BACKLOG §O: launch the Java per-replay processor over the WHOLE replay corpus, detached.
# Prints the java PID. Outputs: <Out>/<id>.{states,actions.jsonl,status.json} + run_info.json;
# log at <Out>.log / <Out>.err. Read the result with scripts/replay_java_summary.py <Out>.
# Classpath = optional harness classes dir (a private javac build) ahead of the jar.
param(
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$Jar = 'D:/ffb_replays/ffb-ai-replay.jar',
    [string]$Classes = '',
    [string]$Xmx = '6g'
)
$corpus = @(
    'D:/ffb_replays/fumbbl',
    'D:/ffb_replays/partial',
    'D:/ffb_replays/window',
    'D:/ffb_replays/fumbbl_night_20261122',
    'D:/ffb_replays/fumbbl_night_20261122/partial'
)
$cp = if ($Classes) { "$Classes;$Jar" } else { $Jar }
$argList = @("-Xmx$Xmx", '-cp', $cp, 'com.fumbbl.ffb.ai.simulation.FumbblReplayParser',
    '--per-replay', $Out, '--tool', $Jar) + $corpus
$p = Start-Process -FilePath java -ArgumentList $argList -RedirectStandardOutput "$Out.log" `
    -RedirectStandardError "$Out.err" -WindowStyle Hidden -PassThru
$p.Id
