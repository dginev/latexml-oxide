#!/usr/bin/env python3
"""Audit the shipped crate tree for native (C/C++/asm) code that cargo-deny cannot see.

WHY THIS EXISTS
---------------
cargo-deny and cargo-about both reason over crate *manifests*. A `-sys` crate that
compiles vendored C reports only its Rust wrapper's `license =` field, so the native
library's real copyright holder and terms never appear in any automated audit. This is
not hypothetical: libmarpa statically linked MIT (Kegler) *and* LGPL-3.0/LGPL-2.1
(libavl, GNU obstack) code into every binary this project ever shipped, attributed
nowhere, while `cargo deny check licenses` reported "licenses ok" the whole time.

So this script audits the thing manifests cannot express: which shipped crates actually
compile or link native code. Every hit must be listed in AUDITED below with a reason.
A new one fails the build until a human looks at what it compiles and attributes it.

This is a *tripwire, not an oracle*: it tells you a crate pulls in native code and must
be checked by hand. It cannot read licenses out of a tarball.

Cross-check: docs/release/LICENSE_INVENTORY.md D.2 (the hand-audit table) and
THIRD-PARTY-NOTICES section 3 (the attributions themselves).

Usage:  python3 tools/audit_vendored_natives.py [--verbose]
Exit:   0 = every native-code crate is audited; 1 = an unaudited crate appeared.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

# Everything here is anchored to the repo, never to the cwd. Both halves of this
# script have now been caught trusting the cwd, and both times the result was a
# confident green over a subset of the truth:
#   * RESOURCE_ROOTS was relative, so from any other directory rglob yielded nothing
#     and the script printed "ok resources/RelaxNG/svg/ (0 file(s))" -- exit 0 having
#     scanned no files at all.
#   * `cargo tree`/`cargo metadata` inherited the cwd, so running one directory down
#     resolved a DIFFERENT manifest: libmimalloc-sys (whose vendored C is (c) Microsoft
#     while its wrapper LICENSE.txt names Octavian Oncescu -- the exact case NOTICES 3.4
#     exists for) dropped out of the tree, the gate printed "OK: all 4 native-code crates
#     are audited", exited 0, and listed the disappearance as a "stale" entry inviting
#     someone to delete its attribution.
# That second one is the libmarpa path-prefix bug reincarnated. The lesson both times:
# a gate that reports success for work it never did is worse than no gate.
REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST_ARGS = ["--manifest-path", str(REPO_ROOT / "Cargo.toml")]

# The feature set the distributed binary ships (CLAUDE.md, RELEASE_CRITERIA 4).
# Audit exactly what we ship -- no more, no less.
#
# RELEASE_EXTRA_FEATURES (release.yml passes `kpathsea-build-from-source` on the
# Windows leg) is folded in rather than ignored: it is part of what we publish, and a
# per-target feature that pulled in a native crate would otherwise be invisible here.
# It adds no crates today (verified: same graph with and without) -- the point is that
# it cannot start to without this gate noticing.
SHIPPED_FEATURES = [
    "--no-default-features",
    "--features",
    "runtime-bindings,kpathsea-build-from-source",
]

# The triples we actually publish (docs/release/RELEASING.md "What ships in a release";
# the build legs in .github/workflows/release.yml). KEEP IN SYNC when a target is added.
#
# Neither default nor `all` is right here. `cargo tree` defaults to the HOST target, so
# an ubuntu-only gate (CI `lint`, the release `notices` job) never sees the
# cfg(windows)/cfg(target_os="macos") crates that our Windows and macOS artifacts link
# -- fail-open. `--target all` overshoots the other way, dragging in wasm/haiku crates
# we never distribute, and a gate that cries wolf gets switched off. Measured here:
# 110 host / 143 all / 116 across these five.
RELEASE_TARGETS = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
TARGET_ARGS = [arg for t in RELEASE_TARGETS for arg in ("--target", t)]

# Crates whose native code has been audited by hand. Keep in sync with
# LICENSE_INVENTORY.md D.2. A crate here is NOT necessarily a licensing problem --
# it means someone checked what it compiles and recorded the answer.
AUDITED = {
    "libmarpa-sys": ("0.3.0", 
        "Vendors libmarpa 8.6.2 as a tarball. Mixed per-file licensing that the "
        "manifest's 'MIT OR Apache-2.0' does not express: marpa.c/marpa_ami.c/"
        "marpa_codes.c are MIT (c) Jeffrey Kegler; marpa_avl.c/marpa_tavl.c are "
        "LGPL-3.0+ (Ben Pfaff's libavl); marpa_obs.c is LGPL-2.1+ (GNU obstack). "
        "Attributed in THIRD-PARTY-NOTICES 3.3; static LGPL link -> relink note 3.5."
    ),
    "libmimalloc-sys": ("0.1.49", 
        "Vendors mimalloc C. The crate-root LICENSE.txt is the wrapper author's "
        "((c) Octavian Oncescu), so cargo-about emits an MIT text naming the WRONG "
        "holder; the C that ships is (c) Microsoft Corporation, Daan Leijen. "
        "Attributed in THIRD-PARTY-NOTICES 3.4."
    ),
    "kpathsea_sys": ("0.2.3", 
        "Links libkpathsea (LGPL-2.1+), fetched at build time at KPSE_REF -- no "
        "license file in the crate to scan. STATICALLY linked on every release leg: "
        "Linux + macOS via tools/build_static_kpathsea.sh, Windows via kpathsea_sys "
        "build_from_source. (The subprocess-kpsewhich backend is a RUNTIME fallback "
        "-- e.g. MiKTeX -- not a build-time one, so it does not avoid the static "
        "link.) Attributed in THIRD-PARTY-NOTICES 3.2; static LGPL -> relink 3.5."
    ),
    "libxml": ("0.3.16",
        "Binds libxml2 -- (c) Daniel Veillard, MIT -- which the crate's own "
        "'MIT OR Apache-2.0' does not name. Linked via pkg-config + rustc-link-lib "
        "rather than compiled here, and STATIC on every release leg "
        "(tools/build_static_libxml.sh, built --without-zlib/lzma/icu so nothing else "
        "comes with it). Permissive, so no relink duty. Attributed in NOTICES 3.1."
    ),
    "libxslt": ("0.1.5",
        "Binds libxslt/libexslt -- (c) Daniel Veillard, MIT -- same shape as libxml "
        "above, same static link, built --without-crypto so libgcrypt (LGPL-2.1) stays "
        "OUT of the closure. If that flag is ever dropped, an LGPL library enters the "
        "static link and 3.5's relink note must cover it. Attributed in NOTICES 3.1."
    ),
    "psm": ("0.1.31",
        "Compiles its own portable stack-manipulation asm shims, and ships a prebuilt "
        "wasm32.o (a target we do not release). Own copyright, covered by the crate's "
        "own MIT OR Apache-2.0 -- no third-party native code, so cargo-about's "
        "per-crate text is accurate. No separate notice owed."
    ),
    "stacker": ("0.1.24", 
        "Compiles its own small C shim (src/arch/windows.c -> GetCurrentFiber). Own "
        "copyright, covered by the crate's own MIT OR Apache-2.0. No notice owed."
    ),
}

# Signals that a crate brings native code into the binary. Deliberately broad: a false
# positive costs one line in AUDITED, a false negative costs an unattributed library.
VENDORED_ARCHIVE_SUFFIXES = (".tar.gz", ".tgz", ".tar.xz", ".tar.bz2", ".zip")
# .m/.mm: ObjC -- two of our five release triples are macOS. .o: a prebuilt object,
# as psm ships for wasm32.
NATIVE_SOURCE_SUFFIXES = (".c", ".cc", ".cpp", ".cxx", ".S", ".s", ".asm", ".m", ".mm", ".cu")
# A crate can ship a PREBUILT library and link it with no build.rs, no `links`,
# and no sources -- native code with none of the other four signals.
PREBUILT_LIB_SUFFIXES = (".a", ".lib", ".so", ".dylib", ".dll", ".o", ".obj")
# libfoo.so.1 / libfoo.so.0.0.0 -- versioned sonames that no suffix test catches.
PREBUILT_SO_RE = re.compile(r"\.so(\.\d+)+$")
# `rustc-link-lib` / pkg_config: the shape libxml/libxslt use -- link a system library
# with NO links key, NO cc::Build, NO vendored sources. They are the largest native
# surface in the binary (and static in release), and matched none of the other four
# signals, so the gate cheerfully reported "all N native-code crates audited" with the
# two biggest ones not in N. bindgen alone is NOT enough: it generates Rust bindings
# and implies nothing about linking.
BUILD_RS_NATIVE_RE = re.compile(
    r"cc::Build|cmake::|Build::new\(\)|\.compile\(|rustc-link-lib|pkg_config|pkg-config"
)


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True, check=True).stdout


def shipped_packages():
    """(name, version) actually linked into the distributed binary, over RELEASE_TARGETS.

    The explicit target list is load-bearing -- see RELEASE_TARGETS above.

    `-e normal` on purpose: build-dependencies (cc, bindgen, clang-sys) run at build
    time and are not linked in, so they owe no distribution notice. The crate whose
    build.rs compiles C is itself a normal dep -- that is what we want to catch.
    """
    out = run(
        ["cargo", "tree", *MANIFEST_ARGS, *SHIPPED_FEATURES,
         "-e", "normal", "--prefix", "none", *TARGET_ARGS]
    )
    pkgs = set()
    for line in out.splitlines():
        line = line.replace(" (*)", "").strip()
        if not line:
            continue
        parts = line.split()
        # `name vX.Y.Z [extra...]` -- the version is always field 1 and always starts
        # with 'v'. Anything else is a header/blank/continuation we do not want, but a
        # line we FAIL to parse must not vanish silently: that is how a shipped crate
        # becomes an unaudited crate. main() cross-checks against cargo metadata.
        if len(parts) >= 2 and re.fullmatch(r"v\d+\.\d+\.\d+.*", parts[1]):
            pkgs.add((parts[0], parts[1][1:]))
    if not pkgs:
        raise SystemExit("error: parsed zero crates from `cargo tree` — refusing to pass blind")
    return pkgs


def package_dirs():
    """(name, version) -> (package dir, links), plus the set of OUR workspace package ids.

    Resolving by manifest_path rather than guessing a registry directory name is what
    makes git-sourced crates like libmarpa-sys (a checkout hash path, not
    `<name>-<version>/`) visible at all -- a find(1)-by-dirname sweep silently misses
    them, which is its own way to lose a library.

    Workspace membership comes from cargo's own `workspace_members` ids, NOT from
    matching path prefixes. Prefix matching is a false-negative machine here: the
    workspace root's *parent* is `~/git`, so `startswith` would silently skip every
    crate checked out beside us -- including `~/git/marpa` when a developer enables the
    local `[patch]` that this repo's own Cargo.toml documents. That skipped libmarpa
    (the LGPL library) while the audit exited 0, and then reported it as a "stale" entry
    inviting someone to delete its attribution. Exactly the bug this script exists to
    catch, so it does not get to make it.
    """
    meta = json.loads(
        run(["cargo", "metadata", "--format-version", "1", *MANIFEST_ARGS, *SHIPPED_FEATURES])
    )
    dirs = {}
    for pkg in meta["packages"]:
        build_script = next(
            (Path(t["src_path"]) for t in pkg.get("targets", []) if "custom-build" in t.get("kind", [])),
            None,
        )
        dirs[(pkg["name"], pkg["version"])] = (
            Path(pkg["manifest_path"]).parent,
            pkg.get("links"),
            pkg["id"],
            build_script,
        )
    return dirs, set(meta["workspace_members"])


def native_evidence(pkg_dir, links, build_script=None):
    """Why we think this crate brings in native code. Empty list = it doesn't."""
    why = []
    if links:
        why.append(f"links = \"{links}\"")

    # From cargo metadata's custom-build target, NOT a hardcoded `build.rs`: Cargo
    # honours `build = "build/main.rs"`, and rustversion in this very tree uses
    # build/build.rs -- so the hardcoded path was already reading nothing there. A
    # crate that fetches its sources at build time (the kpathsea_sys shape) with a
    # custom build-script path and no `links` key would evade every other signal.
    build_rs = build_script if build_script is not None else pkg_dir / "build.rs"
    if build_rs.is_file():
        try:
            if BUILD_RS_NATIVE_RE.search(build_rs.read_text(errors="ignore")):
                why.append(f"{build_rs.name} builds/links native code")
        except OSError:
            pass

    archives, sources, prebuilt = [], 0, []
    for p in pkg_dir.rglob("*"):
        if not p.is_file():
            continue
        # endswith, not `.suffix`: a versioned shared object (libfoo.so.1) has
        # suffix ".1", so a suffix test silently misses ordinary library packaging.
        if p.name.endswith(VENDORED_ARCHIVE_SUFFIXES):
            archives.append(p.name)
        elif p.name.endswith(NATIVE_SOURCE_SUFFIXES):
            sources += 1
        elif p.name.endswith(PREBUILT_LIB_SUFFIXES) or PREBUILT_SO_RE.search(p.name):
            prebuilt.append(p.name)
    if archives:
        why.append(f"vendored archive(s): {', '.join(sorted(archives)[:3])}")
    if sources:
        why.append(f"{sources} native source file(s)")
    if prebuilt:
        why.append(f"prebuilt librar(ies): {', '.join(sorted(prebuilt)[:3])}")
    return why


