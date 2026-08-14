# Out-of-scope papers

Papers where **Perl LaTeXML also fails** on the same host (TL2025,
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings`), so they fail the
in-scope predicate ("in scope iff Perl produces 0 errors"). Not Rust
regressions; no fix is owed under the match-Perl mission. Conclusions only —
the per-attempt investigation narratives were compacted out 2026-08-14 (recover
from `git log` if needed).

## `\emph{ ... $$display$$ ... }` — display math inside inline text

Author error: `<ltx:emph>` is inline and cannot legally contain display math /
matrices; both engines reject it. `$$` inside horizontal-mode `\emph` decomposes
to two `$` toggles, stranding `_`/`^` in text mode (`Unexpected:_`) and poisoning
the emph subtree. The Rust/Perl count delta is **malformed-XML reporting
verbosity** (Rust reports per-position, Perl consolidates) — not a correctness gap.

- **0901.2408** — R=4, P=4. Trigger `\emph{ … $$ … _ … $$ … $…$ … }`. Loci:
  `tex_math.rs` `$$` gate, `\emph` constructor mode `latex_constructs.rs`.
- **math0010241** — R=33, P=19 (`\emph{ … $$Q=\left[array\right]$$ … }`). Same family.

## Kernel CS clobber when `\def` precedes `\documentstyle`

**hep_ph0001306** (R=6/P≈3-ish; both fail) and **cond-mat0106160** (R=6, P=3).
A user `\def\d{…}` / `\def\r{…}` *before* `\documentstyle` runs before the kernel
CS is even defined; the LaTeX-pool load then installs the kernel 1-arg accent
`\d{}`/`\r{}`, which consumes the following `$`, cascading script-mode errors.
`\documentclass` and `\gdef` avoid it. Perl preloads its kernel pool at engine
bootstrap (before reading the user file), so the user redef cleanly overrides.

Root cause reaches a **real latex-dump corruption**: `latex_constructs.rs`
force-reloads `plain_constructs` (with `state_unlocked`) to restore locked math
CSes (`\prime`, `\active@math@prime`) that `latex_dump` clobbers — and that reload
also clobbers the user `\def`. The dump clobber itself is a cmsy font-encoding
gap: `decode_math_char(0x230)` returns `'0'` (U+0030) instead of `'′'` (U+2032),
because the cmsy slot→Unicode map returns the raw slot low-byte (`value & 0xFF`);
`latex.dump.txt` serializes `\prime` with mathglyph 48 and ~85 other symbols
similarly corrupted (`\alpha` 945→11, `\aleph` 8501→64, …).

**Settled dead-ends** (all reverted — every approach that preserves the user
`\def` across the reload regresses a test): `if !IsDefined!` guard on `\d`/`\b`
→ accents_test; snapshot+restore `\d`/`\b` meaning → accents_test; skip the
`plain_constructs` reload → mathtokens_test; extend snapshot to
`\@math@daccent`/`\@math@baccent` → accents_test. The second-pass install is
load-bearing beyond the accent CSes. **Real fix (most upstream):** add cmsy
font-encoding entries so `decode_math_char` yields the right glyph — eliminates
the dump corruption, the restore-reload, and the clobber in one shot.

## XMath leaking into text-context elements

- **math0601451** — 1481× `malformed:ltx:XMTok in <ltx:title>` (amsppt+amstex).
  Math in amsppt `\title`/`\heading` needs `XMText`-wrapped output, not raw XMath.
  Loci: `latexml_engine/src/amsppt*` + the digest path promoting XMath into text.

## Deep-engine / resource failures

- **math0005251** — math-parser cumulative-state OOM: ~28 GB digesting the paper's
  math (Perl: ~10.5 s / 234 MB). Min repros run clean; needs accumulated math state.
  See `memory/project_math_parser_state_cumulative_hangs.md`. Fix is grammar-level
  in `latexml_math_parser`. Acceptance: `(ulimit -v 6291456; … math0005251.zip)` exits 0.
- **math0606553** — `\usepackage{xy}` + `\CompileMatrices` re-tokenization:
  xy.tex compile mode writes a `.xyc` file and `\input`s it back; a catcode
  snapshot (`\xyuncatcodes` via `\edef` of `\the\catcode64`) baked at the wrong
  time re-tokenizes `\lx@dual` as `\lx`+`@dual` → `undefined:\lx`. Affects xy +
  `\CompileMatrices` papers with `\lx@*` CSes in cells. Repro:
  `latexml_oxide/tests/graphics/xycompile.tex`. Empirical band-aid (not applied):
  force `at_letter:true` on `.xyc` paths.
