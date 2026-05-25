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
- **We inherited the good parts.** The `Locator` data model is sound and box
  / whatsit / error nodes already carry a `Locator` — we build on it
  unchanged. (The 2009 XPointer *serialization* is the one part we don't
  reuse for the web attribute — no web-platform support, and latent in our
  port; see §0.)

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
- **Deferred until clearly needed:** column precision (now specified — §3.1,
  the `token-locators` compile flag), the per-leaf char-offset map (§6 rung
  2/`data-srcmap`), and in-equation provenance.

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

A locator is a source range whose two endpoints are each a `(file, line,
col)` triple. Our `Locator` (`common/locator.rs:17`) already carries this
data (`source`, `from_line;col`, `to_line;col`) — we reuse the model
unchanged. For the **web-facing attribute we deliberately do not reuse the
XPointer serialization** that `Locator::to_attribute()` emits (`:166`,
`…#textrange(from='l;c',…)` / `…#textpoint('l;c')`): XPointer is an XML-era
addressing scheme with **zero web-platform support** — no browser, devtools,
or JS API resolves an `xpointer()`/`textrange()` fragment (the only native
fragment resolvers are `#id` and the unrelated text-fragments `#:~:text=`),
so a client would regex-parse it for the same four integers either way. It is
also **latent** in our port — defined and unit-tested, but wired to no
emitted attribute. So the source-map feature serialises the briefer,
sibling-aligned `tag:l:c-tag:l:c` via a focused `Locator::to_sourcepos()`,
preserving identical information; `to_attribute()` is left untouched for any
future internal/Perl-parity use.

#### 0.1 The file table — a Source-Map-v3-flavoured header

The web platform's standard for "where did this generated output come from"
is **Source Map v3** (now **ECMA-426** at TC39), shared by JS bundlers and
CSS preprocessors (Sass/Less/PostCSS); CSS references it with a
`/*# sourceMappingURL=… */` comment. Source maps **reference every source
file by integer index into an ordered `sources` array** — exactly the numeric
file-id ↔ filename map we want — so we follow that convention for the
file dimension:

- **`tag` = index into `sources`.** The per-element `data-sourcepos` integer
  is a `sources` index, never a path — this is the source-map design, and the
  reason the inline markup stays tiny *and* anonymisable.
