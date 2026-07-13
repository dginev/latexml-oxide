#!/usr/bin/env python3
# telemetry_dashboard.py — the RELEASE_CRITERIA §5 tail-latency & RSS dashboard.
#
# The public-quality risk is OUTLIERS, not the mean (a 60s timeout tail, a
# math-ambiguity blow-up, a 4 GiB alloc failure, a high-RSS package load).
# perf_phase_summary.py answers "where does wall time GO on average" (phase
# attribution); this answers "how bad is the TAIL, and is it GROWING". It reports
# P50/P90/P99/max wall + peak-RSS, the phase that drives the P99, and the top
# fatal / timeout / high-RSS / math-ambiguity witnesses — then, with --gate,
# enforces a "no unbounded growth" release gate that is DELIBERATELY SEPARATE
# from "mean beats Perl": absolute red lines (RSS→4 GiB, wall→timeout) plus an
# optional regression check against a committed baseline.
#
# Pure stdlib (json + gzip + statistics) — no pandas/numpy, runnable on any
# stock Python 3. Shares the record format with docs/performance/TELEMETRY.md and
# latexml_core/src/telemetry.rs (`Telemetry::to_json_line`).
#
# Usage:
#   tools/telemetry_dashboard.py <telemetry.jsonl[.gz] | output_dir> [opts]
#     --top N                 witnesses per list (default 10)
#     --html PATH             also write a self-contained HTML dashboard
#     --gate                  exit 1 on a growth violation (for CI/release)
#     --baseline PATH         baseline JSON for relative regression gating
#     --update-baseline PATH  write the current percentiles as a baseline, exit
#     --rss-redline-mb N      absolute peak-RSS red line (default 3500)
#     --wall-redline-frac F   P99 wall may reach this fraction of timeout_s
#                             (default 0.90)
#     --tolerance F           relative regression tolerance vs baseline
#                             (default 0.25 = +25%)
#
# Baseline workflow: capture a representative full-corpus telemetry.jsonl.gz
# from a fleet run, `--update-baseline resources/telemetry/baseline.json`, commit
# it, then run `--gate --baseline …` in CI against each new run.

from __future__ import annotations

import argparse
import gzip
import html
import json
import sys
import zipfile
from pathlib import Path

# 17-phase order — must match telemetry.rs `Phase` and perf_phase_summary.py.
PHASES = [
  "bootstrap", "digest", "build", "rewrite", "math_parse", "post_xml_parse",
  "post_scan", "bibliography", "crossref", "graphics", "math_images",
  "mathml_pres", "mathml_cont", "split", "xslt", "html5_fixups", "serialize",
]

# Outcome categories that count as a hard failure for the growth gate.
FAILURE_CATEGORIES = {"fatal", "timeout", "conversion_error", "error", "oom"}


def load_records(path: Path):
  """Yield Telemetry dicts from a JSONL[.gz] file or a directory of output ZIPs
  (each ZIP carries a telemetry.json member, as cortex_worker writes)."""
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
  opener = gzip.open if path.suffix == ".gz" else open
  with opener(path, "rt", encoding="utf-8") as fh:
    for line in fh:
      line = line.strip()
      if line:
        try:
          yield json.loads(line)
        except json.JSONDecodeError:
          continue


# ---- percentiles (nearest-rank, robust for small N) ----------------------

def pct(sorted_vals, q):
  """q in [0,100]; nearest-rank percentile. Empty -> 0."""
  if not sorted_vals:
    return 0
  if len(sorted_vals) == 1:
    return sorted_vals[0]
  rank = max(1, min(len(sorted_vals), round(q / 100.0 * len(sorted_vals))))
  return sorted_vals[rank - 1]


def summarize(vals):
  """Return {p50,p90,p99,max,mean,n} for a list of numbers."""
  s = sorted(vals)
  n = len(s)
  return {
    "p50": pct(s, 50), "p90": pct(s, 90), "p99": pct(s, 99),
    "max": s[-1] if s else 0,
    "mean": (sum(s) / n) if n else 0,
    "n": n,
  }


def us_to_s(us):
  return us / 1_000_000.0


def kb_to_mb(kb):
  return kb / 1024.0


# ---- report --------------------------------------------------------------

