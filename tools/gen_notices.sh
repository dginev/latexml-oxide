#!/usr/bin/env bash
# Assemble the complete THIRD-PARTY-NOTICES for a release artifact:
# the hand-authored sections 1-4 (embedded TeX dumps LPPL/Knuth, Perl-LaTeXML
# assets, linked syslibs, subprocess tools) plus the auto-generated section 5
# (Rust dependency licenses, from the actual lockfile).
#
# The Rust section is generated with the SAME feature set the distributed binary
# ships (`--no-default-features --features runtime-bindings`), so it attributes
# exactly the crates that are linked in — no more, no less.
#
# Usage:  tools/gen_notices.sh [OUTPUT]     (default: ./THIRD-PARTY-NOTICES.dist)
# Requires: cargo-about  (cargo install cargo-about --features cli)
#
# See docs/release/LICENSE_INVENTORY.md (the audit) and docs/release/RELEASE_CRITERIA.md §4.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

out="${1:-THIRD-PARTY-NOTICES.dist}"

if ! command -v cargo-about >/dev/null 2>&1; then
  echo "error: cargo-about not found. Install with:" >&2
  echo "  cargo install cargo-about --features cli" >&2
  exit 1
fi

echo "Generating Rust dependency license section (shipped feature set)..." >&2
rust_section="$(mktemp)"
trap 'rm -f "$rust_section"' EXIT
cargo about generate \
  --no-default-features --features runtime-bindings \
  about.hbs -o "$rust_section"

# hand-authored sections 1-4  +  generated section 5  +  section 6 (copyleft texts)
#
# Section 6 carries the verbatim license texts that the STATICALLY linked LGPL
# components require us to ship: libkpathsea (LGPL-2.1) and libmarpa's
# libavl/obstack-derived files (LGPL-3.0 / LGPL-2.1) — see THIRD-PARTY-NOTICES
# §3.2-§3.5. LGPL-3.0 is a set of additional permissions on top of GPL-3.0 and
# is not self-contained, so the GPL-3.0 text ships alongside it.
copyleft_section="$(mktemp)"
trap 'rm -f "$rust_section" "$copyleft_section"' EXIT
{
  echo "--------------------------------------------------------------------------------"
  echo "6. COPYLEFT LICENSE TEXTS"
  echo "--------------------------------------------------------------------------------"
  echo
  echo "Verbatim texts of the copyleft licenses referenced from section 3, shipped"
  echo "as those licenses require. They apply to the third-party libraries named"
  echo "there — NOT to latexml-oxide's own source, which is CC0-1.0 (see LICENSE)."
  echo
  for pair in "GNU LESSER GENERAL PUBLIC LICENSE v2.1:licenses/LGPL-2.1.txt" \
              "GNU LESSER GENERAL PUBLIC LICENSE v3.0:licenses/LGPL-3.0.txt" \
              "GNU GENERAL PUBLIC LICENSE v3.0 (incorporated by reference into LGPL-3.0):licenses/GPL-3.0.txt"; do
    title="${pair%%:*}"; path="${pair#*:}"
    [[ -f "$path" ]] || { echo "error: missing license text $path" >&2; exit 1; }
    echo "================================================================================"
    echo "$title"
    echo "--------------------------------------------------------------------------------"
    cat "$path"
    echo
  done
} > "$copyleft_section"

cat THIRD-PARTY-NOTICES "$rust_section" "$copyleft_section" > "$out"
echo "Wrote $out ($(wc -l < "$out") lines)." >&2
