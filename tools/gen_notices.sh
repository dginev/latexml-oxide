#!/usr/bin/env bash
# Assemble the complete THIRD-PARTY-NOTICES for a release artifact:
# the hand-authored sections 1-4 (embedded TeX dumps LPPL/Knuth, Perl-LaTeXML
# assets, linked syslibs, subprocess tools) plus the auto-generated section 5
# (Rust dependency licenses, from the actual lockfile).
#
# The Rust section is generated with the SAME feature set the artifact being
# packaged ships, so it attributes exactly the crates linked in — no more, no
# less. Default = the release binary's set; the cortex-worker container links a
# different graph, so it overrides via NOTICES_CARGO_FEATURES.
#
# Usage:  tools/gen_notices.sh [OUTPUT]     (default: ./THIRD-PARTY-NOTICES.dist)
# Env:    NOTICES_CARGO_FEATURES  cargo feature flags for section 5's graph
#                                 (default: --no-default-features --features runtime-bindings)
#         LATEXML_SELF_REV        our own commit for section 7, when `git` cannot
#                                 answer (the container build excludes .git)
# Requires: cargo-about  (cargo install cargo-about --features cli)
#
# See docs/release/LICENSE_INVENTORY.md (the audit) and docs/release/RELEASE_CRITERIA.md §4.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

out="${1:-THIRD-PARTY-NOTICES.dist}"
read -r -a cargo_features <<<"${NOTICES_CARGO_FEATURES:---no-default-features --features runtime-bindings}"

if ! command -v cargo-about >/dev/null 2>&1; then
  echo "error: cargo-about not found. Install with:" >&2
  echo "  cargo install cargo-about --features cli" >&2
  exit 1
fi

# The input MUST be the hand-authored sections 1-4 and nothing more. If it already
# carries a section 5, this is a re-run over a previously assembled file and `cat`
# below would append a SECOND copy of sections 5/6/7 -- which every downstream gate
# would happily pass, since they all test for the PRESENCE of a marker and a line
# count FLOOR. Duplication makes them *more* likely to pass. That state is reachable:
# make_release.sh swaps this file for the .deb build, so a SIGKILL mid-build leaves
# the assembled content sitting in the tracked path. Refuse rather than compound it.
if grep -qF "5. RUST DEPENDENCY LICENSES" THIRD-PARTY-NOTICES; then
  echo "error: THIRD-PARTY-NOTICES already contains an assembled section 5." >&2
  echo "  This file must hold ONLY the hand-authored sections 1-4; appending to it" >&2
  echo "  would emit sections 5/6/7 twice, and no gate checks for duplicates." >&2
  echo "  A killed 'make_release.sh' can leave the assembled file in this path." >&2
  echo "  Restore it with:  git checkout -- THIRD-PARTY-NOTICES" >&2
  exit 1
fi

echo "Generating Rust dependency license section (${cargo_features[*]})..." >&2
rust_section="$(mktemp)"
trap 'rm -f "$rust_section"' EXIT
cargo about generate "${cargo_features[@]}" about.hbs -o "$rust_section"

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
# LATEXML_SELF_REV first: the container build has no .git in its context (see
# .dockerignore), so `git rev-parse` there answers nothing and section 7 would name
# no commit for the very application whose CC0 source discharges the relink promise.
# docker.yml passes the real sha through the GITSHA build-arg.
self_rev="${LATEXML_SELF_REV:-$(git rev-parse HEAD 2>/dev/null || echo "unknown")}"

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
  echo "  GNU libiconv -- LGPL-2.1-or-later (sec 3.1) -- WINDOWS .exe ONLY"
  echo "      Statically linked there via libxml2's vcpkg 'iconv' default feature;"
  echo "      on Linux/macOS iconv is a dynamically linked host system library and"
  echo "      no libiconv code is in the binary."
  echo "      https://www.gnu.org/software/libiconv/  (recipe: vcpkg ports/libiconv)"
  echo
  echo "To relink: clone latexml-oxide at the commit above, point the relevant dependency"
  echo "at your modified library -- the marpa/kpathsea crate, or (for libiconv) the vcpkg"
  echo "port the Windows leg installs -- and rebuild. latexml-oxide's own source is CC0,"
  echo "so nothing in our terms restricts you from doing so."
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
