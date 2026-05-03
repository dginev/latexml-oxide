#!/usr/bin/env python3
# perf_compare.py — paired A/B comparison of two telemetry corpus runs.
#
# See docs/TELEMETRY.md §4 Step 8. Used after perf-affecting commits per
# the docs/PERFORMANCE.md Optimization Acceptance Checklist.
#
# Usage:
#   tools/perf_compare.py <baseline> <new>
#
# <baseline> and <new> are each either:
#   - a JSONL[.gz] file produced by benchmark_canvas.sh JSONL aggregation, or
#   - a directory of cortex_worker output ZIPs (each containing telemetry.json)
#
# Reports per-paper Δwall, Δphase_us, Δerrors. Joins on paper_id.

from __future__ import annotations

import sys
from pathlib import Path
from statistics import median

# Reuse the loader in perf_phase_summary
sys.path.insert(0, str(Path(__file__).parent))
from perf_phase_summary import PHASES, load_records, fmt_us  # noqa: E402


def index_by_paper(path: Path) -> dict:
  out = {}
  for r in load_records(path):
    pid = r.get("paper_id")
    if pid:
      out[pid] = r
  return out


def main(argv: list[str]) -> int:
  if len(argv) != 3:
    print(__doc__.lstrip())
    return 1

  base = Path(argv[1])
  new = Path(argv[2])
  for p in (base, new):
    if not p.exists():
      print(f"error: {p} not found", file=sys.stderr)
      return 2

  base_idx = index_by_paper(base)
  new_idx = index_by_paper(new)

  shared = sorted(set(base_idx) & set(new_idx))
  only_base = sorted(set(base_idx) - set(new_idx))
  only_new = sorted(set(new_idx) - set(base_idx))

  print(f"baseline: {len(base_idx)} jobs from {base}")
  print(f"new:      {len(new_idx)} jobs from {new}")
  print(f"shared:   {len(shared)} jobs")
  if only_base:
    print(f"only in baseline: {len(only_base)} (sample: {only_base[:3]})")
  if only_new:
    print(f"only in new:      {len(only_new)} (sample: {only_new[:3]})")
  print()

  if not shared:
    print("error: no shared papers; cannot compare", file=sys.stderr)
    return 3

  # Aggregate Δwall + Δphase_us
  d_walls = []
  d_phase_totals = {p: 0 for p in PHASES}
  base_phase_totals = {p: 0 for p in PHASES}
  d_errors = 0
  base_errors = 0
  regressed_papers = []  # (pid, dwall_us, base_wall_us)

  for pid in shared:
    b = base_idx[pid]
    n = new_idx[pid]
    bw = b.get("wall_us", 0)
    nw = n.get("wall_us", 0)
    dw = nw - bw
    d_walls.append(dw)
    base_errors += b.get("errors", 0)
    d_errors += n.get("errors", 0) - b.get("errors", 0)
    bp = b.get("phase_us", [0] * len(PHASES))
    np_ = n.get("phase_us", [0] * len(PHASES))
    for i, ph in enumerate(PHASES):
      if i < len(bp):
        base_phase_totals[ph] += bp[i]
      if i < len(np_) and i < len(bp):
        d_phase_totals[ph] += np_[i] - bp[i]
    # 15% wall regression trigger (per PERFORMANCE.md "Regression trigger")
    if bw > 0 and dw > 0.15 * bw:
      regressed_papers.append((pid, dw, bw))

  base_total_wall = sum(b.get("wall_us", 0) for b in base_idx.values())
  new_total_wall = sum(new_idx[pid].get("wall_us", 0) for pid in shared)
  d_total_wall = new_total_wall - sum(base_idx[pid].get("wall_us", 0) for pid in shared)

  print(
    f"shared total wall:  base={fmt_us(sum(base_idx[pid].get('wall_us', 0) for pid in shared))} "
    f"new={fmt_us(new_total_wall)} "
    f"Δ={fmt_us(d_total_wall) if d_total_wall >= 0 else '-' + fmt_us(-d_total_wall)} "
    f"({100 * d_total_wall / max(1, sum(base_idx[pid].get('wall_us', 0) for pid in shared)):+.2f}%)"
  )
  print(f"errors:             base={base_errors}  Δ={d_errors:+d}")
  print()

  # Per-phase Δ
  print(f"{'phase':<16} {'base_total':>12} {'Δ':>12} {'Δ%':>9}")
  print("-" * 56)
  for ph in PHASES:
    base = base_phase_totals[ph]
    if base == 0 and d_phase_totals[ph] == 0:
      continue
    delta = d_phase_totals[ph]
    pct = (100 * delta / base) if base > 0 else float("inf")
    sign = "+" if delta >= 0 else "-"
    print(
      f"{ph:<16} {fmt_us(base):>12} {sign}{fmt_us(abs(delta)):>11} "
      f"{pct:+8.2f}%"
    )
  print()

  # Per-paper regressions
  if regressed_papers:
    regressed_papers.sort(key=lambda x: -x[1])
    print(f"Per-paper regressions (>15% wall, per PERFORMANCE.md):")
    for pid, dw, bw in regressed_papers[:30]:
      print(f"  {pid:<24}  base={fmt_us(bw):>10}  Δ={fmt_us(dw):>10}  ({100 * dw / bw:+.1f}%)")
    if len(regressed_papers) > 30:
      print(f"  ... and {len(regressed_papers) - 30} more")
    print()
  else:
    print("No per-paper regressions >15% threshold.")
    print()

  # Δwall distribution
  d_walls.sort()
  if d_walls:
    print("Δwall_us distribution across shared papers:")
    print(f"  min     {fmt_us(d_walls[0]) if d_walls[0] >= 0 else '-' + fmt_us(-d_walls[0])}")
    med = median(d_walls)
    print(f"  median  {fmt_us(med) if med >= 0 else '-' + fmt_us(-med)}")
    print(f"  max     {fmt_us(d_walls[-1]) if d_walls[-1] >= 0 else '-' + fmt_us(-d_walls[-1])}")
    n_better = sum(1 for x in d_walls if x < 0)
    n_worse = sum(1 for x in d_walls if x > 0)
    n_same = sum(1 for x in d_walls if x == 0)
    print(f"  faster: {n_better}/{len(d_walls)} | slower: {n_worse} | same: {n_same}")

  return 0


if __name__ == "__main__":
  sys.exit(main(sys.argv))
