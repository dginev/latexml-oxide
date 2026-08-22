# win_perf_bench.ps1 — Windows performance probe for latexml_oxide.
#
# Measures the three things that make Windows behave differently from the Linux
# CI/benchmark host (docs/performance/ARXIV_PERFORMANCE.md):
#   1. Fixed process cost — external wall vs the in-process telemetry `wall_us`
#      (the gap is spawn + global init + I/O the pipeline clock never sees).
#   2. Per-phase budget on a mix of repo fixtures (the 17-phase `--telemetry-out`
#      record; see docs/performance/TELEMETRY.md).
#   3. Subprocess-spawn microbench — the OS `CreateProcess` floor, the exe-load
#      floor, and a `kpsewhich` file lookup (the kpathsea `ls-R` init cost that
#      dominates the subprocess backend — docs/performance/WINDOWS_PERFORMANCE.md).
#
# The kpathsea backend the binary was built with (subprocess kpsewhich vs
# in-process linked libkpathsea, `--features kpathsea-build-from-source`) is
# echoed from the first conversion's log — run the script once per binary to A/B.
#
# Usage (from anywhere; TeX Live for Windows must be reachable):
#   $env:PATH = "C:\texlive\2026\bin\windows;" + $env:PATH   # NOT MiKTeX-first
#   tools\win_perf_bench.ps1
#   tools\win_perf_bench.ps1 -Bin path\to\latexml_oxide.exe -OutDir C:\tmp\perf -K 6
#
# Windows PowerShell 5.1 (no &&/||; native-command stderr is wrapped as
# NativeCommandError records — harmless, we time exit codes not $?).

param(
  [string]$Bin    = (Join-Path $PSScriptRoot '..\target\release\latexml_oxide.exe'),
  [string]$OutDir = (Join-Path $env:TEMP 'lxo-winperf'),
  [int]$K         = 6            # timed reps per doc; report min + median
)

$ErrorActionPreference = 'Continue'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
New-Item -ItemType Directory -Force $OutDir | Out-Null
Set-Location $repo

function Median($xs) {
  $s = @($xs | Sort-Object); $n = $s.Count
  if ($n -eq 0) { return 0 }
  if ($n % 2 -eq 1) { return $s[[int]([math]::Floor($n/2))] }
  return ($s[$n/2 - 1] + $s[$n/2]) / 2.0
}
function TimeMs($sb) {
  $sw = [System.Diagnostics.Stopwatch]::StartNew()
  & $sb 2>$null | Out-Null
  $sw.Stop(); return $sw.Elapsed.TotalMilliseconds
}

Write-Host '==== HOST ===='
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
Write-Host ("CPU: {0} ({1}C/{2}T)" -f $cpu.Name.Trim(), $cpu.NumberOfCores, $cpu.NumberOfLogicalProcessors)
$os = Get-CimInstance Win32_OperatingSystem
Write-Host ("OS: {0}  RAM: {1:N1} GiB free / {2:N1} GiB total" -f $os.Caption, ($os.FreePhysicalMemory/1MB), ($os.TotalVisibleMemorySize/1MB))
try {
  $mp = Get-MpComputerStatus -ErrorAction Stop
  Write-Host ("Defender RealTimeProtection: {0}   BehaviorMonitor: {1}" -f $mp.RealTimeProtectionEnabled, $mp.BehaviorMonitorEnabled)
} catch { Write-Host "Defender status: unavailable ($($_.Exception.Message))" }

Write-Host "`n==== KPATHSEA BACKEND ===="
$he = Join-Path $OutDir 'hello.stderr.txt'
& $Bin (Join-Path $repo 'latexml_oxide\tests\hello\hello.tex') --dest (Join-Path $OutDir 'hello.html') 2> $he | Out-Null
Write-Host ("  " + (Select-String -Path $he -Pattern 'kpathsea:backend' | Select-Object -First 1).Line)

