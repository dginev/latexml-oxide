#!/usr/bin/env bash
# Sweep Perl→Rust port for `locked=>1`, `bounded=>1`,
# `scope=>'global'`, `requireMath=>1`, `robust=>1` attribute parity.
#
# For each attribute, counts active (non-commented) occurrences in each
# Perl .ltxml that has any, then compares to the paired Rust .rs file
# counting both `attr => value` macro-arg and `attr: value` struct-
# literal syntaxes. Prints DRIFT lines when counts differ.
#
# Not every drift is a bug. Common benign causes:
#  - Perl re-declares the same CS twice (TeX last-wins) with the same
#    attribute, Rust deduplicates.
#  - Rust adds defensive `locked` on forward-refs Perl leaves open.
#  - Perl's `scope => 'global'` is sometimes inlined into RawTeX
#    `\global\def`, which the grep can't detect.
#  - Rust delegates to shared engine helpers (e.g. llncs_cls
#    `\spnewtheorem` → latex_constructs::define_new_theorem which
#    handles global scope internally across 60+ sites). Per-file
#    drift count doesn't capture the delegation.
# Investigate each per case.
set -euo pipefail

map_rust_stem() {
  local name="$1"
  local stem="${name%.ltxml}"
  echo "$stem" | sed -E 's/\.(sty|cls|tex|def|ltx)$/_\1/' \
    | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9_' '_' | sed 's/_*$//'
}

find_rust_file() {
  local rstem="$1"
  for root in latexml_package/src/package latexml_package/src/engine latexml_contrib/src; do
    local c="$root/${rstem}.rs"
    [ -f "$c" ] && { echo "$c"; return 0; }
  done
  return 1
}

audit_attr() {
  local perl_pat="$1"  # e.g. "locked *=> *1"
  local rust_pat="$2"  # e.g. "locked *(=>|:) *true"
  local label="$3"
  echo "=== $label ==="
  local files
  files=$(grep -rl -E "$perl_pat" LaTeXML/lib/LaTeXML/Package/ ar5iv-bindings/bindings/ 2>/dev/null || true)
  local n_files=0 n_drift=0
  for pf in $files; do
    n_files=$((n_files + 1))
    local rstem rfile
    rstem=$(map_rust_stem "$(basename "$pf")")
    rfile=$(find_rust_file "$rstem" || true)
    [ -z "$rfile" ] && continue
    local plocked rlocked
    plocked=$({ grep -E "$perl_pat" "$pf" || true; } | { grep -vcE '^\s*#' || true; })
    rlocked=$({ grep -E "$rust_pat" "$rfile" || true; } | { grep -vcE '^\s*//' || true; })
    if [ "$plocked" != "$rlocked" ]; then
      n_drift=$((n_drift + 1))
      printf '  DRIFT\t%s\tperl=%s\trust=%s\t%s\n' \
        "$rstem" "$plocked" "$rlocked" "$rfile"
    fi
  done
  echo "  ($n_files Perl files scanned, $n_drift drifts)"
}

audit_attr 'locked *=> *1'               'locked *(=>|:) *true'                    "locked"
audit_attr 'bounded *=> *1'              'bounded *(=>|:) *true'                   "bounded"
audit_attr "scope *=> *'global'"         "Scope::Global|scope *=> *scope" "scope=>global"
audit_attr 'requireMath *=> *1'          'require_math *(=>|:) *true'              "requireMath"
audit_attr 'robust *=> *1'               'robust *(=>|:) *true'                    "robust"
