param([switch]$SomentePreparar)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$raiz = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$origemApp = Join-Path $raiz 'app'
$pastaBuild = Join-Path $env:LOCALAPPDATA 'Canoa\build-test'
$arquivoPacoteOrigem = Join-Path $origemApp 'package-lock.json'
$arquivoPacoteLocal = Join-Path $pastaBuild 'package-lock.json'
$instalarDependencias = -not (Test-Path -LiteralPath (Join-Path $pastaBuild 'node_modules\.bin\tauri.cmd')) -or -not (Test-Path -LiteralPath $arquivoPacoteLocal)
if (-not $instalarDependencias) {
    $instalarDependencias = (Get-FileHash -LiteralPath $arquivoPacoteOrigem).Hash -ne (Get-FileHash -LiteralPath $arquivoPacoteLocal).Hash
}

New-Item -ItemType Directory -Path $pastaBuild -Force | Out-Null
Push-Location $raiz
try {
    $arquivos = @(git -c core.quotepath=false ls-files --cached --others --exclude-standard -- app)
    if ($LASTEXITCODE -ne 0 -or $arquivos.Count -eq 0) {
        throw 'Não foi possível listar os arquivos do aplicativo no Git.'
    }
    foreach ($relativo in $arquivos) {
        $dentroApp = $relativo.Substring(4).Replace('/', '\')
        $origem = Join-Path $origemApp $dentroApp
        $destino = Join-Path $pastaBuild $dentroApp
        if (Test-Path -LiteralPath $destino) {
            $arquivoOrigem = Get-Item -LiteralPath $origem
            $arquivoDestino = Get-Item -LiteralPath $destino
            if ($arquivoOrigem.Length -eq $arquivoDestino.Length -and
                (Get-FileHash -LiteralPath $origem).Hash -eq (Get-FileHash -LiteralPath $destino).Hash) {
                continue
            }
        }
        $pastaDestino = Split-Path -Parent $destino
        New-Item -ItemType Directory -Path $pastaDestino -Force | Out-Null
        Copy-Item -LiteralPath $origem -Destination $destino -Force
    }
} finally {
    Pop-Location
}

if ($SomentePreparar) {
    Write-Output "Cópia local preparada: $pastaBuild"
    return
}

Push-Location $pastaBuild
try {
    if ($instalarDependencias) {
        npm ci
        if ($LASTEXITCODE -ne 0) { throw 'Falha ao instalar dependências do aplicativo.' }
    }
    npm run tauri build
    if ($LASTEXITCODE -ne 0) { throw 'Falha ao gerar o instalador.' }
} finally {
    Pop-Location
}

$configuracao = Get-Content -LiteralPath (Join-Path $origemApp 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
$pastaInstalador = Join-Path $pastaBuild 'src-tauri\target\release\bundle\nsis'
$pacote = Get-ChildItem -LiteralPath $pastaInstalador -Filter "$($configuracao.productName)_$($configuracao.version)_*-setup.exe" -File |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
if ($null -eq $pacote) { throw 'A compilação terminou sem produzir o instalador NSIS esperado.' }
$saida = Join-Path $raiz "output\Canoa-$($configuracao.version)-Windows-instalador.exe"
Copy-Item -LiteralPath $pacote.FullName -Destination $saida -Force
$hash = (Get-FileHash -LiteralPath $saida -Algorithm SHA256).Hash
Write-Output "Instalador: $saida"
Write-Output "Tamanho: $((Get-Item -LiteralPath $saida).Length) bytes"
Write-Output "SHA-256: $hash"

$executavel = Join-Path $pastaBuild 'src-tauri\target\release\canoa.exe'
if (-not (Test-Path -LiteralPath $executavel)) { throw 'O executável compilado não foi encontrado.' }
$saidaPortatil = Join-Path $raiz "output\Canoa-$($configuracao.version)-Windows-portatil.exe"
Copy-Item -LiteralPath $executavel -Destination $saidaPortatil -Force
Write-Output "Portátil: $saidaPortatil"
Write-Output "Tamanho: $((Get-Item -LiteralPath $saidaPortatil).Length) bytes"
Write-Output "SHA-256: $((Get-FileHash -LiteralPath $saidaPortatil -Algorithm SHA256).Hash)"
