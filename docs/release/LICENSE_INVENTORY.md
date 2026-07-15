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
dynamic LGPL ones — **settled 2026-07-14** by the owner-approved position in
§D.3: source-availability discharges it, with per-artifact commits recorded in
`THIRD-PARTY-NOTICES` §7.

None of this constrains **using** latexml-oxide. It concerns **redistributing**
the binary, and our own source stays CC0 throughout.

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
  MIT (127), CC0-1.0 (9 — our **8** workspace crates *plus* `tiny-keccak`, which
  is third-party CC0 and not ours; the count matching is not the same as the
  attribution matching), BSD-3-Clause (1, `unidecode` — but see §D.4: its
  manifest does not describe its embedded data), MPL-2.0 (1), Unicode-3.0 (1),
  Zlib (1).
- **Attribution:** the per-crate license *texts* for the shipped binary are
  auto-generated into `THIRD-PARTY-NOTICES` §5 by `cargo about` — config
  [`about.toml`](../../about.toml) + template [`about.hbs`](../../about.hbs), assembled
  with the hand-authored §1-4 and the §6 copyleft texts (`licenses/`) by
  [`tools/gen_notices.sh`](../../tools/gen_notices.sh).
  Generated from the (gitignored) lockfile at release time, not committed.
  **Decided 2026-07-14 (owner): `Cargo.lock` stays gitignored.** It is unusual
  for a binary crate, and it is why the built `marpa` revision was not
  recoverable from the repo (the git dep carries no `rev =`) — but the licensing
  need is met by `THIRD-PARTY-NOTICES` §7, which records the exact revisions per
  artifact at release time. Committing the lockfile is a broader
  reproducible-build / supply-chain question, deserving its own PR rather than a
  rider on a licensing fix.
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

### Scope of `THIRD-PARTY-NOTICES` itself (settled 2026-07-15)

One file ships with every artifact, so **it is the union over the artifacts, with
per-entry scope markers** — not a description of `latexml_oxide` alone. That was
already true before anyone said so (`zlib`/`libiconv` are Windows-`.exe`-only
entries), but the file's opening line still read "The distributed binary",
singular, which stopped being true once the `cortex-worker` image began shipping
`cortex_worker` (the only executable of ours that links libzmq).

The alternative — a file whose meaning silently depends on which download you
got — is worse than a union that says so. So the header now declares the scope,
and any entry not present in every artifact states its own. **§5 is the sole
exception**: it is generated per artifact from that artifact's own dependency
graph, which is why the two container images get different §5s (measured: 5
worker-only crates, incl. `zmq`/`zmq-sys`).

What we actually publish, for the record:

| Artifact | Executables it carries |
|---|---|
| tarballs (×4), Windows `.zip`, both `.deb`s | `latexml_oxide` only |
| `ghcr.io/dginev/latexml-oxide` | `latexml_oxide` only |
| `ghcr.io/dginev/latexml-oxide/cortex-worker` | `cortex_worker` **+** `latexml_oxide` |

`latexmlmath_oxide` and `genschema_oxide` are declared `[[bin]]` targets but are
**not shipped in any artifact** — nothing in `make_release.sh`, the cargo-deb
asset list, or the Dockerfile stages them. If that ever changes, re-check this
table and the notices' scope paragraph together.

## B. Embedded resources (`resources/`, `include_str!`/`include_bytes!` at build)

| Asset group | Count | Origin | License |
|---|---|---|---|
| `CSS/` | 13 | Perl LaTeXML | PD (NIST) ≈ CC0 |
| `CSS/relaxng-schema-rustdoc-theme.css` | 1 | **ours, except its `data-theme="ayu"` colour values — taken from rustdoc's Ayu theme** | CC0, except that palette: MIT (rustdoc, © The Rust Project Contributors; itself crediting Ayu, © 2016 Ike Kurghinyan) — **notice given, §2.3** |
| `XSLT/` | 20 | Perl LaTeXML | PD (NIST) ≈ CC0 |
| `RelaxNG/*` (top level) | 25 | Perl LaTeXML schema | PD (NIST) ≈ CC0 |
| **`RelaxNG/svg/`** | **83** | **W3C SVG 1.1 RELAX NG schema, modified by Mozilla** | **W3C/Mozilla permissive — notice required** |
| `DTD/` | 2 | Perl LaTeXML | PD (NIST) ≈ CC0 |
| `Profiles/` | 9 | Perl LaTeXML | PD (NIST) ≈ CC0 |
| `javascript/LaTeXML-maybeMathjax.js` | 1 | Perl LaTeXML | PD (NIST) ≈ CC0 |
| `javascript/relaxng-schema-rustdoc-theme.js` | 1 | **ours** | CC0 |

