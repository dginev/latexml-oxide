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
derives from — each retains its upstream license, enumerated below. The two
that matter for a distribution notice: the **build-time-embedded TeX-Live
dumps** (§C), the **statically-linked libxml2/libxslt** (§D; static since
0.7.1), and — the sharpest, newly universal in 0.7.4 — the **statically-linked
LGPL-2.1 libkpathsea** (§D). A static LGPL link triggers the §6 relink
obligation (unlike the MIT static libs and the dynamic LGPL ones); see the §D
note.

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
  with the hand-authored §1-4 by [`tools/gen_notices.sh`](../../tools/gen_notices.sh).
  Generated from the (gitignored) lockfile at release time, not committed.
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

## D. Dynamically-linked system libraries

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

**libkpathsea is the one STATIC LGPL link** (the LGPL libs above are kept
dynamic precisely to avoid this). LGPL-2.1 §6 permits static linking but obliges
the distributor to let a user relink against a modified libkpathsea. Here both
sides are open, so relinking is possible: latexml-oxide's own source is CC0, and
the kpathsea source is public and **pinned** — `kpathsea_sys` `build_from_source`
fetches it at a recorded commit (`KPSE_REF`), and the Linux/macOS
`build_static_kpathsea.sh` pins the same. What §6 still requires in the shipped
artifact is the LGPL-2.1 license **text** + the kpathsea copyright + a pointer to
that pinned source — currently **MISSING** from `THIRD-PARTY-NOTICES` (its
hand-authored §1–4 carry no kpathsea entry). Tracked as **F5**. (This obligation
is not new to 0.7.4 — the Linux legs have static-linked kpathsea since the
`build_static_kpathsea.sh` legs; 0.7.4 makes it universal by adding Windows.)

Re-verify: `ldd target/<profile>/latexml_oxide` (dev) or the release-artifact
no-dynamic-clib CI assertion (release).

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
- **F5** *(open, 2026-07-14)* — **`THIRD-PARTY-NOTICES` missing libkpathsea
  (LGPL-2.1 static link).** kpathsea is statically linked on Linux
  (`build_static_kpathsea.sh`) and, as of 0.7.4, Windows (`build_from_source`),
  but the hand-authored §1–4 carry no kpathsea entry and no LGPL-2.1 §6 relink
  note. Add the LGPL-2.1 license text + the kpathsea copyright + a pointer to the
  pinned source (`KPSE_REF`) so a user can relink; both sides are open so relink
  is possible. **Owner to confirm** this source-availability posture satisfies §6
  for the static link (vs. shipping object files / a written offer).
