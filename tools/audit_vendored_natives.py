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

# The feature set the distributed binary ships (CLAUDE.md, RELEASE_CRITERIA 4).
# Audit exactly what we ship -- no more, no less.
SHIPPED_FEATURES = ["--no-default-features", "--features", "runtime-bindings"]

# Crates whose native code has been audited by hand. Keep in sync with
# LICENSE_INVENTORY.md D.2. A crate here is NOT necessarily a licensing problem --
# it means someone checked what it compiles and recorded the answer.
AUDITED = {
    "libmarpa-sys": (
        "Vendors libmarpa 8.6.2 as a tarball. Mixed per-file licensing that the "
        "manifest's 'MIT OR Apache-2.0' does not express: marpa.c/marpa_ami.c/"
        "marpa_codes.c are MIT (c) Jeffrey Kegler; marpa_avl.c/marpa_tavl.c are "
        "LGPL-3.0+ (Ben Pfaff's libavl); marpa_obs.c is LGPL-2.1+ (GNU obstack). "
        "Attributed in THIRD-PARTY-NOTICES 3.3; static LGPL link -> relink note 3.5."
    ),
    "libmimalloc-sys": (
        "Vendors mimalloc C. The crate-root LICENSE.txt is the wrapper author's "
        "((c) Octavian Oncescu), so cargo-about emits an MIT text naming the WRONG "
        "holder; the C that ships is (c) Microsoft Corporation, Daan Leijen. "
        "Attributed in THIRD-PARTY-NOTICES 3.4."
    ),
    "kpathsea_sys": (
        "Links libkpathsea (LGPL-2.1+), fetched at build time at KPSE_REF -- no "
        "license file in the crate to scan. Statically linked on Linux and Windows; "
        "subprocess kpsewhich elsewhere. Attributed in THIRD-PARTY-NOTICES 3.2; "
        "static LGPL link -> relink note 3.5."
    ),
    "psm": (
        "Compiles its own portable stack-manipulation asm shims. Own copyright, "
        "covered by the crate's own MIT OR Apache-2.0 -- no third-party native code, "
        "so cargo-about's per-crate text is accurate. No separate notice owed."
    ),
    "stacker": (
        "Compiles its own small C shim (src/arch/windows.c -> GetCurrentFiber). Own "
        "copyright, covered by the crate's own MIT OR Apache-2.0. No notice owed."
    ),
}

# Signals that a crate brings native code into the binary. Deliberately broad: a false
# positive costs one line in AUDITED, a false negative costs an unattributed library.
VENDORED_ARCHIVE_SUFFIXES = (".tar.gz", ".tgz", ".tar.xz", ".tar.bz2", ".zip")
NATIVE_SOURCE_SUFFIXES = (".c", ".cc", ".cpp", ".cxx", ".S", ".s", ".asm")
BUILD_RS_NATIVE_RE = re.compile(r"cc::Build|cmake::|Build::new\(\)|\.compile\(")


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True, check=True).stdout


def shipped_packages():
    """(name, version) actually linked into the distributed binary.

    `-e normal` on purpose: build-dependencies (cc, bindgen, clang-sys) run at build
    time and are not linked in, so they owe no distribution notice. The crate whose
    build.rs compiles C is itself a normal dep -- that is what we want to catch.
    """
    out = run(["cargo", "tree", *SHIPPED_FEATURES, "-e", "normal", "--prefix", "none"])
    pkgs = set()
    for line in out.splitlines():
        line = line.replace(" (*)", "").strip()
        if not line:
            continue
        parts = line.split()
        if len(parts) >= 2 and parts[1].startswith("v"):
            pkgs.add((parts[0], parts[1][1:]))
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
    meta = json.loads(run(["cargo", "metadata", "--format-version", "1", *SHIPPED_FEATURES]))
    dirs = {}
    for pkg in meta["packages"]:
        dirs[(pkg["name"], pkg["version"])] = (
            Path(pkg["manifest_path"]).parent,
            pkg.get("links"),
            pkg["id"],
        )
    return dirs, set(meta["workspace_members"])


