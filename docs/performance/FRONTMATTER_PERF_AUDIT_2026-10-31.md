# Frontmatter pipeline performance audit — 2026-10-31

This is a frozen study. Its open levers live in [`PERFORMANCE.md`](PERFORMANCE.md) as **P9** (the frontmatter pipeline), **P10** (the XPath compiled-expression cache) and **P11** (the `_font` attribute traffic).

**User directive, 2026-10-31.**
- "performance is critical for us. Both the work on reorganizing and segmenting the frontmatter needs to be performance-optimal, as well as any detection algorithms that will raise warnings. Be precise."
- Then: "rearchitect our current solution for max performance. Some phases can be coordinated better. We can use the string arena more."

**Basis:**
- Two read-only measurements on the tip `82adbc1802`: perf453 (stage shares and scaling) and perf453b (the re-architecture bounds).
- 18 documents:

| group | documents |
|---|---|
| many-author | 2005.08745, 2402.17260, 1707.07081, 2409.19633, 2407.00879, hep-ex0508047, 2201.11593, 1205.2739, 2603.24341, 2006.14918 |
| median | 2005.03240 (IEEEtran), 2504.05250 (icml), 1601.01776 (amsart), 1004.1370 (revtex4), 1812.02463 (llncs) |
| ≥973 authors | 2103.04797 (975, revtex4-1), 1710.09320 (1,108, aastex61), 1901.01540 (1,202, aastex62) |

**Method:**
- Every gain is `perf stat -e instructions:u`. Repeats agree within 0.05–0.15 %.
- Profiles come from a gdb stop-sampler: 1 ms of wall time, full stacks, 4 passes, 1,927–5,897 samples per document. perf call graphs do not work on this host: Zen 5 has no LBR, the binary has no frame pointers, and perf 7.0's DWARF unwinding fails under the mold layout.
- XPath and attribute costs were bracketed by `LD_PRELOAD` instruction shims.
- Scripts are kept in the job scratch, `perf453{,b}/bin`.

## Where frontmatter time goes

| quantity | value |
|---|---|
| frontmatter share, median paper (3 creators) | 0.68–1.05 % |
| frontmatter share, many-author papers | 3.5–15.6 %; 24–26 % at 1,108–1,202 authors; 40.9 % at 975 (written revtex) |
| cost per creator | 1.1–4.2 M instructions; linear up to 1,202 creators, except the two quadratics below |
| fleet-weighted frontmatter share (30k census, 29,803 papers) | 0.5–0.9 % of fleet CPU; the top 1 % of papers by authors carry 5 % of it |

Author calls per paper in the census: median 1, p90 4, p99 17, p99.9 290. 0.21 % of papers have ≥100 calls, and they hold 24.7 % of all `\author` calls.

## Duplicated work, phase by phase (2103.04797, 975 creators)

```
\author/\affiliation calls ─ ledger record_* ........................... 2.24 %
\maketitle → digest_front_matter (place.rs:403)
  ├ shadow_flush: rehearse the block tokens (shadow.rs:154) ............ 1.67 %  DIGEST #1
  │               rehearse the API content  (shadow.rs:192) ............ 5.96 %  DIGEST #1 (incl. lex_events / font_key)
  ├ real digestion of the queue (place.rs:564) ......................... 15.67 % DIGEST #2 of the same content
  │   keyvals per API call (frontmatter_api.rs:211, :482) ............... 5.91 % (only in the real digestion)
insert_frontmatter ─ absorb 5.46 % ─ old reader 2.54 % ─ 9 post-insertion passes:
  relocate_annotations 3.66 (QUADRATIC) · write_observed_frontmatter 1.57 · flag_merged_creators 0.67 ·
  respell_emails 0.30 · show_unanswered_marks 0.27 · coalesce_empty_creators 0.25 · answered_labels 0.17 ·
  drop_empty_contacts 0.15 · distribute_upfront_contacts 0.12
  └ the writer: resolve 0.80 → dom::read 0.37 → concede 0.30 → difference → plan = None ⇒ nothing written
```

**The observer writes nothing on these papers.** With `LATEXML_FM_SHADOW=0` (observer off), the XML is byte-identical on 17 of the 18 documents. Only 1812.02463 differs, because the writer splits a `{a,b,c}@airbus.com` group.

The observer's cost on those 17 documents is the instruction delta with it off:

| creators | observer cost |
|---|---|
| 975 | 16.0 % |
| 417 | 3.2 % |
| median paper | 0.01–0.1 % |
| fleet-weighted | about 0.05 % |