# --- embedded resources (LICENSE_INVENTORY.md §B) ----------------------------
#
# The same failure mode, one level up: `latexml_core/build.rs` walks resources/RelaxNG/
# with NO filtering and include_str!s every file, so anything dropped in there ships
# verbatim inside the binary. The inventory called all 108 RelaxNG files "PD (NIST)"
# until resources/RelaxNG/svg/ turned out to be W3C + Mozilla. Markers, not counts:
# a count gate fails on every legitimate new schema file (noise), while what actually
# matters is a NEW third-party copyright holder appearing among the embedded assets.
# Anchored to REPO_ROOT, never the cwd -- see the note there.
RESOURCE_ROOTS = [REPO_ROOT / "resources"]
# Broad on purpose. The previous six fixed phrases missed "Copyright (c) 2024 Foo Inc"
# (a real notice with no license keyword) and, live in this repo,
# resources/dumps/texlive.YYYY.version -- which says "GNU Lesser GPL", not the
# "GNU Lesser General" the regex demanded. Match any copyright/licence assertion and let
# AUDITED_RESOURCES carry the verdicts; noise here costs one line, a miss costs a library.
THIRD_PARTY_MARKER_RE = re.compile(
    r"copyright\s*(?:\(c\)|\u00a9|[0-9]{4}|holder)"
    r"|\(c\)\s*[0-9]{4}"
    r"|all rights reserved"
    r"|licen[sc]ed under|license[sd]? under"
    r"|GNU (?:General|Lesser|Library)"
    r"|(?:Apache|MIT|BSD|MPL|ISC|Zlib|LGPL|GPL)[- ](?:License|[0-9])"
    r"|Mozilla Foundation|World Wide Web Consortium|W3C \(MIT, INRIA, Keio\)",
    re.IGNORECASE,
)