Perl LaTeXML is **public domain** (NIST, 17 U.S.C. §105; upstream `LICENSE`
explicitly equates it to CC0) — so those are CC0-compatible with no notice
burden. `LaTeXML-maybeMathjax.js` only *conditionally loads* MathJax from a CDN;
it does **not** bundle MathJax, so no MathJax (Apache-2.0) redistribution
occurs.

**`RelaxNG/svg/` is the largest exception** (found 2026-07-14; this table
previously called all 108 RelaxNG files "PD (NIST)"). It was called "the *one*
embedded resource that is not public domain" until 2026-07-15, when the Ayu
palette in `CSS/relaxng-schema-rustdoc-theme.css` turned out to be a second —
the same over-broad-claim reflex this section exists to catch. It is the W3C's
SVG 1.1 RELAX NG schema (© 2001, 2002 W3C —
MIT/INRIA/Keio) with modifications © 2007 Mozilla Foundation. `latexml_core/build.rs`
walks the whole RelaxNG tree with **no filtering** and `include_str!`s each file,
so all 83 ship verbatim inside the binary. The grant is permissive
(use/copy/modify/distribute, in perpetuity, no fee) with one condition: *"the
above copyright notice and this paragraph appear in all copies."* Discharged by
`THIRD-PARTY-NOTICES` §2.2, and belt-and-braces by the verbatim comment headers
that travel inside each embedded file. Same species of error as §D.2: an
attribution claim broader than what was checked.

Re-verify (should list `RelaxNG/svg/` and nothing else):

