Set-Location 'C:\Users\Admin\niels\ffb-rust\ffb-rust'
$races = @('amazon','chaos_chosen','chaos_dwarf','chaos_pact','dark_elf','dwarf','elf','goblin','halfling','high_elf','human','khemri','lizardman','necromantic','norse','nurgle','ogre','orc','renegades','skaven','slann','undead','underworld','vampire','wood_elf','lineman')
$pass = 0; $fail = 0; $failRaces = @()
foreach ($race in $races) {
    $r = & .\target\debug\ffb-parity.exe --home $race --away $race --edition bb2025 --seeds 100 --no-abort 2>$null | Select-String 'PARITY:'
    Write-Host "$race`: $r"
    if ($r -match '100/100') { $pass++ } else { $fail++; $failRaces += $race }
}
Write-Host "---"
Write-Host "Total: $pass/26 races 100/100, $fail failed"
if ($failRaces.Count -gt 0) { Write-Host "Failed: $($failRaces -join ', ')" }