# There is deliberately NO "clear it if it carries the NIST public-domain notice" escape
# hatch. 43 embedded files do carry that notice, but none of them trips the marker regex
# above, so such a rule would be dead code -- dead code that could only ever ACT by
# clearing a genuinely third-party file that happened to quote the NIST phrase. If an
# upstream Perl-LaTeXML file ever grows a copyright header, the right outcome is that
# this gate flags it and a human looks, which is what happens with no rule at all.

# Embedded subtrees whose third-party origin is recorded. Prefixes are used ONLY where
# the whole subtree has one known upstream, never as a blanket for a mixed directory.
AUDITED_RESOURCES = {
    "resources/RelaxNG/svg/": (
        "W3C SVG 1.1 RELAX NG schema (c) 2001,2002 W3C, modifications (c) 2007 "
        "Mozilla Foundation. Permissive, but the notice must accompany all copies. "
        "Attributed in THIRD-PARTY-NOTICES 2.2; LICENSE_INVENTORY.md B."
    ),
    "resources/dumps/": (
        "TeX-Live-derived format dumps + their texlive.YYYY.version stamps, embedded by "
        "latexml_engine/build.rs. Attributed in THIRD-PARTY-NOTICES 1 (LaTeX kernel "
        "LPPL-1.3c; plain TeX, Knuth). The stamp also carries kpathsea's own banner "
        "((c) Karl Berry & Olaf Weber, LGPLv2.1+) -- kpathsea is attributed in 3.2. "
        "Gitignored build artifacts, so this prefix is usually empty outside a release."
    ),
}


