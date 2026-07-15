# License Inventory — the redistributable-binary audit

The [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §4 deliverable: a *living*
inventory of everything the shipped `latexml_oxide` binary carries or touches,
with each item's origin and license, so the public-domain claim is scoped
correctly and CI can check the release artifact against it.

**Not a dated snapshot** — keep this current when a dependency, embedded asset,
or linked/subprocess tool changes. (Re-verify commands are inline per section.)

## Claim scope (the headline)

latexml-oxide's **own source code and original resources are CC0-1.0**
(public-domain dedication; `LICENSE`, all 8 workspace-crate manifests). The CC0
claim does **NOT** extend to third-party material the binary embeds, links, or
derives from — each retains its upstream license, enumerated below. The ones
that matter for a distribution notice: the **build-time-embedded TeX-Live
dumps** (§C), the **statically-linked libxml2/libxslt** (§D; static since
0.7.1), and the two **static LGPL links** — `libkpathsea` (LGPL-2.1, universal
from 0.7.4) and parts of **`libmarpa`** (LGPL-3.0 / LGPL-2.1, in every build
since the math parser landed). A static LGPL link triggers the relink
obligation (LGPL-2.1 §6 / LGPL-3.0 §4), unlike the MIT static libs and the
dynamic LGPL ones; see the §D note.

**The trap that hid `libmarpa` (§D.2).** `cargo-about` attributes each *crate*
as its **manifest** declares it, and harvests license texts from the crate's
own files. A `-sys` crate that compiles **vendored C** therefore reports the
*Rust wrapper's* license, silently masking the native library's real copyright
holder and terms. Three crates in the shipped tree have this shape, and
**auditing by cargo-deny/cargo-about alone cannot catch any of them** — §D.2
enumerates them by hand. When adding a `-sys` dependency, check what its
`build.rs` actually compiles, not just its `license =` field.

## A. Rust dependencies — GATED

Vetted allow-list in [`deny.toml`](../../deny.toml); the CI `lint` job runs
`cargo-deny` with `--all-features`, so an unlisted license fails the build.
Allowed set: `CC0-1.0`, `MIT`, `Apache-2.0` (+ LLVM-exception), `BSD-2/3-Clause`,
`ISC`, `Zlib`, `Unicode-3.0`, `MPL-2.0` (weak per-file copyleft — safe for a CC0
binary).

- **Status:** `cargo deny --all-features check licenses` → **licenses ok**.
- **Distributed artifact** (`--no-default-features --features runtime-bindings`)
  → **licenses ok** with no warnings. Shipped-crate license breakdown:
  MIT (127), CC0-1.0 (9, our workspace crates), BSD-3-Clause (1, unidecode),
  MPL-2.0 (1), Unicode-3.0 (1), Zlib (1).
- **Attribution:** the per-crate license *texts* for the shipped binary are
  auto-generated into `THIRD-PARTY-NOTICES` §5 by `cargo about` — config
  [`about.toml`](../../about.toml) + template [`about.hbs`](../../about.hbs), assembled
  with the hand-authored §1-4 and the §6 copyleft texts (`licenses/`) by
  [`tools/gen_notices.sh`](../../tools/gen_notices.sh).
  Generated from the (gitignored) lockfile at release time, not committed.
- **Scope limit — this gate does NOT cover vendored C.** cargo-deny and
  cargo-about both reason over *crate manifests*; a `-sys` crate that compiles
  third-party C reports only its Rust wrapper's license. Those native libraries
  are hand-audited in **§D.2** and attributed in `THIRD-PARTY-NOTICES` §3 —
  a "licenses ok" result here says nothing about them.
- **~~Known warning~~ RESOLVED 2026-07-13:** `pericortex` (our own
  `dginev/cortex-peripherals`, git source) previously had no `license` field. It
  is pulled in **only** under the `cortex` feature — absent from the shipped
  binary. *F1 closed:* pericortex was relicensed MIT → `CC0-1.0` upstream
  (`license = "CC0-1.0"` in its `Cargo.toml` + CC0 `LICENSE` text), clearing the
  cargo-deny/cargo-about warning; a fresh build resolves the branch HEAD that
  carries it.

Re-verify: `cargo deny --all-features check licenses`

## B. Embedded resources (`resources/`, `include_str!`/`include_bytes!` at build)

| Asset group | Count | Origin | License |
|---|---|---|---|
| `CSS/` | 14 | Perl LaTeXML + our schema-docs theme | PD (NIST) / CC0 |
| `XSLT/` | 20 | Perl LaTeXML | PD (NIST) ≈ CC0 |
| `RelaxNG/` | 108 | Perl LaTeXML schema | PD (NIST) ≈ CC0 |
| `DTD/` | 2 | Perl LaTeXML | PD (NIST) ≈ CC0 |
| `Profiles/` | 9 | Perl LaTeXML | PD (NIST) ≈ CC0 |
| `javascript/LaTeXML-maybeMathjax.js` | 1 | Perl LaTeXML | PD (NIST) ≈ CC0 |
| `javascript/relaxng-schema-rustdoc-theme.js` | 1 | **ours** | CC0 |

Perl LaTeXML is **public domain** (NIST, 17 U.S.C. §105; upstream `LICENSE`
explicitly equates it to CC0) — so these are CC0-compatible with no notice
burden. `LaTeXML-maybeMathjax.js` only *conditionally loads* MathJax from a CDN;
it does **not** bundle MathJax, so no MathJax (Apache-2.0) redistribution
occurs.

## C. Compiled-in dumps — the sharp edge (TeX-Live-derived)

`resources/dumps/{plain,latex}.YYYY.dump.txt` are a **serialized engine-state
snapshot** produced by running the latexml-oxide dumper over TeX Live's format
sources inside a pinned per-TL-year container at release time
(`tools/make_formats.sh`, `.github/workflows/release-dumps.yml`). The embedded
state is *derived from*:

- **plain TeX** (`plain.tex`/`tex.ini`) — Knuth's TeX license (permissive;
  redistribution allowed, modified copies must be renamed).
