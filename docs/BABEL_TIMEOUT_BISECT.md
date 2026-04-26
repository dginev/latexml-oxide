# Babel.sty token_limit Timeout — Bisection (2026-04-26)

> Investigation of the babel.sty raw-load loop affecting ~20 sandbox papers.

## Problem

`\usepackage[english]{babel}` (and any other language) causes Rust's
raw babel.sty load to hit `token_limit:Timeout 100M tokens, infinite
loop?`. Perl LaTeXML and pdflatex handle the same load cleanly in
~3.10 seconds.

## Bisection method

Truncate `babel.sty` at line N, append `\endinput`, observe whether
loop fires. Test:
```latex
\documentclass{article}
\usepackage[english]{babel}
\begin{document}
hi
\end{document}
```

Loaded via `TEXINPUTS=/tmp/babel_bisect/tex::` to override the
TeXLive babel.sty.

## Bisection results

| head -N | loop? | notes |
|---------|-------|-------|
| 250     | NO    | brace-imbalance error (truncation side effect) |
| 500     | NO    | |
| 1000-4000 | NO  | |
| 4290    | NO    | (rc=124 — actually times out via timeout(15s), but no token_limit) |
| 4300    | NO    | last definitions before \endinput |
| 4304    | TIMES OUT | natural \endinput on this line |
| 4306 (full) | TIMES OUT | natural ending |

The loop fires when the file is processed past line 4304 (the natural
`\endinput`). The lines after `\endinput` (4305-4306) are just
comments and shouldn't be processed.

## Loop trigger zone (lines 4290-4304)

```latex
4297:  \bbl@foreach\bbl@toload{\bbl@tempc#1\@@}
4298:  \bbl@tempb
4299:  \DeclareOption*{}
4300:  \ProcessOptions
4301:  \bbl@exp{%
4302:    \\\AtBeginDocument{\\\bbl@usehooks@lang{/}{begindocument}{{}}}}%
4303:  \def\AfterBabelLanguage{\bbl@error{late-after-babel}{}{}{}}
4304:  \endinput
```

Most-likely loop sources (highest priority for next investigation):
1. **`\bbl@foreach\bbl@toload{\bbl@tempc#1\@@}`** — iterates over the
   user's language list (`\bbl@toload` = `english` for our probe).
   Each iteration calls `\bbl@tempc{english\@@}` which routes through
   `\bbl@load@language` → `\input english.ldf`.
2. **`\ProcessOptions`** (NOT *) — final option pass; should be a
   no-op since `\DeclareOption*{}` made unknown options silent.
3. **`\bbl@exp{...}`** with `\AtBeginDocument` — protected@edef-style.

Most likely: `\bbl@foreach` triggers a loop in our gullet when
processing the language code `english`. The `\bbl@foreach` macro
itself is a babel-internal for-each iterator.

## Why `\endinput` doesn't fire

Despite the bisection showing line 4304's `\endinput` is reached,
SOMETHING in the lines 4297-4304 expansion enters an infinite-token
loop BEFORE `\endinput` actually closes the file. The file's natural
end is reached only AFTER the timeout fires.

When my probe replaces line 4304's natural `\endinput` with one of my
own appended (e.g. `head -4290 + \endinput`), no loop fires —
because the truncation cuts away the `\bbl@foreach\bbl@toload` line
or its execution context.

## Next-iteration plan

1. Add gullet instrumentation (env-gated) that prints CS names every
   ~1M tokens so we can SEE which CS is looping.
2. Compare against Perl LaTeXML's behavior: trace `\bbl@foreach` and
   `\bbl@load@language` execution and match Rust's behavior.
3. Look for our gullet/expansion mechanism that may be diverging from
   Perl's around tag definitions, expandable callbacks, or `\@empty`
   list edge cases.

## Cross-reference

- `latexml_package/src/package/babel_sty.rs` — current Rust binding
- `LaTeXML/lib/LaTeXML/Package/babel.sty.ltxml` — Perl 3-line wrapper
- `LaTeXML/lib/LaTeXML/Package/babel.def.ltxml` — sets `\bbl@opt@safe`
- `LaTeXML/lib/LaTeXML/Package/babel_support.sty.ltxml` — quote chars,
  language map, `\select@language` override, `\iflanguage` fake
- TL2025 raw `babel.sty` v25.15 (4306 lines)