Write-Host "`n==== SPAWN MICROBENCH (min ms of $K) ===="
$floor = 1..$K | ForEach-Object { TimeMs { & cmd /c rem } }
Write-Host ("  cmd /c rem (OS spawn floor):      min {0,7:N1}  median {1,7:N1}" -f ($floor|Measure-Object -Minimum).Minimum, (Median $floor))
$ver = 1..$K | ForEach-Object { TimeMs { & $Bin --version } }
Write-Host ("  latexml_oxide --version:          min {0,7:N1}  median {1,7:N1}" -f ($ver|Measure-Object -Minimum).Minimum, (Median $ver))
$kpseV = 1..$K | ForEach-Object { TimeMs { & kpsewhich --version } }
Write-Host ("  kpsewhich --version (no lookup):  min {0,7:N1}  median {1,7:N1}" -f ($kpseV|Measure-Object -Minimum).Minimum, (Median $kpseV))
$kpse = 1..$K | ForEach-Object { TimeMs { & kpsewhich cmr10.tfm } }
Write-Host ("  kpsewhich cmr10.tfm (ls-R init):  min {0,7:N1}  median {1,7:N1}" -f ($kpse|Measure-Object -Minimum).Minimum, (Median $kpse))
$kpse5 = 1..$K | ForEach-Object { TimeMs { & kpsewhich cmr10.tfm article.cls tikz.sty xcolor.sty pgf.sty } }
Write-Host ("  kpsewhich x5 (one process):       min {0,7:N1}  median {1,7:N1}" -f ($kpse5|Measure-Object -Minimum).Minimum, (Median $kpse5))

$inputs = @(
  'latexml_oxide\tests\hello\hello.tex',
  'latexml_oxide\tests\structure\book.tex',
  'latexml_oxide\tests\complex\si.tex',
  'latexml_oxide\tests\benchmark\equality_big.tex',
  'latexml_oxide\tests\tikz\various_colors.tex'
)
$PHASES = @('bootstrap','digest','build','rewrite','math_parse','post_xml_parse','post_scan','bibliography','crossref','graphics','math_images','mathml_pres','mathml_cont','split','xslt','html5_fixups','serialize')

Write-Host "`n==== PER-DOC WALL (ms) ===="
Write-Host ("  {0,-16} {1,10} {2,10} {3,10} {4,10}" -f 'doc','ext_min','ext_med','tele_wall','delta')
$agg = @{}; foreach ($p in $PHASES) { $agg[$p] = [double]0 }
foreach ($rel in $inputs) {
  $name = [System.IO.Path]::GetFileNameWithoutExtension($rel)
  $src  = Join-Path $repo $rel
  if (-not (Test-Path $src)) { Write-Host ("  {0,-16} MISSING" -f $name); continue }
  $dest = Join-Path $OutDir ($name + '.html')
  $tele = Join-Path $OutDir ($name + '.telemetry.json')
  & $Bin $src --dest $dest --telemetry-out $tele 2>$null | Out-Null   # warm
  $ext = 1..$K | ForEach-Object { TimeMs { & $Bin $src --dest $dest --telemetry-out $tele } }
  $extMin = ($ext | Measure-Object -Minimum).Minimum; $extMed = Median $ext
  $tMs = 0
  if (Test-Path $tele) {
    $j = Get-Content $tele -Raw | ConvertFrom-Json
    $tMs = [double]$j.wall_us / 1000.0
    for ($i=0; $i -lt $PHASES.Count; $i++) { $agg[$PHASES[$i]] += [double]$j.phase_us[$i] }
  }
  Write-Host ("  {0,-16} {1,10:N1} {2,10:N1} {3,10:N1} {4,10:N1}" -f $name, $extMin, $extMed, $tMs, ($extMed-$tMs))
}

Write-Host "`n==== AGGREGATE PHASE BUDGET (sum over docs) ===="
$sum = ($agg.Values | Measure-Object -Sum).Sum
if ($sum -gt 0) {
  foreach ($p in $PHASES) {
    if ($agg[$p] -gt 0) { Write-Host ("  {0,-16} {1,7:N1}%  ({2,12:N0} us)" -f $p, (100.0*$agg[$p]/$sum), $agg[$p]) }
  }
}
Write-Host "`n[done]"
