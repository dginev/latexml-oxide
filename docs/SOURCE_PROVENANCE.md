# Source Provenance — the beyond-Perl showcase (issues #47, #92)

**The product: live source ↔ preview, two clients on one substrate.** The
flagship client is the **ar5iv-editor** — a two-panel web UI, **CodeMirror**
editing LaTeX on the left, the rendered **HTML preview** on the right —
where every edit (addition / deletion / modification) **auto-syncs** into
the preview. The value is deliberately simple: *type LaTeX, watch the page
update in place.* Source **locators** are the mechanism: they map each
source range to its preview region and back, so an edit updates (ideally
just) the affected region and the two panels stay aligned — and clicking
the preview jumps to the source, clicking the source scrolls the preview.

The **second client is a VSCode extension** that consumes the *same*
locators the *same* way: a webview HTML preview beside the `.tex` editor,
edit → preview update, click ↔ source. Build the substrate (locators +
conversion server) **once**; both clients are thin shells over one locator
contract — VSCode talks to the server over **LSP** (which also carries the
#92 diagnostics natively), the ar5iv-editor over HTTP/WebSocket. The two
differ only in their shell, not in the sync logic.

The same locator substrate then powers two further *beyond-Perl*
capabilities, for free:

- **#47** — accurate **linting** over the visible preview text, mapped back
  to the exact source span (even across macro expansion).
- **#92** — **Rust-compiler-grade author error messages**: point at the
  construct that actually caused the failure, with its expansion origin —
  not "somewhere near the end of the environment."

Perl LaTeXML chased the locator accuracy these need for a *decade* (upstream
[brucemiller/LaTeXML#101](https://github.com/brucemiller/LaTeXML/issues/101),
2009–2019) and never cracked it. Its data model made the cost prohibitive.
Rust's does not. That gap is the demonstration of new value.

## Why Perl stalled (a decade of #101)

1. **`getLocator` = "where the parser *is* now", not where a token
   *started*.** The `from` of a range was guesswork (`$c-$nc` — "random
   numbers"). Fixing it meant recording the start position at *every*
   `readToken` entry — painful to retrofit across Perl's Mouth.
2. **The efficiency stopper.** Storing origin *in* the Token loses Perl's
   constant-Token sharing. Deyan's 2016 escape — keep shared constants,
   store locators only where needed — was floated but never built.
3. **Two conflated cases.** *Invocation span* (where `\foo{…}` sits in the
   source) vs *macro origin* (where the expanded body came from). They need
   different mechanisms and reconciling them was "rather heavy."
4. **"Eating disorders".** Mouth/Gullet run *ahead* of the construct
   (look-ahead, delimiter put-back, KeyVals). Typos in an environment
   report at the environment's *end*. "Where are you now" ≠ the span of the
   expression.

Upstream eventually landed only basic XPath-form locator *attributes*
(#1013/#736) and closed #101 in 2019 assuming "working to satisfaction" —
i.e. it gave up on the accuracy goal.

## Why Rust breaks the deadlock

- **`Token` stays 8 bytes** (`SymStr`+`Catcode`); provenance lives
  **out-of-band** — a side table / interval map keyed by mouth+offset. This
  is exactly Deyan's 2016 "keep constants shared" idea, now actually
  realizable because nothing forces origin into the Token (RELEASE_CRITERIA
  §9 Tier B constraint).
- **One clean capture point.** `mouth.rs::read_token` (`:628`) is a single
  entry; record `(lineno, colno)` there *before* `read_char` advances and
  the `from` becomes exact — the "modify readToken" Bruce said was required,
  trivial here. `get_locator` (`:139`) currently does the same Perl
  "where am I now" approximation; this sharpens it.
- **We inherited the good parts.** `Locator::to_attribute()` already emits
  the 2009-designed XPointer form `range(from='l;c',to='l;c')` /
  `point('l;c')`; box / whatsit / error nodes already carry a `Locator`.

## Plan (phased — Tier A is near-term and parity-neutral)

- **Tier A — element-level invocation span (MVP, ship first).** Plumb the
  *existing* box-level `Locator` onto DOM nodes behind an opt-in
  `--source-map`. This is what the ar5iv-editor sync runs on: each preview
  element carries its source range, so a CodeMirror edit highlights/scrolls
  the matching preview region and a preview click maps back (via
  `querySelector` on the locator attribute). Same data gives
  **construct-level error locators** (the #92 win — fixes the
  "eating-disorder" mis-pointing). Capture token-start in `read_token` to
  make `from` exact. Mostly wiring; no parity change.
- **Tier B — token/char expansion provenance.** Out-of-band origin per
  token, distinguishing a **literal source span** from a **macro-expansion
  span**. Delivers: accurate linting over visible text where it crosses
  macro boundaries (`\def\au{au}\au{}tor` → visible `autor`, source span
  over `\au{}tor`) and **macro-origin error traces** (#92's "climb the
  expansion stack" pain). This is the payoff Perl never reached.
- **Process model — the shared backend (required, not optional).** Both
  clients need sub-second reconversion per keystroke/save; "near-instant"
  (the #47 word) is impossible from a cold binary that re-parses ~24k dump
  entries per run. So a **persistent server** holds warm engine state and
  reconverts on a debounce. **MVP: full-document reconversion** (warm state
  + debounce — simple, and fast enough to start; the whole point of the
  Rust rewrite is that a full reconvert is already cheap). **Region-
  incremental** reconversion — re-running only the edited span where
  locators permit — is a later optimization, taken only if the full-doc MVP
  proves too slow on large papers. The server speaks LSP to VSCode and
  HTTP/WebSocket to the ar5iv-editor, and is the single host for preview
  sync (#47) and diagnostics (#92).

## MVP granularity: line-level, math-opaque (decided 2026-05-24)

For linting and preview the *useful* granularity is **line correctness** —
exactly SyncTeX's granularity, and enough to "scroll to roughly where I'm
editing" and to map a linter finding back to a source line. So the MVP
deliberately relaxes scope:

- **Line-level, at the block/inline-element level.** Stamp each element with
  a `(tag, from_line[, to_line])` range; columns are nice-to-have, not
  required. The mouth tracks `line` robustly; line attribution is far more
  stable than column attribution under the eating-disorder.
- **Math is opaque.** Stamp one line-range on the `ltx:Math` / `<math>`
  wrapper at the `absorb` hook; do **not** descend into the Marpa-built
  MathML. This defers §7 A.3 (the math-parser provenance gap — the single
  biggest item) until there is a *clear, tested* way to do in-equation
  mapping.
- **Deferred until clearly needed:** column precision, the per-leaf
  char-offset map (§6 rung 2/`data-srcmap`), and in-equation provenance.

This makes the MVP bar simply *"match SyncTeX — line-level, block-element,
math opaque"*: achievable, parity-neutral, and it sidesteps the hardest
correctness stages while still delivering the ar5iv-editor sync and the
linter. The richer rungs in §6 and §7 remain the documented growth path.

## Cost & the switch (off by default)

Source locators are not free on **two** axes, so a single switch
(`--source-map`, plus an env equivalent) gates the whole feature and is
**disabled by default**:

1. **Runtime / memory.** Token-start capture in `read_token`, the
   out-of-band provenance table, and locator propagation through boxes all
   cost CPU and RAM. When the switch is off, *none of that machinery runs* —
   no side table, no start bookkeeping — so the corpus/parity conversion
   path (where speed, RAM, and the ~99.4% number are measured) pays
   **nothing**. The switch must gate *tracking*, not merely *emission*;
   tracking-then-discarding would defeat the point.
2. **Markup verbosity.** Per-element locator attributes measurably enlarge
   HTML/XML and can leak local paths. Default-off keeps shipped output
   compact and clean (RELEASE_CRITERIA §6); the ar5iv-editor and linter turn
   it on explicitly.

This makes the showcase strictly additive: a normal conversion is identical
to today, and the provenance cost is borne only by the tools that want it.

## The #92 connection (same substrate)

#92 wants author-facing error UX that TeX/LaTeXML can't give: *where did
this construct start*, and *what expanded into the offending token*. Those
are precisely Tier A (construct start, killing the eating-disorder
mis-pointing) and Tier B (expansion origin). One provenance layer, two
products — build it once.

## Precise implementation — correctness for LaTeX emulation

This is the detailed, correctness-first plan. It cites our current pipeline
by `file:line`, borrows SyncTeX's proven model, and heeds the specific traps
Bruce documented over a decade in brucemiller/LaTeXML#101.

### 0. The model (first principles + how it differs from SyncTeX)

A locator is `(tag, from_line;from_col, to_line;to_col)` over a source file
`tag`. Our `Locator` (`common/locator.rs:17`) already *is* this shape, and
`Locator::to_attribute()` (`:166`) already serializes the 2009-designed
XPointer form `…#textrange(from='l;c',to='l;c')` / `…#textpoint('l;c')`.

What SyncTeX does and where we improve on it:

| SyncTeX | latexml-oxide |
|---|---|
| Unit = TeX **box/glue/kern node** on a PDF page | Unit = **DOM element** in the HTML tree |
| Records `(tag, line)`; **column unused** (TeX has no columns) | Records `(tag, line, *column*)` — the Mouth tracks `colno` (`mouth.rs:78`) |
| Resolve output→source by **geometric nearest-box** on page (x,y) | Resolve by **range containment** in the DOM (the element *is* the addressable unit — no geometry) |
| `Input: <tag>:<file>` preamble maps tag→filename | Same: emit a tag→file table once; per-element attrs carry the integer `tag`, **not** a path (avoids local-path leakage, RELEASE_CRITERIA §6) |
| Node tagged with the line current **at node creation** | Same risk, same fix — see §1 |

Two directions, in SyncTeX's terms: **backward** (output→source: clicked
element → its range → editor selection — a direct attribute read) and
**forward** (source→output: cursor `(l,c)` → the *tightest* element whose
range contains it — a containment walk, the exact analog of SyncTeX's
deepest-box query but without geometry).

### 1. The one correctness invariant: tag a construct's START, not "where the parser is now"

This is the rock both Bruce and SyncTeX broke on. Our `gullet::get_locator()`
(`gullet.rs:180`) returns the *current* mouth position (`mouth.rs:139`),
which by construction is **ahead** of the construct being built — TeX's Mouth
is always a bit past the token (look-ahead to find token end; delimiter
put-back; KeyVals reading to `]` before chewing). SyncTeX hit the same wall
with `\item` (parser-time node creation vs later use; fixed in v1.17 with
node-weight/mean-line heuristics); Bruce called it the "eating disorder."

We fix it *exactly*, not heuristically, in two layers:

1. **Token-start capture in the Mouth.** `mouth.rs::read_token` (`:628`) is a
   *single* entry point — the thing Bruce said was needed but was painful to
   retrofit across Perl. Record `(self.lineno, self.colno)` into a new
   `last_token_start` field **after** inter-token skips (leading spaces,
   comment skipping, `^^`-decoding at `read_char` `:556`) and **before**
   consuming the token's own chars. Heed Bruce's warning: capture *after*
   skips so `from` lands on the first significant char. Then `get_locator()`
   returns an accurate `from = last_token_start`, `to = current pos` — no
   more `$c-$nc` guesswork.
2. **Construct-start snapshot at digestion.** A box/whatsit's true span is
   `[start of its first contributing token … end of its last]`. So snapshot
   `gullet::get_locator()` as `open_loc` when a digestion frame *opens*
   (before reading args/body), and range it with the *close* position via
   `Locator::new_range(open_loc, gullet::get_locator())` (`:80`) when it
   closes. The Stomach already grabs `gullet::get_locator()` at box creation
   (`stomach.rs:158,342,353,822`); the change is to also stash the *open*
   locator at frame entry and combine. This yields the invocation span and
   makes put-back/look-ahead irrelevant (the range is fixed at open→close,
   not the look-ahead tail).

### 2. Tier A — element-level invocation span (the MVP)

The data is already flowing; the work is to make locators *ranges* (§1) and
to *stamp* them on nodes behind the switch.

- **Carrier:** digested items already hold `.locator` — `Tbox`
  (`tbox.rs:30`, set at `:79` from `gullet::get_locator`), `Whatsit`
  (`whatsit.rs:39`), `List` (`list.rs:23`); `digested.rs:260-266` exposes
  `get_locator()` for every variant. Make these the §1 ranges.
- **Attachment hook:** `document.rs::absorb` (`:641`). In the `TBox`,
  `Whatsit`, and `Alignment` arms (`:654-675`) the box being absorbed sits in
  `self.box_to_absorb` and the nodes it produced are the freshly-recorded
  `constructed_nodes` (`:742`). After `be_absorbed` + `close_constructed_nodes`,
  when the switch is on, stamp each node in that frame with
  `box_to_absorb.get_locator().to_attribute()`. Also stamp in `open_element`
  (`:916`) / `insert_element` (`:835`) from the current `box_to_absorb`
  locator so constructor-opened elements (which bypass `absorb`'s box arms)
  are covered.
- **Attribute:** reuse `to_attribute()`'s value but with the **tag table**
  form (integer tag, not path). Pick one attribute name (proposal:
  `data-src` to stay HTML-valid and obviously non-semantic) present *only*
  under the switch.
- **Gate:** off by default; when off, §1 capture and this stamping are
  both skipped (no side cost — Cost & the switch).

### 3. Tier B — token/char expansion provenance (the linting payoff)

First principles: a visible output char came from either **(i)** a literal
source token (it has a real `mouth+offset` span the author can edit), or
**(ii)** a token *spliced in by macro expansion* (no literal source span —
its provenance is the invocation site, with a chain back to the macro's
definition site). Bruce's `\def\au{au}\au{}tor → autor` is case (ii): the
`au` chars are not editable text at any `autor` span; their provenance is the
expansion of `\au` at the invocation.

Mechanism (out-of-band — **never widen the 8-byte `Token`**):

- When the Gullet expands a macro it pushes the body onto the mouthstack
  (`gullet.rs` `open_mouth`/`mouthstack`). Tag that **expansion frame** with
  `(invocation_loc, definition_loc)`: `invocation_loc = gullet::get_locator()`
  at expansion time; `definition_loc` = the definition's own `.locator`
  (`definition/expandable.rs:56`, already recorded at define time).
- Keep an **expansion-provenance stack** parallel to the mouthstack. A token
  consumed for digestion gets: literal span if it came from a *file* mouth,
  else the enclosing expansion frame's `(invocation span → definition site)`
  chain. Store this in a **side table keyed by the digested item / node id**,
  not in the token.
- Emit a `kind=literal|expanded` marker alongside the range so the linter
  knows whether a visible-text offset maps to an editable source span
  (case i) or only to an invocation range (case ii). This *is* the #47 ask.

### 4. Perl's unsolved hard cases — concrete handling

- **Eating disorder / `\item`:** solved by §1 (open→close range), not
  heuristics; forward sync then uses "tightest enclosing range wins."
- **Auto-created elements** (font-switch `ltx:text` from `\rm`): the element
  has no originating box. Take the locator from the *causing* whatsit (its
  `.locator` is in `box_to_absorb`); failing that, inherit the parent node's
  range (a font switch lies within its parent's span). Bruce flagged exactly
  this case as unhandled.
- **Attribute-only constructs** (`\label`, `\ref`): attributes can't hold a
  child locator node. Keep a **sidecar map** `(element → {attr → locator})`
  in source-map mode rather than dropping the info (Bruce noted attributes
  "don't really have a place to store a locator").
- **Adjacent-text coalescing:** coalesce sibling text with equal/adjacent
  locators into one ranged element instead of a span-per-token (Bruce feared
  the XML-bloat; SyncTeX v1.19 added `=` run-compression for the same
  reason). Source-map mode only.
- **Containment invariant (boxes don't overlap):** SyncTeX assumes LaTeX
  boxes nest (the ConTeXt-overlap caveat, v1.2). Our DOM nests by
  construction, so a child's range must be ⊆ its parent's range — see §5.

### 5. Output contract & validation

- **Tag table:** emit `tag→file` once (document-level, SyncTeX preamble
  style); per-element `data-src` carries `tag#textrange(...)`.
- **Round-trip tests** (pin like `tests/math/norm_kerned_delims`): for a
  corpus sample assert (a) every range is within its file bounds; (b) a
  child element's range ⊆ its parent's (nesting invariant); (c) for
  `kind=literal` text, the source substring at the range equals the visible
  text (modulo normalization). These catch the off-by-one/whitespace drift
  that plagued the Perl fork (dginev, 2010: "prone to whitespace offsets").
- **Switch:** one flag gates §1 capture, §3 side table, and §2/§4 emission
  together; off by default (Cost & the switch).

### 6. Output-side: reflow, viewport variability, and sub-element navigation

The preview is **reflowing HTML**, not a fixed-geometry PDF page: the same
paragraph is ~10 visual lines on a wide screen and ~20 on a narrow one, and
re-renders on every edit. The model must scroll/highlight correctly through
all of that.

**Core principle — never store output coordinates; store source provenance
on stable nodes and compute geometry on demand.** This is the one place we
deliberately diverge from SyncTeX: SyncTeX maps source → *PDF (x,y)*, which
is valid only because PDF pages don't reflow. We map source → *DOM node +
source range* and let the browser compute the current position when asked:

- `element.scrollIntoView()` / `element.getBoundingClientRect()` are
  evaluated against the *live* layout every call. When a paragraph reflows
  10→20 lines, the target node is unchanged; the browser simply returns its
  new position. There is nothing baked-in to invalidate. A "visual line" is
  an ephemeral layout artifact with no DOM identity — we address the
  *character/element*, and the browser reports which line it currently sits
  on.

**Node-level vs internal-text-level.** Correct: CSS selectors and `data-*`
attributes only address **nodes** — you cannot CSS-select "the 5th character
of this text node." The escape hatch is the **DOM Range API**, which is *not*
limited to node granularity and *also* tracks reflow:

- forward (source→preview): `range.setStart(textNode, off)` +
  `range.getClientRects()` → the live rectangle(s) of an arbitrary character
  span → scroll to it;
- backward (preview→source): `caretRangeFromPoint(x, y)` → text node +
  offset → map back to source.

So sub-element navigation *is* achievable — via Range, not via CSS.

**Granularity knob (engine-side implication).** To get text-level precision
without exploding the DOM into a `<span>`-per-word (the coalescing cost in
§4), a text-bearing **leaf** element carries a compact monotonic
**char-offset map**: DOM-text-offset ↔ source-offset (e.g. a packed
`data-srcmap` of breakpoints). The client resolves a source offset to a
`(textNode, offset)` and builds a Range at query time. Three rungs, pick per
need:

1. **Element-level (Tier A MVP):** `data-src` *line* range on each element;
   `scrollIntoView` the containing element. Reflow-safe already; good enough
   to land the ar5iv-editor. **This is the chosen MVP** (see "MVP
   granularity" above).
2. **Text-offset (post-MVP):** add the leaf `data-srcmap`; Range-based
   precise scroll/highlight of the exact edited word. *Deferred.*
3. **Span-per-run (fallback):** only where a leaf can't carry a clean map
   (e.g. ligature/normalization splits). *Post-MVP.*

This means the §4 coalescing decision must **preserve an internal offset
table** on the coalesced leaf, not flatten it away, if rung 2 is wanted.

**Across a reconvert (full-doc MVP).** On edit we replace the preview DOM.
Re-locate the target by its `data-src` *source range* (the stable key,
viewport-independent) and restore scroll/selection from that — provenance-
driven scroll preservation, immune to both reflow and re-render.

**Both clients use this same preview code — including VSCode.** A VSCode
HTML preview is a **Webview** = a Chromium iframe with a full DOM, so the
Range API / `scrollIntoView` / `caretRangeFromPoint` all work there
unchanged; the preview-pane logic is *shared* with the ar5iv-editor. The
only difference is the **editor half**:

- *ar5iv-editor:* CodeMirror is itself in the DOM, so both panes share one
  document and JS coordinates them directly.
- *VSCode:* the source editor is **not** in the DOM — it's driven from the
  extension host via the VSCode API (`TextEditor.revealRange`,
  `.selection`, `TextDocument.offsetAt`/`positionAt`). The webview and host
  exchange `(source range ↔ element)` messages over
  `postMessage`/`onDidReceiveMessage`. (This is exactly how VSCode's own
  Markdown preview does editor↔webview scroll-sync; we are the
  locator-precise LaTeX analog.)

So "two thin clients on one substrate" is literal: identical preview code,
different editor binding, same `data-src`/`data-srcmap` contract. VSCode
packaging caveats (not blockers): webview **CSP** + `webview.asWebviewUri`
for our CSS/JS (RELEASE_CRITERIA §6); `caretRangeFromPoint` is non-standard
but safe in the Chromium webview (`caretPositionFromPoint` is the standard
fallback used by the ar5iv-editor in arbitrary browsers).

### 7. The crux: correctness obligations & how we verify

The encouraging part (client/preview) is *derived* — geometry computed on
demand (§6). The hard, critical part is the **engine**: generating locators
faithful to the TeX model and binding them accurately to the DOM. Two
obligations, each spanning multiple stages.

**A. Provenance must survive every pipeline stage.** Losing it anywhere
breaks the chain:

1. digestion → XML construction — the `absorb` hook (§2). *Designed.*
2. rewrite passes (ligatures, math-token declarations) — nodes move/merge;
   locators must follow the moved node.
3. **math parse (XMath → MathML)** — `latexml_math_parser` has **zero**
   locator awareness today; the Marpa actions build entirely new nodes. Each
   action would have to set its node's range to span the locators of the
   XMath tokens it consumed. **Largest single gap — and therefore deferred:
   the MVP treats math as opaque** (one line-range on the `ltx:Math`
   wrapper; see "MVP granularity"). Only attempt in-equation mapping with a
   clear, tested path.
4. serialization.
5. `latexml_post` XSLT (ltx XML → HTML) — `data-src`/`data-srcmap` must ride
   the `copy-attribute`/`add_attributes` path (`LaTeXML-common.xsl:327,390,
   481`), *including* reconstructed elements (math, tables) that don't merely
   copy their source attributes.

**B. User-source vs foreign-source (never scroll into a `.sty`).** Tag every
input file (SyncTeX-style table), but the editor/linter navigates only
within the *opened user document(s)*. Mark provenance
`kind = literal(user) | expanded | foreign`. Engine-injected tokens
(dump-loaded defs, package/class code, constructed tokens) are `foreign` or
synthetic — resolve those to the nearest *user-source* ancestor rather than
pointing at an uneditable file.

**C. TeX-model hazards for `from`/`to` accuracy** (the test surface): catcode
changes; active characters; `\input`/file-boundary mouth switches (tag
changes mid-stream); `\verb`/verbatim; `\scantokens`; `^^`-decoding and
line-ending normalization (the 2010 whitespace-offset drift); tokens with no
user source. §1 (start capture) and §3 (expansion) are the *mechanism*; this
is the list each must be tested against.

**D. How we earn confidence:**

- **Corpus round-trip gate** (extend §5): on a sample, assert literal ranges'
  source substring == visible text; every range within file bounds; child ⊆
  parent. Run it like the parity corpus, as a regression gate.
- **Invariants as debug asserts** (debug profile only): every emitted range
  non-empty, monotonic, within its mouth bounds, nested in its parent.
- **Golden/pinned tests** for the hard cases (math, `\item`, font-switch,
  `\input` boundary, verbatim) — same discipline as
  `tests/math/norm_kerned_delims`.
- **SyncTeX as an external oracle:** compile the same `.tex` with
  `pdflatex -synctex=1`, parse the `.synctex` (the cloned reference parser),
  and diff TeX's `(tag,line)` for a construct against ours. SyncTeX *is* the
  reference answer to "what source line did TeX attribute this to," so it
  cross-checks our line attribution wherever both produce a node.

## Status

**Prioritized showcase** (2026-05-24). Tier A is the near-term deliverable
and is parity-neutral, so it can proceed alongside the corpus mission.
Build order: locator substrate (Tier A + `--source-map`) → warm-state
conversion server (full-doc reconvert MVP) → the two clients (ar5iv-editor
web UI and VSCode extension) over the shared locator contract. Cross-refs:
[`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §9 (gates/context),
[`ISSUE_AUDIT.md`](ISSUE_AUDIT.md) #47/#92. Issue #199 (HTML-dialect
RelaxNG) gives the preview a validation contract.
