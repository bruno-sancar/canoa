param(
    [Parameter(Mandatory = $true)]
    [int]$ProcessoCanoa,
    [int]$Amostras = 5,
    [int]$IntervaloSegundos = 10
)

if ($Amostras -lt 1 -or $IntervaloSegundos -lt 0) {
    throw 'Use pelo menos uma amostra e intervalo não negativo.'
}

$cpuAnteriorPorId = @{}
$momentoAnterior = $null
$processadoresLogicos = [Environment]::ProcessorCount

for ($amostra = 1; $amostra -le $Amostras; $amostra++) {
    $inventario = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name)
    $ids = [System.Collections.Generic.HashSet[int]]::new()
    [void]$ids.Add($ProcessoCanoa)
    do {
        $antes = $ids.Count
        foreach ($item in $inventario) {
            if ($ids.Contains([int]$item.ParentProcessId)) {
                [void]$ids.Add([int]$item.ProcessId)
            }
        }
    } while ($ids.Count -gt $antes)

    $processos = @(foreach ($idAtual in $ids) {
        Get-Process -Id $idAtual -ErrorAction SilentlyContinue
    })
    $privado = ($processos | Measure-Object -Property PrivateMemorySize64 -Sum).Sum
    $trabalho = ($processos | Measure-Object -Property WorkingSet64 -Sum).Sum
    $momento = Get-Date
    $cpuAtualPorId = @{}
    $cpuSegundos = 0.0
    foreach ($processo in $processos) {
        if ($null -eq $processo.CPU) { continue }
        $cpu = [double]$processo.CPU
        $cpuAtualPorId[$processo.Id] = $cpu
        if ($null -ne $momentoAnterior) {
            if ($cpuAnteriorPorId.ContainsKey($processo.Id)) {
                $cpuSegundos += [math]::Max(0, $cpu - $cpuAnteriorPorId[$processo.Id])
            } elseif ($processo.StartTime -ge $momentoAnterior) {
                $cpuSegundos += $cpu
            }
        }
    }
    $cpuPctSistema = $null
    if ($null -ne $momentoAnterior) {
        $segundos = ($momento - $momentoAnterior).TotalSeconds
        if ($segundos -gt 0 -and $processadoresLogicos -gt 0) {
            $cpuPctSistema = [math]::Round(100 * $cpuSegundos / ($segundos * $processadoresLogicos), 1)
        }
    }
    [pscustomobject]@{
        Momento = $momento.ToString('o')
        Amostra = $amostra
        Processos = $processos.Count
        PrivadoMiB = [math]::Round($privado / 1MB, 1)
        TrabalhoMiB = [math]::Round($trabalho / 1MB, 1)
        CpuPctSistema = $cpuPctSistema
        IDs = (($processos | Select-Object -ExpandProperty Id | Sort-Object) -join ',')
    }
    $cpuAnteriorPorId = $cpuAtualPorId
    $momentoAnterior = $momento
    if ($amostra -lt $Amostras) {
        Start-Sleep -Seconds $IntervaloSegundos
    }
}
