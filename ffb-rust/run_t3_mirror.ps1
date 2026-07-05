# T3b: Mirror matchups — each race vs itself, real activation (tier 3)
# Runs all canonical races in order of complexity.
# Dwarf and goblin are excluded until G-RULE-1 (Secret Weapon H2 ejection) is fixed.
# Usage: .\run_t3_mirror.ps1
# Usage (single race): .\run_t3_mirror.ps1 -Race amazon

param(
    [string]$Race = "",
    [string]$Seeds = "1-100"
)

$races = @(
    'amazon',
    'human',
    'orc',
    'chaos',
    'skaven',
    'dark_elf',
    'elf',
    'high_elf',
    'wood_elf',
    'norse',
    'nurgle',
    'undead',
    'necromantic',
    'vampire',
    'chaos_pact',
    'chaos_dwarf',
    'halfling',
    'ogre',
    'lizardman',
    'khemri',
    'underworld',
    'slann'
    # 'goblin',  # blocked by G-RULE-1
    # 'dwarf',   # blocked by G-RULE-1
)

if ($Race -ne "") {
    $races = @($Race)
}

$pass = 0
$fail = 0
$failed_races = @()

foreach ($race in $races) {
    Write-Host ""
    Write-Host "=== T3b: $race vs $race (seeds $Seeds) ===" -ForegroundColor Cyan
    cargo run --release -p ffb-parity -- --tier 3 --home $race --away $race --seeds $Seeds --no-abort
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  PASS: $race" -ForegroundColor Green
        $pass++
    } else {
        Write-Host "  FAIL: $race" -ForegroundColor Red
        $fail++
        $failed_races += $race
    }
}

Write-Host ""
Write-Host "=== T3b Summary ===" -ForegroundColor Cyan
Write-Host "  Passed: $pass / $($pass + $fail)"
if ($failed_races.Count -gt 0) {
    Write-Host "  Failed: $($failed_races -join ', ')" -ForegroundColor Red
}
