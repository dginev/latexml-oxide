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
| **internal-attr leak** (304k of all errors, 2 docs) — **ROOT-CAUSED, DEFERRED** | tcolorbox (35562), pgf-spectra LSE (268654) | `attribute "_font"/"_autoclose"/… not allowed here` on `svg:g`/`svg:path`/`text`/`p` | RUST-ONLY. The streaming **fatal-stop "cheap partial"** branch (`core_interface.rs:1518-1540`) returns the document before pass-2 finalize, so the `_`-strip (`document.rs:811-822` `finalize_rec` PostWork, mirror of Perl `Document.pm:452`) never runs on the spine, and spilled segments splice in verbatim (`splice_segment_text`, document.rs:1783). Fires ONLY when a memory/timeout Fatal hits in streaming pass 1 — hence only the 2 biggest docs; control: pgf-spectra **NIST** completes pass 2 → 0 leaks, **LSE** hits the cheap partial → 168532. Verified repro (leak.tex: 900 tikz blobs at `--max-memory=1400` → cheap partial → 1105 `_font`). **Deferred: LOW value / MEDIUM risk.** The leak is only in salvage partials of docs that stay Fatal ([[feedback_fatal_stays_fatal]]) — they are FAILED conversions; their real fix is fitting in memory (a perf problem), not the attr strip. Fix plan if pursued: a strip-only DOM walk (`_`-prefixed attrs, memory-neutral) before the cheap-partial return, plus a per-segment strip decision (parse vs splice-time scan); `_font` cannot be stripped at spill time (it is the node→Font linkage, document.rs:5205). |
| **math content-model** | ~~frege (84+54)~~ **FIXED 56eg**, numerica (45), tikz-network (87) | `element "rule"/"inline-block"/"text"/"logical-block" not allowed here; expected "Math"/"MathBranch"/…` | **frege subclass = bare pieces unwrapped under `<MathFork>`; FIXED 56eg** (`cleanup_math` keeps the `<Math>` wrapper when the unwrap would be an invalid MathFork child — frege 28/80→0/80; witness `TeX_Math.pool.ltxml:219` had no parent guard, surpass-Perl). Remaining: numerica's `inline-block`/`logical-block` and tikz-network are a DISTINCT tier (Math under `text`/`td`/`equation`, not MathFork) — per-construct, heterogeneous |
| **`<tags>` misplacement** | ~~algorithm2e (221)~~ **FIXED 56eh (221→20)** | `element "tags" not allowed here` | **algorithm2e numbered-indented-line rule-before-tags FIXED 56eh** (`\lx@prepend@indentation@` keeps a leading `<tags>` first — RUST-ONLY parity, LaTeXML-block.rnc:189). Residual 20 = double-`\nl` (two tags) + block-end `}`-before-tags, a separate sub-issue |
| **dangling IDREF** | biblatex-chicago cms-notes-intro (41) | `IDREF "Hendnote." without matching ID` | endnote/footnote cross-refs emit an idref with no matching id (note the malformed trailing-dot id) |

## Signal 2 — explicit `ltx_ERROR` fallbacks (post HTML)

Only **15 of 1850** manuals emit any `ltx_ERROR` element, and the dominant class
is `{forest}` (6) — the known forest.sty stub ([[project_forest_full_support]]).
Visible error-fallbacks are rare; semantic gaps are mostly the subtler schema
violations above, not `ERROR` elements.

## Takeaways / ranked targets

Big picture: quality is already largely met — content median 98.5 (recall) and
82% schema-valid — with a **long heterogeneous tail** of small per-construct
violations. There is no single high-leverage fix.

1. **Internal-attr leak** — highest error-COUNT but DEFERRED (see table): it lives
   only in the salvage partials of 2 memory-Fatal docs, which are failed
   conversions; their real need is fitting in memory (perf), and error-count is
   the weak proxy we moved away from. Affects 0 successful conversions.
2. **Math content-model violations** — the real semantic-quality target (affects
   SUCCESSFUL docs): a `rule`/`inline-block`/`text` node emitted into a
   `MathBranch`/math container the schema forbids. **The frege `<MathFork>`
   subclass is FIXED (56eg)** — `cleanup_math` no longer unwraps a trivial-math
   `<Math>` into bare pieces under `<MathFork>` (28/80→0/80). Remaining and
   DISTINCT: numerica's `inline-block`/`logical-block` and tikz-network, where a
   Math sits under `text`/`td`/`equation` (not MathFork) — heterogeneous,
   root-cause per class.
3. Dangling IDREFs (biblatex-chicago endnotes) — narrower, per-package;
   note the biblatex dangling-`\hyperlink` class is RULED-KEEP. **`<tags>`
   misplacement (algorithm2e) FIXED 56eh (221→20).**

The two biggest schema offenders (tcolorbox, pgf-spectra LSE) are a **memory/perf**
problem, not a markup problem — they Fatal on memory; the markup axis will only be
satisfiable for them once they convert completely (PLANS 12 raw-interpreter perf /
streaming memory reduction).

## Method (reusable)

`tools/perfect_kernel/validate.sh <outroot>` writes `validate_verdicts.tsv`
(`bundle name rng_errors`). To get violation CLASSES, rebuild a jing-resolvable
copy of `latexml_core/resources/RelaxNG` (rewrite the `urn:x-LaTeXML:RelaxNG:*`
includes to relative hrefs, as validate.sh does) and run
`jing $RNG <doc>.xml | sed -E 's/^[^:]+:[0-9]+:[0-9]+: //' | sort | uniq -c`.
Error-count is a weak proxy; schema validity + violation-class is the axis-2
signal.
