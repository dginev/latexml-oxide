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

## Design checklist (Perl's unsolved hard cases — handle explicitly)

- **Auto-created elements** (font-switch `ltx:text`): take the locator from
  the *causing* box/whatsit (e.g. `\rm`), not the surrounding text.
- **Attribute locators** (`\label`): attributes have no node to carry a
  `Locator` — decide a sidecar map rather than dropping the info.
- **Eating disorder:** record construct start; never trust "where the
  parser is now" as the span start.
- **Adjacent-token coalescing:** coalesce equal/adjacent locators instead of
  a span-per-token (Perl feared that overhead); keep it opt-in.
- **Opt-in only:** one switch gating both tracking and emission, off by
  default — see "Cost & the switch" above.

## Status

**Prioritized showcase** (2026-05-24). Tier A is the near-term deliverable
and is parity-neutral, so it can proceed alongside the corpus mission.
Build order: locator substrate (Tier A + `--source-map`) → warm-state
conversion server (full-doc reconvert MVP) → the two clients (ar5iv-editor
web UI and VSCode extension) over the shared locator contract. Cross-refs:
[`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §9 (gates/context),
[`ISSUE_AUDIT.md`](ISSUE_AUDIT.md) #47/#92. Issue #199 (HTML-dialect
RelaxNG) gives the preview a validation contract.
