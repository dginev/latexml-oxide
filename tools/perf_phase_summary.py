#!/usr/bin/env python3
# perf_phase_summary.py — read per-job telemetry JSONL, print rollups.
#
# See docs/performance/TELEMETRY.md §4 Step 6. Pure stdlib (json + gzip + statistics).
# Pandas / numpy intentionally avoided — keeps this runnable on any
# stock Python 3 install.
#
# Usage:
#   tools/perf_phase_summary.py <telemetry.jsonl[.gz]>
#   tools/perf_phase_summary.py <output_dir>           # auto-discovers
#                                                       # telemetry.json files
#                                                       # in output ZIPs
#
# Reading mode 1 (jsonl): each line is one job record (Telemetry::to_json_line).
# Reading mode 2 (dir): each *.zip in the directory contains a telemetry.json
# member written by cortex_worker. This is what benchmark_canvas.sh produces
# today before the JSONL aggregation step lands.

from __future__ import annotations

import gzip
import json
import sys
import zipfile
from pathlib import Path
from statistics import median, quantiles


PHASES = [
  "bootstrap",
  "digest",
  "build",
  "rewrite",
  "math_parse",
  "post_xml_parse",
  "post_scan",
  "bibliography",
  "crossref",
  "graphics",
  "math_images",
  "mathml_pres",
  "mathml_cont",
  "split",
  "xslt",
  "html5_fixups",
  "serialize",
]


def load_records(path: Path):
  """Yield Telemetry dicts from either a JSONL[.gz] file or a directory of
  output ZIPs."""
  if path.is_dir():
    for zip_path in sorted(path.glob("*.zip")):
      try:
        with zipfile.ZipFile(zip_path) as zf:
          if "telemetry.json" not in zf.namelist():
            continue
          with zf.open("telemetry.json") as f:
            line = f.read().decode("utf-8").strip()
            if line:
              yield json.loads(line)
      except (zipfile.BadZipFile, json.JSONDecodeError):
        continue
    return

  if path.suffix == ".gz":
    fh = gzip.open(path, "rt", encoding="utf-8")
  else:
    fh = open(path, encoding="utf-8")
  with fh:
    for line in fh:
      line = line.strip()
      if not line:
        continue
      try:
        yield json.loads(line)
      except json.JSONDecodeError:
        continue


def fmt_us(us: int) -> str:
  if us >= 1_000_000:
    return f"{us / 1e6:.2f}s"
  if us >= 1_000:
    return f"{us / 1e3:.1f}ms"
  return f"{us}us"


def fmt_pct(num, denom):
  if denom == 0:
    return "  -  "
  return f"{100 * num / denom:5.2f}%"


def main(argv: list[str]) -> int:
  if len(argv) != 2:
    print(__doc__.lstrip())
    return 1

  src = Path(argv[1])
  if not src.exists():
    print(f"error: {src} not found", file=sys.stderr)
    return 2

  records = list(load_records(src))
  if not records:
    print(f"error: no telemetry records found in {src}", file=sys.stderr)
    return 3

  n = len(records)
  total_wall = sum(r.get("wall_us", 0) for r in records)
  phase_totals = {p: 0 for p in PHASES}
  for r in records:
    pus = r.get("phase_us", [])
    for i, p in enumerate(PHASES):
      if i < len(pus):
        phase_totals[p] += pus[i]
  total_phase = sum(phase_totals.values())

  print(f"Read {n} jobs from {src}")
  print(f"Total wall: {fmt_us(total_wall)}")
  print(f"Sum of phase_us: {fmt_us(total_phase)} = {fmt_pct(total_phase, total_wall)} of wall")
  print()

  # Per-phase share
  print(f"{'phase':<16} {'total':>10} {'%wall':>8} {'%phase':>8}  {'mean':>9}")
  print("-" * 60)
  rows = sorted(phase_totals.items(), key=lambda kv: -kv[1])
  for phase, t in rows:
    if t == 0:
      continue
    print(
      f"{phase:<16} {fmt_us(t):>10} "
      f"{fmt_pct(t, total_wall):>8} "
      f"{fmt_pct(t, total_phase):>8}  "
      f"{fmt_us(t // n):>9}"
    )
  print()

  # Top-30 by each phase
  for phase in PHASES:
    if phase_totals[phase] == 0:
      continue
    idx = PHASES.index(phase)
    by_phase = sorted(
      records,
      key=lambda r, i=idx: r.get("phase_us", [0] * len(PHASES))[i] if i < len(r.get("phase_us", [])) else 0,
      reverse=True,
    )
    top = by_phase[:5]
    print(f"top-5 by {phase}:")
    for r in top:
      pus = r.get("phase_us", [])
      v = pus[idx] if idx < len(pus) else 0
      if v == 0:
        break
      pid = r.get("paper_id", "?")
      wall = r.get("wall_us", 0)
      print(f"  {pid:<24} {fmt_us(v):>10}  ({fmt_pct(v, wall)} of paper wall)")
    print()

  # sum-of-phase / wall distribution
  ratios = [
    (sum(r.get("phase_us", [])) / r["wall_us"]) if r.get("wall_us", 0) > 0 else 0
    for r in records
  ]
  ratios = [x for x in ratios if x > 0]
  if ratios:
    ratios.sort()
    print("sum_phase_us / wall_us distribution:")
    print(f"  min    {ratios[0]:.3f}")
    if len(ratios) >= 4:
      qs = quantiles(ratios, n=4)
      print(f"  q25    {qs[0]:.3f}")
    print(f"  median {median(ratios):.3f}")
    if len(ratios) >= 4:
      qs = quantiles(ratios, n=4)
      print(f"  q75    {qs[2]:.3f}")
    print(f"  max    {ratios[-1]:.3f}")
    above_92 = sum(1 for x in ratios if x >= 0.92)
    print(
      f"  {above_92}/{len(ratios)} jobs ({fmt_pct(above_92, len(ratios)).strip()}) "
      f"meet the docs/performance/TELEMETRY.md §6 acceptance ratio of >=0.92"
    )

  return 0


if __name__ == "__main__":
  sys.exit(main(sys.argv))