```bash
grep -rliE "World Wide Web Consortium|Mozilla Foundation|Apache License|GNU General" resources/
```

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
| libgcrypt | LGPL-2.1 | **not linked at all** — `build_static_libxml.sh` builds libxslt `--without-crypto` (#261). Was previously listed as "dynamic, keep dynamic"; dropping that flag would pull an LGPL library into the *static* link, so it is a §D.3 decision, not a build tweak. |
| libgpg-error | LGPL-2.1 | **not linked at all** (transitive via libgcrypt, which is gone) |
| zlib / liblzma / ICU | Zlib / … | **Linux+macOS: not linked at all** — libxml2 is built `--without-zlib --without-lzma --without-icu`. No `libz-sys`/`lzma-sys` in the lock either (flate2 → pure-Rust miniz_oxide). **Windows: zlib IS statically linked** — vcpkg's libxml2 port default-features are `["iconv","zlib"]` and `release.yml` installs with no feature spec (F13). liblzma/ICU are not (no `lzma` feature in the port; `icu` is non-default). |
| **GNU libiconv** | **LGPL-2.1+** | **Windows: STATIC** (vcpkg `iconv` default feature → F13) — a third static LGPL link, covered by the same §D.3 posture; `THIRD-PARTY-NOTICES` §3.1/§3.5/§7. **Linux: not linked** (iconv is in glibc). **macOS: dynamic** against the host system libiconv. |
| libzmq | MPL-2.0 (+MIT, LGPL-2.0+ parts) | **cortex-worker container image only**, dynamic against the image's own `libzmq5` package — not in any binary download. Reached via `zmq-sys` (`links = "zmq"`, declares `MIT/Apache-2.0` — the binding's licence, not libzmq's: the §D.2 pattern again). Only the `cortex` feature pulls it, which `SHIPPED_FEATURES` excludes, so the audit never sees it. Noted in `THIRD-PARTY-NOTICES` §3.1; the image also carries `/usr/share/doc/libzmq5/copyright`. |
| **libkpathsea** | **LGPL-2.1** | **static on every release leg** — Linux + **macOS** (`tools/build_static_kpathsea.sh`; both macOS legs run it, `release.yml` build-macos/build-macos-intel), Windows 0.7.4 (`kpathsea_sys` `build_from_source`). The subprocess-`kpsewhich` backend is a **runtime** fallback (MiKTeX; a host with no libkpathsea), not a build-time one — it does **not** avoid the static link. See §D.3. |

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
same. The LGPL-2.1 **text** + the kpathsea copyright + the relink pointer + the
per-artifact commit are now shipped (`THIRD-PARTY-NOTICES` §3.2/§3.5/§6/§7) —
**F5 is closed**; the posture is settled in §D.3. (The obligation is **older than
0.7.4 and was never partial**: `46502fb90c` added `build_static_kpathsea.sh` to
the Linux *and* macOS legs together, and Windows had no release before 0.7.4 — so
every leg we have ever published already carried the static LGPL link. 0.7.4 adds
no obligation; it is the first release to *disclose* the one already there.)

Re-verify: `ldd target/<profile>/latexml_oxide` (dev) or the release-artifact
no-dynamic-clib CI assertion (release).

### D.2 Vendored native libraries — the cargo-about blind spot

Statically compiled into the binary by a `-sys` crate's `build.rs`. In each row
the **crate manifest's license is not the license of the code that ships**, so
none of these are catchable by cargo-deny (§A) — they are audited by hand here
and attributed in `THIRD-PARTY-NOTICES` §3.2-§3.5.

Note the framing "vendored **native**" understates the class: the defining trait
is *a manifest that does not describe the material shipped*, and C is merely its
commonest form. `unidecode` (§D.4) is the same failure in pure Rust. Two crates
here — `libxml`/`libxslt` — are also not "vendored" at all: they link a system
library via pkg-config, which is why they tripped none of the audit's original
five signals despite being the largest native surface in the binary.

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

Each artifact now **names its own inputs**: `THIRD-PARTY-NOTICES` §7 (generated
by `gen_notices.sh` at release time) records the exact latexml-oxide commit, the
marpa commit resolved from `Cargo.lock`, and `KPSE_REF`. This closes a real hole
in the earlier "everything is version-pinned" wording: kpathsea *is* pinned
in-repo, but the `marpa` git dep carries no `rev =`, so the built revision lived
only in the **gitignored** lockfile — a user rebuilding would have resolved
branch HEAD instead. Relinking is now reproducible rather than theoretical, and
the generator refuses to emit an unresolved pointer.

### Position (owner-approved 2026-07-14)

> The LGPL components (libkpathsea; parts of libmarpa) are statically linked.
> The relink obligation (LGPL-2.1 §6 / LGPL-3.0 §4) is discharged by **source
> availability**: latexml-oxide's own source is CC0-1.0 and public, and every
> input commit is recorded per-artifact in `THIRD-PARTY-NOTICES` §7. Both sides
> being open, a recipient can rebuild the combined work against a modified
> library. We therefore do **not** ship prelinkable object files or a written
> offer — the heavier alternatives LGPL permits for distributors who cannot
> provide source.

This mirrors the §C position on the TeX-Live-derived dumps: state the posture,
scope the CC0 claim honestly, and ship the notice that makes it checkable.
Revisit if the binary ever links an LGPL library whose source is *not* public and
pinned, or if latexml-oxide's own source ceases to be CC0 — either would break
the premise this rests on.

### D.4 Embedded third-party *data* — the same blind spot, in pure Rust

`unidecode 0.3.0` (reached from `latexml_core/src/common/cleaners.rs`) ships a
~1.9 MB Unicode→ASCII `MAPPING` table in `src/data.rs`, headed *"File
autogenerated with /scripts/generate_map.pl"*. That script converts the data set
from Sean M. Burke's Perl `Text::Unidecode`; the crate's own README says as much.

| | Declares | What actually ships |
|---|---|---|
| `unidecode` | BSD-3-Clause, © 2015 Amit Chowdhury | the Rust code — accurate |
| its `data.rs` table | *(nothing separate)* | data generated from `Text::Unidecode`, © 2001, 2014, 2015, 2016 **Sean M. Burke**, "same terms as Perl" = Artistic-1.0-Perl OR GPL-1.0-or-later |

Why no tool catches it: `cargo-deny` and `cargo-about` read the manifest (which
is not wrong about the code it covers); `audit_vendored_natives.py` looks for
*native* code and for third-party notices under `resources/`, and this is neither.
`deny.toml` allows neither Artistic nor GPL-1.0 — it passes **only** because the
manifest says BSD-3, so the "explicit review before it enters the distribution"
that the allow-list comment promises had never actually happened for this crate.

**Position (owner-approved 2026-07-15).** Attribute Burke by name and record the
ambiguity (`THIRD-PARTY-NOTICES` §3.6). We treat the mapping table as
**uncopyrightable data** rather than a creative work — it is a factual mapping,
and Burke notes much of it derives from the Unicode Consortium's own data — so no
copyleft obligation attaches, and no Artistic/GPL text ships in §6. This is a
judgement, not a certainty; it is written down precisely so it can be revisited
rather than silently inherited from a manifest. Attribution is owed under either
reading, and is given. Revisit if the table is ever used as more than a lookup, or
if upstream asserts otherwise.

## E. Subprocess-only tools (never linked → no license propagation)

All graphics helpers are invoked via `std::process::Command` in
`latexml_post/src/graphics.rs`, never linked — so their (A)GPL licenses do not
propagate to our binary. For the binary downloads, the host TeX Live ecosystem
provides them.

**Except in the container images** (F10): those `apt-get install` ghostscript,
imagemagick, poppler-utils and mupdf-tools (and all of TeX Live) **into** the
image, so there we *redistribute* them rather than borrow them from the host.
"Never linked" still holds and is what matters — they remain separate programs
merely aggregated alongside latexml-oxide, which does not place latexml-oxide
under their terms — and each arrives with its own Ubuntu copyright file under
`/usr/share/doc`. Worth stating because "the host provides them" was written for
the tarball and quietly assumed to hold everywhere.

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
- **F4** *(landed 2026-07-14)* — **CI gate for the classes no manifest audit can
  see** (RELEASE_CRITERIA §7). `tools/gen_notices.sh` runs in a dedicated
  `notices` job, the assembled file is bundled into **every** artifact, and the
  job content-gates it (§2.2, §3.2/3.3/3.4, §5, §6, plausible length) so a
  truncated notice fails the release. **`tools/audit_vendored_natives.py`** (CI
  `lint` job, beside cargo-deny) is the standing backstop for both blind spots:
  it fails when a shipped crate compiles/links native code without a §D.2 entry,
  and when an embedded resource carries an unaudited third-party notice. Both
  tripwires were verified to actually trip (removing an `AUDITED` entry, and
  dropping an Apache-2.0-marked file into `resources/`) — a gate that cannot fail
  is not a gate. Markers, not counts, for resources: a count gate fails on every
  legitimate new schema file while missing the thing that matters (a new
  copyright holder). Tracked with #51.
- **F8** *(landed 2026-07-14)* — **`resources/RelaxNG/svg/` is not public
  domain.** §B claimed all 108 RelaxNG files were "PD (NIST)"; 83 of them are the
  W3C SVG 1.1 RELAX NG schema (© 2001, 2002 W3C) with Mozilla's modifications
  (© 2007), embedded verbatim by `latexml_core/build.rs`. Permissive, but the
  notice must accompany all copies. Attributed in `THIRD-PARTY-NOTICES` §2.2;
  §B row corrected; `audit_vendored_natives.py` now guards the class. Found by
  asking the §D.2 question one level up — *what does this claim actually cover?*
  — rather than by any tool. Nothing else under `resources/` is third-party.
- **F5** *(CLOSED 2026-07-14)* — **libkpathsea (LGPL-2.1 static link).** kpathsea is statically linked on
  **every release leg**: Linux **and macOS** via `build_static_kpathsea.sh`, and,
  as of 0.7.4, Windows via `build_from_source`. (This line said "Linux … and
  Windows" until 2026-07-15, contradicting §D.1 two hundred lines above it.)
  `THIRD-PARTY-NOTICES` now carries the kpathsea copyright (§3.2), the relink
  note (§3.5), the verbatim LGPL-2.1 text (§6), and the **exact commit each
  artifact was built from** (§7) — all via `tools/gen_notices.sh` + `licenses/`.
  **Posture settled** (owner-approved 2026-07-14, §D.3): source-availability
  discharges the §6 relink obligation; no object files or written offer. Same
  call covers libmarpa (F6).
- **F6** *(CLOSED 2026-07-14)* — **vendored native libs were absent
  from every notice: libmarpa + mimalloc.** Found by auditing what `-sys`
  `build.rs` files actually compile, rather than trusting manifest `license =`
  fields (§D.2). libmarpa was attributed **nowhere** despite statically linking
  MIT (Kegler) **and LGPL-3.0/LGPL-2.1** (libavl, GNU obstack) code into every
  binary ever shipped; mimalloc was attributed to the *wrapper author* instead of
  Microsoft. Both now in `THIRD-PARTY-NOTICES` §3.3/§3.4. The LGPL half of
  libmarpa inherits F5's settled relink posture (§D.3), and F4's
  `audit_vendored_natives.py` is now the standing check for the class.

  **Decided 2026-07-14 (owner):** leave the upstream `libmarpa-sys` manifest
  declaring `MIT OR Apache-2.0`. Declaring only the wrapper's license is the
  `-sys` convention (e.g. `openssl-sys` declares MIT though OpenSSL is
  Apache-2.0), the crate is not published to crates.io so no downstream consumer
  is misled, and an "honest" combined field would trip our own `deny.toml` (LGPL
  is not in the allow list) for no real gain. **The control is §3.3 + the CI
  audit, not the manifest** — which is precisely why the audit exists.
- **F13** *(landed 2026-07-15)* — **the Windows `.exe` statically links zlib and GNU
  libiconv (LGPL-2.1+), via a vcpkg default nobody chose.** `release.yml` runs `vcpkg
  install libxml2:x64-windows-static` with **no feature spec** and there is no
  `vcpkg.json` manifest, so the port's `default-features` — `["iconv","zlib"]` — apply.
  On a fully-static triplet both land inside the `.exe`. So the Windows artifact carries
  a **third** static LGPL link, undisclosed, on the platform we are shipping for the
  first time. Found by asking what the *other* build path does: `build_static_libxml.sh`
  governs Linux and macOS only, and a §3.1 claim written from it ("nothing else C is
  linked in") was silently a five-platform claim sourced from a two-platform script —
  this PR's own bug class, committed while fixing this PR's own bug class.
  **Decided 2026-07-15 (owner):** keep the features and attribute. Dropping to
  `libxml2[core]` would remove both, but libxml2 without iconv handles only
  UTF-8/Latin-1/ASCII internally, which risks regressing `\inputencoding` documents on a
  platform we cannot test locally before the tag. libiconv now appears in
  `THIRD-PARTY-NOTICES` §3.1 (Windows-only static), §3.5's relink list, and §7's
  provenance; zlib is attributed too, though its licence only *invites* acknowledgement.
  Same §D.3 posture discharges the relink duty: the vcpkg port and GNU upstream are both
  public and versioned. **No gate covers this class** — `audit_vendored_natives.py`
  reasons over the cargo graph, and a vcpkg feature default is invisible to it.
- **F10** *(landed 2026-07-15)* — **the container images shipped no notices, and no
  LICENSE.** `docker.yml` pushes `ghcr.io/dginev/latexml-oxide` and `.../cortex-worker`
  to GHCR on release-publish, so both are a distribution channel — each hands a user the
  binary with statically linked LGPL in it (libkpathsea; libmarpa's libavl/obstack), and
  the worker additionally copies `resources/` in, carrying the 83 W3C/Mozilla SVG schemas
  whose licence requires the notice travel with every copy. `grep -c 'LICENSE|THIRD-PARTY'
  Dockerfile` → **0**. Same lesson as F7 and F9 for the **third** time: an artifact
  assembled by a different tool inherits none of the staging — and this time the tool was
  one nobody had enumerated as a channel at all. `make_release.sh` never runs for a
  container, so the Dockerfile now has its own `notices` stage that generates the file
  (per-image: the CLI and worker link **different** feature graphs, so §5 differs),
  content-gates it, and copies it plus `LICENSE` into both runtime images under
  `/usr/local/share/doc/latexml-oxide/`. §4 was also corrected: "provided by the user's
  system" is false for these images, which `apt-get install` ghostscript/imagemagick/
  poppler/mupdf **into** the image (aggregation, so latexml-oxide's terms are unaffected,
  and each keeps its own Ubuntu copyright file).
- **F11** *(landed 2026-07-15)* — **`unidecode`: the blind spot is not limited to C.**
  The crate embeds a ~1.9 MB transliteration table machine-generated from Sean M. Burke's
  Perl `Text::Unidecode`; its `LICENSE` is BSD-3-Clause © 2015 Amit Chowdhury, while
  upstream is © 2001–2016 Burke under "the same terms as Perl" (Artistic-1.0-Perl OR
  GPL-1.0-or-later). Structurally identical to the mimalloc case in §D.2 — a manifest
  naming a different holder than the material shipped — but in a **pure-Rust** crate, so
  neither `cargo-deny` nor `audit_vendored_natives.py` (not native, not under
  `resources/`) can see it, and the notices' own "-sys crates compile third-party C"
  caveat did not reach it. `deny.toml` allows neither Artistic nor GPL-1.0: it passes
  **only** because the manifest says BSD-3. **Decided 2026-07-15 (owner):** attribute
  Burke and record the ambiguity (`THIRD-PARTY-NOTICES` §3.6); we treat the mapping table
  as uncopyrightable data — a factual mapping, much of it derived from the Unicode
  Consortium's own — so no copyleft attaches and no Artistic/GPL text ships in §6. A
  judgement, written down rather than asserted away. See §D.4.
- **F12** *(landed 2026-07-15)* — **the schema-docs Ayu palette came from rustdoc.**
  `resources/CSS/relaxng-schema-rustdoc-theme.css` is ours, but its `data-theme="ayu"`
  colour values are rustdoc's (11/11 verbatim; its light/dark palettes are original), and
  rustdoc itself credits Dempfi's Ayu. §B booked the file as "ours/CC0" and called
  `RelaxNG/svg/` "the one embedded resource that is not public domain". Legally thin —
  colour values are arguably facts — but it fails **our own** stated standard: §2.1
  attributes NIST PD where "no notice is legally required; attribution is provided in
  gratitude." Attributed in `THIRD-PARTY-NOTICES` §2.3; §B row corrected. Note the
  resource gate cannot catch this class: it flags files that *announce* themselves, and
  all 16 embedded CSS/JS files carry no notice at all — they clear by **silence**.
- **F9** *(landed 2026-07-14)* — **the `.deb` shipped the committed notices, not the
  assembled ones.** Found by the PR's own code review, not by any gate. F7 fixed the
  tarballs and the `.zip` but missed the `.deb` entirely: `cargo deb` builds its payload
  from the asset list in `latexml_oxide/Cargo.toml` (`["../THIRD-PARTY-NOTICES", ...]`),
  i.e. the **committed** repo-root file — sections 1–4 only. Staging the assembled file
  into `${stage_dir}` did nothing for it, so `apt install ./latexml-oxide_*.deb` — the
  path the README calls the easiest way in — installed notices with no §5 (~140 Rust
  crate texts), no §6 (the copyleft texts the static LGPL links oblige), and no §7.
  Fixed in `make_release.sh`: point `../THIRD-PARTY-NOTICES` at the assembled file for
  the `cargo deb` run, restore the committed file after, and then **read the notices back
  out of the built `.deb`** (`dpkg-deb --fsys-tarfile`) asserting §5/§6/§7 — the failure
  was invisible from outside, since the `.deb` builds and installs fine either way.
  The lesson generalizes: an artifact assembled by a *different tool* does not inherit
  the staging you did for the others.
- **F7** *(landed 2026-07-14)* — **the notices shipped differed per platform.**
  `tools/gen_notices.sh` ran **only in the `release` job** (ubuntu), so only the
  x86_64-linux **tarball** bundled the complete file (the `.deb` never did — see F9). The macOS (both) and
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