def native_evidence(pkg_dir, links):
    """Why we think this crate brings in native code. Empty list = it doesn't."""
    why = []
    if links:
        why.append(f"links = \"{links}\"")

    build_rs = pkg_dir / "build.rs"
    if build_rs.is_file():
        try:
            if BUILD_RS_NATIVE_RE.search(build_rs.read_text(errors="ignore")):
                why.append("build.rs compiles native code (cc/cmake)")
        except OSError:
            pass

    archives, sources = [], 0
    for p in pkg_dir.rglob("*"):
        if not p.is_file():
            continue
        if p.name.endswith(VENDORED_ARCHIVE_SUFFIXES):
            archives.append(p.name)
        elif p.suffix in NATIVE_SOURCE_SUFFIXES:
            sources += 1
    if archives:
        why.append(f"vendored archive(s): {', '.join(sorted(archives)[:3])}")
    if sources:
        why.append(f"{sources} native source file(s)")
    return why


# --- embedded resources (LICENSE_INVENTORY.md §B) ----------------------------
#
# The same failure mode, one level up: `latexml_core/build.rs` walks resources/RelaxNG/
# with NO filtering and include_str!s every file, so anything dropped in there ships
# verbatim inside the binary. The inventory called all 108 RelaxNG files "PD (NIST)"
# until resources/RelaxNG/svg/ turned out to be W3C + Mozilla. Markers, not counts:
# a count gate fails on every legitimate new schema file (noise), while what actually
# matters is a NEW third-party copyright holder appearing among the embedded assets.
RESOURCE_ROOTS = ["resources"]
THIRD_PARTY_MARKER_RE = re.compile(
    r"World Wide Web Consortium|W3C \(MIT, INRIA, Keio\)|Mozilla Foundation|"
    r"Apache License|GNU General Public|GNU Lesser General",
    re.IGNORECASE,
)
# Embedded paths known to carry a third-party notice, with where it is attributed.
AUDITED_RESOURCES = {
    "resources/RelaxNG/svg/": (
        "W3C SVG 1.1 RELAX NG schema (c) 2001,2002 W3C, modifications (c) 2007 "
        "Mozilla Foundation. Permissive, but the notice must accompany all copies. "
        "Attributed in THIRD-PARTY-NOTICES 2.2; LICENSE_INVENTORY.md B."
    ),
}


def audit_resources(verbose):
    """Flag embedded resources carrying an unaudited third-party copyright."""
    hits = []
    for root in RESOURCE_ROOTS:
        for p in Path(root).rglob("*"):
            if not p.is_file():
                continue
            try:
                text = p.read_text(errors="ignore")
            except OSError:
                continue
            if THIRD_PARTY_MARKER_RE.search(text):
                hits.append(p.as_posix())

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
        pkg_dir, links, pkg_id = entry
        # Skip only OUR OWN workspace crates (CC0, audited by definition), identified
        # by cargo's package ids -- not by path shape. See package_dirs().
        if pkg_id in workspace_ids:
            continue
        why = native_evidence(pkg_dir, links)
        if why:
            found[name] = (version, why)

    if unresolved:
        print("ERROR: shipped crate(s) that cargo metadata could not locate:")
        for u in unresolved:
            print(f"  {u}")
        print("\nCannot audit what cannot be found; failing rather than passing blind.")
        return 1

    unaudited = {n: v for n, v in found.items() if n not in AUDITED}
    stale = [n for n in AUDITED if n not in found]

    if verbose or unaudited:
        print("Shipped crates that compile or link native code:\n")
        for name, (version, why) in sorted(found.items()):
            mark = "NEW " if name in unaudited else "ok  "
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
        for name, (version, _) in sorted(unaudited.items()):
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
