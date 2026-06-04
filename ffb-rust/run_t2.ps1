Set-Location 'C:\Users\Admin\niels\ffb-rust\ffb-rust'
$races = @('amazon','chaos','chaos_dwarf','chaos_pact','dark_elf','dwarf','elf','goblin','halfling','high_elf','human','khemri','lizardman','necromantic','norse','nurgle','ogre','orc','renegades','skaven','slann','undead','underworld','vampire','wood_elf')
$pass = 0; $fail = 0
foreach ($race in $races) {
    $r = & .\target\release\ffb-parity.exe --home $race --away $race --edition bb2025 --seeds 100 --no-abort 2>&1 | Select-String 'PARITY:'
    $line = "$race`: $r"
    Write-Host $line
    if ($r -match '100/100') { $pass++ } else { $fail++ }
}
Write-Host "---"
Write-Host "Total: $pass/25 races 100/100, $fail failed"