def audit_resources(verbose):
    """Flag embedded resources carrying an unaudited third-party copyright."""
    hits = []
    for root in RESOURCE_ROOTS:
        if not root.is_dir():
            print(f"ERROR: resource root missing: {root}")
            return 1
        for p in root.rglob("*"):
            if not p.is_file():
                continue
            try:
                text = p.read_text(errors="ignore")
            except OSError:
                continue
            if not THIRD_PARTY_MARKER_RE.search(text):
                continue
            hits.append(p.relative_to(REPO_ROOT).as_posix())

    unaudited = [h for h in hits if not any(h.startswith(k) for k in AUDITED_RESOURCES)]
    if verbose:
        print(f"Embedded resources carrying a third-party notice: {len(hits)} file(s)")
        for prefix in AUDITED_RESOURCES:
            n = sum(1 for h in hits if h.startswith(prefix))
            print(f"  ok  {prefix} ({n} file(s))")
        print()

    if unaudited:
        print("ERROR: embedded resource(s) carry an UNAUDITED third-party notice:")
        print()
        for h in sorted(unaudited)[:20]:
            print(f"  {h}")
        if len(unaudited) > 20:
            print(f"  ... and {len(unaudited) - 20} more")
        print()
        print("latexml_core/build.rs embeds resources/ verbatim, so these SHIP inside")
        print("the binary. Identify the holder + terms, attribute in")
        print("THIRD-PARTY-NOTICES section 2, add a row to LICENSE_INVENTORY.md B, and")
        print("record the path in AUDITED_RESOURCES in this file.")
        print()
    return 1 if unaudited else 0


