# Error/Fatal parity audit (2026-05-03)

> Verifies that every condition Perl LaTeXML marks with `Error(...)` or
> `Fatal(...)` is also marked with `Error!(...)` or `Fatal!(...)` in
> the Rust translation. **This audit underpins the "0 Rust regressions"
> headline**: if Rust silently succeeds where Perl flags an error, the
> parity_check.sh log-grep pipeline can record `Rust=0` but the page
> is structurally wrong.

## Headline numbers

| Source | Error/Fatal callsites | Notes |
|---|---:|---|
| Perl `LaTeXML/lib/**` | 269 | grep `^\s*(Error\|Fatal)\s*\(` excluding `Common/Error.pm` |
| Rust workspace | 153 | grep `\b(Error\|Fatal)!\s*\(` excluding macro definitions |
| **Gap** | **≈116** | **43%** of conditions silently absent |

The gap is **not uniformly distributed**. Some Rust crates have
proportional coverage; two are at zero.

## Per-crate coverage

| Crate | lines | `Error!`/`Fatal!` | density |
|---|---:|---:|---|
| latexml_core | 70,500 | 78 | 1 per 904 lines (proportional) |
| latexml_engine | 32,428 | 54 | 1 per 600 lines (proportional) |
| latexml_package | 57,119 | 17 | **1 per 3,360 lines (10× sparser than core)** |
| latexml_post | 19,993 | **0** | **zero coverage** |
| latexml_math_parser | 11,101 | **0** | **zero coverage** |
| latexml_oxide | 2,657 | 1 | thin (mostly thin glue) |
| latexml_contrib | 10,341 | 3 | thin (early bindings) |

## Critical gaps (verified 2026-05-03)

### Whole-crate zero-coverage

* **`latexml_post` (19,993 lines, 0 Error!/Fatal!)** — Perl `Post.pm`
  alone has 13 callsites, plus 7 in `Util/Image.pm`, 5 in `Post/LaTeXImages.pm`,
  several more in `Post/MathML/*`, `Post/Crossref.pm`, etc. The
  entire post-processing pipeline currently *cannot* report an
  Error condition. Failure paths return defaults silently.

* **`latexml_math_parser` (11,101 lines, 0 Error!/Fatal!)** — Perl
  `MathParser.pm` has 5 callsites. Math-parser failures, malformed
  grammars, and unrecoverable math constructs all currently return
  *something* without flagging.

### Whole-package zero-coverage (within latexml_package)

| Package | Rust lines | Rust `Error!`/`Fatal!` | Perl callsites |
|---|---:|---:|---:|
| `siunitx_sty.rs` | 2,604 | **0** | 6 |
| `pgfmath_code_tex.rs` | 1,438 | **0** | 6 |
| `xcolor_sty.rs` | 1,569 | **0** | 4 |
| `calc_sty.rs` | 393 | **0** | 4 |
| **subtotal** | **6,004** | **0** | **20** |

These four packages together represent ≈10% of all `latexml_package`
code yet contribute zero error reporting. `siunitx` is the
arxiv-sandbox's most common unit-of-measure package; `pgfmath` is
loaded transitively by `tikz`; `xcolor` is near-universal in
academic LaTeX.

### Stubbed-out TODO sites in well-covered files

These have a comment showing the original Perl `Error(...)` /
`Fatal(...)` but the Rust call site is commented-out or replaced
with a no-op:

| File:line | Comment-only Perl call | Status |
|---|---|---|
| `latexml_core/src/document.rs:1446` | `Error('unexpected', 'multiple-nodes', $self, …)` | TODO; whole `set_node` frag-node check is commented |
| `latexml_core/src/document.rs:1454` | `Error('unexpected', 'empty-nodes', $self, …)` | TODO; same block |
| `latexml_core/src/binding/content.rs:818` | `Error("missing_file", request, …)` | replaced by `fatal!(Package, MissingFile, …)` — divergence: Perl is recoverable, Rust terminates. Could over-fatal-ize on missing-but-optional inputs |
| `latexml_core/src/binding/content.rs:2246` | `Stomach::makeError($_[0], 'undefined', token)` | inline comment only |
| `latexml_core/src/state.rs:2189` | `Fatal('unexpected', '<endgroup>', …)` | comment-only |
| `latexml_core/src/state.rs:2281` | `Fatal('unexpected', '<endgroup>', …)` | comment-only |
| `latexml_core/src/stomach.rs:182-183` | `Error('misdefined', $x, $self, "Expected a Box\|List\|Whatsit, but got '" . Stringify($x))` | TODO |
| `latexml_core/src/stomach.rs:762` | `Fatal('internal', '<EOF>', …)` | TODO |
| `latexml_core/src/stomach.rs:973` | `Error('unexpected', '&', …)` | TODO |
| `latexml_engine/src/setup_binding_language.rs:342,355` | `Error('expected', 'conditional', …)` | TODO |
| `latexml_engine/src/base_parameter_types.rs:703` | `Error('expected', '<box>', $gullet, …)` | TODO |
| `latexml_engine/src/base_parameter_types.rs:1010` | `Error('expected', '{', $gullet, "Missing keyval arguments")` | TODO |
| `latexml_engine/src/latex_constructs.rs:2695` | `Stomach::makeError(undefined, $undef)` | inline comment only |
| `latexml_package/src/package/verbatim_sty.rs:181` | `Error("expected", "delimiter", $stomach, …)` | TODO |
| `latexml_package/src/package/color_sty.rs:42` | `Error('unexpected', $spec, $STATE->getStomach, …)` | TODO |
| `latexml_package/src/package/mathtools_sty.rs:125` | `Error('ignore', …)` for redefinition skip | TODO |

