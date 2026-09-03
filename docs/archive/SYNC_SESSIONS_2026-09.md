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
