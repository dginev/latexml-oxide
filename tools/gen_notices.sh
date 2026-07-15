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

# Section 7: the exact revisions this artifact was built from.
#
# THIRD-PARTY-NOTICES 3.5 promises a recipient can relink the statically linked
# LGPL components (libkpathsea, parts of libmarpa) against a modified library.
# That promise is only as good as knowing WHICH sources went in. kpathsea is
# pinned in-repo (KPSE_REF), but the marpa git dep carries no `rev =`, so the
# revision actually built is recorded only in Cargo.lock -- which is gitignored.
# Rather than assert "version-pinned" and hope, resolve the revisions at release
# time and write them down, so each artifact names its own inputs exactly.
provenance_section="$(mktemp)"
trap 'rm -f "$rust_section" "$copyleft_section" "$provenance_section"' EXIT

# cargo-about already resolved the graph above, so Cargo.lock exists by now.
marpa_rev="$(sed -n 's|^source = "git+https://github.com/dginev/marpa#\(.*\)"$|\1|p' \
  Cargo.lock 2>/dev/null | head -1)"
kpse_ref="$(sed -n 's|^KPSE_REF="${KPSE_REF:-\(.*\)}"$|\1|p' \
  tools/build_static_kpathsea.sh 2>/dev/null | head -1)"
self_rev="$(git rev-parse HEAD 2>/dev/null || echo "unknown")"

{
  echo "--------------------------------------------------------------------------------"
  echo "7. SOURCE PROVENANCE (for relinking the LGPL components)"
  echo "--------------------------------------------------------------------------------"
  echo
  echo "Section 3.5 states that a recipient may relink the statically linked LGPL"
  echo "libraries against a modified version. These are the exact sources this"
  echo "artifact was built from, so that is reproducible rather than theoretical:"
  echo
  echo "  latexml-oxide (CC0-1.0, the application)"
  echo "      https://github.com/dginev/latexml-oxide"
  echo "      commit: ${self_rev}"
  echo
  echo "  libmarpa 8.6.2 -- MIT (Kegler) + LGPL-3.0/LGPL-2.1 parts (sec 3.3)"
  echo "      vendored verbatim as libmarpa-8.6.2.tar.gz in the libmarpa-sys crate"
  echo "      https://github.com/dginev/marpa"
  echo "      commit: ${marpa_rev:-unresolved}"
  echo
  echo "  libkpathsea -- LGPL-2.1-or-later (sec 3.2)"
  echo "      https://github.com/TeX-Live/texlive-source (texk/kpathsea)"
  echo "      commit: ${kpse_ref:-unresolved}"
  echo
  echo "To relink: clone latexml-oxide at the commit above, point the marpa/kpathsea"
  echo "dependency at your modified library, and rebuild. latexml-oxide's own source"
  echo "is CC0, so nothing restricts you from doing so."
  echo
} > "$provenance_section"

if [[ -z "$marpa_rev" || -z "$kpse_ref" ]]; then
  echo "error: could not resolve source provenance (marpa_rev='${marpa_rev}'," >&2
  echo "  kpse_ref='${kpse_ref}'). Section 7 would ship an unresolved relink" >&2
  echo "  pointer, so the LGPL relink promise in section 3.5 could not be kept." >&2
  echo "  Check Cargo.lock exists and tools/build_static_kpathsea.sh still sets" >&2
  echo "  KPSE_REF in the expected shape." >&2
  exit 1
fi

cat THIRD-PARTY-NOTICES "$rust_section" "$copyleft_section" "$provenance_section" > "$out"
echo "Wrote $out ($(wc -l < "$out") lines)." >&2
