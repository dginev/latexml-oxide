#!/usr/bin/env bash
# lint_raw_log_diag.sh — the diagnostic vehicle stays SINGLE.
#
# Perl LaTeXML has exactly one emission vehicle per severity (Error.pm's
# Info/Warn/Error/Fatal), which is what makes its tally and cortex's
# aggregation lossless by construction. The Rust equivalents are the
# Info!/Warn!/Error!/Fatal! macros and their function forms
# (latexml_core::common::error::emit_info/emit_warn/emit_error/emit_fatal for
# non-Result contexts). A raw `log::warn!`/`log::error!`/`log::info!` call
# bypasses ALL severity semantics: the tally (the 131 MB witness logged
# 12,105 Warning: lines and reported "2 warnings"), the MAX_ERRORS cap, the
# consecutive-error runaway breaker, output suppression, fatal demotion, and
# the category:object taxonomy cortex classifies by.
#
# This lint FAILS on any raw log::{info,warn,error}! call in workspace crates.
# The only legitimate homes are allowlisted below: the vehicle's own
# implementation (error.rs), and the logger backend. `log::debug!`/`trace!`
# stay free — they are developer chatter, not conversion diagnostics.
#
# The logger backend still counts any raw record it prints
# (note_status_from_logger) — that net exists for FOREIGN crates logging
# through the `log` facade, not as licence for workspace code.

set -euo pipefail
cd "$(dirname "$0")/.."

ALLOW_RE='^(latexml_core/src/common/error\.rs|latexml_core/src/util/logger\.rs)'

violations=$(grep -rn --include="*.rs" -E 'log::(info|warn|error)!' \
  latexml_core/src latexml_engine/src latexml_package/src latexml_contrib/src \
  latexml_post/src latexml_oxide/src latexml_oxide/bin latexml_math_parser/src \
  cortex_worker/src 2>/dev/null \
  | grep -vE "$ALLOW_RE" \
  | grep -vE '^\S+:\s*[0-9]+:\s*//' || true)

if [ -n "$violations" ]; then
  echo "ERROR: raw log::{info,warn,error}! diagnostics found — use the single"
  echo "vehicle instead: Info!/Warn!/Error!/Fatal! macros, or"
  echo "latexml_core::common::error::emit_{info,warn,error,fatal} in"
  echo "non-Result contexts. Rationale in this script's header."
  echo
  echo "$violations"
  exit 1
fi
echo "OK: no raw log-crate diagnostics in workspace crates."
