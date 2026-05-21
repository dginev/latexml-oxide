# tools/

Utility scripts for development, triage, sandbox sweeps, kernel-dump
generation, schema work, and porting audits. Most are bash; a few are
Python or Perl. Run from the repo root unless noted.

## Quick lookup

| I want to … | Use |
|---|---|
| Classify Rust vs Perl on one paper | `parity_check.sh <arxiv_id>` |
| Triage a failing paper (test profile, backtrace) | `triage_failure.sh <arxiv_id>` |
| Bisect to a minimum repro | `bisect_repro.sh <arxiv_id> [canary]` |
| Extract first-error class from a log | `first_error.sh <paper.log>` |
| Run a stage sweep across N papers | `benchmark_canvas.sh --input-dir <dir> --stage N --stage-size 10000` |
| Aggregate stage-sweep results to TSV | `parity_stats.sh <stage_dir>` |
| Generate kernel dumps for ambient TL | `make_formats.sh` |
| Compile RNC → RNG schema | `compileschema.sh` |
| Generate HTML docs for a schema | `generate-scholarly-schema-docs --schema FOO.rnc --output DIR` |
| Run tests against TL2023 (CI parity) | `test_with_tl2023.sh` |
| Lint Claude Code memory tree | `claude_check_memory.py` |

## Categories

### Triage (paired with skills `canvas-triage`, `min-repro`)

- **`parity_check.sh`** — primary triage. Per-paper verdict: BOTH CLEAN /
  OUT-OF-SCOPE / REAL_REGRESSION / PERL_REGRESSION / Perl-capped / Perl-timeout.
  Reads `SANDBOX_DIR`; respects `TIMEOUT_SECS=180`.
- **`triage_failure.sh`** — phase-2 single-paper triage under test profile
  with `RUST_BACKTRACE=full`. `KEEP_TMP=1` preserves the extracted dir.
- **`bisect_repro.sh`** — coarse window-bisection from the first-error line,
  given an arxiv_id and optional canary pattern.
- **`first_error.sh`** — first non-cascade error class from a paper log,
  with source context.

### Sandbox sweeps (cortex_worker + post-sweep analysis)

- **`benchmark_canvas.sh`** — Phase-1 stage sweep. Auto-builds release
  `cortex_worker`; staged, resumable, structured logging. Tune
  `--workers`, `--timeout`, `MAX_RAM_KB` per machine.
- **`parity_stats.sh`** — TSV-emitting Rust-vs-Perl delta + verdict per
  paper from a completed stage directory.

### Build / setup

- **`make_formats.sh`** — generate versioned kernel dumps
  (`resources/dumps/{plain,latex}.YYYY.dump.txt` + `texlive.YYYY.version`).
  TL year detected via `kpsewhich -var-value=SELFAUTOPARENT` with
  `pdflatex --version` fallback. Run after checkout, TL upgrade, or
  whenever post-`\dump` engine state changes.
- **`compileschema.sh`** — RNC → RNG via `trang`, producing
  `resources/RelaxNG/LaTeXML.model`.
- **`generate-scholarly-schema-docs`** — standalone HTML schema-doc
  tree (trang + `genschema_oxide` + `latexml_oxide --split`).

### Perl-porting audits (one-shot per binding migration)

- **`audit_attrs.sh`** — Perl→Rust attribute parity sweep
  (locked / bounded / scope / requireMath / robust).
- **`audit_def_parity.py`** — Perl `Def*` vs Rust `Def*!` macro-kind
  parity counter.
- **`audit_locked.sh`** — `locked=>1` parity.

### Performance

- **`perf_compare.py`** — paired A/B comparison of two telemetry corpus
  runs (Δwall, Δphase_us).
- **`perf_phase_summary.py`** — per-phase rollups from telemetry JSONL.
- **`run_perf_corpus.sh`** — Tier A serial regression baseline runner.

See `docs/PERFORMANCE.md` for the full perf-track methodology.

### CI parity

- **`test_with_tl2023.sh`** — run the test suite against a TeXLive 2023
  install (matching the sibling canvas-machine).

### AI agent tools (`claude_` prefix)

- **`claude_check_memory.py`** — lint
  `~/.claude/projects/<slug>/memory/` for broken `[[link]]` /
  `(file.md)` refs, orphan files, `MEMORY.md` budget overrun (200
  lines), stale `\cs` claims. Per-contributor; portable across
  machines via auto-derived slug. Run periodically or `--strict` in
  pre-push.

### Reference / one-shot

- **`compile_metrics.pl`** — Bruce Miller's upstream TFM compilation
  tool (Perl). Kept for cross-reference; not used in normal builds.
- **`convert_metrics.pl`** — one-shot converter from Perl
  `StandardMetrics.pm` to Rust `standard_metrics.rs`. Job done;
  preserved for archaeology.

## Conventions

- **`claude_<name>.<ext>`** — tools that only operate on Claude Code
  local state (e.g. memory tree). Sibling machines / human
  contributors don't run these.
- Everything else is general-purpose; both humans and AI agents use
  the same triage / sweep / build tooling.
- Scripts assume they're invoked from the repo root unless their
  header says otherwise.
