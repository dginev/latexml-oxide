# SYNC_STATUS Session Logs — Lifted 2026-09-03

Completed entries, historical post-mortems, and deferred explorations lifted out of the live `../SYNC_STATUS.md` worklist to keep the live file concise and actionable (<500 lines).

---

## 1. Upstream R1 — `brucemiller/LaTeXML#2852`: Subfile `\documentclass` Options
* **Status:** Merged in `latexml-oxide` as PR #310; open upstream on `brucemiller/LaTeXML`.
* **Details:** CI all-green (15 checks across Perl 5.34–5.42 and TeX Live matrices). No further code or automatable step remains in this repository.
* **Mechanism:** The upstream allowlist was hand-split on `,` and missed valued forms (`[varwidth=5cm]` $\to$ `Error:undefined:{varwidth}`). Rust fix uses `OptionalKeyVals` and matches keys.

---

## 2. Math Mode Glossary Display Tokens (`\gls` / `\acrshort`, 1705.10306)
* **Status:** Confirmed Parity / Non-Bug (2026-06-27). Deferred.
* **Findings:**
  - 293 errors `ltx:XMTok isn't allowed in <ltx:glossaryref>`: a glossary command in math mode digests the acronym term as math $\to$ bare letter `<XMTok>`s, rejected by the `glossaryref` content model.
  - Source analysis confirmed this is identical to Perl: Perl's `Stomach.pm::enterHorizontal` is a no-op in math mode, so `\lx@glossaries@gls@link` does not force text in either engine. Both engines raw-load the same `glossaries.sty` and produce identical output.
  - Perl 0.8.8 times out in `expl3-code.tex` on this paper, making live oracle capture impractical.

---

## 3. MathML Core Element Deprecation: `m:menclose`
* **Status:** Deferred by user directive (2026-07-30).
* **Details:** We emit `m:menclose` for `\cancel` and `\boxed` (`latexml_post/src/mathml/presentation.rs`). MathML Core removed `menclose`.
  - `\boxed` (`notation="box"`): Translates to an `m:mrow` with CSS border.
  - `\cancel` (`notation="updiagonalstrike"`): Has no MathML Core equivalent; requires SVG overlay or CSS diagonal strike.
  - Deferred to a dedicated MathML Core styling pass because it involves visual rendering decisions and golden XML diffs.

---

