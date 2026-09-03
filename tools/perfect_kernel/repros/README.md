# Perfect-kernel topical repro corpus

Minimal, self-contained `.tex` reproductions of perfect-kernel failures, grouped
by the *mechanism* they exercise (not by package), so one topic can be worked
to green at a time and re-checked as a unit. Run a topic with
`tools/perfect_kernel/repros.sh <topic>`; see `repros.sh --help`.

This directory is deliberately outside `latexml_oxide/tests/` — a `.tex` under a
fixture root without a golden `.xml` silently passes. A repro that turns green
gets a real guard test (`cluster_package_guards.rs`, `perfect_kernel_batchNN`)
and STAYS here as a topic regression check.

## File conventions

One repro per file, ≤ ~20 lines, `article` class unless the class is the subject,
raw-load preload assumed (`[rawstyles,rawclasses]latexml.sty`; add
`% preload: [luatex,rawstyles,rawclasses]latexml.sty` for a LuaTeX-oracle doc).
The header comment block is machine-read by `repros.sh`:

```latex
% witness: numerica/numerica (numerica.tex:898)     <- corpus doc + line
% oracle:  pdflatex 0 errors                        <- the ground truth
% engines: rust=6 perl=6 (2026-09-03)               <- last measured
% expect:  0 errors; cell Math contains XMTok 0.125 <- 0 errors + ONE structural check
% status:  RED                                      <- RED | GREEN | CONTROL
```

`CONTROL` marks a shape that pdflatex ALSO rejects (e.g. `${$b$}$`); it must
keep producing an error and documents the boundary of the mechanism.

Name files `<mechanism>_<witness>.tex`; a topic `NOTES.md` holds the root-cause
summary per repro (mechanism with `file:line` into latex.ltx / the `.sty` /
tex.web / Perl, classification RUST-ONLY / SHARED / PERL-ORIGIN, fix site,
dead ends) — conclusions only, the play-by-play stays in the agent transcript.

## Topics

| Topic | Mechanism |
|---|---|
| `alignment` | `&`/`\\`/`\cr` handling, the per-cell hidden `$` pairing (`\lx@dollar@in@mathmode`), `\halign` templates, column types |
| `boxes-groups` | a box/group opened in one macro and closed in another (`\hbox\bgroup…\egroup`, ulem word boxes, `\begingroup`/`\endgroup` across mode frames), mode-frame errors |
| `index` | `\index` entry writing/expansion (`\protected@write`), makeindex-round-trip packages, `\edef`+`\write` of mode-dependent conditionals |
| `string-mouth` | tokens re-read from a string (`SanitizedVerbatim`, `\scantokens`, `\write`+`\input`, pre-tokenized bodies): lost catcodes, invented EOFs, conditionals cut at a mouth boundary |
| `sectioning-frontmatter` | `\@startsection` seam, `\maketitle`/`\@maketitle`, class-owned `\section`, sectioning inside lists/items |
| `luatex-profile` | LuaTeX-oracle docs under `[luatex]`: engine probes, `\directlua` bridge, Unicode text commands |
| `expl3` | l3 kernel behaviour under raw load: regex, keys, hooks, `\mode_if_math`, file boundaries |
