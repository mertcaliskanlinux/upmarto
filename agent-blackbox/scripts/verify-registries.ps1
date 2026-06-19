# Pre-publish registry checks — delegates to cross-platform Node verifier.
# Usage: .\scripts\verify-registries.ps1 [-Strict]
param([switch]$Strict)

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

$args = @()
if ($Strict) { $args += "--strict" }
& node (Join-Path $PSScriptRoot "verify-registries.mjs") @args
exit $LASTEXITCODE
