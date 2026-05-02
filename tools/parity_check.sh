#!/usr/bin/env bash
# parity_check.sh — Compare Rust vs Perl LaTeXML error counts on sandbox
# papers, classifying each as one of:
#   - BOTH CLEAN          (Rust=0, Perl=0): conversion clean for both
#   - OUT-OF-SCOPE        (Rust==Perl, both >0): paper Perl can't handle
#                          either; not a Rust regression
#   - REAL REGRESSION     (Rust > Perl): genuine Rust-only divergence,
#                          worth investigating
#   - PERL_REGRESSION     (Rust < Perl): rare but theoretically possible
#
# This is the primary triage tool for the canvas-failure list. Most
# 100k_noproblem_sandbox_html "failures" turn out to be OUT-OF-SCOPE
# under the in-scope predicate "Perl=0 with current TL2025 + ar5iv-bindings",
# because the canvas list was generated under different conditions. Use
# this tool to filter the list down to the actual Rust-only regressions.
#
# Usage:
#   tools/parity_check.sh <arxiv_id_1> [arxiv_id_2] [...]
#   tools/parity_check.sh $(cat /tmp/sample.txt)
#
# Env:
#   SANDBOX_DIR  default: $HOME/data/100k_noproblem_sandbox
#                (root containing arxmliv/<yyMM>/<id>/<id>.zip)
#   AR5IV_PATH   default: $HOME/git/ar5iv-bindings/bindings
#   TIMEOUT_SECS default: 90
#
# Requires:
#   - target/debug/latexml_oxide  (run `cargo build --bin latexml_oxide`
#     beforehand if missing — script does NOT auto-rebuild to keep it fast)
#   - latexml in $PATH (Perl LaTeXML)

set -uo pipefail

SANDBOX_DIR="${SANDBOX_DIR:-$HOME/data/100k_noproblem_sandbox}"
AR5IV_PATH="${AR5IV_PATH:-$HOME/git/ar5iv-bindings/bindings}"
TIMEOUT_SECS="${TIMEOUT_SECS:-90}"
WORKSPACE="$(cd "$(dirname "$0")/.." && pwd)"
RUST_BIN="$WORKSPACE/target/debug/latexml_oxide"

if [[ ! -x "$RUST_BIN" ]]; then
  echo "ERROR: $RUST_BIN missing — run 'cargo build --bin latexml_oxide' first" >&2
  exit 1
fi