Neither the writer's apply, nor its second `dom::read`, nor `hide_answered_marks` ran on any of the documents (plan None).

## Ranked levers (upper bound: 975-author / 417-author / median / fleet-weighted)

| # | lever | bound | site |
|---|---|---|---|
| 1 | compiled-XPath cache inside `findnodes` (engine-wide; P10) | −0.60 / −0.99 / −0.81 / ≈ −0.8 %, measured with the shim, XML identical on 18 of 18 | `latexml_core/src/common/xml.rs:215` → rust-libxml `node_evaluate` → `xmlXPathNodeEval` compiles on every call; 57–63 distinct expressions per document, 15k–32k calls |
| 2 | `_font` attribute get/set (engine-wide; P11) | 1.1–2.3 % inside libxml2 alone (254k–651k calls per document); Rust side unmeasured | Document font get/set |
| 3 | observer work that writes nothing | 16.0 / 3.2 / 0.01–0.1 / ≈0.05 % | rehearsal, lexing, ledger, resolve, `dom::read` |
| 4 | the API content's rehearsal digested again by the real digestion (inside #3) | 7.6 / 1.3 / 0.03 / <0.05 % | `shadow.rs:192` vs `place.rs:564` (rehearsal/real cost ratio 0.4–0.7) |
| 5 | keyval digestion per API call | 5.9 / 0.5 / 0.02 / ≈0.02 % | `frontmatter_api.rs:211`, `:482` (≈1.2 M instructions per author) |
| 6 | `relocate_annotations` `held()`, **quadratic** | 3.7 / 0.11 / 0 / ≈0 | `relocate.rs:335-339` rebuilds `annotation_value` per note × author × child |
| 7 | XPath queries per creator in the post-insertion passes | 1.8 / 0.6 / 0.05 / ≈0.01 % | about 12 relative `ltx:personname…` queries per creator; `show_unanswered_marks` makes 4,871 calls at 975 creators |
| 8 | String-keyed hashing owned by frontmatter code | 1.5 / 0.07 / 0 / ≈0 | `TagAttrs = HashMap<String,String>` (`latexml_core/src/document/tag.rs:14`), `get("role"/"_num")` at `place.rs:574-600` and `frontmatter_api.rs:529`, `:739` |
| 9 | `font_key` `format!` per event | 1.25 % at 975 | `observer/events/mod.rs:622` |
| 10 | `get_frontmatter_name` re-digests the role's name macro for every contact | ≈1.0 % at ≈1,000 | `queue_digest.rs:656`, `:668` |
| 11 | `queued_author_count`, **quadratic** | 0.37 % at ≈1,000 | `queue_digest.rs:827-834` |
| 12 | three whole-document scans at `\end{document}` | ≈0.03 % of instructions (0.33 % of wall time: memory-bound walks) | `sect02.rs:543-547`, `titlepage_layout.rs:623`, `field_ink.rs:180`, `:245` |
| 13 | private frontmatter DOM attributes moved to a typed table | ≤0.1–0.2 % | `_annotations`, `_label`, `_bymark`, `_textmarks`, `_plainmarks`, `_picturemarks`, `_answers`, `_merged_checked` (11k–13k calls at ≈1,000 creators) |

## The self-check Warnings' budget (96a)

The budget is ≤0.1 % of instructions on every document. perf453 priced comparable walks:

| walk style | per-node cost | worst case | verdict |
|---|---|---|---|
| one traversal of the frontmatter subtree, constant work per node | — | ≤0.057 % | fits |
| an XPath query per node or per creator | 3,300–7,336 instructions | 0.05–0.23 % | fails |
| any walk from the document root | — | 0.09–0.23 % on every document | fails |

The self-check must therefore ride the shared index of lever 7 and never walk from the root.

## Conclusions

- **Fleet-wide**, the largest gains are engine-wide (#1, #2). The whole frontmatter pipeline is at most 0.5–0.9 % of fleet CPU, and its removable overhead is a fraction of that.
- **On the many-author tail**, the frontmatter levers are large: #3–#11 together are ≥35 % of a 975-author conversion. The two quadratics (#6, #11) are correctness-of-scaling defects and come first.
- **Digesting the API content once (#4)** is the phase-coordination lever. The lexer would read the real digestion's boxes instead of a rehearsal's. It is a design change gated on readings being identical (`ab_audit fmshadow`), not a tweak.

Not measured: dhat allocation counts (no `dhat-heap` build was available), exact per-pass instructions outside XPath (needs uprobes as root or an instrumented build), and memory.
