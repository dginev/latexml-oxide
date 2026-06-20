# Known Rust-side crashes reproducible with small/medium .tex inputs

## ✅ RESOLVED: 1804.01117 — xint raw-load stack overflow (SIGABRT), ar5iv profile (fixed 2026-06-20)

Runaway native recursion (number-arg-macro chain, ~25 000 deep) overflowed the
256 MB conversion-thread stack under `--preload=ar5iv.sty`/`INCLUDE_STYLES=true`.
**Fixed** by wrapping the per-expansion `defn.invoke()` in `read_x_token`
(`gullet.rs`, `Outcome::Invoke`) in `stacker::maybe_grow` — the SIGABRT is gone;
the paper now exits 124 (graceful wall-clock timeout) like Perl's fail-soft empty
doc, full suite 1459/0. Robustness win, not coverage (paper still doesn't convert
in either engine). Full gdb backtrace + analysis:
[`1804.01117_xint_stack_overflow_2026-06-20.md`](1804.01117_xint_stack_overflow_2026-06-20.md);
`docs/STABILITY_WITNESSES.md` Cluster F.

---

## RESOLVED: 0705.0790 — SIGSEGV **fixed in cycle 236** (commit `18c9640ee`, RAII wrapper in `9a5aa401c`)

**Resolution:** `DocOwnedNode` RAII wrapper (`latexml_post/src/doc_owned_node.rs`)
suppresses the `_Node::drop` → `xmlFreeNode` path for unlinked-but-doc-owned
nodes. Applied at two call sites: `PostDocument::drop` idcache teardown and
`math_processor::process_math_node` when detaching the xmath subtree.
Scan/crossref perf wins followed in cycles 239/241 (`0bc04e3eb`, `22adfc355`).

**Post-fix verification:**
- 3/3 minimal `$X$`+ar5iv repros exit cleanly.
- arXiv:0705.0790 cortex_worker --standalone: 6.88s (was >60s abort).
- 512-paper sandbox slice: 512/512 OK (no timeouts).
- 9/9 latexml_post unit tests pass.

The original investigation notes are retained below as a reproducer
template for any future UAF that surfaces with similar drop-order symptoms.

---

## 0705.0790 — `latexmlpost_oxide` SIGSEGV (cycle 236, 2026-04-23) — HISTORICAL

**Paper:** hep-th supergravity, 315 KB .tex, ~5900 line `<document>` XML
after conversion.

**Perl:** succeeds, exit 0, ~91 s with 61 warnings.

**Rust `latexml_oxide`:** succeeds, exit 0, ~5 s, peak RSS ~1 GB.
Produces a 5.8 MB XML.

**Rust `latexmlpost_oxide`:** **reliably SIGSEGV** (5/5 runs) when fed
either the Rust-produced XML or the Perl-produced XML. Happens with
all tested output formats (`--format=html5`, `--format=html`,
`--format=xhtml`, and the default XML writer), with or without
`--dest`. The HTML output (~2 MB) reaches stdout before the crash
fires on teardown/cleanup — so the bug is in drop order of the
libxml2 document tree, not in the emission path itself.

**Why cortex_worker reports abort exit 134:** the benchmark harness
runs with a 60 s watchdog; the actual segfault in post-processing
escalates to SIGABRT via the watchdog thread. Changing the default
watchdog timeout does not help — the underlying post-proc crash is
deterministic.

**Reproducer steps:**
```sh
cd /tmp && mkdir -p repro && cd repro
cp ~/git/latexml-oxide/docs/known_crashes/0705_0790.tex .
cd ~/git/latexml-oxide
target/release/latexml_oxide \
    --preload=ar5iv.sty \
    --path=~/git/ar5iv-bindings/bindings \
    --dest=/tmp/repro/out.xml /tmp/repro/0705_0790.tex
# ^ succeeds in ~5s
target/release/latexmlpost_oxide --dest=/tmp/repro/out.html /tmp/repro/out.xml
# ^ SIGSEGV every time
```

**Cycle 236 bisection — 4-line `.tex` minimal repro:**

```tex
\documentclass[12pt]{article}
\begin{document}
Hello $X$.
\end{document}
```

Crash: `latexmlpost_oxide --dest=/tmp/min.html /tmp/min.xml` → SIGSEGV
(exit 139), reliably.

Required ingredients (all necessary):
- `--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` during
  the core conversion. Without ar5iv, no crash.
- Any inline math (`$X$`, `$(0,2)$`, `${\mathbb P}^1$`, etc.) that
  produces a real XMTok child inside XMath. `${\mathbb P}$` (single
  XMTok, no script) crashes too; plain `hello world` with no math
  does not.
- The `xml:id` attribute on the XMath subtree (ar5iv sets nested
  xml:id like `p1.m1.1`). Stripping `xml:id` from the `<XMath>`
  wrapper makes the crash disappear. Stripping `_ID_counter__`
  alone does NOT help.

Workaround: `latexmlpost_oxide --keepXMath ...` exits cleanly — the
PMML-conversion code path is the culprit; preserving the XMath tree
skips it.

**Perl behavior on the same minimal `.tex`:** exit 0, produces
proper `<math id="p1.1.m1" ...><mi>X</mi></math>`. The
`ltx_markedasmath` rewrite (TeX_Math.pool.ltxml:190 `cleanup_Math`
afterClose hook) fires only when the XMath child set is
XMText/XMHint/single-PUNCT/PERIOD — not for a real `XMTok role="UNKNOWN"`.
So Perl keeps the math; the Rust crash is in the Rust-specific PMML
path's handling of `xml:id` on the XMath/XMTok nodes.

**Next steps:**
1. Read `latexml_post/src/mathml/presentation.rs::convert_to_pmml`
   and trace how xml:id is carried from `<XMath>` into the emitted
   `<m:math>` tree. Likely a `fragid → xml:id` remap (see mod.rs
   :1296-1306 which drops xml:id for XMText; presentation path may
   be doing the opposite and creating a dangling reference).
2. Run under `valgrind --track-origins=yes` with the 4-line repro to
   pinpoint the UAF address. The XML is tiny (15 lines) so the trace
   should be readable.
3. Compare against Perl's MathML::Presentation which copies xml:id
   onto the m:math element as `id=` — the Rust port may be doing
   this while also freeing the source libxml Node prematurely.

**Status:** not a conversion regression. Core `latexml_oxide` converts
cleanly; only `latexmlpost_oxide` crashes. 1/512 failure rate on the
10k-sandbox benchmark slice corresponds to this paper (and ~19/7898 in
the full sandbox — some of those 19 may share this root cause).
