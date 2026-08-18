# Multi-document join — main paper + Supplementary Material into one output

An arXiv submission may ship several **top-level** `.tex` files: a main paper plus
one or more Supplementary-Material documents (each its own `\documentclass`, each
with its own `.bbl`). LaTeXML historically converted only one. This doc records how
we detect and join them, what has landed, and the queued streaming extension.

## Status

- **LANDED (in-memory join).**
  - Detection: [`main_tex::find_top_level_texs`](../../latexml_oxide/src/main_tex.rs)
    (#639) returns the ordered set, main first — template-safe (`.bbl` sibling +
    self-identifying `\title`/filename).
  - Join: [`multidoc::join_core_documents`](../../latexml_oxide/src/multidoc.rs)
    (#640) converts each top-level file **independently** (own session), then
    splices each supplement into the main `<document>` as a top-level appendix
    `<section>` titled by the supplement's own `\title`, with the supplement's
    id/label space prefixed (`as1_`, `as2_`, …) so intra-document `\ref`s resolve
    and never collide.
  - Front-end: `bin/latexml_oxide.rs` — `--whatsin=directory` expands to the set;
    several files side-by-side on the CLI (`latexml_oxide main.tex supplement.tex`)
    are joined the same way. Everything downstream of the single joined `<document>`
    (the whole post pipeline) is unchanged.
  - Guards: `main_tex::tests`, `latexml_oxide/tests/121_multidoc_join.rs`.
  - **Limit:** holds the parsed supplements in memory — fine for the overwhelmingly
    common small main+supplement case, wrong for a supplement large enough to need
    streaming (below).

- **QUEUED: streaming-scale join** (this doc's main subject).

## Why streaming needs a different join

A large document streams precisely so its whole DOM never sits in RAM
([`STREAMING_POST_DESIGN_2026-07-06.md`](STREAMING_POST_DESIGN_2026-07-06.md)).
An in-memory join of two full DOMs defeats exactly that. So at scale the join must
be **per-document independent** on the convert side and assembled on the **post**
side, where the streaming machinery already lives.

## The fit: the post pipeline is already a join engine

Streaming post is two-pass, and only the first pass is global:

```
pass 1  stream → Scan each page → ONE shared ObjectDB (+ page-tree metadata) → free page
pass 2  stream → per-page CrossRef/Graphics/MathML/XSLT → write → free page
```

`ObjectDB` is built to **outlive** individual page DOMs (it `adopt_xml`-deep-copies
what it keeps), and "only Scan needs global knowledge; everything after it is
page-local." The split feature already assembles tens of thousands of separate
page-DOMs into one cross-referenced, navigable site under streaming. **A
multi-document join is that same problem with pages sourced from N core-XML files
instead of one split tree** — a generalization of split, inheriting its streaming
memory profile.

XSLT stays **per-page and unchanged** (the structural join is pre-XSLT /
ObjectDB-level); only the supplement's appendix heading is a thin XSLT/CSS
presentation hook.

## Planned mechanics

1. **Convert phase** emits each top-level file to its own core XML on disk
   (`main.xml`, `supp1.xml`, …). Each conversion streams within its own RAM budget;
   no two full DOMs coexist. Same ordered list as directory-detection /
   CLI-multi-file.
2. **Post "join"** feeds all N core XMLs through the existing two-pass model:
   - **Pass 1 (Scan)** scans each file in order into **one** ObjectDB, applying a
     **per-source id/label prefix** as it adopts (`S1` → `d1:S1`, `LABEL:x` →
     `LABEL:d1:x`) — the collision fix lives exactly where ids normalize into the
     DB — and attaches each supplement into the combined **page-tree** as a
     top-level appendix node.
   - **Pass 2 (emit)** streams every page (from any source), resolving refs against
     the combined DB, writing the joined navigable output. Pagination stays the
     existing split decision (small join → one page; large → split pages, degrading
     identically to a large single document today).

## Seams

- `latexml_oxide/src/post.rs::run_post_processing_impl`, the `PostInput::File(path)`
  arm — generalize `PostInput` to an **ordered list** of core-XML inputs; Scan loops
  them (prefix per index), Pass 2 unchanged.
- `latexml_post/src/stream_split.rs` — the two-pass template.
- `latexml_post/src/scan.rs` / `object_db.rs::adopt_xml` — where the per-source
  id-prefix hooks in.
- Parity gate template: `latexml_oxide/tests/118_streaming_split_parity.rs` — the
  streaming join needs its own byte-identical-vs-in-memory-join gate before the
  auto-threshold widens.

## Residuals / deferred (both paths)

- A supplement that cross-`\ref`s **into the main** does not resolve (per-source id
  spaces) — faithful: arXiv's separately-compiled PDFs cannot cross-reference either.
- **Archives** (`.tar.gz`/`.zip`) still expand to a single main; multi-top-level in
  an archive is a small follow-up (`unpack_archive` → `find_top_level_texs`).
- Supplement conversion diagnostics are not yet folded into the persisted
  `.latexml.log` (the main's log is kept).
- Bibliography **cross-document** cite-key collisions are not prefixed beyond the
  bibitem `xml:id` (each document's bibliography is separate; a shared key resolving
  to the wrong doc's entry is a rare edge).

## First increment when resumed

Generalize `PostInput` to an ordered list + multi-input Scan with the per-source
prefix (Pass 2 untouched), guarded by an in-memory-join-vs-streaming-join parity
fixture. That is the smallest slice that makes the join streaming-safe.