def main():
    verbose = "--verbose" in sys.argv
    dirs, workspace_ids = package_dirs()
    shipped = shipped_packages()

    found, unresolved = {}, []
    for name, version in sorted(shipped):
        entry = dirs.get((name, version))
        if entry is None:
            # A shipped crate cargo metadata could not place. Never shrug this off:
            # an unplaceable crate is an unaudited crate.
            unresolved.append(f"{name} v{version}")
            continue
        pkg_dir, links, pkg_id, build_script = entry
        # Skip only OUR OWN workspace crates (CC0, audited by definition), identified
        # by cargo's package ids -- not by path shape. See package_dirs().
        if pkg_id in workspace_ids:
            continue
        why = native_evidence(pkg_dir, links, build_script)
        if why:
            # Keyed by (name, version), matching AUDITED. Keying by name alone silently
            # drops all but the last version of a crate shipped at two versions -- live
            # here today (getrandom x3, hashbrown x2). Two versions of one native crate
            # can vendor two different libraries.
            found[(name, version)] = why

    if unresolved:
        print("ERROR: shipped crate(s) that cargo metadata could not locate:")
        for u in unresolved:
            print(f"  {u}")
        print("\nCannot audit what cannot be found; failing rather than passing blind.")
        return 1

    # Match on (name, version), not name. AUDITED's text pins what a SPECIFIC version
    # vendors ("libmarpa 8.6.2"); a bump can change the vendored library wholesale, so
    # inheriting the old verdict by name is how a stale "ok" outlives the thing it
    # described. A bump is cheap to clear -- re-check and edit one line.
    unaudited = {k: v for k, v in found.items() if k[0] not in AUDITED}
    rebumped = {
        k: AUDITED[k[0]][0]
        for k in found
        if k[0] in AUDITED and k[1] != AUDITED[k[0]][0]
    }
    stale = [n for n in AUDITED if n not in {k[0] for k in found}]

    if rebumped:
        print("ERROR: audited crate(s) changed version — the recorded verdict may no longer hold:")
        print()
        for (name, got), was in sorted(rebumped.items()):
            print(f"  {name}: audited at v{was}, tree has v{got}")
            print(f"      recorded: {AUDITED[name][1][:100]}...")
        print()
        print("A version bump can swap the vendored library (and its license) entirely.")
        print("Re-check what this version compiles, then update AUDITED + LICENSE_INVENTORY §D.2.")
        return 1

    if verbose or unaudited:
        print("Shipped crates that compile or link native code:\n")
        for (name, version), why in sorted(found.items()):
            mark = "NEW " if (name, version) in unaudited else "ok  "
            print(f"  {mark}{name} v{version}")
            for w in why:
                print(f"        - {w}")
        print()

    if stale:
        # Deliberately NOT phrased as "prune these". An earlier version of this script
        # skipped crates by path prefix, which made libmarpa-sys vanish from the tree
        # and surface here -- so the note was cheerfully inviting someone to delete the
        # attribution of a library that was still very much being linked in. Removing
        # an entry is only safe once you have confirmed the crate is genuinely gone.
        print("NOTE: audited crate(s) not seen in the shipped tree this run:")
        for n in stale:
            print(f"  - {n}")
        print()
        print("  Do NOT prune them from AUDITED on this basis alone. First confirm the")
        print("  crate is really gone (`cargo tree -e normal | grep <name>`) rather than")
        print("  merely unresolved -- a crate that is still linked but went unseen is a")
        print("  silently unattributed library, which is the failure this script exists")
        print("  to prevent. If it is genuinely gone, drop it here and from")
        print("  LICENSE_INVENTORY.md §D.2 together.")
        print()

    if unaudited:
        print("ERROR: unaudited crate(s) bring native code into the shipped binary:")
        print()
        for name, version in sorted(unaudited):
            print(f"  {name} v{version}")
        print()
        print("cargo-deny/cargo-about CANNOT catch this: they read the crate manifest,")
        print("which describes the Rust wrapper, not the C it compiles. Someone must")
        print("look at what the build script actually compiles and its real copyright.")
        print()
        print("For each crate above:")
        print("  1. Read its build.rs -- which sources does it compile?")
        print("  2. Find those sources' real license + copyright holder (check inside")
        print("     any vendored tarball; check per-FILE headers, not just COPYING --")
        print("     libmarpa is MIT overall but has LGPL files).")
        print("  3. If it differs from the crate's `license =`, attribute it by name in")
        print("     THIRD-PARTY-NOTICES section 3 and add a row to LICENSE_INVENTORY.md D.2.")
        print("  4. If it is copyleft AND statically linked, also ship the license text")
        print("     (licenses/ + tools/gen_notices.sh) and check the relink note 3.5.")
        print("  5. Record the verdict in AUDITED in this file.")
        return 1

    print(f"OK: all {len(found)} native-code crates in the shipped tree are audited.")
    return audit_resources(verbose)


if __name__ == "__main__":
    sys.exit(main())