### Localised file-level gaps

| Perl file | Perl `Error/Fatal` | Rust file | Rust calls | Δ | Likely cause |
|---|---:|---|---:|---:|---|
| `latex_constructs.pool.ltxml` | 10 | `latex_constructs.rs` | 8 | -2 | `Error('undefined', 'dimension')` Perl L4956 + `Error('misdefined', itemtype)` Pair-item L4897 absent |
| `TeX_Macro.pool.ltxml` | 8 | `tex_macro.rs` | 7 | -1 | `Fatal('expected', "#n")` parameter-spec parsing Perl L117 absent |
| `xkeyval.sty.ltxml` | 8 | `xkeyval_sty.rs` | 7 | -1 | `Error('undefined', 'xkeyval')` package load-order check Perl L260 absent |

## Impact assessment

**Does this invalidate the "0 Rust regressions" headline?**

Spot-check on the 34 PERL_REGRESSION papers (those classified
"Rust strictly better than Perl" in stages 01-10) — count of how
many load each suspect package:

| Package | PERL_REGRESSION papers loading it |
|---:|---|
| siunitx | 0 / 34 |
| pgfmath | 0 / 34 |
| xcolor | 4 / 34 |
| calc | 0 / 34 |

So the unported-error-conditions packages **do not appear to be
manufacturing fake "Rust beats Perl" results** on the current 100k
corpus. The 30+ unaffected PERL_REGRESSION papers are genuine
Rust-superiority cases. xcolor's 4 papers should be re-verified
once xcolor error guards land.

**However**, the missing coverage is still real. Two scenarios where
it bites today:
1. A paper uses a malformed `\sisetup{}` argument; Perl says
   `Error:misdefined:siunitx`, Rust silently substitutes a default
   value. parity_check.sh records `Rust=0, Perl=1` and classifies it
   PERL_REGRESSION. The user sees a doc that's structurally wrong.
2. A paper has a legitimate `\definecolor` typo; Perl errors, Rust
   uses `#000000` silently. Same pattern.

These cases are present in the corpus but appear rare. The audit
gap is structurally serious (we'd discover any new ones only by
spot-rendering output) but not currently inflating the parity claim
materially.

## Prioritized fix list

Ordered by user-visible impact:

1. **TODO frag-node check in `document.rs:1446-1460`** (~10 lines).
   Smallest-blast-radius. Easy to land. Currently the whole
   `setInsertionPoint` validation is commented out; reactivate with
   proper `Error!("unexpected", "multiple-nodes", …)` / `("empty-nodes", …)`.

2. **`binding/content.rs:818` divergence — `Error` vs `Fatal`**. Perl
   uses recoverable `Error("missing_file", …)` and continues; Rust
   uses `fatal!(Package, MissingFile, …)` and terminates. May
   over-fatal-ize on optional-input scenarios. Soften to
   `Error!("missing_file", …)` per Perl.

3. **The 5 stomach.rs / state.rs `<endgroup>` and `<EOF>` Fatal sites** —
   these guard real malformed-input paths. Wire them.

4. **The 4 unported-error-condition packages** (siunitx, pgfmath,
   xcolor, calc). Largest LOC scope. Do it after smoke-testing one
   PERL_REGRESSION xcolor paper to confirm the fix actually changes
   classification.

5. **Whole-crate zero-coverage** (latexml_post, latexml_math_parser):
   long-tail; scope a per-file plan as a follow-up project. These
   touch substantial code and need careful per-file translation.

## How to extend / re-run this audit

```bash
# Perl callsites
grep -rnE '^\s*(Error|Fatal)\s*\(' \
  /home/deyan/git/latexml-oxide/LaTeXML/lib/LaTeXML/ \
  | grep -v Common/Error.pm > /tmp/perl_errors.txt

# Rust callsites
grep -rnE '\b(Error|Fatal)!\s*\(' \
  /home/deyan/git/latexml-oxide/{latexml_core,latexml_engine,latexml_package,latexml_post,latexml_oxide,latexml_math_parser,latexml_contrib}/src \
  > /tmp/rust_errors.txt
```

Compare counts per file/class. Any Perl callsite without a
corresponding Rust match is a candidate gap — verify by reading the
surrounding code path before treating it as a true gap (Rust may use
`Result::Err`, `panic!`, or `Warn!` in lieu of the `Error!` macro,
which is acceptable when intentional).
