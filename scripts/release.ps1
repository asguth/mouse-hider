<#
    Publica uma versao nova:

        .\scripts\release.ps1 1.1.0

    Grava a versao no Cargo.toml, roda os testes e o selftest, commita, cria a tag e envia.
    O GitHub Actions (.github/workflows/release.yml) compila e anexa o MouseHider.exe ao
    release. O nome do arquivo nunca muda, entao o link de download tambem nao:

        https://github.com/asguth/mouse-hider/releases/latest/download/MouseHider.exe
#>
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Versao
)

$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)

# Executaveis nativos nao disparam excecao no PowerShell: o codigo de saida e conferido na mao.
# E com 'Stop' qualquer linha no stderr (o cargo escreve progresso ali) viraria erro fatal.
function Executar($descricao, $comando, $argumentos) {
    Write-Host "==> $descricao" -ForegroundColor Cyan
    $anterior = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try { & $comando @argumentos 2>&1 | ForEach-Object { "$_" } }
    finally { $ErrorActionPreference = $anterior }
    if ($LASTEXITCODE -ne 0) { throw "$descricao falhou (exit $LASTEXITCODE)" }
}

# 1. O Cargo.toml e a fonte unica da versao: a tela Sobre e a checagem de atualizacao leem dele.
$toml = Get-Content Cargo.toml -Raw
$toml = [regex]::Replace($toml, '(?m)^version = "[\d.]+"', "version = `"$Versao`"", 1)
# WriteAllText: o Set-Content do Windows PowerShell 5.1 gravaria com BOM.
[IO.File]::WriteAllText((Resolve-Path Cargo.toml), $toml, [Text.UTF8Encoding]::new($false))

Executar "testes" 'cargo' @('test', '--quiet')
Executar "compilando" 'cargo' @('build', '--release', '--quiet')

# Start-Process -Wait: o exe e de janela, entao "& exe" volta na hora e o codigo de saida
# nao valeria nada.
Write-Host "==> selftest" -ForegroundColor Cyan
$teste = Start-Process 'target\release\MouseHider.exe' -ArgumentList '--selftest' -Wait -PassThru
if ($teste.ExitCode -ne 0) { throw "selftest falhou (exit $($teste.ExitCode))" }

Executar "commit" 'git' @('commit', '-am', "Versao $Versao")
Executar "tag" 'git' @('tag', '-a', "v$Versao", '-m', "Versao $Versao")
Executar "push" 'git' @('push', 'origin', 'HEAD', '--follow-tags')

Write-Host ""
Write-Host "pronto: v$Versao enviada. O GitHub Actions publica o exe em alguns minutos." -ForegroundColor Green
