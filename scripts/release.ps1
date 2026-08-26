<#
    Publica uma versao nova de ponta a ponta:

        .\scripts\release.ps1 1.0.1

    Grava a versao no csproj, publica o exe, compila o instalador, commita, marca a
    tag e cria o release no GitHub. O asset sai sempre com o mesmo nome, entao o link
    do site nunca muda:

        https://github.com/asguth/mouse-hider/releases/latest/download/MouseHiderSetup.exe
#>
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Versao
)

$ErrorActionPreference = 'Stop'
$raiz = Split-Path $PSScriptRoot -Parent
Set-Location $raiz

$dotnet = "$env:ProgramFiles\dotnet\dotnet.exe"
if (-not (Test-Path $dotnet)) { throw "nao encontrei o dotnet em $dotnet" }

# O winget instala o Inno por usuario; instalacao manual costuma cair em Program Files (x86).
$iscc = @(
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $iscc) { throw "Inno Setup nao encontrado (winget install JRSoftware.InnoSetup)" }

# Executaveis nativos nao disparam excecao no PowerShell: o codigo de saida tem que ser conferido na mao.
function Executar($descricao, $comando, $argumentos) {
    Write-Host "==> $descricao" -ForegroundColor Cyan
    & $comando @argumentos
    if ($LASTEXITCODE -ne 0) { throw "$descricao falhou (exit $LASTEXITCODE)" }
}

# 1. O csproj e a fonte unica da versao — a tela Sobre e o instalador leem daqui.
$csproj = "MouseHider\MouseHider.csproj"
(Get-Content $csproj -Raw) -replace '<Version>[\d.]+</Version>', "<Version>$Versao</Version>" |
    Set-Content $csproj -Encoding utf8 -NoNewline

# 2. Executavel unico, self-contained: roda sem .NET instalado.
Executar "publicando o exe" $dotnet @(
    'publish', 'MouseHider\MouseHider.csproj', '-c', 'Release', '-r', 'win-x64',
    '--self-contained', 'true', '-p:PublishSingleFile=true', '-p:EnableCompressionInSingleFile=true',
    '-p:IncludeNativeLibrariesForSelfExtract=true', '-p:DebugType=none', '-o', 'dist', '--nologo'
)

Executar "verificando o exe publicado" "dist\MouseHider.exe" @('--selftest')

# 3. Instalador. OutputBaseFilename esta fixo no .iss: e o que sustenta o link permanente.
Executar "compilando o instalador" $iscc @("/DMyAppVersion=$Versao", 'installer\MouseHider.iss')

# 4. Commit e tag.
git add -A
git diff --cached --quiet
if ($LASTEXITCODE -ne 0) {
    Executar "commit" 'git' @('commit', '-m', "Versao $Versao")
}
Executar "tag" 'git' @('tag', '-a', "v$Versao", '-m', "Versao $Versao")
Executar "push" 'git' @('push', 'origin', 'main', '--follow-tags')

# 5. Release no GitHub com o asset de nome fixo.
Executar "release" 'gh' @(
    'release', 'create', "v$Versao", 'installer\Output\MouseHiderSetup.exe',
    '--title', "v$Versao", '--generate-notes'
)

Write-Host ""
Write-Host "pronto: v$Versao publicada" -ForegroundColor Green
Write-Host "link do site (nao muda): https://github.com/asguth/mouse-hider/releases/latest/download/MouseHiderSetup.exe"