def build_stats(records):
  """Compute the full stat bundle the text/html/gate all consume."""
  walls = [r.get("wall_us", 0) for r in records]
  rss = [r.get("max_rss_kb", 0) for r in records]
  wall_stats = summarize(walls)
  rss_stats = summarize(rss)

  # Per-phase P99 (µs) — which phase drives the tail.
  phase_p99 = {}
  for i, name in enumerate(PHASES):
    col = [r.get("phase_us", [0] * len(PHASES))[i]
           for r in records
           if i < len(r.get("phase_us", []))]
    phase_p99[name] = summarize(col)

  # Outcome breakdown.
  categories = {}
  for r in records:
    c = r.get("category", "unknown")
    categories[c] = categories.get(c, 0) + 1

  # A "heavy" math parse = a formula in the ≥100 ms bucket (index 8 of 9).
  def heavy_parses(r):
    b = r.get("math_parse_buckets", [])
    return b[8] if len(b) > 8 else 0

  return {
    "n": len(records),
    "wall": wall_stats,
    "rss": rss_stats,
    "phase_p99": phase_p99,
    "categories": categories,
    "heavy_parses": heavy_parses,
  }


def top(records, key, n, predicate=None):
  pool = [r for r in records if predicate(r)] if predicate else records
  return sorted(pool, key=key, reverse=True)[:n]


def pid(r):
  return r.get("paper_id", "?")


def print_report(records, stats, top_n):
  n = stats["n"]
  shas = sorted({r.get("git_sha", "")[:12] for r in records if r.get("git_sha")})
  hosts = sorted({r.get("host", "") for r in records if r.get("host")})
  print(f"=== Tail-latency & RSS dashboard — {n} jobs ===")
  if shas:
    print(f"git_sha: {', '.join(shas)}   host: {', '.join(hosts) or '?'}")
  print()

  print("Outcomes:")
  for c, k in sorted(stats["categories"].items(), key=lambda kv: -kv[1]):
    flag = "  <-- failure" if c in FAILURE_CATEGORIES else ""
    print(f"  {c:<20} {k:>7}  ({k / n * 100:.1f}%){flag}")
  print()

  w, r = stats["wall"], stats["rss"]
  print(f"{'metric':<14} {'p50':>10} {'p90':>10} {'p99':>10} {'max':>10} {'mean':>10}")
  print(f"{'wall (s)':<14} {us_to_s(w['p50']):>10.2f} {us_to_s(w['p90']):>10.2f} "
        f"{us_to_s(w['p99']):>10.2f} {us_to_s(w['max']):>10.2f} {us_to_s(w['mean']):>10.2f}")
  print(f"{'peak RSS (MB)':<14} {kb_to_mb(r['p50']):>10.0f} {kb_to_mb(r['p90']):>10.0f} "
        f"{kb_to_mb(r['p99']):>10.0f} {kb_to_mb(r['max']):>10.0f} {kb_to_mb(r['mean']):>10.0f}")
  print()

  print("Phase P99 (s) — the tail's cost centres (top 6):")
  ranked = sorted(stats["phase_p99"].items(), key=lambda kv: -kv[1]["p99"])[:6]
  for name, ps in ranked:
    if ps["p99"] == 0:
      continue
    print(f"  {name:<16} p99={us_to_s(ps['p99']):>8.2f}  p90={us_to_s(ps['p90']):>8.2f}  max={us_to_s(ps['max']):>8.2f}")
  print()

  def witness_list(title, rows, fmt):
    if not rows:
      return
    print(f"{title}:")
    for r in rows:
      print(f"  {fmt(r)}")
    print()

  witness_list(
    f"Slowest {top_n} (wall)", top(records, lambda r: r.get("wall_us", 0), top_n),
    lambda r: f"{pid(r):<24} {us_to_s(r.get('wall_us', 0)):>8.2f}s  "
              f"rss={kb_to_mb(r.get('max_rss_kb', 0)):>6.0f}MB  [{r.get('category', '?')}]")
  witness_list(
    f"Highest {top_n} peak-RSS", top(records, lambda r: r.get("max_rss_kb", 0), top_n),
    lambda r: f"{pid(r):<24} {kb_to_mb(r.get('max_rss_kb', 0)):>7.0f}MB  "
              f"wall={us_to_s(r.get('wall_us', 0)):>7.2f}s  [{r.get('category', '?')}]")
  witness_list(
    "Failure witnesses (fatal/timeout/error)",
    top(records, lambda r: (r.get("fatal_errors", 0), r.get("errors", 0)), top_n,
        predicate=lambda r: r.get("category") in FAILURE_CATEGORIES
        or r.get("fatal_errors", 0) or r.get("errors", 0)),
    lambda r: f"{pid(r):<24} [{r.get('category', '?'):<16}] "
              f"fatal={r.get('fatal_errors', 0)} err={r.get('errors', 0)} "
              f"wall={us_to_s(r.get('wall_us', 0)):.2f}s")
  hp = stats["heavy_parses"]
  witness_list(
    "Math-ambiguity witnesses (≥100 ms parses / huge parse counts)",
    top(records, lambda r: (hp(r), r.get("math_parse_count", 0)), top_n,
        predicate=lambda r: hp(r) or r.get("math_parse_count", 0)),
    lambda r: f"{pid(r):<24} heavy={hp(r):>4}  parses={r.get('math_parse_count', 0):>10}  "
              f"math_parse={us_to_s(r.get('phase_us', [0]*len(PHASES))[4]):.2f}s")


