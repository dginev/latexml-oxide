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
# See docs/LICENSE_INVENTORY.md (the audit) and docs/RELEASE_CRITERIA.md §4.
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

# hand-authored sections 1-4  +  generated section 5
cat THIRD-PARTY-NOTICES "$rust_section" > "$out"
echo "Wrote $out ($(wc -l < "$out") lines)." >&2
