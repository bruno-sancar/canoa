Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$raiz = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$preparar = Join-Path $PSScriptRoot 'gerar-instalador.ps1'
$pastaBuild = Join-Path $env:LOCALAPPDATA 'Canoa\build-test'

& $preparar -SomentePreparar

Push-Location $pastaBuild
try {
    npm ci
    if ($LASTEXITCODE -ne 0) { throw 'Falha ao instalar dependências.' }
    npm run build
    if ($LASTEXITCODE -ne 0) { throw 'Falha na compilação da interface.' }

    Push-Location (Join-Path $pastaBuild 'src-tauri')
    try {
        cargo fmt --all -- --check
        if ($LASTEXITCODE -ne 0) { throw 'Falha na formatação Rust.' }
        cargo test --locked --lib
        if ($LASTEXITCODE -ne 0) { throw 'Falha nos testes Rust.' }
    } finally {
        Pop-Location
    }
} finally {
    Pop-Location
}

Write-Output 'Verificação concluída. Nenhuma janela do Canoa foi aberta.'