# ---- growth gate ---------------------------------------------------------

def percentile_snapshot(stats):
  """The committable baseline: the numbers the gate compares."""
  return {
    "n": stats["n"],
    "wall_p50_us": stats["wall"]["p50"], "wall_p90_us": stats["wall"]["p90"],
    "wall_p99_us": stats["wall"]["p99"], "wall_max_us": stats["wall"]["max"],
    "rss_p50_kb": stats["rss"]["p50"], "rss_p90_kb": stats["rss"]["p90"],
    "rss_p99_kb": stats["rss"]["p99"], "rss_max_kb": stats["rss"]["max"],
  }


def run_gate(records, stats, args):
  """Return (ok, lines). Absolute red lines + optional baseline regression.
  Deliberately independent of any mean/Perl comparison."""
  lines = []
  ok = True
  rss_p99_mb = kb_to_mb(stats["rss"]["p99"])
  rss_max_mb = kb_to_mb(stats["rss"]["max"])

  # Absolute red line 1: peak RSS approaching the 4 GiB alloc-failure wall.
  if rss_max_mb > args.rss_redline_mb:
    ok = False
    lines.append(f"FAIL rss: max peak-RSS {rss_max_mb:.0f}MB > red line {args.rss_redline_mb}MB "
                 "(approaching the 4 GiB alloc-failure wall)")
  else:
    lines.append(f"ok   rss: max peak-RSS {rss_max_mb:.0f}MB <= {args.rss_redline_mb}MB "
                 f"(p99 {rss_p99_mb:.0f}MB)")

  # Absolute red line 2: P99 wall approaching the timeout.
  timeouts = [r.get("timeout_s", 0) for r in records if r.get("timeout_s", 0)]
  if timeouts:
    tmax = max(timeouts)
    limit = tmax * args.wall_redline_frac
    wall_p99_s = us_to_s(stats["wall"]["p99"])
    if wall_p99_s > limit:
      ok = False
      lines.append(f"FAIL wall: p99 wall {wall_p99_s:.1f}s > {args.wall_redline_frac:.0%} of "
                   f"{tmax}s timeout ({limit:.1f}s)")
    else:
      lines.append(f"ok   wall: p99 wall {wall_p99_s:.1f}s <= {limit:.1f}s "
                   f"({args.wall_redline_frac:.0%} of {tmax}s timeout)")

  # Absolute red line 3: any hard-timeout job at all is a growth signal.
  n_timeout = sum(1 for r in records if r.get("category") == "timeout")
  if n_timeout:
    lines.append(f"warn wall: {n_timeout} job(s) hit the hard timeout")

  # Relative regression vs a committed baseline.
  if args.baseline:
    try:
      base = json.loads(Path(args.baseline).read_text())
    except (OSError, json.JSONDecodeError) as e:
      lines.append(f"warn baseline: could not read {args.baseline} ({e}) — skipping regression check")
      base = None
    if base:
      cur = percentile_snapshot(stats)
      for key, label in [("wall_p99_us", "wall p99"), ("rss_p99_kb", "rss p99"),
                         ("wall_max_us", "wall max"), ("rss_max_kb", "rss max")]:
        b, c = base.get(key, 0), cur.get(key, 0)
        if b and c > b * (1 + args.tolerance):
          ok = False
          lines.append(f"FAIL regress: {label} grew {(c / b - 1) * 100:.0f}% "
                       f"(> {args.tolerance:.0%} tolerance) vs baseline")
        elif b:
          lines.append(f"ok   regress: {label} {(c / b - 1) * 100:+.0f}% vs baseline")
  else:
    lines.append("note baseline: none supplied — only absolute red lines enforced "
                 "(pass --baseline for regression gating)")
  return ok, lines


