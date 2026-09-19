# Semantic-markup audit — 2026-09-19

First instrumentation of the **semantic-markup axis** (axis 2 of
[`../PERFECT_KERNEL.md`](../PERFECT_KERNEL.md)): does every construct emit its
LaTeXML-schema element rather than a presentational/generic fallback? Two
measurable signals, from sweep #104 (`latexml_oxide.56dz`) — frozen snapshot,
revalidate on current `HEAD`.

## Signal 1 — RelaxNG schema validity (S2, `validate_verdicts.tsv`)

`jing` against the authoritative `LaTeXML.rng` over every core XML.

| | docs |
|---|---|
| total | 2372 |
| schema-valid | 1955 (82%) |
| invalid | 417 (18%) |
| invalid with ≥50 rng errors | 15 |

Schema-invalid = broken semantic structure (an element in a position the schema
forbids, a leaked attribute, a dangling IDREF). The systemic violation classes,
by jing message over representative offenders:

| class | witnesses | jing message | fix locus |
|---|---|---|---|
| **internal-attr leak** (304k of all errors, 2 docs) | tcolorbox (35562), pgf-spectra (268654) | `attribute "_font"/"_autoclose"/"_autoopened"/"_fontswitch"/"_scopebegin" not allowed here` on `svg:g`/`svg:path`/`text`/`p` | construction-time `_`-prefixed internals leak into the core XML for the 2 largest (streaming-path) docs; the pre-serialization strip that clears them for the other 2483 docs is bypassed. Under root-cause (streaming/restart finalize skip hypothesis). A SEMANTIC/serialization fix. |
| **math content-model** | frege (84+54), numerica (45), tikz-network (87) | `element "rule"/"inline-block"/"text"/"logical-block" not allowed here; expected "Math"/"MathBranch"/…` | a block/rule/text node lands inside a math (MathBranch) or logical-block context the schema forbids — a construct emitting into the wrong container; per-construct, heterogeneous |
| **`<tags>` misplacement** | algorithm2e (221) | `element "tags" not allowed here` | the float/bibitem `<tags>` element in an invalid position (known algorithm2e residual) |
| **dangling IDREF** | biblatex-chicago cms-notes-intro (41) | `IDREF "Hendnote." without matching ID` | endnote/footnote cross-refs emit an idref with no matching id (note the malformed trailing-dot id) |

## Signal 2 — explicit `ltx_ERROR` fallbacks (post HTML)

Only **15 of 1850** manuals emit any `ltx_ERROR` element, and the dominant class
is `{forest}` (6) — the known forest.sty stub ([[project_forest_full_support]]).
Visible error-fallbacks are rare; semantic gaps are mostly the subtler schema
violations above, not `ERROR` elements.

## Takeaways / ranked targets

1. **Internal-attr leak** — highest error-count, cleanest fix (restore the
   `_`-prefixed strip on the large/streaming-doc path), clears the 2 worst docs.
   Generalizes to a serialization guarantee: no `_`-prefixed attribute in output.
2. **Math content-model violations** — more docs, heterogeneous; each is a
   construct emitting into a forbidden container. Triage per class.
3. Dangling IDREFs and `<tags>` misplacement — narrower, per-package.

## Method (reusable)

`tools/perfect_kernel/validate.sh <outroot>` writes `validate_verdicts.tsv`
(`bundle name rng_errors`). To get violation CLASSES, rebuild a jing-resolvable
copy of `latexml_core/resources/RelaxNG` (rewrite the `urn:x-LaTeXML:RelaxNG:*`
includes to relative hrefs, as validate.sh does) and run
`jing $RNG <doc>.xml | sed -E 's/^[^:]+:[0-9]+:[0-9]+: //' | sort | uniq -c`.
Error-count is a weak proxy; schema validity + violation-class is the axis-2
signal.
