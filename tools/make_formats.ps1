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
$Bin = Join-Path "target\$BuildProfile" 'latexml_oxide.exe'

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

Write-Host "[make_formats] generating plain.$TlYear.dump.txt (--init=plain.tex)..."
& $Bin --init=plain.tex
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[make_formats] generating latex.$TlYear.dump.txt (--init=latex.ltx)..."
& $Bin --init=latex.ltx
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

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
