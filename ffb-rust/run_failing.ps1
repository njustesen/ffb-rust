Set-Location 'C:\Users\Admin\niels\ffb-rust\ffb-rust'
$races = @('elf','goblin','dwarf','high_elf','human','dark_elf','amazon','chaos_dwarf','chaos_pact','norse')
foreach ($race in $races) {
    $r = & .\target\debug\ffb-parity.exe --home $race --away $race --edition bb2025 --seeds 100 --no-abort 2>&1 | Select-String 'PARITY:'
    Write-Host "$race`: $r"
}