- **`sourceRoot`** — factor the common directory prefix out of `sources` (the
  spec's stated purpose, "removing repeated values"), so a deep project path
  is stored once rather than on every entry.
- **`sourcesContent`** (optional) — embed the original source text so the
  table is **self-contained** (works with no filesystem — our portability
  ethos, and the editor-server already holds the text). **Omitting**
  `sources`/`sourcesContent` is the anonymisation lever: ship structure-only.

**Where the decoder lives (decided 2026-05-24): out-of-band, never inlined.**
The per-element `data-sourcepos` tags ship *in* the output (our `mappings`
analogue); the `tag→file` `sources` decoder ships **out-of-band**, so the
shipped HTML/XML is anonymisable *by construction* — only opaque integers, no
filenames. Two channels, no new artifact:

- **`.log`** — latexml-oxide's existing conversion-metadata sink. With
  `--source-map` on, the engine writes one `source-map` record per source
  (`[tag] file`, array index = tag), gated (`converter.rs`, before the
  "Conversion complete" note). This is the decoder for CLI / file-based
  consumers — no standalone `.map` sidecar or `sourceMappingURL` plumbing,
  which would only earn their keep for the *generic* sourcemap toolchain
  (browser devtools) we don't target.
- **In-process** — embedders read the same table programmatically via
  `state::source_table_snapshot()` (reset per `from_config`, so it is exactly
  the current conversion's table — no cross-request/cross-user bleed on a
  shared worker). The ar5iv-editor server does this and forwards file
  **basenames** on its WebSocket envelope (`ConvertResponse.sources`, never the
  HTML); a VSCode client would carry the same over LSP.

(A standalone sidecar + `sourceMappingURL` remains a trivial future add if a
generic-tooling consumer ever needs it.)

We borrow the *header* (`sources`/`sourceRoot`/`sourcesContent`) but **not**
the VLQ `mappings` blob: our per-element `data-sourcepos` attributes *are* the
inline analogue of `mappings`. That is the one deliberate divergence — source
maps externalise **all** position data into one compact file optimised for
load-once, binary-search **stack-trace symbolication**, whereas we keep ranges
**inline on DOM nodes** because our consumer is **live DOM navigation**
(`querySelector`, the Range API, survival across reflow — §6). Different
consumer, different placement; same file-indexing convention. (Source maps
offer **no** attribute-naming guidance — they put nothing on output nodes — so
the attribute *name* stays with the cmark-gfm `data-sourcepos` lineage of §2.
The two standards are complementary: `data-sourcepos` = our mappings; the file
table = our `sources` header.)

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

**Status & priority (2026-05-25): this is the near-term lever, and it is *not yet
implemented*.** The Mouth's `get_locator` (`mouth.rs:147`) still derives `from`
heuristically (`from_column = if to_column ≥ max_col { 0 } else { to_column }`) —
the eating-disorder approximation; ranges only look right for single-line
constructs. Implementing §1 (one `last_token_start` field, captured in
`read_token` `:629` *after* inter-token skips; an open→close snapshot at each
digestion frame) gives **every existing element, leaves included, an accurate,
correctly-containing `(file;line;col)` range** — with **no `Token` change and no
markup change**. That accurate, *containing* element range is the precondition
for the content-window character localization in §2.1.

**Experiment 1 (2026-05-25) — the capture point matters.** A first spike landed
the `last_token_start` primitive (`mouth.rs`: captured in `read_token` at the
token-start point) + `gullet::get_locator_from_start()`, and snapshotted the
open locator at the **Constructor digest entry**, ranging it with the post-args
close. Result on `\section{First Section}` (line 12): `from` moved from col 1 to
**col 9** — the `{`, not `\section`. For a *macro-wrapped* construct the
element-building Constructor fires *after* the command and its opening brace are
consumed, so `last_token_start` there is the brace, not the user command.
**Conclusion:** the open snapshot must be taken when the *user command token* is
read — `stomach`'s invoke loop, right before `invoke_token`
(`stomach.rs:856`) — and threaded into digest as a **stack** of open-locators
(frames nest), not captured at the constructor itself. The `last_token_start`
mouth primitive and `get_locator_from_start()` stand; the open-locator stack is
the next step. (The spike's constructor wiring was reverted to keep the tree
green; the mouth/gullet primitives remain in place.)

**Experiment 2 (2026-05-25) — even the constructor *entry* is too late, which
reframes the whole lever.** Moving the snapshot to `invoke_primitive`'s entry
(*before* before-digest) still yielded `0:12:9` for `\section`. So by the time
the element-building Constructor digests, the gullet has already **expanded**
`\section` (it is a macro) and the file mouth already sits at the `{`. **For a
macro-defined command — i.e. most of LaTeX — there is no mouth position at
constructor-digest time equal to the user-command start;** recovering it needs
**expansion-chain provenance** (tag the expansion frame with the invocation
locator *when the macro expands*, and propagate it to the constructor — that is
§3 Tier B), not a mouth snapshot. The cheap §1 snapshot only nails constructs
invoked *directly* on their own token (rare in real LaTeX).

*The reframing this forces.* The §2.1 content-window client does **not** need
command-*tight* ranges — only correct *containing* ones (the search slice must
include the target). And the **existing heuristic ranges are already
whole-construct supersets** for single-line constructs: `\section{First Section}`
→ `0:12:1-0:12:24`, which *contains* the rendered title. So perfecting engine
range-accuracy is likely **not** the highest-value next step. The better
experiment is to **build the §2.1 client (DFS-descent + content-window) against
the *current* ranges and measure where they actually fail** — then spend engine
effort (§1 multi-line `to`, or §3 expansion-provenance for `from`) only on the
measured gaps. The two spikes' value was precisely to find this: the engine lever
is dearer than it looked, and the client is the cheaper place to learn what
accuracy is truly required. (The committed `last_token_start` primitive remains
useful for the direct-construct case and for error locators; it is not wasted.)

**Experiment 3 (2026-05-25) — assembly from digested children: no regression,
but it does not fix the target.** Replaced the constructor's mouth-snapshot with
a union of its *digested children's* locators (`Tbox`/`Whatsit`/`List` `.locator`,
which survive expansion). All `52_source_map` tests pass and the structural
golden is unchanged. But a mid-line probe — `Some \textbf{bold} and \emph{italic}
words here.` — exposed the failure: the bold `ltx:text` got `0:3:19` and the emph
`0:3:37`, **point locators at the column *after* each construct**, not the content
spans (`bold` = cols 14–17, `italic` = 30–35). Cause: assembly is only as accurate
as the **leaf** `Tbox` locators, and a leaf Tbox takes its locator from
`get_locator()` at *build* time — which for argument text is **after `readBalanced`
has consumed the whole `{…}`** (the eating-disorder END column). The per-token
starts (14,15,16,17 for `bold`) existed transiently in the mouth but were
overwritten by digest time. **Decisive conclusion: the leaf bottleneck cannot be
solved by re-derivation or assembly — the source position must travel *with the
token*.** This confirms the original anchoring intuition empirically. Per the
plan we proceed to **handle-on-Token** (§3.1.1 option 2): a `u32` origin handle on
`Token` (8→12, behind the `token-locators` compile flag) indexing a per-conversion
side arena, set at `read_token`, so a digested run recovers its true span from its
constituent tokens. Assembly-from-children (Experiment 3) then returns as the
*construct*-level rule layered atop accurate leaves. (The Experiment 3 wiring was
reverted to keep the baseline clean.)

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
  `box_to_absorb.get_locator().to_sourcepos()`. Also stamp in `open_element`
  (`:916`) / `insert_element` (`:835`) from the current `box_to_absorb`
  locator so constructor-opened elements (which bypass `absorb`'s box arms)
  are covered.
- **Attribute name — `data-sourcepos` (decided 2026-05-24).** Adopt the
  cmark-gfm / GitHub / GitLab convention for exactly this task (markup
  source → HTML, source ranges on block elements). cmark-gfm is our
  engine's spiritual sibling — *LaTeXML : LaTeX :: cmark-gfm : Markdown* —
  so wearing its attribute is what "friendly to the web ecosystem" means
  here, not a name we'd be coining. Explicitly **not** `data-src`: that is
  the de-facto lazy-load idiom (lazysizes' `data-src`→`src` swap) and would
  collide. The file path is kept **out** of the value — the React/Vue
  dev-inspectors that inline paths (`data-v-inspector="file:line:col"`) are
  the §6 / RELEASE_CRITERIA-§6 path-leak anti-pattern. Present *only* under
  the switch.
- **Attribute value — a file-tagged extension of cmark's `line:col-line:col`.**
  The source **file is first-class**: each endpoint is a full `(file, line,
  col)` triple, so the value is `<tag>:<l>:<c>-<tag>:<l>:<c>` (e.g.
  `0:12:1-0:12:240`). This is the one deliberate superset of cmark, whose
  single-document model has *no* file axis — but LaTeX projects are
  multi-file (`\input`/`\include`) and the editor must scroll the **exact**
  file, so file belongs in the triple, not as an afterthought. We keep
  cmark's recognizable name and `l:c…` shape and prefix the integer file
  **tag** to each endpoint. Splitting the value on `-` yields two
  `tag:line:col` triples — unambiguous, since every component is a
  non-negative int. This maps 1:1 onto `Locator` (`source`, `from_line;col →
  to_line;col`); `Locator` today carries a single `source` and `new_range`
  rejects cross-file ranges, so the two tags are currently always equal, yet
  the endpoint-complete format future-proofs a per-endpoint-source `Locator`
  (the `locator.rs` source-ownership TODO). It also stays a strict superset
  of VSCode's line-only `data-line` (a line-only client reads the first
  line).
- **Line authoritative; column a best-effort within-line refinement (decided
  2026-05-24, refined for nested constructs).** The `:col` components ride
  along (cmark-gfm shape). Construct-start columns are heuristic under macro
  expansion (Bruce #101), so the **line stays the authoritative axis** — but
  the consumer *does* use the column as a tie-break **within** the anchor line,
  to descend from a containing paragraph to the exact inline construct (e.g. a
  `\textbf` span) the caret sits in. It can only narrow to a descendant: if a
  start column is heuristically ahead of the caret, that construct drops out of
  the anchor set and the line-level ancestor is used — never a wrong line.
- **File resolution — integer tag + doc-level `tag→file` table (SyncTeX
  `Input:` preamble analog).** The attribute never inlines a path: it
  carries a small integer `tag` resolved to its file via a once-per-document
  table. That gives exact-file sync in multi-file projects *and* avoids both
  path leakage (RELEASE_CRITERIA §6) and the markup bloat of repeating a
  path on every element. Tag `0` = the main user document; each
  `\input`/`\include` mouth allocates the next tag as it opens (B below
  marks user vs foreign so the editor never scrolls into a `.sty`). Because
  the per-element value is a **bare integer**, output shipped *without* the
  table is inherently **anonymised**: a consumer lacking the map can still
  use the structure (ranges nest, endpoints order) but cannot recover any
  filename or local path. This is the same indirection the Source Map v3
  `sources` array uses — see §0.1. **The table itself is serialised
  out-of-band** (§0.1: the `.log` for file consumers, `source_table_snapshot()`
  / the WS envelope for in-process ones), **never inlined into the output** —
  so "anonymised without the table" is the *default*, not an opt-in.
- **Gate:** off by default; when off, §1 capture and this stamping are
  both skipped (no side cost — Cost & the switch).

### 2.1 The client model: DFS-descent + content-window character localization (decided 2026-05-25)

Markup is capped at LaTeXML's narrative-semantics schema: a `data-sourcepos`
range on *existing* elements (leaves included, via §1), and **nothing heavier** —
no per-char `data-srcmap`, no span-splitting. Character-level sync is therefore a
**client** problem solved against accurate element ranges. Inspiration:
[Playwright locators](https://playwright.dev/docs/locators) — *relative* (locate
by surrounding content, not absolute offsets), *lazy* (re-evaluated against the
current text, no precomputed map), and *strict* (resolve to exactly one position,
else widen / fall back). Two phases, both directions:

**Phase 1 — DFS-descent to the tightest containing leaf.** Ranges nest (§5: child
⊆ parent), so from the root descend into the child whose `(line,col)` range
contains the target, recursively, to the deepest element that still contains it.
O(depth), no per-node scan. Requires *correct, containing* ranges — **§1 is the
precondition** (today's heuristic can yield a range that does not contain its own
content, landing the descent wrong).

**Phase 2 — locate the character by its left/right textual windows.** Within the
leaf, do **not** interpolate an offset (the ceiling forbids the per-char map that
would make interpolation exact, and interpolation drifts on every `---`,
collapsed space, and splice). Instead match *content context*: take a short
window of text on each side of the target and find the position whose neighbours
align — robust to source↔render character-count mismatch because it matches *what
is there*, not *how many*.

- *Reverse (preview click → editor caret), extends `bindPreviewSourceNav`:*
  `caretPositionFromPoint` / `caretRangeFromPoint` (on the preview **shadow
  root**) → `(textNode, offset)`; DFS gives the leaf; read its `data-sourcepos`
  source span; align the rendered left/right windows against that **source slice**
  to find `(line,col)`; `editor.revealPosition`.
- *Forward (editor caret → preview), extends `scrollPreviewToSource`:* DFS by the
  caret's `(line,col)` to the leaf; align the **source** left/right windows
  against the leaf's rendered text to find the offset; a DOM `Range` at that
  offset → `getClientRects()` → scroll + flash.

**Obligations (the correctness surface):**

1. **Containment is a hard precondition.** If the leaf range doesn't contain the
   target, the slice excludes it and the match fails — §1 + a debug-assert (child
   ⊆ parent; range contains its rendered text) guard this.
2. **Source-vs-rendered alignment, not string search.** A leaf's source slice
   contains markup (`\emph{b}` between "a" and "c") absent from the rendered text;
   the match must align *tolerating* non-literal stretches (fuzzy / subsequence),
   not assume equality. The main implementation subtlety.
3. **Strictness / uniqueness (Playwright).** If a window matches more than one slice
   position, **grow it** until unique; if still ambiguous at the slice bounds, fall
   back to the construct-level reveal (today's behavior). Never guess.
4. **Bidirectional consistency.** Forward and reverse must be inverses sharing one
   alignment routine, or a glyph round-trips to the wrong place — pin as a tested
   invariant.
5. **Non-text leaves** (math, images, `\ref`→"Fig 3", generated content) have no
   literal window — resolve to the leaf's range start (construct-level). Honest.

*Direction asymmetry (product judgement):* glyph precision matters more in
**reverse** (land on the char I clicked) than **forward** (the reading eye wants
the *region*) — the forward client may deliberately flash the enclosing leaf
rather than a single glyph.

Cost: per-event one DFS (O(depth)) + one bounded alignment over a single leaf's
slice — trivial for both per-click (reverse) and per-conversion (forward).

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

### 3.1 Tier B′ — in-band per-token start, behind a `token-locators` compile flag (proposed, then deferred — 2026-05-25)

**Status: deferred the same day (2026-05-25), superseded as the next step by
§1 + a content-window client (§2.1).** Two reasons, both decisive: (a) the
**markup ceiling** — we prohibit any markup beyond LaTeXML's narrative-semantics
schema (only `data-sourcepos` on *existing* elements; no `data-srcmap`, no
span-splitting), and per-token tightness cannot be *expressed* without exactly
that prohibited leaf markup; (b) the cheaper **§1 + content-window** path reaches
glyph-level sync without touching `Token` at all. This subsection is kept as the
considered design — do not implement it until §1 + the client are built and
*measured* insufficient. §3.1.1 records why all three engine variants are
deferred.

§3's out-of-band stack is the right home for the *expansion chain* (the
literal-vs-expanded `kind`, the macro-origin trace — #92's payoff). But on its
own it does **not** sharpen **column** accuracy for literal text that reaches a
construct through a *macro argument* — the Bruce #101 sore spot: `\textbf{Hello}`
digests with every "Hello" char box reporting the *construct's end column*
(verified, see "Bruce's wall" below), because the chars' source columns are gone
before digestion. §1's open→close snapshot fixes columns for text typed
*directly* in the stream; it cannot for argument text, whose tokens are collected
and replayed.

The cleanest fix is to let each token **carry its own source start**, so the
position rides the token through argument collection, put-back, and `Copy` with
**no parallel bookkeeping** — the constructor then computes a Tbox/text-run's
true span from its *contributing tokens'* starts, not from a look-ahead
approximation. The §3 objection ("never widen the 8-byte `Token`") is a
*shipping* invariant, not an absolute: a **compile-time feature** preserves it
for every normal/corpus/parity/distribution build and pays the width only in an
explicit precision build.

**Decision.**

- **`token-locators` Cargo feature** on `latexml_core` (re-exposed by
  `latexml_oxide`). A *compile* flag, not runtime: it changes the size/layout of
  a `Copy` type, which a runtime switch cannot. Off by default.
- **Payload — start-only, 3 fields** (`TokenStart { source: SymStr, line: u32,
  col: u32 }`, 12 bytes; `Token` 8→20). A token's **end is derived** at digestion
  (start + consumed length, or the next significant token's start), so the
  redundant `to_*` of a full per-token `Locator` is not stored. `NONE` sentinel =
  `source` empty / `line == 0` — the same "no real position" test §2 already uses
  (`loc.from_line == 0`).
- **Single populating site:** `mouth.rs::read_token` (`:628`) stamps
  `(source, lineno, colno)` captured **after** inter-token skips, **before**
  consuming the token's chars (§1, exactly). **Every other construction site gets
  `NONE`** via a `#[cfg(feature = "token-locators")] loc: TokenStart::NONE`
  field-init — confirmed to compile both ways; a scratch mirror of `Token` gives
  `size_of` **8 off, 20 on** (the 8-byte invariant is provably untouched when
  off).
- **Runtime `--source-map` still gates emission.** The feature only makes the
  field *exist* and be *captured*; whether a `data-sourcepos` is stamped remains
  the runtime switch. A `token-locators` binary run **without** `--source-map`
  behaves as today (the field is filled but nothing is emitted).

**Relationship to the rest of the plan.** This is the "column precision" rung the
MVP explicitly deferred (§"MVP granularity"); it does **not** replace §3 — the
out-of-band frame still owns the expansion chain and the `kind` marker. They
compose: in-band start = exact *literal* columns; out-of-band frame = *provenance*
of expanded text. Math stays opaque (§7 A.3) regardless.

**Emergent property (and its caveat).** Macro *body* tokens are read at
`\def`-time, so under the feature they carry their **definition-site** start;
literal tokens carry their **invocation-site** start. That yields §3's
literal-vs-expanded distinction almost for free — but it *conflates* the two
without an explicit bit (the consumer would have to infer "is this start in the
file I'm editing"). An explicit `kind` still belongs in the §3 frame; do not lean
on the emergent signal for linting correctness.

**Cost & risks.**
- *Default build: zero* — not one of the ~150 literal sites is touched when the
  feature is off; `Token` stays 8 bytes.
- *Feature build:* `Token` 8→20 (a `Copy` type cloned/moved millions of times),
  plus ~150 literal sites (**109** outside `token.rs` + the `T_*!` macros/statics
  inside it) each needing the cfg field-init. Acceptable for an opt-in precision
  build, but **a feature-on CI lane is required** so a newly-added raw
  `Token { … }` literal can't silently break the feature build.
- *`SymStr` source per token* keeps the existing arena model; tag resolution
  stays the document-level `source_tag()` table (§0.1) — unchanged.

**Verification (extends §7).** The §7.C column hazards are exactly the test
surface: `^^`-decoding / line-ending offset drift (the 2010 bug), catcode /
active-char changes, `\verb`/verbatim, the `\input` mouth-boundary tag change,
and tokens with no user source (`NONE`). Earn confidence with: golden tests
pinning **exact** columns on argument text (`\textbf{Hello}`, nested
`\emph{a \textbf{b}}`, a cross-line `\textbf{…\n…}`) — a feature-gated variant of
`52_source_map.rs`'s currently line-accurate golden; debug-asserts that every
emitted range is non-empty, monotonic, and within its mouth bounds; and the §7.D
corpus round-trip "literal range's source substring == visible text," which
becomes a *column-exact* assertion under the feature.

**Non-goals here.** Expansion-chain / `kind` marker (§3), in-equation columns
(math stays opaque), and any change to the default 8-byte build.

#### 3.1.1 Alternatives considered, and how each serves the two frontend directions (2026-05-25)

*The impossibility that frames the choice.* `Token` is a pure `Copy` value with
**no identity** (`token.rs:286`; `PartialEq` is by *meaning* — text+code), so a
provenance side table **keyed by token is impossible**: once argument chars are
collected into `Tokens(Vec<Token>)` (`tokens.rs:38`) and replayed, nothing
distinguishes them. Any approach at the *same* token-exact precision must
attach position to something with identity — the `Token`, or the heap container
it rides in. Three are viable (all gated; default build unchanged):

1. **In-band full start** (this section's decision) — `Token` 8→20; position in
   the token. Simplest, zero indirection, obviously correct.
2. **In-band provenance *handle*** — `Token` 8→**12** (one `u32` id) indexing a
   per-conversion side arena of `TokenStart`. Same precision, half the hot-type
   bloat, and the arena is the natural home for §3's expansion chain + `kind` —
   so it **unifies Tier B and B′** into one mechanism. Costs an indirection per
   read and a side arena (bounded by tokens read; freed at end).
3. **Argument-granular container locator** — **no `Token` change**.
   `read_balanced` (`gullet.rs:762`) sits between the open/close mouth
   positions — the commented-out `startloc` at `:772` is the exact hook — so
   capture open→close as a range on the (today locator-less) `Tokens`/`ArgWrap`
   container; the constructor's Tbox inherits it. Cheapest by far.

*Why all three are deferred under the markup ceiling.* Per-token precision (1/2)
buys accuracy the permitted markup cannot carry: a bare text run can't get its own
range without a wrapper element, which the ceiling forbids — so the finest
*expressible* range is the existing leaf *element*, which **§1 already delivers**
without touching `Token`. The character work then lives in the client (§2.1), not
the markup. Hence:

- **Near-term (the experiment):** §1 (accurate, containing element ranges) + §2.1
  (DFS-descent + content-window client). No `Token` change, no markup change.
- **Deferred (this subsection):** per-token in-band start. Revisit *only* if
  measured — if §2.1's content-window matching cannot disambiguate within some
  leaf's slice (e.g. pathological repetition that defeats window-growth), per-token
  data could shrink the slice — and even then it would feed the client via the
  out-of-band table, **not** new markup. Of the three, prefer the **handle (2)**
  (leaner token, unifies with §3).

The shipped client-fingerprint heuristic (see Status) already gives *word*-level
reverse at zero engine cost — §2.1 must clearly beat it (window-exact, both
directions) to earn its keep.

#### 3.1.2 article.tex locator audit (2026-05-25, token-locators build)

All 133 `data:sourcepos` locators from `tests/structure/article.tex` converted
with `--source-map` under the `token-locators` build, compared char-for-char to
the source. The result cleanly splits on **how a construct's content reaches
digestion**:

**Accurate — content-exact spans (inline-processed content).** The token-origin
→ leaf-`Tbox` → child-assembly path delivers exact ranges wherever content is
digested *inline*:
- `\author{John Q.~Author \and Someone Else}` → `personname` `0:3:9-0:3:23`
  ("John Q.~Author") and `0:3:29-0:3:40` ("Someone Else") — both exact.
- `\emph{italic}` → `0:3:30-0:3:35`, `\textbf{bold}` → `0:3:14` (probe).
- list `item` bodies, description-label `text` (`0:59:7` = the `[A thing]` label).

**Discrepant — fall back to `get_locator()` (the eating-disorder position).**
Every discrepancy has *one* root cause: child-assembly finds **no located
leaves** and falls back to the post-expansion mouth position. That happens
exactly when content is **stored / replayed / deferred**, or when the
construct's own start is **consumed by expansion**:
- **Sectioning** (`section`/`title` → `0:12:1-0:12:24`): line-accurate but the
  *whole* `\section{…}` line, not the title's `10-22`. The title is stored
  (TOC/runninghead) and replayed without origins → fallback. Matches the pinned
  golden; acceptable (line-accurate).
- **Environments** (`figure` → `0:23:7`, `table` → `0:30:7`): point at the `{`
  of the *inner* `\begin{centering}`, not `\begin{figure}` (line 22). The
  `\begin` start is consumed by expansion before the float constructor digests
  (Experiment 2's wall).
- **Float captions** (`caption` → `0:27:5`): **wrong line** — line 27 is
  `\end{figure}`, the caption is line 26. The float defers caption digestion to
  `\end`, where `get_locator()` points; its content carries no origins through
  the deferral. The one genuinely wrong-*line* case.
- **Plain paragraphs** (`p`/`para` → `0:13:1-0:13:1`): a start-*point*, not a
  span — the paragraph takes its first absorbed box's locator and never extends
  it. Start-accurate.

**Correction path.** These are precisely the **§3 expansion/replay-provenance**
territory, confirming the tiering: the cheap token-origin mechanism nails the
inline/leaf cases (the editor/linter's common path) and the rest need
(a) origin-preserving token **storage/replay** (sectioning, `\caption`),
(b) §3 **expansion-frame** provenance to recover a `\begin`/command start, and
(c) a paragraph locator computed as the **span of its absorbed content**. Until
then the fallback stays *line*-accurate except for deferred floats — so the audit
also pins `\caption`-in-float as the highest-priority correctness fix.

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
  style); per-element `data-sourcepos` carries `tag:l:c-tag:l:c` (file
  first-class in each endpoint; integer tag, no inlined paths).
- **MVP fixture:** `latexml_oxide/tests/structure/article.tex` with
  `--source-map` on — a structural article (sections/paragraphs/lists,
  math-light) pinned as a golden of `data-sourcepos` attributes.
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

1. **Element-level (Tier A MVP):** `data-sourcepos` range on each element;
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
Re-locate the target by its `data-sourcepos` *source range* (the stable key,
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
different editor binding, same `data-sourcepos`/`data-srcmap` contract. VSCode
packaging caveats (not blockers): webview **CSP** + `webview.asWebviewUri`
for our CSS/JS (RELEASE_CRITERIA §6); `caretRangeFromPoint` is non-standard
but safe in the Chromium webview (`caretPositionFromPoint` is the standard
fallback used by the ar5iv-editor in arbitrary browsers).

### 7. The crux: correctness obligations & how we verify

The encouraging part (client/preview) is *derived* — geometry computed on
demand (§6). The hard, critical part is the **engine**, and it reduces to
two pieces:

1. **Compute the locator correctly** (faithful to the TeX model — §1), and
2. **Get it onto the most accurate DOM node** (§2 + obligation A below).

We **build on LaTeXML's existing `Locator` model unchanged**
(`common/locator.rs`): `Locator`, `new_range`, and the `.locator` fields
already carried by `Tbox`/`Whatsit`/`List` are the foundation; the source-map
adds exactly one focused serialiser (`to_sourcepos`, §0) and otherwise uses
them unchanged. The work is *using* them correctly and propagating them — not
redefining the model. (SyncTeX was conceptual grounding for §1's
start-invariant and the line-granularity choice; it is **not** a runtime
dependency or a required comparison.)

The two pieces span multiple stages, hence two obligations:

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
5. `latexml_post` XSLT (ltx XML → HTML) — `data-sourcepos`/`data-srcmap` must ride
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

These are **self-contained** — they validate our locators against the
source text directly, on our own model, with no external tool. (A one-off
`pdflatex -synctex=1` diff of `(tag,line)` is a *handy* bring-up sanity
check, but not a required gate and not an ongoing comparison.)

## Status

**Prioritized showcase** (2026-05-24). Tier A is the near-term deliverable
and is parity-neutral, so it can proceed alongside the corpus mission.
Build order: locator substrate (Tier A + `--source-map`) → warm-state
conversion server (full-doc reconvert MVP) → the two clients (ar5iv-editor
web UI and VSCode extension) over the shared locator contract. Cross-refs:
[`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §9 (gates/context),
[`ISSUE_AUDIT.md`](ISSUE_AUDIT.md) #47/#92. Issue #199 (HTML-dialect
RelaxNG) gives the preview a validation contract.

**As-built — ar5iv-editor source→preview scroll MVP (2026-05-24).** The first
client consuming the substrate is live. End-to-end contract, single direction
(source → preview), line-granular, single-file-clean:

1. **Engine** (`feat/source-locators`): `--source-map` stamps anonymous
   `data:sourcepos="tag:l:c[-…]"` (→ HTML `data-sourcepos` via the XSLT
   `copy_foreign_attributes` path); the `tag→file` decoder goes to the `.log`
   and `state::source_table_snapshot()` — never the HTML (§0.1). Gated;
   corpus path byte-identical.
2. **ar5iv server**: `source_map: Some(true)` always on; after convert it
   reads the snapshot and forwards file **basenames** as
   `ConvertResponse.sources` (WS envelope, out-of-band).
3. **ar5iv frontend**: each *edit-driven* convert records the caret's 1-based
   line (`editor.getCursorLine()`) against its request id; when that render
   lands, `preview.ts::scrollPreviewToSource` resolves `active_file → tag` via
   `sources`, picks the best element by the **anchor rule** (below), then
   `scrollIntoView({block:"center"})` + a brief accent flash. Boot / example-
   swap / file-navigation converts deliberately don't scroll.

**Selection rule — tightest range that *contains* the caret, else the
reading-order anchor.** When an element's range genuinely contains the caret
`(line,col)` (well-ranged constructs — a section title `0:490:1-0:490:26`, an
equation), pick the **tightest** such; on an identical range the **deeper**
element wins (the `<h2>` over its wrapping `<section>`, which currently shares
the heading's range — see the engine gap below). Otherwise — collapsed-point
inline constructs, which contain nothing — fall back to the construct that most
recently *started* at or before the caret in source reading order `(line,col)`;
each stamped element gets a single ordering key, lower wins: `start ≤ caret → [0, -fromLine,
-fromCol, span]` (an *anchor*: greatest start at/before the caret — latest
line, then latest column — then tightest range), else `[1, fromLine, fromCol,
span]` (an *after*: first construct beyond the caret — fallback only). One key
subsumes four behaviours that a "tightest-containing-range" rule gets wrong:

- *Containment* — the element you edit *inside* has the greatest start ≤ caret,
  so it beats its own lower-starting ancestors. No `contains` test.
- *Soft recovery (the user's "no node for line N")* — a blank line in a gap, a
  blank line inside a big container (where "tightest containing" would wrongly
  pick the whole section), and **latexml's error-truncated tail** (no node for
  N) all degrade to the nearest *preceding* construct automatically — never a
  freeze, never a coarse jump to the top.
- *Nested constructs on one line (the user's "subtree pointing to the same
  line")* — the **column** breaks the line tie and descends to the exact inline
  construct the caret sits in (paragraph `509:2` vs its `\textbf` child
  `509:47`, caret at `509:50` → the bold span). Line stays authoritative; the
  column can only narrow to a descendant, with safe fallback to the line-level
  ancestor when a start column is heuristically ahead of the caret.
- *Coincident starts* — when two nested nodes share an exact `(line, col)`
  start, the smallest `span` wins = the innermost. `span` is free from the
  parse, so **no DOM-depth walk** is needed.

**Performance — one pass, no layout reads.** A single `querySelectorAll` +
linear scan keeping the best key (integer compares only); the scroll/flash are
the only layout-touching work and are deferred to one `requestAnimationFrame`,
so the selection never forces a reflow and the UI doesn't lag. A second running
best that *ignores* the tag is kept as the soft-recovery fallback for a
mis-resolved tag.

**Word-level inline precision — content-fingerprint (landed 2026-05-24).** A
construct's source columns are only reliable for text typed *directly* in the
stream: a `\textbf{…}` (macro-argument) run has its columns destroyed before
the stomach runs — the argument is read into a position-less token list, so
**every** char box reports the construct's *end* column (verified: "Hello
there" → cols 3:2…3:12, but `\textbf{bold words here}` → all 3:37). Recovering
those columns engine-side needs token-level provenance (Tier B). Near-term, the
client recovers word-level precision *without* columns: capture the word under
the caret, then **scope to the caret's enclosing block** (the column anchor's
nearest `.ltx_p`/`.ltx_para`/list-item/cell) and pick the tightest element in it
whose **rendered text** contains the word (shortest `textContent`). Block-scope
— not line — is essential: a `\textbf{…}` that wraps across source lines has its
locator point on a *later* line than the caret, so a line filter misses it
entirely (verified: editing line 467 of a wrapped bold, the only line-467 element
was the `\citep`, so the word fell through to it); but the bold span is a DOM
child of the same paragraph regardless of wrapping. Literal text only — a macro
arg that doesn't render verbatim (`\ref` → "Fig 3") won't match and the column
anchor stands, so it never does worse.

**Tier-B bookmark (decided 2026-05-24): long-term goal, but NOT in the
`Token`.** Storing the 5 locator numbers on every `Token` is rejected — `Token`
identity (`sym`+`catcode`) drives macro dispatch / catcode lookup / `\ifx` /
interning / shared macro bodies, so source position *cannot* enter `Eq`/`Hash`
and is necessarily out-of-identity metadata; in-`Token` storage would also
collapse constant-token sharing (clone+restamp every macro-body expansion),
~4× the hottest value (8→~32 B), and tax cache + by-value copies on every run
for provenance almost none consume. Tier B, when taken, captures `(line,col)`
at the single `mouth::read_token` point into an **out-of-band** side structure
(parallel per-source-mouth locator stream / mouth+offset map), only for
user-source mouths — `Token` stays 8 bytes (RELEASE_CRITERIA §9).

Consumer contract for the next client (VSCode): *cursor `(line, col[, word])` →
tag (via the out-of-band `sources` table) → anchor element (reading-order rule
above), refined by the content-fingerprint when a distinctive word is present →
reveal*. Line authoritative; column refines within the line (§2); word
disambiguates where columns are destroyed. Deferred: column *accuracy* (Tier B,
out-of-band only), reverse direction (preview-click → source), multi-file
beyond basename matching, region-incremental reconvert.

**Known engine gap — `<section>` locator spans only its heading.** A section
element is stamped `0:L:1-0:L:C` covering just its `\section{…}` title line (the
same range as its `<h2>`), because the locator is taken from the heading
construct at *open* and never extended to cover the body at section auto-close.
Harmless for the scroll MVP — body edits self-locate to their own paragraphs,
and the client prefers the deeper `<h2>` on the shared range — but wrong for
"select the whole section" and any consumer that trusts the section's `to`.
Fixing it means extending the element's locator at section auto-close (gated,
sensitive); left as a follow-up. The same shape affects other auto-closed
containers (`enumerate`/`itemize`/`description` bodies).