## 4. BibTeX `.bst` Support & The `\Dbar` Historical Retraction
* **Status:** Deferred family. Pointers in `parity/DEFERRED_FAMILIES.md`.
* **Retraction (2026-07-27):** The initial hypothesis that witness 2605.11579 proved `.bst` files vendor macro definitions was refuted: `alpha.bst` contains zero `Dbar` macros; `\Dbar` is defined by `mathscinet.sty` (which the witness failed to load). Undefined `\Dbar` is strict parity.
* **True Scope:** `.bst` interpretation is only relevant when a document ships `.bib` + `.bst` without a `.bbl` (a very small population on arXiv). Entry selection is already computed from `BIBLABEL` records (Divergence #80). Remaining `.bst` questions involve custom sort order, label formatting, and field selection.

## 2026-09-27 — closed ranked rows R1, R2, R5 and the 2026-08-02 status snapshot (lifted from SYNC_STATUS)

Ranked-table rows as they stood:

| # | item | state | size | detail |
|---|---|---|---|---|
| **R1** | **Fatal-Seed: Perl-0 vs Rust-101 Error Floods** (`2605.22927`, `2606.11121`) | **SEEDS RESOLVED** (re-verified 2026-09-27: 2605.22927, 2606.11121, 2606.01136, 2605.10685 all 0 errors, 0 fatals; the 117-paper cluster not re-measured) | — | Open items §R1 |
| **R2** | **Springer Nature `sn-jnl.cls` Dependency Drop** (witness `2606.00121`) | **RESOLVED** (batch 56kt: the Rust-only `sn_jnl_cls.rs` binding bypassed OmniBus's dependency scan and loaded a hand-picked subset; it now runs that scan over the shipped class, less article/natbib/apacite/program; 2606.00121 5 → 0 errors; residual in §R2) | — | Open items §R2 |
| **R5** | **Physical In-Place Image Cropping (`trim`/`clip`)** | **RESOLVED** (batch 56kx: rasters are cropped and turned into files of their own, once per job; witness `2510.17772`; DIVERGENCES #337) | — | Open items §R5 |

### Current status (2026-08-02 snapshot)

- **2026-08-02 — rc4-recut full rerun of sandbox-arxiv-2605+2606 (60,505 docs):**
  - **Overall:** no_problem 6,078/6,359 · warning 19,744/19,724 · error 3,991/4,102 · fatal 266/241.
  - **Fatal clusters:**
    | cluster | size | verdict |
    |---|---|---|
    | `panic:caught` | 3 | **FIXED (PR #491)** — pooled-worker math parser `PENDING_DISCARDS` stale handle sweep on abort. |
    | `TooManyErrors:MaxLimit(100)` | 117 | **REAL seed** — 4/8 sampled REAL, led by **2605.22927 & 2606.11121** (Perl 0 vs Rust 101-flood). |
    | `Stomach:Recursion` | 55 | **MIXED** — 3/8 REAL-by-count (`2605.17696` R144/P56, `2606.05321` R35/P15, `2606.08524` R94/P50). |
    | `Timeout:PushbackLimit` | 120 | Environmental/budget caps, not conversion bugs. |
    | `Timeout:TokenLimit` | 88 | Performance ceiling; legitimate heavy papers. |

---

### R1 — Fatal-Seed: Perl-0 vs Rust-101 Error Floods (`2605.22927`, `2606.11121`) — SEEDS RESOLVED
- Re-verified 2026-09-27 (56ks3/56kt): the seeds `2605.22927`, `2606.11121`, `2606.01136` and `2605.10685` all convert with
  0 errors and 0 fatals. The 117-paper `TooManyErrors:MaxLimit(100)` cluster they were sampled from has not been re-measured.

### R2 — Springer Nature `sn-jnl.cls` Dependency Drop (witness `2606.00121`) — RESOLVED (batch 56kt)
- **Cause:** the Rust-only `latexml_contrib/src/sn_jnl_cls.rs` binding bypasses the OmniBus fallback, whose
  `maybeRequireDependencies` (Package.pm:2776-2813) loads each package the raw class names that has a binding; the binding
  loaded a hand-picked subset instead, so booktabs was missing (2606.00121: `\toprule`/`\midrule`/`\bottomrule` undefined
  and two `\omit` errors).
- **Fix:** the binding runs the kernel's scan (`require_dependencies_except`, content.rs) over the shipped class, less article
  (already loaded; its `\LoadClass[twoside,fleqn]` re-load set equations flush left), natbib, apacite and program; the scan reads
  a CR-only class (2402.17342's copy) by its lines (DIVERGENCES #333).
- **Residual (parity):** newer copies (2404+) define `\toprule`/`\midrule`/`\botrule`/`\cmidrule` themselves (e.g. 2605.00003's
  copy L1280-1321) instead of loading booktabs; neither Perl nor this binding runs the class code, so a paper that relies on
  them without its own `\usepackage{booktabs}` still has `\toprule` undefined (the binding `\let`s only `\botrule`).
- **Side finding (fixed in 56ku):** the amsmath binding never processed its options (Perl amsmath.sty.ltxml:61) — repro
  `tools/perfect_kernel/repros/math-parse/amsmath_options_processed.tex`, GREEN.
- **Side finding (fixed in 56kv):** a class's `\LoadClass[opts]` options became global options, as in Perl (KPE #304,
  DIVERGENCES #334): webofc/USG `\LoadClass[fleqn]{article}` + amsmath without `fleqn` gave `ltx_fleqn` where pdflatex
  centres. Repro `tools/perfect_kernel/repros/loader/loadclass_options_stay_local.tex`, GREEN.
- **Side finding (fixed in 56kw):** elsarticle set `ltx_fleqn` for every layout (KPE #305, DIVERGENCES #335).
  Repro `tools/perfect_kernel/repros/loader/elsarticle_fleqn_follows_journal_type.tex`, GREEN.
- **Side findings of 56kv's review:** (1) FIXED in 56kw (DIVERGENCES #336, KPE #306): amsmath.sty:53
  `\newif\iftagsleft@` discards a class's `leqno` (repro `loader/amsmath_tagsleft_discards_class_leqno`, GREEN);
  ams_core/amsbook pass `leqno`/`reqno` to amsmath as amsart.cls:159-162 does; acmart passes `reqno` to amsart
  (acmart.cls:282); siamart/aomart/imsart-journal stubs pass `leqno`. Open: (2) `eqnarray` after a class's `fleqn`
  + amsmath: pdflatex flush left, Rust centred (`ltx_fleqn` is document-wide; DIVERGENCES #334). (3)
  `document_class_filename` (content.rs) is still overwritten by nested class loads; xkeyval's fallback reads it
  (`xkeyval_sty.rs:1524`). (4) elsarticle's `\ifpreprint` is always false in Rust; elsarticle.cls:112 makes
  `preprint` the default (true) and `5p`/`3p`/`1p`/`final` clear it (:71-87).

- **Side finding (57b review, RED):** a DefMath constructor in text mode converts silently. Perl warns
  `unexpected:\binom … should only appear in math mode` because `defmath_common_constructor_options`
  (Package.pm:1698-1711) calls `requireMath` in every DefMath constructor, unconditionally; Rust's
  `transfer_common_constructor_options` (dialect.rs) calls it only for `require_math => true`. That gate was added
  against a Rust-only warning on `\rightarrowfill` (a DefMath ARROW), which Perl builds with `defmath_prim` (no
  `requireMath`), so the fix belongs in the prim/cons branch choice of `def_math`. Repro
  `math-parse/text_mode_defmath_constructor_is_reported`.

### R5 — Physical In-Place Image Cropping (`trim` / `clip`) — RESOLVED (batch 56kx)
- A raster with `trim`/`viewport`/`angle` takes Plan::Convert with a generated name (a web-native one is copied, not
  converted); the worker crops (`graphicx_crop_rect`, Perl Util/Image.pm:400-418) and turns it in one `convert` run, once
  per job (`transform_raster_inplace`). The plain use keeps the untouched copy (before, `angle=` rotated it in place, and
  two nodes sharing a converted job rotated it twice). Display size: the remaining ops on the cropped size. Pixels per bp:
  the raster's own dpi over 72 (DIVERGENCES #337, KPE #307). 2510.17772: 12 figures cropped, aspect now the crop's.
- A small PDF's vector render is cropped through its root `viewBox` (`crop_svg_inplace`); only `angle=` sends it to
  the raster path. Every source's relative path is reserved before outputs are named, so a generated `xN` never
  overwrites an author's `xN.png` (KPE #308). Residuals: `reflect` is not applied; a negative trim (padding) is clamped,
  as in Perl; anisotropic resolutions crop by the x one; the job key holds the full options, so
  `[trim=X,clip,width=3cm]` and `[…,width=5cm]` crop twice into identical files.