- **LaTeX kernel** (`latex.ltx`, expl3) — **LPPL 1.3c** (LaTeX Project Public
  License; redistribution and derived-format distribution allowed).

Key facts:
- **NOT committed to the git repo** (`resources/dumps/*.dump.txt` are
  gitignored build artifacts — verified `git check-ignore`). The repo's CC0
  claim therefore ships no TeX-Live-derived content.
- **Embedded into the release binary** at build time (gzip, DEP-12). A
  compiled format derived from LPPL/Knuth sources is a well-trodden TeX-world
  artifact (analogous to a `.fmt` file), and both licenses permit it.

### Position (owner-approved 2026-07-09)

> The embedded dumps are derived from TeX Live's LaTeX kernel (LPPL 1.3c) and
> plain TeX (Knuth). The CC0 dedication covers latexml-oxide's source, **not**
> this build-time-embedded, TeX-Live-derived content, which retains its
> upstream license. The release ships a `THIRD-PARTY-NOTICES` file attributing
> the LaTeX kernel (LPPL 1.3c) and plain TeX (Knuth), and the README's license
> statement scopes the public-domain claim to "the latexml-oxide source and
> original resources," excluding the embedded dumps.

This mirrors how the binary already relies on system libxml2/libxslt (§D)
without claiming CC0 over them. Landed: [`THIRD-PARTY-NOTICES`](../../THIRD-PARTY-NOTICES)
§1 + the README License section (F2/F3 below).

## D. Linked native (C) libraries

### D.1 System libraries

`ldd` on the built binary (host TeX Live ecosystem is out of scope per
CLAUDE.md; these are standard OS libraries):

| Library | License | Linkage (release / dev) |
|---|---|---|
| libxml2 | MIT | **static** (0.7.1, PIC source-built) / dynamic in `cargo build` |
| libxslt | MIT | **static** (0.7.1) / dynamic in `cargo build` |
| libexslt | MIT | **static** (0.7.1) / dynamic in `cargo build` |
| libgcrypt | LGPL-2.1 | dynamic (transitive via libxslt) — **keep dynamic** |
| libgpg-error | LGPL-2.1 | dynamic (transitive via libgcrypt) — keep dynamic |
| zlib | Zlib | dynamic |
| **libkpathsea** | **LGPL-2.1** | **static** where linked — Linux (`tools/build_static_kpathsea.sh`), Windows 0.7.4 (`kpathsea_sys` `build_from_source`); subprocess `kpsewhich` otherwise (macOS/MacTeX ships no lib; MiKTeX fallback). See the §6 note below. |

The release binary statically links libxml2/libxslt/libexslt (§3, shipped
0.7.1); MIT requires their copyright notice when statically linked → covered in
`THIRD-PARTY-NOTICES` §3. LGPL-2.1 (libgcrypt/libgpg-error) is satisfied by
dynamic linking; the build deliberately keeps `-lgcrypt`/`-lz` dynamic. The
default dev build (`cargo build`) links all of these dynamically against the
host.

