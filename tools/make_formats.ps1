# make_formats.ps1 — PowerShell mirror of tools/make_formats.sh for native
# Windows (no bash). KEEP THE TWO IN SYNC: the .sh is authoritative; this
# mirrors its contract exactly. See that file's header for the full design
# rationale (two --init invocations, TL-year-versioned dump filenames,
# when to re-run).
#
# Usage (from anywhere):
#   tools\make_formats.ps1                       # debug build (default)
#   $env:PROFILE='release'; tools\make_formats.ps1
#
# Env:
#   PROFILE              release | debug (default) | ci
#   LATEXML_DUMP_DIR     optional override — where to COPY the dumps
#                        (they are always written to resources/dumps/)
#
# Requires kpsewhich/pdflatex on PATH (TeX Live for Windows or MiKTeX).
# Written for Windows PowerShell 5.1 (no &&/||, manual exit-code checks).

Set-Location (Join-Path $PSScriptRoot '..')

# NOTE: $PROFILE is a PowerShell automatic variable (profile script path),
# but environment variables live in a separate namespace, so $env:PROFILE
# is safe; just never assign the plain $Profile variable.
$BuildProfile = if ($env:PROFILE) { $env:PROFILE } else { 'debug' }
switch ($BuildProfile) {
  'release' { $CargoFlags = @('--release') }
  'debug'   { $CargoFlags = @() }
  'ci'      { $CargoFlags = @('--profile', 'ci') }
  default   {
    Write-Host "PROFILE must be 'release', 'debug', or 'ci' (got: $BuildProfile)"
    exit 2
  }
}

# Always (re)build — cargo is the staleness authority (see make_formats.sh
# for the stale-binary-in-build-cache incident this guards against).
Write-Host "[make_formats] building latexml_oxide ($BuildProfile)..."
cargo build @CargoFlags --bin latexml_oxide
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
# Honour CARGO_TARGET_DIR, as the .sh does.
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
$Bin = Join-Path (Join-Path $TargetDir $BuildProfile) 'latexml_oxide.exe'

# TL-year detection, mirroring the .sh (and the runtime's
# dump_paths::detect_ambient_texlive_year two-step strategy):
#   1. kpsewhich -var-value=SELFAUTOPARENT, trailing path component
#      `YYYY[suffix]` (TeX Live for Windows installs to C:\texlive\YYYY).
#   2. pdflatex --version banner ("TeX Live YYYY").
# MiKTeX matches neither (rolling release) — its handling is
# docs/WINDOWS_COMPATIBILITY_PLAN.md Phase 2.2; this script currently
# requires TeX Live semantics, like the .sh it mirrors.
# (stderr is routed through cmd's nul to avoid PS 5.1's NativeCommandError
# wrapping of native-command stderr.)
$TlYear = ''
$SelfAuto = cmd /c "kpsewhich -var-value=SELFAUTOPARENT 2>nul"
if ($SelfAuto) {
  $Leaf = (("$SelfAuto".Trim() -replace '\\', '/') -split '/')[-1]
  if ($Leaf -match '^([0-9]{4})[A-Za-z]*$') { $TlYear = $Matches[1] }
}
if (-not $TlYear) {
  $Banner = (cmd /c "pdflatex --version 2>nul" | Select-Object -First 3) -join ' '
  if ($Banner -match 'TeX Live ([0-9]{4})') { $TlYear = $Matches[1] }
}
if (-not $TlYear) {
  Write-Host '[make_formats] could not detect TeXLive year via kpsewhich SELFAUTOPARENT or pdflatex --version'
  exit 3
}

# Pin the binary's kpathsea to the SAME TeX Live the year was detected from
# (the .sh's dual-TL guard: a linked libkpathsea anchors on its compile-time
# tree, so a host with two TeX distributions could otherwise detect one year
# and raw-load latex.ltx / expl3-code.tex from the other tree).
$TlRoot = cmd /c "kpsewhich -var-value=SELFAUTOPARENT 2>nul"
if ($TlRoot) {
  $TlRoot = "$TlRoot".Trim()
  if (Test-Path (Join-Path $TlRoot 'texmf-dist') -PathType Container) {
    $env:TEXMFROOT = $TlRoot
    $env:TEXMFDIST = "$TlRoot/texmf-dist"
    $env:TEXMFCNF = "$TlRoot/texmf-dist/web2c"
    Write-Host "[make_formats] pinned TEXMFROOT=$TlRoot (dual-TL guard)"
  }
}

# The .sh's run_init: the --init run is gated on ZERO `Error:`/`Fatal:` lines
# (LATEXML_INIT_DEBUG=1 keeps them visible; without it init suppresses them,
# so a broken dump would pass silently). A non-zero exit fails too.
function Invoke-Init([string]$Init) {
  $Log = [System.IO.Path]::GetTempFileName()
  $env:LATEXML_INIT_DEBUG = '1'
  cmd /c "`"$Bin`" --init=$Init > `"$Log`" 2>&1"
  $Status = $LASTEXITCODE
  Remove-Item Env:LATEXML_INIT_DEBUG
  $Bad = @(Get-Content -LiteralPath $Log -Encoding UTF8 |
    ForEach-Object { $_ -replace "$([char]27)\[[0-9;]*m", '' } |
    Where-Object { $_ -match '^(Error|Fatal):' })
  Remove-Item -LiteralPath $Log
  if ($Status -ne 0 -or $Bad.Count -ne 0) {
    Write-Host "[make_formats] --init=${Init}: exit=$Status errors=$($Bad.Count)"
    $Bad | Select-Object -First 20 | ForEach-Object { Write-Host $_ }
    exit 4
  }
}

Write-Host "[make_formats] generating plain.$TlYear.dump.txt (--init=plain.tex)..."
Invoke-Init 'plain.tex'

Write-Host "[make_formats] generating latex.$TlYear.dump.txt (--init=latex.ltx)..."
Invoke-Init 'latex.ltx'

if ($env:LATEXML_DUMP_DIR) {
  New-Item -ItemType Directory -Force $env:LATEXML_DUMP_DIR | Out-Null
  Copy-Item "resources\dumps\plain.$TlYear.dump.txt" $env:LATEXML_DUMP_DIR -ErrorAction SilentlyContinue
  Copy-Item "resources\dumps\latex.$TlYear.dump.txt" $env:LATEXML_DUMP_DIR
  Copy-Item "resources\dumps\texlive.$TlYear.version" $env:LATEXML_DUMP_DIR -ErrorAction SilentlyContinue
  Write-Host "[make_formats] dumps copied to $env:LATEXML_DUMP_DIR"
}

Write-Host '[make_formats] done.'
Write-Host "[make_formats]   plain dump: resources/dumps/plain.$TlYear.dump.txt"
Write-Host "[make_formats]   latex dump: resources/dumps/latex.$TlYear.dump.txt"
$KpseBanner = cmd /c "kpsewhich --version 2>nul" | Select-Object -First 1
Write-Host "[make_formats] texlive: $KpseBanner"
