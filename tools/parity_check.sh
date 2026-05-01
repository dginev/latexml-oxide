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
  for f in *.tex; do
    [[ -f "$f" ]] || continue
    if grep -lE '^\\documentstyle|^\\documentclass' "$f" >/dev/null 2>&1; then
      mainfile="$f"; break
    fi
  done
  [[ -z "$mainfile" ]] && mainfile=$(ls *.tex 2>/dev/null | head -1)
  if [[ -z "$mainfile" ]]; then
    echo "$paper: SKIP (no .tex)"
    cd / && rm -rf "$td"
    continue
  fi
  ulimit -v 6291456
  rust_errs=$(timeout "$TIMEOUT_SECS" "$RUST_BIN" --preload=ar5iv.sty --path="$AR5IV_PATH" "$mainfile" 2>&1 | grep -cE 'Error:')
  perl_errs=$(timeout "$TIMEOUT_SECS" latexml --preload=ar5iv.sty --path="$AR5IV_PATH" "$mainfile" 2>&1 | grep -cE 'Error:')
  status="UNKNOWN"
  if [[ "$rust_errs" -eq 0 && "$perl_errs" -eq 0 ]]; then
    status="BOTH CLEAN"
  elif [[ "$rust_errs" -eq "$perl_errs" ]]; then
    status="OUT-OF-SCOPE (Perl=$perl_errs)"
  elif [[ "$rust_errs" -gt "$perl_errs" ]]; then
    status="REAL REGRESSION (P=$perl_errs vs R=$rust_errs)"
  else
    status="PERL_REGRESSION (P=$perl_errs vs R=$rust_errs)"
  fi
  printf "%-30s Rust=%-3s Perl=%-3s %s\n" "$paper" "$rust_errs" "$perl_errs" "$status"
  cd / && rm -rf "$td"
done