# ---- optional HTML -------------------------------------------------------

def write_html(records, stats, path):
  w, r = stats["wall"], stats["rss"]
  rows = "".join(
    f"<tr><td>{html.escape(pid(x))}</td><td class=n>{us_to_s(x.get('wall_us',0)):.2f}</td>"
    f"<td class=n>{kb_to_mb(x.get('max_rss_kb',0)):.0f}</td>"
    f"<td>{html.escape(x.get('category','?'))}</td></tr>"
    for x in top(records, lambda x: x.get("wall_us", 0), 25))
  phase_rows = "".join(
    f"<tr><td>{name}</td><td class=n>{us_to_s(ps['p99']):.2f}</td>"
    f"<td class=n>{us_to_s(ps['max']):.2f}</td></tr>"
    for name, ps in sorted(stats["phase_p99"].items(), key=lambda kv: -kv[1]['p99'])[:8]
    if ps["p99"])
  doc = f"""<!doctype html><meta charset=utf-8><title>Tail-latency dashboard</title>
<style>
:root{{color-scheme:light dark}}body{{font:14px/1.5 system-ui,sans-serif;margin:2rem;max-width:60rem}}
h1{{font-size:1.3rem}}table{{border-collapse:collapse;margin:.5rem 0 1.5rem;width:100%}}
th,td{{padding:.3rem .6rem;border-bottom:1px solid #8884;text-align:left}}
td.n,th.n{{text-align:right;font-variant-numeric:tabular-nums}}
.kpi{{display:inline-block;margin:.3rem 1.2rem .3rem 0}}.kpi b{{font-size:1.5rem}}
.over{{overflow-x:auto}}
</style>
<h1>Tail-latency &amp; RSS — {stats['n']} jobs</h1>
<div class=kpi>wall p99 <b>{us_to_s(w['p99']):.1f}s</b></div>
<div class=kpi>wall max <b>{us_to_s(w['max']):.1f}s</b></div>
<div class=kpi>RSS p99 <b>{kb_to_mb(r['p99']):.0f}MB</b></div>
<div class=kpi>RSS max <b>{kb_to_mb(r['max']):.0f}MB</b></div>
<h2>Phase P99 (s)</h2><div class=over><table><tr><th>phase</th><th class=n>p99</th><th class=n>max</th></tr>{phase_rows}</table></div>
<h2>Slowest 25 (wall)</h2><div class=over><table><tr><th>paper</th><th class=n>wall s</th><th class=n>RSS MB</th><th>category</th></tr>{rows}</table></div>
"""
  Path(path).write_text(doc, encoding="utf-8")


# ---- main ----------------------------------------------------------------

def main(argv):
  ap = argparse.ArgumentParser(description="Tail-latency & RSS dashboard (RELEASE_CRITERIA §5).")
  ap.add_argument("source", help="telemetry.jsonl[.gz] file or a directory of output ZIPs")
  ap.add_argument("--top", type=int, default=10)
  ap.add_argument("--html")
  ap.add_argument("--gate", action="store_true")
  ap.add_argument("--baseline")
  ap.add_argument("--update-baseline")
  ap.add_argument("--rss-redline-mb", type=float, default=3500)
  ap.add_argument("--wall-redline-frac", type=float, default=0.90)
  ap.add_argument("--tolerance", type=float, default=0.25)
  args = ap.parse_args(argv)

  records = list(load_records(Path(args.source)))
  if not records:
    print(f"telemetry_dashboard: no records in {args.source}", file=sys.stderr)
    return 2
  stats = build_stats(records)

  if args.update_baseline:
    Path(args.update_baseline).write_text(
      json.dumps(percentile_snapshot(stats), indent=2) + "\n", encoding="utf-8")
    print(f"wrote baseline ({stats['n']} jobs) -> {args.update_baseline}")
    return 0

  print_report(records, stats, args.top)
  if args.html:
    write_html(records, stats, args.html)
    print(f"[html] wrote {args.html}")

  if args.gate:
    ok, lines = run_gate(records, stats, args)
    print("=== growth gate (no unbounded growth — separate from mean-vs-Perl) ===")
    for ln in lines:
      print(f"  {ln}")
    print(f"GATE: {'PASS' if ok else 'FAIL'}")
    return 0 if ok else 1
  return 0


if __name__ == "__main__":
  sys.exit(main(sys.argv[1:]))