**libkpathsea is a STATIC LGPL link** — one of two, with libmarpa (§D.2); the
LGPL libs in the table above are kept dynamic precisely to avoid this. LGPL-2.1
§6 permits static linking but obliges the distributor to let a user relink
against a modified libkpathsea. Here both sides are open, so relinking is
possible: latexml-oxide's own source is CC0, and the kpathsea source is public
and **pinned** — `kpathsea_sys` `build_from_source` fetches it at a recorded
commit (`KPSE_REF`), and the Linux/macOS `build_static_kpathsea.sh` pins the
same. The LGPL-2.1 **text** + the kpathsea copyright + the pinned-source pointer
are now shipped (`THIRD-PARTY-NOTICES` §3.2/§3.5/§6) — see **F5**, which stays
open only on the owner's posture call (§D.3). (The obligation is not new to
0.7.4 — the Linux legs have static-linked kpathsea since
`build_static_kpathsea.sh`; 0.7.4 makes it universal by adding Windows.)

Re-verify: `ldd target/<profile>/latexml_oxide` (dev) or the release-artifact
no-dynamic-clib CI assertion (release).

### D.2 Vendored native libraries — the cargo-about blind spot

Statically compiled into the binary by a `-sys` crate's `build.rs`. In each row
the **crate manifest's license is not the license of the code that ships**, so
none of these are catchable by cargo-deny (§A) — they are audited by hand here
and attributed in `THIRD-PARTY-NOTICES` §3.2-§3.5.

