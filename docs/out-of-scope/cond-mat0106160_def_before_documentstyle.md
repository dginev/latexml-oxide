# cond-mat0106160 — `\def\r{\rho}` before `\documentstyle`

**Status:** out-of-scope under "in scope iff Perl=0" predicate (Perl=3, Rust=6
as of 2026-05-01) — but a related, more verbose error cascade in Rust.

## Source pattern (a10.tex L1-4)

```latex
\def\r{\rho}
\def\beg{\begin{equation}}
\def\eeq{\end{equation}}
\documentstyle[12pt]{article}
```

`\def`s precede `\documentstyle`. Same root cause as
`hep_ph0001306_documentstyle_clobber.md`: class-load re-installs the kernel
`\r` (the over-ring text accent), clobbering the user's `\r=\rho` redef.
Subsequent `\r_{xy}` in equation contexts then hits the kernel's text-accent
`\r`, and `_{xy}` arrives in horizontal mode → script-in-text-mode error.

## Perl vs Rust delta

* **Perl**: 3 errors `expected:{ Missing sub/superscript argument` at lines
  37, 42, 121.
* **Rust**: 6 errors — the same 3 underscore positions plus extra
  `Unexpected:_` / `Unexpected:^` / `\lx@applyaccent Attempt to end mode
  restricted_horizontal in math`. Rust's recovery path emits a cascade where
  Perl emits one error per offending position.

## Why deferred

The architectural issue (kernel-reinstall clobbers user `\def` from
pre-class context) is in
`docs/out-of-scope/hep_ph0001306_documentstyle_clobber.md` — needs either:

* **(a)** class-load skips re-binding kernel CSes the user already redefined
  before `\documentstyle`, OR
* **(b)** plain_constructs' kernel CSes load once at engine bootstrap and
  never re-load on `\documentstyle` dispatch.

Until that root cause is fixed, the verbose-cascade aspect of the Rust
divergence is a secondary symptom.

## Possible quick-win (separate fix)

Even before fixing (a)/(b), Rust's stomach could match Perl's
"emit one error per position" by deduplicating the script-mode error
chain at recovery: after the first `Unexpected:_/^` outside math, swallow
the followup tokens at the same source position rather than re-emitting.
This wouldn't recover the paper but would bring Rust to error-count-parity
with Perl, which is what the canvas predicate measures.
