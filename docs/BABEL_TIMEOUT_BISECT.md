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

## Deeper localization (next iteration's findings)

`\bbl@foreach\bbl@toload{\bbl@tempc#1\@@}` calls `\bbl@tempc` for
each entry in `\bbl@toload`. `\bbl@tempc` is defined at line 4251:

```latex
\def\bbl@tempc#1/#2//#3//#4/#5\@@{%
  \count@\z@
  \ifnum#2=\@m % if no \BabelDefinitionFile
    \ifnum#1=\z@
      \ifnum\bbl@ldfflag>\@ne\bbl@tempc 0/0//#3//#4/#3\@@
      \else\bbl@tempd{#1}{#2}{#3}{#4}{#5}%
      \fi
    \else
      \ifodd\bbl@ldfflag\bbl@tempc 10/0//#3//#4/#3\@@
      \else\bbl@tempd{#1}{#2}{#3}{#4}{#5}%
      \fi
    \fi
  \else
    ...
    \bbl@tempd{#1}{#2}{#3}{#4}{#5}%
  \fi}
```

The recursive call: `\bbl@tempc 0/0//#3//#4/#3\@@` (lines 4255 and
4259) RE-INVOKES `\bbl@tempc` with first args = "0/0/...".

For our probe with just `english`:
- `\bbl@ldfflag` = 0 (default, line 360)
- `\bbl@iniflag` = 0 (default, line 356)
- `\@ne` = 1, `\@m` = 1000 (LaTeX kernel)

So `\ifnum 0>\@ne` = `\ifnum 0>1` = FALSE → no recursion. Same for
`\ifodd 0` = FALSE.

**Verified locally**: `\ifnum\foo>\@ne` (with `\chardef\foo=0`)
returns "NO" in BOTH Rust and Perl. The basic conditional works.

So the loop trigger is NOT in `\bbl@tempc`'s `\ifnum`/`\ifodd` paths.
It must be earlier — in `\bbl@toload`'s value parsing, or in
`\bbl@foreach` / `\bbl@vforeach`'s for-each iteration itself.

`\bbl@toload@last` at line 4193 is set as:
```latex
\edef\bbl@toload@last{0/\bbl@tempa//\CurrentOption//#1/\bbl@tempb}
```

For `english`, this becomes: `0/<\bbl@tempa>//english//<#1>/<\bbl@tempb>`.

If `\bbl@tempa` or `#1` or `\bbl@tempb` expands incorrectly in our
gullet (e.g. expanding to itself or a recursive form), the `\edef`
expansion could blow up. Or the resulting `\bbl@toload` value
might confuse `\bbl@vforeach`'s comma-delimited parsing.

## Next-iteration tools

- Add a token-counter eprintln! gated on `LATEXML_TRACE_TOKENS=1`
  that prints the current CS being expanded every 1M tokens. This
  would identify the looping CS.
- Run Perl LaTeXML on the same probe with `--verbosity=3` to see
  what `\bbl@toload` ends up as. Compare to Rust.
- Suspect `\bbl@tempa` and `\bbl@tempb` expansion in `\edef`.

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

## CRITICAL FINDING (2026-04-26 iteration B): NODUMP works

Setting `LATEXML_NODUMP=1` (which uses `latex_base.rs` source-level
definitions instead of the `latex_dump.txt` precompiled state)
makes the same babel probe load CLEANLY:

```
$ LATEXML_NODUMP=1 ./target/release/latexml_oxide /tmp/babel_simplest.tex
(Loading "babel.sty" definitions...
Info:latexml::converter Conversion complete: No obvious problems
```

Without NODUMP (default — uses `resources/dumps/latex.dump.txt`):
```
$ ./target/release/latexml_oxide /tmp/babel_simplest.tex
Error:unexpected:babel.sty Error loading binding for 'babel.sty':
Error:token_limit:Timeout Token limit of 100000000 exceeded
```

**The bug is in our dump-loaded state.** Some entry in
`latex_dump.txt` corrupts a definition that babel.sty's option
processing depends on. Likely candidates:
- `\@ifpackagewith`, `\DeclareOption`, `\ProcessOptions`, `\ProcessOptions*`
- `\edef`-friendly internals: `\bbl@trim@def`, `\bbl@xin@`, `\bbl@add`
- LaTeX2e option list bookkeeping: `\@unprocessedoptions`,
  `\@unusedoptionlist`, `\@optionlist`

## Distilled minimal probes (TDD reds + greens)

Tested isolation of babel's recursive constructs in `/tmp/babel_minrec*.tex`:

1. `\bbl@vforeach` over `english,french,german`
   → `[english][french][german]` ✅ matches Perl
2. `\bbl@tempc 0/1000//english//english/x\@@`
   → `[0/1000/english/english/x]` ✅ matches Perl
3. `\bbl@foreach\bbl@toload{\bbl@tempc#1\@@}` with
   `\def\bbl@toload{0/1000//english//english/x}`
   → `[0/1000/english/english/x]` ✅ matches Perl

The babel constructs WORK on a fresh state — the loop is from a
state interaction with the dump-loaded LaTeX kernel. Regression
test: `latexml_oxide/tests/regression/babel_recursive_loop.tex`.

## Next-iteration plan (UPDATED)

1. Diff dump-loaded `\@ifpackagewith` definition vs source-level
   (latex_base.rs). Same for `\DeclareOption`, `\ProcessOptions`,
   `\bbl@*` internals.
2. Suspect: macro encoded in dump uses different param-spec or
   body that produces infinite recursion when babel calls it.
3. If a specific dump entry is the culprit, either fix the
   dump-writer or add an explicit override for that CS.