| Native lib | Carrier crate | Crate declares | What actually links in | Notice |
|---|---|---|---|---|
| **libmarpa 8.6.2** | `libmarpa-sys` (git, `dginev/marpa`) | `MIT OR Apache-2.0` | `marpa.c`, `marpa_ami.c`, `marpa_codes.c` → **MIT**, © 2018 Jeffrey Kegler; `marpa_avl.c`, `marpa_tavl.c` (from Ben Pfaff's libavl) → **LGPL-3.0-or-later**, © FSF; `marpa_obs.c` (from GNU obstack) → **LGPL-2.1-or-later**, © FSF | §3.3 |
| **mimalloc** | `libmimalloc-sys` | `MIT` + a crate-root `LICENSE.txt` © 2019 **Octavian Oncescu** (the wrapper author) | `c_src/mimalloc/**` → **MIT**, © 2018-2025 **Microsoft Corporation, Daan Leijen** — a different holder than the harvested text | §3.4 |
| **libkpathsea** | `kpathsea_sys` | `MIT OR Apache-2.0`, **no LICENSE file in the crate** | kpathsea C, fetched at `KPSE_REF` → **LGPL-2.1-or-later**, © Karl Berry, Olaf Weber et al. | §3.2 |

Why each evades the automated gate:
- **libmarpa** vendors its sources as a **tarball** (`libmarpa-8.6.2.tar.gz`), so
  the `COPYING` / `COPYING.LESSER` inside it are invisible to cargo-about's file
  scan — and its per-file license split is not expressible in a manifest field.
- **mimalloc** *does* ship a crate-root `LICENSE.txt`, which is exactly what makes
  it dangerous: cargo-about harvests it and emits a **plausible-looking MIT text
  naming the wrong copyright holder**.
- **kpathsea** fetches source at build time; nothing to scan.

Checked and **not** in this class: `stacker` and `psm` compile only their own
small C/asm shims (own copyright, covered by their manifests); `libxml2`/`libxslt`
are source-built by the release workflow, not by a `-sys` crate, and are
attributed in §3.1.

Re-verify (lists every shipped crate that compiles native code — inspect any new
hit by hand, and note git-sourced crates like `libmarpa-sys` need a checkout-path
lookup rather than a registry one):

```bash
cargo tree --no-default-features --features runtime-bindings -e normal,build --prefix none \
  | sed 's/ (\*)//' | sort -u | grep -iE '\-sys |^cc '
```

### D.3 The LGPL static-link (relink) obligation

`libkpathsea` (D.1) and part of `libmarpa` (D.2) are **statically linked LGPL**.
LGPL-2.1 §6 / LGPL-3.0 §4 permit static linking provided a recipient can relink
against a modified version of the library. Every input is public and pinned:
latexml-oxide's source is CC0; libmarpa 8.6.2 is vendored verbatim in
`libmarpa-sys`; kpathsea is pinned at `KPSE_REF`. `THIRD-PARTY-NOTICES` §3.5
states this and points at the sources; §6 now ships the verbatim LGPL-2.1,
LGPL-3.0 and GPL-3.0 texts (LGPL-3.0 is additional permissions on top of
GPL-3.0, so it is **not** self-contained — both texts are required).

**Owner to confirm** this source-availability posture satisfies the relink
obligation for the static links, vs. the heavier alternatives (shipping
prelinkable object files, or a written offer). See F5.

## E. Subprocess-only tools (never linked → no license propagation)

All graphics helpers are invoked via `std::process::Command` in
`latexml_post/src/graphics.rs`, never linked — so their (A)GPL licenses do not
propagate to our binary. The host TeX Live ecosystem provides them.

| Tool | Package | License | Call sites |
|---|---|---|---|
| `gs` | Ghostscript | AGPL-3.0 (or commercial) | graphics.rs:1554 |
| `mutool` | MuPDF | AGPL-3.0 | graphics.rs:840, 1366 |
| `pdftocairo` | Poppler | GPL-2.0 | graphics.rs:880, 1418, 1492 |
| `convert` | ImageMagick | Apache-2.0-style | graphics.rs:688, 1723 |

Re-verify: `grep -rn 'Command::new' latexml_post/src/graphics.rs`

## F. Open items (the remaining audit work)

- **F1** *(closed 2026-07-13)* — `pericortex` relicensed MIT → `CC0-1.0`
  upstream (`license` field + CC0 `LICENSE`); clears the cargo-deny/cargo-about
  warning. Was cortex-only, never distribution-blocking.
- **F2** *(landed 2026-07-09)* — **`THIRD-PARTY-NOTICES`** (hand-authored §1-4 +
  cargo-about §5): LaTeX kernel (LPPL 1.3c) + plain TeX (Knuth) for the embedded
  dumps (§C); Perl-LaTeXML assets; the statically-linked libxml2/libxslt/libexslt
  (MIT, since 0.7.1); Rust crates. Assembled by `tools/gen_notices.sh`.
- **F3** *(landed 2026-07-09)* — **README License section** scoping the CC0 claim
  to our source + original resources (§C wording).
- **F4** *(open)* — **CI release-time gate** (RELEASE_CRITERIA §7): run
  `tools/gen_notices.sh` in the release workflow, bundle the assembled notices in
  the artifact, and verify the artifact's embedded resources match §B/§C.
  Complements the existing cargo-deny (§A) gate. Tracked with #51.
- **F5** *(attribution landed 2026-07-14; posture call still OPEN)* —
  **libkpathsea (LGPL-2.1 static link).** kpathsea is statically linked on Linux
  (`build_static_kpathsea.sh`) and, as of 0.7.4, Windows (`build_from_source`).
  `THIRD-PARTY-NOTICES` now carries the kpathsea copyright (§3.2), the relink
  note + pinned-source pointer (§3.5), and the verbatim LGPL-2.1 text (§6, via
  `tools/gen_notices.sh` + `licenses/`). **Still owner-to-confirm:** that
  source-availability satisfies the §6 relink obligation, vs. shipping
  prelinkable object files or a written offer. Same call covers libmarpa (F6).
- **F6** *(attribution landed 2026-07-14)* — **vendored native libs were absent
  from every notice: libmarpa + mimalloc.** Found by auditing what `-sys`
  `build.rs` files actually compile, rather than trusting manifest `license =`
  fields (§D.2). libmarpa was attributed **nowhere** despite statically linking
  MIT (Kegler) **and LGPL-3.0/LGPL-2.1** (libavl, GNU obstack) code into every
  binary ever shipped; mimalloc was attributed to the *wrapper author* instead of
  Microsoft. Both now in `THIRD-PARTY-NOTICES` §3.3/§3.4. The LGPL half of
  libmarpa inherits F5's relink posture (§D.3). **Follow-up:** F4's CI gate
  should assert the §D.2 table matches the shipped tree, since no existing
  automated check can see this class.
- **F7** *(landed 2026-07-14)* — **the notices shipped differed per platform.**
  `tools/gen_notices.sh` ran **only in the `release` job** (ubuntu), so only the
  x86_64-linux tarball + `.deb` bundled the complete file. The macOS (both) and
  aarch64-linux tarballs are packaged in their own jobs, where `make_release.sh`
  found no `THIRD-PARTY-NOTICES.dist` and fell back to the **committed §1-4** —
  shipping **without §5 (all ~140 Rust crate MIT/Apache texts)**. The Windows
  deliverable was a bare `.exe`, bundling **no notices at all**. Fixed by:
  (1) a **`notices` job** that assembles the file once, gates it on a
  content check (§3.2/3.3/3.4/§5/§6 present, plausible length), and hands it to
  every packaging leg as an artifact — so all platforms now ship byte-identical
  notices; (2) **Windows ships a `.zip`** (`latexml_oxide.exe` + notices +
  LICENSE + README), so no download lacks its notices. One generated file is
  valid for every target because `about.toml` sets no `targets` and krates treats
  an empty filter as `include_all_targets` (§5 is the union over all platforms,
  not the build host's subset) — **do not add a `targets` key** without
  revisiting the `notices` job. `make_release.sh` now also warns loudly when it
  falls back to the committed file.