if [[ $# -eq 0 ]]; then
  echo "Usage: $0 <arxiv_id> [<arxiv_id> ...]" >&2
  echo "  Reads from stdin if no args:  cat ids.txt | $0" >&2
  exit 2
fi

# Allow piped input too
if [[ "$1" == "-" ]]; then
  mapfile -t papers
else
  papers=("$@")
fi

for paper in "${papers[@]}"; do
  zip=$(find "$SANDBOX_DIR" -name "${paper}.zip" 2>/dev/null | head -1)
  if [[ -z "$zip" ]]; then
    echo "$paper: SKIP (no zip in $SANDBOX_DIR)"
    continue
  fi
  td=$(mktemp -d)
  cd "$td" && unzip -q "$zip" 2>/dev/null
  mainfile=""
  # 1) Strong rule — file matching .tex/.TEX/.latex extension AND containing
  #    \documentclass/\documentstyle. Some sandbox papers use uppercase .TEX
  #    (gr-qc0003030 KERR2a.TEX), some use .latex (nucl-ex0203017
  #    collectif.latex), some use no extension at all (physics0103096 part1).
  shopt -s nullglob
  for f in *.tex *.TEX *.latex; do
    [[ -f "$f" ]] || continue
    if grep -lE '^\\documentstyle|^\\documentclass' "$f" >/dev/null 2>&1; then
      mainfile="$f"; break
    fi
  done
  # 2) Fallback — extension-less plain files containing \documentstyle/class.
  #    grep -I skips binary files (the .ps/.eps figures).
  if [[ -z "$mainfile" ]]; then
    for f in *; do
      [[ -f "$f" ]] || continue
      [[ "$f" == *.* ]] && continue  # has extension; was checked above or is binary
      if grep -IlE '^\\documentstyle|^\\documentclass' "$f" >/dev/null 2>&1; then
        mainfile="$f"; break
      fi
    done
  fi
  # 3) Last-resort fallback — alphabetical .tex/.TEX/.latex.
  [[ -z "$mainfile" ]] && mainfile=$( (ls *.tex *.TEX *.latex 2>/dev/null) | head -1)
  shopt -u nullglob
  if [[ -z "$mainfile" ]]; then
    echo "$paper: SKIP (no .tex)"
    cd / && rm -rf "$td"
    continue
  fi
  ulimit -v 6291456
  # Match LaTeXML's "Error:<category>:" prefix (e.g. "Error:undefined:",
  # "Error:unexpected:") — NOT the LaTeX kernel's "LaTeX hooks Error:"
  # pass-through which is just an info-level rendering of kernel
  # diagnostics.
  rust_log="$td/.rust.log"
  perl_log="$td/.perl.log"
  timeout "$TIMEOUT_SECS" "$RUST_BIN" --preload=ar5iv.sty --path="$AR5IV_PATH" "$mainfile" >"$rust_log" 2>&1
  rust_rc=$?
  timeout "$TIMEOUT_SECS" latexml --preload=ar5iv.sty --path="$AR5IV_PATH" "$mainfile" >"$perl_log" 2>&1
  perl_rc=$?
  rust_errs=$(grep -cE 'Error:[a-zA-Z_]+:' "$rust_log")
  perl_errs=$(grep -cE 'Error:[a-zA-Z_]+:' "$perl_log")
  # Some sandbox papers genuinely need >>2min for Perl LaTeXML; bumping
  # the local timeout doesn't scale. If Perl timed out AND its partial
  # log shows zero errors, treat the run as Perl=0 (paper is fine in
  # Perl, just slow). See feedback_perl_parity_timeout_handling.md.
  perl_to_tag=""
  if [[ "$perl_rc" -eq 124 && "$perl_errs" -eq 0 ]]; then
    perl_to_tag=" PERL_TIMEOUT_OK"
  elif [[ "$perl_rc" -eq 124 ]]; then
    perl_to_tag=" PERL_TIMEOUT(partial=$perl_errs)"
  fi
  rust_to_tag=""
  if [[ "$rust_rc" -eq 124 ]]; then
    rust_to_tag=" RUST_TIMEOUT(partial=$rust_errs)"
  fi
  status="UNKNOWN"
  # Perl's MAX_ERRORS default is 100 + Fatal('too_many_errors'), i.e. it hits
  # exactly 101 and bails. When Perl=101 the count is a CAP — Perl's true
  # count is unknown and likely much larger. Don't classify as "Rust > Perl"
  # in that case. Rust's default MAX_ERRORS is 10000.
  perl_capped=""
  if [[ "$perl_errs" -ge 101 ]]; then
    perl_capped=" PERL_CAPPED@$perl_errs"
  fi
  if [[ "$rust_errs" -eq 0 && "$perl_errs" -eq 0 ]]; then
    status="BOTH CLEAN"
  elif [[ "$rust_errs" -eq "$perl_errs" ]]; then
    status="OUT-OF-SCOPE (Perl=$perl_errs)"
  elif [[ "$perl_errs" -ge 101 ]]; then
    # Both engines have many errors; Perl truncated. Verdict undetermined.
    status="OUT-OF-SCOPE? (Perl-capped P=$perl_errs vs R=$rust_errs; cannot compare)"
  elif [[ "$rust_errs" -gt "$perl_errs" ]]; then
    status="REAL REGRESSION (P=$perl_errs vs R=$rust_errs)"
  else
    status="PERL_REGRESSION (P=$perl_errs vs R=$rust_errs)"
  fi
  printf "%-30s Rust=%-3s Perl=%-3s %s%s%s\n" "$paper" "$rust_errs" "$perl_errs" "$status" "$perl_to_tag" "$rust_to_tag"
  cd / && rm -rf "$td"
done
