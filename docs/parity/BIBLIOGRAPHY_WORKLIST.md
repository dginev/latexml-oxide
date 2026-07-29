# Bibliography worklist — targets + the MakeBibliography re-port

> Lifted out of `docs/SYNC_STATUS.md` on 2026-07-25. Two related bodies of
> work: the surveyed "missing references" target list (2026-07-12) and the
> MakeBibliography full-parity re-port (user directive 2026-07-04 — reuse TeX
> interpretation, no special-case parser).
>
> **State, 2026-07-29.** The MakeBibliography re-port is **DONE** — items 1, 2
> and 3 all landed. The only open work in this file is the missing-references
> target list. The `.bib`-as-DATA family closed as divergences
> **#73 #74 #75 #78 #79 #80**; item 2's collation approximation is **#84** and
> its one non-gap is **KNOWN_PERL_ERRORS #67**.
>
> **Two reading rules for this file.** (1) The three **INTERIM** blocks under the
> re-port are HISTORY — every identifier they name was deleted with the string
> route; they are kept for the properties, witnesses and traps that outlive the
> code, and they carry their own banner. (2) **Re-measure any error count dated
> before 2026-07-27**: divergence #80 stopped digesting uncited entries, so a
> count can have dropped without anything being fixed.

### The governing design tension: two regimes collapsed into one pass

Read this before touching anything in the `.bib` path. It is the frame that
decides what is a bug and what is correct behaviour, and every bibliography
defect found so far is a violation of it.

The real toolchain has **two regimes separated by a hard boundary**:

| | input → tool | what the bytes ARE | consequences |
|---|---|---|---|
| **A. Data** | `.bib` → `bibtex(1)` | data | `%` is not a comment, `&` is not an alignment tab, `_` is not a subscript. Braces ARE significant (delimit fields, protect case). `\`-sequences pass through as inert text. The `.bst` program selects which fields survive. |
| **B. TeX** | `.bbl` → `pdflatex` | TeX source | `\emph`, accents, `$…$` mean what they mean. |

latexml-oxide runs **one pass over the live core State** — no `bibtex`
subprocess, no second `pdflatex` invocation, no `.bbl` on disk. Both regimes
therefore happen in the same place, and the discipline is:

> **Be `bibtex` first, then be `pdflatex` on the `.bbl` you just synthesized.**

Collapsing the boundary is what produces the whole defect family: field bytes
that should still have been *data* arrive at the tokenizer as *TeX*.

**Three consequences that follow directly, and are the reason to keep this frame
explicit rather than fixing symptoms:**

1. **Field selection is part of the emulation, not a rendering convenience.**
   No standard `.bst` (plain/unsrt/alpha/abbrv) declares `abstract`, `keywords`
   or `contents` in its `ENTRY` list, so in the real pipeline those fields never
   cross into regime B at all. That — not "nothing renders `ltx:bib-extract`" —
   is the principled justification for OXIDIZED_DESIGN **#73** reading them
   verbatim. The weaker rendering-based argument in that entry predates this
   frame. **Entry selection is the same rule one level up:** `bibtex(1)` copies
   only the `.aux`'s cited keys (plus `crossref` targets, plus `\nocite{*}`) into
   the `.bbl`, so an uncited entry never crosses into regime B either —
   OXIDIZED_DESIGN **#80**, and the section below.

2. **The right seam for specials is an escape at the A→B boundary, not catcode
   suppression during digestion.** When a field's data-string crosses into the
   TeX regime, emit what a careful `.bst` author would have written: `%`→`\%`,
   `&`→`\&`, `#`→`\#`, `_`→`\_`. Legitimate TeX in the field (`\emph{...}`,
   `$x_1+x_2$`, `{\v S}pakov`) is already valid and passes through untouched —
   correct by construction, instead of fighting to keep it alive while
   suppressing catcodes. Authorized as surpass-Perl **and** surpass-pdflatex
   (user decision 2026-07-26): we read `.bib` directly, so we are the component
   that decides what reaches the tokenizer, and the real toolchain's breakage on
   these characters is a property of that toolchain, not a semantic to reproduce.

3. **The balance is delicate in exactly two places**, and both need guards:
   - **Math-awareness** — `_` inside `$…$` must stay a subscript
     (`title = {Bounds on $x_1+x_2$}`) while `AT1G01010_v2` becomes `\_`.
   - **Idempotency** — most real `.bib` files already contain correctly-escaped
     `\&`, `\%`, `\_`. Escaping blindly turns `\&` into `\\&`, a line break
     followed by an ampersand. Only a special NOT already preceded by an
     escaping backslash may be escaped, and `\\&` (a real `\\` then `&`) is the
     tricky case. Mixed conventions inside one file are real: witness
     `booktitle = {... Medical Measurements \&amp; Applications ...}`.

#### The two treatments, and which mechanism belongs to each (settled 2026-07-26)

An earlier draft of this section proposed choosing ONE seam per character by
"blast radius". That was wrong, and the correction is worth keeping because it
is the whole design: **both treatments apply, at different points.** The real
`pdflatex -> bibtex -> pdflatex` dynamic handles these characters twice, and
collapsing the pipeline collapsed both handlings into one.

| | treatment | what it does | mechanism |
|---|---|---|---|
| **1** | reading the `.bib` — *be `bibtex`* | field bytes are inert data: `%` is not a comment, `&` not an alignment tab, `#` not a parameter, `_` not a subscript, `^` not a superscript. Only braces and the entry/field delimiters are structural. **The text is not altered** — the stored value keeps its exact bytes. | a per-Mouth opt-in property on the entry mouth, plus a companion (`tokenize_bib_literal`) for handlers that re-read the raw field |
| **2** | synthesizing the `.bbl` and digesting it — *be `pdflatex` pass 2* | the content must now be valid TeX, and we are the ones writing the `.bbl`, so we escape what the author plainly meant literally | `escape_bib_data_specials` at the data→TeX boundary: math spans skipped, idempotent |

**The corollary that makes the exclusion list principled.** A handler that
consumes the field's characters *itself*, under its own catcode regime — `url`'s
`Semiverbatim` href, `doi` — is still in **treatment 1, still on data**. It must
receive the *unescaped* value. Escaping there is what would plant a literal
`\%` inside a URL. `reads_field_raw` / `bib_field_source` implement this; that
is their justification, not a workaround for a symptom.

**Treatment 2 has three seams, not one.** `\ProcessBibTeXEntry`'s entry line is
the obvious one; `\bib@@title` and `\bib@@pages` both **re-read the RAW field**
rather than the value handed to them, so escaping only the entry line silently
misses every title. Measured: fixing only the mouth took witness `2605.02131`
from 28 errors to **37**. Any change here must cover all three, plus the
name-, date- and MR/Zbl-assembly sites that share that path.

**Treatment-1 changes must stay opt-in per Mouth, never a State catcode.** A
State catcode is inherited by a `.sty` raw-load triggered from inside a field
handler, where `%` must still be a comment.

**Correction to the table above, measured while implementing it: `_` and `^`
belong to treatment 2 ONLY.** The row is right that a `_` in a field is data,
but wrong that treatment 1 is where that gets expressed. A catcode is decided at
tokenization, before anything knows whether the character is inside `$…$`, and
a subscript in a title's math (`title = {Bounds on $x_1+x_2$}`) is legitimate
TeX that must keep working. Putting `_` in `Mouth::with_bib_data_literals`
silently flattened every subscript in a bibliography title — caught by the
existing `bib_bare_ampersand_leaves_live_markup_alone`, which asserts
`role="SUBSCRIPTOP"`. Only the escaper can be math-aware, so treatment 1 covers
`% & #` and treatment 2 covers all five. Landed that way in OXIDIZED_DESIGN #74.

**`^` is treatment-2-only for exactly the same reason as `_`, and joined it
2026-07-27.** The two are twins in this flow: both are TeX scripting characters,
both are plain data to `bibtex(1)`, both raise "Script … can only appear in math
mode" outside math — verified end-to-end rather than assumed (`note = {q _ r ^ s}`
renders `q _ r ^ s`, zero errors). The one asymmetry is the escape spelling, and
it is a silent trap: `\_` is the underscore command but `\^` is the circumflex
**accent**, so the generic `\` + character arm would render `^o` as "ô". `^` gets
its own arm emitting `\textasciicircum{}`, braces included. This is what closed
"the `^` third remains open" below.

#### `@preamble` is treatment 2, and it already executes — verified 2026-07-27

`@preamble` is the one part of a `.bib` that `bibtex(1)` does **not** treat as
data: it copies the block verbatim to the top of the `.bbl` (plain.bst
`begin.bib`, `preamble$ write$`, ahead of `\begin{thebibliography}`), so pdflatex
has the definitions before the first `\bibitem`. It is how a `.bib` ships the
macros its own fields use, and MathSciNet's exporter emits one as a matter of
course — `@preamble{"\def\cprime{$'$} "}` in 2605.00097's `referLiu.bib` (L11)
and fourteen times over in 2605.11579's `biblo.bib` (L4768, L6910, …).

We read `.bib` directly, so that copy is ours to make — and it is made. Perl
`Pre/BibTeX.pm::toTeX` L118-122 joins the preamble lines ahead of
`\begin{bibtex@bibliography}`; `pre_bibtex::to_tex` mirrors it
**verbatim** — no `escape_bib_data_specials`, no
`Mouth::with_bib_data_literals` — which is exactly right for treatment-2
content. Measured end to end with a probe `@preamble` defining a macro nothing
else defines: **Rust 0 errors, same-host `latexmlc` 0 errors, both expand it**;
delete the `@preamble` and both raise `undefined:`. A fresh name is still the
right probe: at the time this was measured `\cprime` was a **vacuous** one,
because an always-on stub in `latex_constructs_rust_only.rs` defined it either
way. That stub is gone (below), so `\cprime` would work now — but any name the
kernel or a binding might also supply re-opens the same trap, so the fixture
keeps macro names unique to itself.

Guard:
`06_cluster_bibliography::bib_preamble_defines_macros_for_the_whole_bibliography`
(fixture `cluster_regressions/bib_preamble.{tex,bib}`), pinning a `#1`
parameter, `\&`/`\%` in a macro body, the `$'$` MathSciNet shape inside a name,
and — via a second entry — that the definitions are installed **once before any
entry**, not per entry. Red-verified twice: dropping the preamble from `to_tex`
costs 4 errors, and routing it through `escape_bib_data_specials` kills the
parameterized macro. The pre-existing `pre_bibtex::to_tex_includes_preamble`
does **not** cover any of this — it asserts on the emitted *string*, so it stays
green even if that string is never executed.

#### Entry selection: digest the CITED entries, not the library — LANDED 2026-07-27

Full design record: OXIDIZED_DESIGN **#80**. What belongs on this worklist:

* **A `.bib` is a library, not a document.** Perl `Pre/BibTeX.pm::toTeX` L110-122
  emits `\ProcessBibTeXEntry` for every entry unconditionally. That was cheap
  under the old string parser; since the raw `.bib` became a real conversion
  (item 1 below) each entry is a full expand/digest/construct cycle.
* **Cost, measured.** `anthology.bib` = 80,576 ACL entries for 9 cited. Witness
  **2605.07796**: 112 s / 4.8 GB RSS / memory budget tripped / **0 bibentries**
  (the whole bibliography lost) / killed by the fleet's 60 s timeout →
  **10 s / 9 bibentries / 0 errors**. Same shape in **59 of the 69** 2605/2606
  `never_completed_with_retries` papers (median 80,597 entries).
* **It is more faithful, not less** — `bibtex(1)` has always filtered on the
  `.aux`'s `\citation` records. Selection is closed over `crossref` and over a
  `\cite` made from inside a selected entry; every entry is still *registered*, so
  by-key lookup still resolves; `None` (= digest everything) covers `\nocite{*}`
  and a missing `BIBLABEL` record.
* **Standing consequence for this worklist: re-measure before believing any
  bibliography error count recorded before 2026-07-27.** An error raised only by
  an uncited entry now disappears without the underlying macro becoming
  available. It already invalidated the `\cprime` stub verdict directly below and
  2605.11579's `undefined:\Dbar` residual.

#### The always-on `\cprime` stub — DELETED 2026-07-27 (it was "it stays" for one day)

**Current rule:** `\cprime`/`\Cprime`/`\cdprime`/`\Cdprime` are `mathscinet.sty`
vocabulary and live only in `latexml_package/src/package/mathscinet_sty.rs`. A
paper gets them by loading `mathscinet` (or `amsrefs`, `amsrefs.sty` L217
`\RequirePackage{mathscinet}[2002/01/01]`), or by carrying the definition in its
own `.bib` `@preamble` — which executes, per the section above. Divergence
**#78**.

**Why the earlier "it stays" verdict was overturned, and it is worth keeping.**
That verdict was measured against a binary that digested **every** entry of a
`.bib` library. Three of its four regression papers only regressed because their
`\cprime`-bearing entry is **uncited** — `bibtex(1)` never copies such an entry
into the `.bbl`, so pdflatex never sees the macro, and the diagnostic was
manufactured by us. Divergence **#80** (digest the cited entries only) removed
that asymmetry structurally, and the justification collapsed with it. *The
measurement was right; its baseline moved under it within the day. A "regression
without the fix" measured against a defective baseline measures the defect.*

Original scan, kept because re-deriving it costs a corpus sweep: across the first
600 papers of `/data/arxiv/2605/`, **7** use `\cprime` inside a `.bib` and **6 of
the 7 carry no `@preamble` at all** (the seventh, 2605.00097, has one but never
uses `\cprime`); 2605.11579 sits outside that window and is the one paper whose
`@preamble` covers real uses.

Re-measured on current main (stub gone), `--includestyles`, idle box, serial —
TOTAL document errors and `undefined:\cprime`:

| paper | `@preamble` defines `\cprime`? | errors | `undefined:\cprime` |
|---|---|---|---|
| 2605.00173 | no | 0 | 0 — `MR2562222` (`bibliography.bib` L885) is uncited |
| 2605.00186 | no | 0 | 0 — same shape |
| 2605.00190 | no | 0 | 0 — same shape |
| 2605.00305 | no | **1** | **1** — the only real cost |
| 2605.11579 | yes, 17 uses in `AUTHOR`/`TITLE`/`BOOKTITLE`/`MRREVIEWER` | 0 | 0 |

2605.00305 is the honest residual and the row to keep: it **cites** `MR710121`
(`mybib.bib` L26, `MRREVIEWER = {V.\ Z.\ Enol\cprime ski\u i}`), loads neither
`mathscinet` nor `amsrefs`, ships no `@preamble`, and uses
`\bibliographystyle{plain}` — `plain.bst` contains zero `cprime`. Real pdflatex
raises the same undefined control sequence, so this is PARITY, and supplying the
macro anyway would push our error count below the author's own toolchain.
2605.11579 is informative twice over: its 14 `@preamble` blocks (`biblo.bib`
L4768/L6910/…) carry all 17 uses, and its formerly-standing `undefined:\Dbar`
went away too — `KacNilpotentorbits` (`biblo.bib` L2059) is uncited (#80), not
newly defined.

Two rows from the with/without table are dropped as uninformative once the
baseline changed but are recorded so they are not re-measured: 2605.00316's
`\cprime` sits in `fjournal` (undigested — 2 errors either way) and 2605.00584
was 0 → 0.

Same-host Perl is **not a usable oracle for this question** (production profile,
verbose, all eight): it never raises `undefined:\cprime` anywhere, for reasons
unrelated to the macro — 2605.00305 and 2605.00316 hit `MAX_ERRORS` (101 + Fatal)
on a pgfparse flood long before MakeBibliography, 2605.00173/.00186/.00190 take
their shipped `.bbl`, and on 2605.11579 Perl emits **zero** bibliography entries
at all ("Missing Entry for citation" × 36, against Rust's 36 rendered) — a silent
total loss, not a clean run. The mechanism was confirmed against Perl on the
controlled fixture instead, where both engines reach 0 errors with every preamble
macro expanded. Perl defines `\cprime` **nowhere** in `LaTeXML/lib`, so the
`mathscinet.sty` binding that now owns it is a surpass-Perl divergence — see #78.

### Why the re-port was the real fix: eager tokenization defeats parameter types — DIAGNOSIS 2026-07-26, FIX LANDED 2026-07-26/27

The 2026-07-26 sandbox rerun quantified what the simplified parser costs. In
`sandbox-arxiv-2605` (30,079 docs) the rc2→rc3 window moved **90 papers
`no_problem → error`** and 61 `warning → error`; **87 %** of the errors in the
latter bucket, and 38 of the first 40 sampled in the former, were raised
`at Anonymous String` — i.e. from `make_bibliography.rs`'s field digest.

Three sub-causes, and they are NOT three bugs — they are one architectural gap:

| symptom | count (sampled) | why |
|---|---|---|
| `undefined:\url` | 21 + 18 | a real `.bbl` opens with `\providecommand{\url}…` from the `.bst`; we digest the raw FIELD, one step earlier |
| `expected:}` | 32 + 17 | percent-ENCODED URLs — `%` at catcode 14 comments out the rest of the line, closing brace included |
| `unexpected:_` / `&` / `^`, `misdefined:#` | 61 | TeX specials that are literal inside a URL |

The first two are fixed (#391: `BBL_STANDARD_FALLBACKS`, `BibCatcodeScope` — the
latter has since **retired**, as predicted below once the engine route landed; it
exists in no source file today, `BBL_STANDARD_FALLBACKS` lives on in
`latexml_oxide/src/bib_session.rs` as the recursive session's preamble).
The third is fixed too (below), but NOT by widening the catcode phase, and that
constraint is why: `$a_b$` and `$x^2$` in a title are legitimate, so
neutralizing `_`/`^` file-wide would trade one regression for another. Only the
math-aware escaper can carry a scripting character.

**LANDED 2026-07-27 on the `.bib` POOL route, exactly as consequence 2 above
prescribes** — an escape at the A→B boundary, not catcode suppression.
`escape_bib_data_specials` (`bibtex.rs`) emits `%`→`\%`, `&`→`\&`, `#`→`\#`,
`_`→`\_`, `^`→`\textasciicircum{}`; `\emph{…}`, `$x^2+y_1$` and `{\v S}pakov`
pass through untouched.
The mechanism, its four hazards (math, idempotency, raw-read fields, nested
`\url` data regions) and — the part that was NOT obvious — the **three** seams
it had to cover are in OXIDIZED_DESIGN #74: the entry line in
`\ProcessBibTeXEntry` plus `\bib@@title` and `\bib@@pages`, both of which
re-read the RAW field instead of using the value the entry line passed them, so
escaping only the entry line silently missed every `title`. Guard
`06_cluster_bibliography::bib_field_specials_are_data_not_tex` plus the
`escape_specials_*` unit tests in `bibtex.rs` (six at the time; **13** today,
after the `^` arm and #79's five unmatched-`$` cases). Seven witnesses, TOTAL document errors,
`--release` before→after: 2605.06926 8→0, 2605.01936 13→0, 2605.04604 2→0,
2605.08986 2→0, 2605.11300 1→0, 2605.05898 1→0, 2605.06249 3→0. **30 → 0** —
every one converts clean.

Two corrections this work turned up: those witnesses raise their errors in the
**pool route's own Mouth**, so the 61 above are not all
`make_bibliography.rs`'s eager digest as this section assumes; and
`volume`/`language`, previously read as genuine parity to leave erroring, are
covered by the DATA-regime policy like everything else.

Perl does not need any of these patches, and the reason is structural.
`BibTeX.pool.ltxml` declares a parameter type per field — `url` **Verbatim**
(L740), `crossref`/`doi`/`isbn`/`issn`/`lccn`/`pii` **Semiverbatim**
(L684, L750-779), `key`/`type`/`review`/`links` **Digested**, the other 34
ordinary — and L92-93 states the rule outright: *"Semiverbatim for fields that
may contain something like a url."* Those types only work because Perl assembles
the entry as TeX **source lines** and hands them to a fresh Mouth (L134-166), so
tokenization is **lazy** and a parameter type can set the catcode table *before*
its argument's characters are read.

Note `note` and `howpublished` are in the ordinary 34 — Perl survives
`note = {\url{…%20…}}` not because the FIELD is Semiverbatim but because
`\url` itself is, and lazy consumption lets it act.

`latexml_engine/src/bibtex.rs` already did this correctly for the pool route:
`open_mouth(Mouth::new(&lines.join("\n"))?, true)` with
`\bib@field@default@url Verbatim`, and a comment there saying why it is
load-bearing — a pre-tokenized `Explode!` stream "did neither" (witness
2508.17585). `make_bibliography.rs` instead called `tokenize()` then `digest()`,
which is **eager**: every catcode is fixed before any handler runs, so no
parameter type could ever take effect. **That eager path is gone** — item 1 below
deleted it, and every `.bib` now reaches the tokenizer through the pool route's
lazy Mouth. (Line numbers into `bibtex.rs` are deliberately dropped here: the
file has been edited in every PR of this campaign and the old `L1577`/`L1809`/
`L1888` anchors no longer point where they did. Perl `file:line` citations are
stable and are kept.)

**Relation to issue 386** (build XML through libxml, not string concatenation).
Separate axes that met in this one file, and the dependency ran one way.
Issue 386 is about OUTPUT — `interpret_tex_markup` returned a *serialized XML
string* spliced into the bibliography behind three trust gates, which was exactly
issue 386's complaint. This section is about INPUT — how the field is tokenized.
Routing field interpretation through the pool path made the output arrive as DOM
nodes, so **the re-port settled the `make_bibliography.rs` portion of issue 386 as
a side effect** — landed with item 1 below (`PostDocument::new(XmlDoc)`, no
serialize/reparse). Do not attack that portion of issue 386 separately.

**Consequence for planning — SETTLED, the interim was never taken.** The 61
residual errors were bounded by this gap, not by a new defect. The considered
interim was a per-field `Tokens::neutralize` (`tokens.rs`, the existing
"retroactively imitate what Semiverbatim would have done" helper — its comment
explains why it deliberately does NOT cover `%`); it imitates the mechanism
rather than using it, and was **not** taken. The durable fix landed instead:
item 1 routes field interpretation through the lazy Mouth, and #74/#79 close the
specials at the data→TeX boundary.

**Maintainer policy, 2026-07-26 — a `.bib` field's content is DATA, not TeX.**
A bare `%`, `&`, `_`, `#`, `^` is the literal character; "Taylor & Francis"
renders with its ampersand. Authorized surpass-Perl *and* surpass-pdflatex:
LaTeXML reads `.bib` directly, with no `.bst` and no `bibtex(1)`, so it decides
what reaches the tokenizer.

> **RETRACTED, and the retraction is the design.** This block once read "the
> boundary is blast radius, not character", splitting the five specials into a
> per-Mouth *regime A* (`% & # _`) and a per-field-parameter-type *regime B*
> (`_` via `Semiverbatim` on `eprint`/`preprint`/`archive`). **Neither half
> survived implementation.** `_` is NOT in the Mouth set — `mouth.rs`'s
> `with_bib_data_literals` is `% & #` only, and its doc says why — and no
> `Semiverbatim` was ever put on `eprint`/`preprint`/`archive`: they are plain
> `\bib@@field{ltx:bib-links}` macros in both engines (`bibtex.rs` L2030/L2053/
> L2055, `BibTeX.pool.ltxml` L712/L724/L728). The correct frame is the **two
> treatments** above — both apply, at different points, and the split is by
> *treatment*, not by blast radius: treatment 1 covers `% & #`, treatment 2
> covers all five. The retired divergence **#76** was this reading's entry.

**Both treatment-1 seams are required.** The value reaches a tokenizer by two
independent routes: the per-entry Mouth, and the handlers that re-read the
stored RAW field and tokenize it themselves (`\bib@@title` recasing,
`current_entry_field`, name splitting, date/pages assembly, MR/Zbl). The
companion for the second is `mouth::tokenize_bib_literal`, behind
`bibtex.rs::tokenize_bib_field`. Fixing only the mouth made 2605.02131 *worse*
(28 → 37 errors) — a title's value never passes through it.

*The `&` third, measured.* Witnesses 2605.01936, 2605.06249, 2605.00462,
2605.03054, 2605.06624, 2605.08753, 2605.10409. The `&` sits in `publisher` /
`journal` / `booktitle` / `author` / `copyright` — all `Digested` in
`BibTeX.pool.ltxml`, so **no parameter type was ever going to cover them and the
eager-tokenization gap above was never this third's cause**. Before: Rust
matched same-host `latexmlc` 1:1 on every re-measured witness, and pdflatex
broke identically. After: **3→0, 1→0, 1→0, 1→0, 1→0, 13→6, 4→3**; the two
residuals are unrelated `undefined:` errors, no `unexpected:&` remains anywhere.
`#` was checked and deliberately left alone — one bare `#` across all seven
witnesses, in 2605.00462's JabRef `file` path, already covered as an unknown
field.

*A separate ampersand bug, also fixed under #74:* the doubly escaped `\&amp;` /
`{\&}amp;` / `&amp;`, where an HTML entity survived into the `.bib`. Those raise
no error at all — Perl and pdflatex both print "&amp;" — so the mouth-level
change does not reach them; `undouble_escaped_ampersand` in `bibtex.rs` does.
Guard `bib_escaped_amp_entity_decodes_to_one_ampersand`. **The `^` third is
CLOSED** (2026-07-27) — the same treatment-2 escape as `_`, with its own arm
because `\^` is the circumflex accent; see the two-treatments section above.

#### What the 2026-07-26 prototype measured (and the two claims it corrected)

Before designing the re-port, the existing `--bibtex` route was probed directly
against same-host Perl (`latexml --bibtex`). It is further along than this
document assumed:

| probe | Rust | Perl |
|---|---|---|
| clean `.bib` (`url` with `_ ^ & #`, `pages={1--10}`, `doi`) | 0 errors, XML **byte-identical to Perl** | 0 errors |
| `howpublished={\url{…%20…}}` with `--preload=url.sty` | **0 errors**, `%20` intact in `<ref href=…>` | identical |

So the `_ ^ & #` residual and the percent-encoded-URL family are both solved by
*using* the route — the `url` field's **Verbatim** type and `\url`'s
**Semiverbatim** parameter do it by construction. Two claims made above needed
correcting:

1. **The `\providecommand` block is NOT redundant under the engine route.** It
   is load-bearing: with no url package the same probe gives 4 errors and *zero*
   entries, because `\url` is undefined and its Semiverbatim parameter therefore
   never runs — the `%` comments out the closing brace. Defining `\url` is what
   arms the protection. `BBL_STANDARD_FALLBACKS` becomes the recursive session's
   **preamble**; `BibCatcodeScope` (the file-wide `%` catcode hack) does retire.
2. **The ObjectDB metadata fallback is NOT an artifact of the string route.**
   Perl's `getBibEntries` has no equivalent at all — `MakeBibliography.pm`
   L342-343 records a missing key and moves on. The Rust path
   (`match_metadata_field` / `get_metadata_content`, L1241/L1277) serves entries
   whose data comes from the ObjectDB, so the re-port must leave it alone; it is
   a separate divergence with its own coverage.

**Blocker the prototype also found (fixed 2026-07-26, before the switch).** Rust
lost the WHOLE bibliography where Perl keeps every entry, from two independent
causes — and reported FEWER errors while doing it, the worst combination:

* `read_balanced` crossed out of the `\ProcessBibTeXEntry` entry mouth into the
  wrapper, swallowing every following entry and `\end{bibtex@bibliography}`.
  Perl's readBalanced reads the current mouth only (`Gullet.pm` L465-472).
  Fixed by `BalancedBoundary::Opaque` on that mouth (`gullet.rs`,
  `bibtex.rs`), leaving the xint `\scantokens` divergence untouched.
* `digest_next_body` pushed Perl's EOF trailer box (`Stomach.pm` L130,
  `push(@LaTeXML::LIST, Box()) unless $token`) only when the body had read *no
  token at all* — a strictly narrower condition than Perl's "the read returned
  undef". A `Digested` argument that ran to EOF therefore had `readDigested`'s
  `pop` (meant to strip the closing-brace box) eat **real content**.

A bare `%` in a field triggers either one — BibTeX has no comment syntax inside
an entry, TeX does. Mis-reading the field stays parity (real bibtex+pdflatex
break on it too); only the blast radius was ours. Guard:
`55_bibtex::runaway_field_costs_only_its_own_entry`. After both fixes every
probe matches Perl's entry count exactly (1/1, 2/2, 3/3 across eight shapes).

### Bibliography "missing references" — NEXT-TARGET list (surveyed 2026-07-12)

Per the user follow-up ("detect docs where References are entirely missing … the
next target for beyond-Perl bibliography work"). Playwright scan over all 297
reported papers (correct mains) → only **4** genuinely lack a rendered
bibliography, and the dominant root cause is **NOT** bibliography markup — it is a
**mid-body digestion error that truncates the document** before the (end-of-doc)
bibliography, which is then collateral damage:

- `2507.21938` (ICML): document tree **truncates** mid-section-2 (empty table
  cells + empty figure); `\bibliography{example_paper}` + co-located
  `example_paper.bbl` never reached. Body-truncation bug.
- `2508.13557` (IEEEtran): undefined `\node` (tikz outside a picture) corrupts
  `display_math` mode → `\lx@end@display@math` cascade → **truncation** before the
  bibliography. `main.bbl` *is* input (the `\jobname.bbl` fallback works). Body-error bug.
- `2510.25135`: **source path-doubling** — main is `submissio-v0/main.tex` and
  `\bibliography{submissio-v0/mypub,submissio-v0/ref}` resolves relative to that
  dir → `submissio-v0/submissio-v0/…`. Source quirk (assumes top-level compile).
- `2606.25280`: **source filename case/extension quirk** —
  `\bibliography{EvoFlock.bib}` vs the shipped `Evoflock.bib` (fails only on a
  case-sensitive FS; parity with Perl on Linux).

So the real beyond-Perl lever is **body-error resilience** (2 papers where a
mid-body digestion error truncates the tail); the other 2 are source quirks/parity.
Post-release (release-week bias is stabilize, and these are deep digestion work).

#### Corpus-scale confirmation (swept 2026-07-14, sandbox-2605, 30,058 result ZIPs)

The 297-paper scan above is confirmed at corpus scale, and the split is now
measured. Detection is a **rendering** property, not a cortex category (there is
no "empty References" category): read the produced HTML out of each result ZIP
and count `ltx_bibitem`. Baseline (pre-fix run): **ok 29,308 (97.5%) / EMPTY 324
(1.1%) / no-bib 359 / no-html 67**.

The EMPTY bucket is NOT one defect — decomposing it is the whole point, and a
first pass that lumps them together mis-ranks the work:

| class | n | what it is |
|---|---|---|
| **TRUNCATED** | 169 | citations rendered but **no bibliography element at all** — the document died before reaching `\bibliography`. NOT a bibliography bug. |
| **NO-CITES** | 92 | a literal "References" heading but no citations/bibliography markup — mostly author-hand-rolled lists; largely parity. |
| **EMPTY-SECTION** | 63 | bibliography element present, **zero entries** — the genuine bibliography defect. |

So **truncation, not bibliography code, is the dominant cause of a missing
References section** (169 vs 63) — the 2026-07-12 hypothesis, now quantified.
Body-error resilience remains the top lever.

**TRUNCATED (169) is REAL, not a stale-ZIP artifact** — spot-checked 4 witnesses
of the largest sub-cluster on the current binary: 3 still truncate
(`2605.00025` 455 errors, `2605.09913` 91, `2605.12696` 14; only `2605.09761`
recovered). Contrast the EMPTY-SECTION side, where stale ZIPs DO dominate
(`2605.02024` shows 38 bibitems / 0 dangling on the current binary) — so
**re-convert before chasing any EMPTY-SECTION paper**.

Dominant TRUNCATED trigger (first error, not the cascade):

| trigger | n | note |
|---|---|---|
| `unexpected:\lx@end@inline@math` | 25 | math-mode desync |
| `unexpected:\lx@begin@alignment` | 19 | alignment opened inside inline math |
| *no errors at all* | 17 | **silent** content loss — worst kind |
| `unexpected:_` / `^` | ~37 | sub/superscript outside math (same family) |
| `unexpected:\lx@tag@intags` | 4 | the `\fnum@figure` cascade above |

The math-desync + alignment families together are ~66/169 (39%) and look like one
root family: a group/mode nesting break around inline math. **11 of the 169 are
the known mhchem `\ce`-in-`align` parity limit** (`2605.12696`: `\ce{CO2(aq) +
H2O &<=> H2CO3}` inside `align` — identical in same-host Perl, investigated
2026-06-27, NOT a Rust gap). The remaining ~147 are the concrete next target for
body-error resilience.

**One TRUNCATED sub-cluster is now FIXED — inline `\end{lstlisting}`** (7 of the
169, 3 of them in the silent subset). See OXIDIZED_DESIGN #61 /
KNOWN_PERL_ERRORS #51: Perl anchors the terminator regex at the line start, so
`</body></html> \end{lstlisting}` never terminates and the reader eats the rest
of the file, `\end{document}` included — **zero `Error:`**. pdflatex accepts the
same input and renders the leading text as the listing's last line, so both
LaTeXML engines were wrong vs the PDF (same-host Perl: "No obvious problems", tail
gone). Fix = match `\end{<env>}` anywhere in the line. 5 of the 7 witnesses
recover: `2605.11619` 0 → 32 refs (Conclusion + appendix restored), `2605.29675`
107, `2605.21677` 66, `2605.29786` 42, `2605.07451` 28 — **275 references**. The
other 2 (`2605.08378`, `2605.08915`) have unrelated causes.

**This is why the "17 silent" subset is the highest-value slice**: no `Error:`
means no cortex signal, so these never surface in any severity report — the only
way to find them is a rendering sweep like this one.

Within EMPTY-SECTION the one clean, landed win was the **non-UTF-8 `.bib`**
cluster (below). Remaining EMPTY-SECTION sub-clusters, not yet triaged:
`undefined:\affiliations/\emails` (7), `post:convert` (8), a revtex4-2 +
bibunits `bu1.bbl`/`bu2.bbl` group (7).

**Trap (hit and corrected):** the classifier's citation needle must be real
citation markup (`ltx_cite`/`ltx_bibref`/`ltx_missing_citation`) — `ltx_ref`
also matches `\ref` to a figure, which over-counted TRUNCATED 231 → 169.
Second trap: **pick the main file cortex picked** (`Processing content …` in
the log). A `grep -rl '\begin{document}'` harness picks the first match, which
for `2605.30360` was the decoy `proof.tex`, not `polyhist.tex` — that alone
manufactured a false "still broken" verdict.

#### `\renewcommand*{\fnum@figure}[1]` truncates the document — ANALYSED 2026-07-14, NOT fixed (needs a decision)

Witness `2605.01731` (cas-sc): 18 figures × 3 errors
(`\lx@tag@intags`/`\lx@tag`/`\end{figure}` "Attempt to end mode
restricted_horizontal") → body collapses to ONE section, 19 `<bibref>` survive
but **no `<bibliography>` element at all**. Breadth: **18 papers corpus-wide**
(`grep 'lx@tag@intags'`), 5 of them in the EMPTY set.

Root cause is a real-world author hack that pdflatex tolerates:

```tex
% Change Fig. 1: to Fig. 1.
\makeatletter
\renewcommand*{\fnum@figure}[1]{\figurename~\thefigure.}
\makeatother
```

Real `\fnum@figure` takes **no** argument. In LaTeX, `\@caption` passes it to
`\@makecaption{\csname fnum@\@captype\endcsname}{…}`, whose body is
`\sbox\@tempboxa{#1: #2}` — so the author's 1-arg version **eats the `:`**,
which is exactly their stated intent. It works in pdflatex.

LaTeXML has no `:` token to eat: `\format@title@figure` is
`\lx@tag[][: ]{\lx@fnum@@{figure}}#1` — the separator is a **tag attribute**, not
a token. So `\csname fnum@figure\endcsname` (Base_Utility L1041-1043) grabs the
group's closing `}` instead, wrecking the caption and cascading.

This is **PARITY** — Perl's `\lx@fnum@@` is identical — so fixing it is a
surpass-Perl divergence, and both engines are wrong vs the PDF. Candidate fix:
expand as `\csname fnum@#1\endcsname{}` so an arg-taking `\fnum@<type>` eats a
harmless empty group (reproducing pdflatex's result) while a normal 0-arg one
just gains an empty group. **Not done**: `\lx@fnum@@` formats every figure/table
caption in every document — blast radius far out of proportion to 18 papers, and
release-week bias is stabilize. Needs a user decision + a full-suite diff.

Minimal repro (article + subfigure + the `\renewcommand*` above) reproduces the
exact 3-error signature; `cas-sc` is NOT implicated (it was the first
hypothesis and it was wrong — plain `article` reproduces).

#### Non-UTF-8 `.bib` silently dropped the whole bibliography — LANDED 2026-07-14

`std::fs::read_to_string` hard-errors on the first non-UTF-8 byte, so a legacy
`.bib` lost **every** entry and rendered an empty References section with **no
`Error:`** — a silent, total loss. Witness `2605.00490`: a JabRef file
self-declaring `% Encoding: Cp1252`. Real `bibtex` 0.99d is 8-bit clean, and
Perl never fails here (`Mouth.pm` L75-80: decode with `Encode::FB_DEFAULT`, or
pass the raw bytes through when `PERL_INPUT_ENCODING` is undef) — so this was
**GENUINE-RUST-ONLY**, not parity.

Fix: both `.bib` read sites decode via the shared
`latexml_core::mouth::decode_input_bytes` — UTF-8, else a **Latin-1
passthrough** (lossless byte → char, so accented names survive intact instead of
collapsing to U+FFFD; legacy `.bib` files are overwhelmingly Latin-1/Cp1252).
The sites are `pre_bibtex::new_from_file` engine-side and — since item 1 deleted
`make_bibliography::convert_bib_file_to_xml`, which was the post-side one — the
multi-file concatenation in `bib_session::payload`, which inherited the
obligation and the witness.
The Mouth's own no-encoding branch now calls the same helper, so there is one
implementation rather than three (the "bespoke duplicate shadowing a faithful
port" anti-pattern has already bitten twice here).

Breadth: 17 papers corpus-wide, 10 of them EMPTY. All 10 recovered: **0 → 336
references** (7/15/5/22/57/25/39/48/108/10), 0 dangling.

Red/green tests: `pre_bibtex::tests::non_utf8_bib_file_is_read_not_rejected`
(engine reader) **and** `06_cluster_bibliography::non_utf8_bib_file_still_yields_a_bibliography`
(post path — where the production failure actually was; the engine-side test
alone would NOT have guarded it). Fixture `cluster_regressions/cp1252_refs.bib`
carries a raw `0xe9`; it is asserted non-UTF-8 so the test cannot go vacuous.
Note when asserting on rendered author names: `author = {Café, André}` is
BibTeX's `Last, First`, so the style abbreviates the given name — the entry
renders `A. Café`, and only the SURNAME is a safe needle.

Third test: `one_bad_byte_does_not_mojibake_the_rest_of_the_file` pins the
per-line granularity (a whole-buffer fallback turns a valid-UTF-8 `Ü` into
`Ã\u{9c}` — verified by reverting).

### MakeBibliography full parity re-port (user directive 2026-07-04: reuse TeX interpretation, no special-case parser)

Audit 2026-07-04 (agent, both files read end-to-end): `make_bibliography.rs`
(3,545 lines) vs Perl `MakeBibliography.pm` (818 lines) is a **faithful port
with one large divergent subsystem**: ~11 of 18 Perl subs are structural
ports (FMT_SPEC stays table-driven; getBibEntries referrer/suffix logic,
formatBibEntry, all do_* formatters track Perl), BUT the .bib->XML route
replaces Perl's 63-line recursive-core-session `convertBibliography` with
~770 lines (~22% of the file) of hand-rolled string parsing
(`parse_bibtex`, `read_bib_value`, `parse_bib_authors`, `strip_braces`,
`is_braced_group`, `convert_bib_file_to_xml`, plus the whole
metadata-fallback path that exists only because no real bibentry XML is
produced).

> **The three INTERIMs below are HISTORY, superseded by item 1 (landed
> 2026-07-26).** Every identifier they name — `convert_bib_file_to_xml`,
> `interpret_tex_markup`, `interpret_tex_text`, `parse_bibtex`,
> `read_bib_value`, `strip_braces` — was **deleted** with the string route
> (−727 lines); `grep`ping for them in `make_bibliography.rs` today returns
> nothing. They are kept, not pruned, for the three things that outlive the
> code: the *properties* the current route must also satisfy (digest each field
> exactly once, keep markup, emit every field kind), their **witnesses**
> (2607.00045, the eleven field kinds, `KNOWN_PERL_ERRORS #60`), and the traps
> that cost a wrong first cut. The live guards are
> `06_cluster_bibliography::bib_field_markup_survives_into_the_bibliography` and
> `105_bib_field_digest_once`, both of which were re-pointed at the current
> route.

INTERIM (landed 2026-07-04): field VALUES now go through the real engine —
`interpret_tex_text` = `digest(mouth::tokenize(v)).to_string()` against the
LIVE in-process state (Perl's `ToString(Digest(Tokenize($x)))`; article-
class macros like `\aap` expand because aa.cls is loaded); the ~150-line
`decode_tex_accents` transliterator is DELETED. DOI identifiers emit
absolute `https://doi.org/` hrefs (percent-encoded, Perl BibTeX.pool
L750-756) and scheme-less bib URLs are forced absolute — normalized both at
.bib conversion AND in `format_links` (covers .bbl-borne/pre-compiled XML).

SECOND INTERIM — field values keep their MARKUP (landed 2026-07-25). The
2026-07-04 step digested field values but then **stringified** them, and a
Whatsit stringifies to its *reversion*: `note = {\url{https://x}}` came back as
its own TeX source, which `strip_braces` mashed into the dead literal
`\urlhttps://x`; `\href{u}{text}` additionally **lost its link text** (the
reversion keeps only the first argument). A second, independent flatten sat
downstream in `apply_formatter`, which took `get_content()` of the field node and
discarded element children — Perl's formatters are `do_any`-shaped and return
`$doc->cloneNodes(@nodes)` (`MakeBibliography.pm` L525-531, L550-552). **Both had
to be fixed; either one alone keeps the bug.** Same-host Perl renders all of
`\url`/`\href`/`\emph`/math correctly, so this was GENUINE-RUST-ONLY.

* `interpret_tex_markup` (make_bibliography.rs) digests into a scratch
  `Document`, runs the new `Document::finalize_subtree` (font resolution +
  `_font`/`_autoopened` bookkeeping-attribute removal), and serializes the
  scratch `ltx:text` wrapper's children. It must be the SUBTREE variant:
  whole-document `finalize()` returns `Ok` but, recursing from the root,
  legitimately UNWRAPS that redundant font-only `ltx:text` — measured, the
  content survives at the root while the caller's handle is left detached and
  childless (`wrapper_children 1 → 0`, `parent = None`), serializing to nothing.
  (An earlier note here said `finalize()` "fails"; it does not — that was
  inferred from the empty output rather than measured.)
* Applied to `title`/`journal`/`journaltitle`/`booktitle`/`publisher`/`note`;
  every other field is untouched, and a wholly plain field still takes the
  plain-text path, so the 99 % case is byte-identical.
* Four fail-safe gates return `None` (→ escaped plain text) rather than splice
  something unsound, because a bad fragment would break the XML parse and lose
  the WHOLE bibliography: failed digest, **escaped content**, **unparsed math**,
  and any prefixed element name.
* The **escaped-content** gate is the one that a first cut got WRONG, and it is
  the most important: BLOCK-level content (`\begin{itemize}`, `\par`,
  `\begin{quote}`, `\footnote`) CLOSES the `ltx:text` wrapper and continues as a
  SIBLING, so serializing only the wrapper's children dropped everything past
  that point — `note={before \begin{itemize}\item X\end{itemize} after}`
  rendered as just `before`, **silently, with zero errors**, i.e. precisely the
  fail-OPEN the design claims to prevent. The gate compares the scratch
  document's whole text against the wrapper's and declines on any difference.
  It compares TEXT, so block content carrying none (a lone floated image) is
  still invisible to it; every realistic escape carries text. Guarded by the
  `INSIDELIST`/`afterblock`/`AFTERPAR` entries in the fixture.
* The math gate is the one remaining sub-Perl case — the Marpa
  pass lives in `latexml_math_parser`, which `latexml_post` does not depend on,
  so a `<Math>` built here keeps unparsed `<XMath>` and would emit malformed
  MathML (`x^2` → `msup` with an empty base). Falling back to the TeX source is
  exactly today's behaviour for math; item 1 below fixes it properly.
* **Digest each field exactly once.** The markup path runs BEFORE
  `interpret_tex_text` and suppresses it on success — both digest the same
  value, and digesting twice re-reports every error the field raises (measured:
  a `_` in a note counted its `unexpected:_` **twice**, silently inflating the
  document's error count, which is the canvas pass/fail signal) and re-runs any
  side effect the macros have. Undefined macros hide this: the first digest
  defines them as `<ltx:ERROR/>`, so they are self-healing on a second pass —
  a guard fixture must use a non-self-healing error, and must contain a
  backslash or both paths short-circuit and the test goes vacuously green.
  Guard: `105_bib_field_digest_once`.
* One serialization note settled during review: a finalized LaTeXML document is
  entirely in the LaTeXML namespace and serializes **unprefixed**, so the single
  `xmlns` on the generated `<bibliography>` root covers the spliced fragment —
  no second `xmlns:ltx` declaration is needed (an early draft added one).

Witness 2607.00045 (sn-jnl, reported by email 2026-07-25): 44 of 78 rendered
entries carry `note = {\url{...}}`. Raw-TeX tokens in the rendered bibliography
**46 → 2**, live links **0 → 45**; the 2 residuals are the gated `$\psi$` title
and one `url={\url{...}}` that **Perl renders equally broken**
(`href="\urlhttps://arxiv.org/abs/quant-ph/0510095"`) — parity, not ours.
Guard: `06_cluster_bibliography::bib_field_markup_survives_into_the_bibliography`
(fixture carries a `\&` so a text-escaping regression cannot pass silently).
*(Guards moved out of `06_cluster_regressions` when PR #400 split the
bibliography cluster into its own test file.)*

THIRD INTERIM — fields that reached NO emit branch (landed 2026-07-25). Found by
asking what else the reported family ("content missing from References") could
cover. `convert_bib_file_to_xml` parsed every field but emitted only a subset,
so eleven field kinds were **silently discarded** — most damagingly
`howpublished = {\url{...}}`, which is how a `@misc` conventionally carries its
URL. Same-host Perl emits all of them, so this was GENUINE-RUST-ONLY.

The format specs ALREADY query the matching elements (`ltx:bib-organization`,
`ltx:bib-place`, `ltx:bib-edition`, `ltx:bib-part[@role='series'|'part']`,
`ltx:bib-status`, `ltx:bib-language`, `ltx:bib-type`, `ltx:bib-note`), so only
the emitter was missing — mappings taken from `bibtex.rs` L1340-1573 (the port of
`BibTeX.pool` `\bib@field@default@*`): `howpublished`→`bib-note[role=publication]`,
`note`/`annote`→`bib-note[role=annotation]`,
`institution`/`organization`/`school`→`bib-organization`, `address`→`bib-place`,
`edition`→`bib-edition`, `series`/`part`→`bib-part[@role=…]`, `type`→`bib-type`,
`status`→`bib-status`, `language`→`bib-language`.

Deliberately still NOT emitted: `chapter`, `subtitle`, `translator`. Perl builds
elements for them, but **no format spec queries** `ltx:bib-part[@role='chapter']`,
`ltx:bib-subtitle` or `ltx:bib-name[@role='translator']`, so emitting them adds
nodes that can never render (confirmed: Perl leaves `chapter={5}` unrendered
too). The entry-type boilerplate Perl synthesizes in `\bib@entry@<type>@prepare`
("Ph.D. Thesis", "Technical Report") is engine-side, not a `.bib` field, and
arrives with item 1 below.

`bib-type` is safe to emit here even though Perl's `getBibEntries` unions it with
`bib-date`: the Rust port queries the two separately (make_bibliography.rs
L1029/L1048), so a `type` field cannot displace the publication year.

One visible consequence of emitting `type`, checked and accepted: real BibTeX
treats it as an **override** of the entry-type label, while LaTeXML renders both,
so `type = {Technical Report}` on a `@techreport` now reads "Technical Report
Technical Report SIDL-WP-1999-0120". Verified byte-identical in same-host Perl —
this is **parity**, KNOWN_PERL_ERRORS #60, and suppressing it would be a
surpass-Perl divergence. It was previously hidden by dropping the field
entirely, which also lost genuinely distinct types (`type = {Technical Memo}`) —
a strictly worse trade. Only 1 `type` field exists across the nine witness
`.bib` files.

Measured: 7/7 probe fields now match same-host Perl on a per-entry fixture;
across the nine 2607 witnesses this recovers **45 `bib-place` + 8 `bib-edition`
+ 1 `bib-type`** that were previously dropped, at **unchanged error counts**
(the wider interpretation surfaces no new diagnostics on real input).
Guard: the `BIGINSTITUTE`/`TECHMEMO`/`LECTURENOTES`/`SECONDED`/`BERLINPLACE`/
`SOMEUNIVERSITY` + `howpub` assertions in
`bib_field_markup_survives_into_the_bibliography`.

#### Why this file is 3,735 lines, and why the interims did NOT refactor it

User observation (2026-07-25): `make_bibliography.rs` builds XML by **string
concatenation** instead of using libxml2 directly, and the file is large partly
as a consequence. Measured: **3,735 lines vs Perl's 818** (4.6×), with 33
`xml.push_str` / 31 `xml_escape` / 71 `format!` sites; the `.bib`→XML route
alone is **573 lines** (`convert_bib_file_to_xml` 296, `interpret_tex_markup`
105, plus the parse/escape helpers).

The cost is concrete, not stylistic: the 2026-07-25 markup work had to add a
serialize→string→reparse round-trip AND a namespace-prefix scanner purely
because the target is a string rather than a node tree. With
`PostDocument::add_nodes`/`NodeData` those two would not exist, and `xml_escape`
would be unnecessary (libxml escapes on set).

**Decision (user, 2026-07-25): do NOT refactor it to node-building as an
intermediate step** — item 1 below already deletes the entire route, so a
node-based rewrite of `convert_bib_file_to_xml` would be thrown away, while
adding regression risk to a validated fix. Item 1 is the answer; it removes the
string machinery *and* fixes the residual math gap and the entry-type
boilerplate for free.

FULL RE-PORT — item 1 LANDED 2026-07-26:
1. **DONE.** `convert_bib_file_to_xml` and the whole string route are gone
   (**-727 lines**; `latexml_post` no longer depends on `latexml_engine` at
   all). A raw `.bib` is converted by the recursive BibTeX session
   (`latexml_oxide/src/bib_session.rs`), injected through the
   `set_bib_converter` hook `latexml_post` declares —
   `latexml_post` cannot depend on the converter, since `convert_document`
   needs the model loader. `get_bibliographies` now follows Perl's shape:
   accumulate `@rawbibs`, ONE combined pass (cross-bib `@string` sharing),
   `literal:` data, kpsewhich fallback, prefer `<name>.bib.xml`.
   The bib document crosses as a **node tree** (`PostDocument::new(XmlDoc)`),
   not a serialize/reparse — which is the `make_bibliography.rs` portion of
   issue 386.

   Two things went differently from the plan above, both measured:
   * The session **reuses the live core State** instead of building a fresh
     one — Perl's own TODO (`MakeBibliography.pm` L174-177). A fresh session
     lost the LaTeX layer (103 errors, 0 bibitems), and a second
     `initialize_singletons` on one thread is a non-unwinding abort
     (WISDOM #67). Sharing the state also gives Perl's `MergeStatus` for
     free — the REPORT counters never left.
   * The **metadata fallback stays**. It is not a string-route artifact:
     Perl's `getBibEntries` has no equivalent at all (L342-343 just records a
     missing key), and the Rust path serves entries whose data comes from the
     ObjectDB.

   Measured on `bib_field_no_url_package` (full pipeline, both engines):
   Rust **0 errors / 3 bibitems**, same-host `latexmlc` **7 errors / 3
   bibitems** with the note truncated. The gap is divergence #72.
2. **DONE 2026-07-29.** The four secondary parity gaps from the audit, plus a
   fifth found while checking them and a latent panic the first fix made
   reachable. All five live in `latexml_post/src/make_bibliography.rs`; the
   whole set is guarded by
   `06_cluster_bibliography::cluster_bib_alpha_style_labels` over
   `cluster_regressions/bib_alpha_style.{tex,bib}`, whose every expectation was
   ground-truthed against same-host Perl LaTeXML 0.8.8 — after which the Rust
   and Perl bibliographies are **byte-identical** on that fixture (modulo the
   pre-existing `biblist` id noted below).

   * **citestyle semantics were SWAPPED** — the one with real corpus reach.
     Perl L481-517 has exactly three branches: `numbers` → `[1]`; **`AY` → the
     abbreviated `[AS64]` label** (class `ltx_bib_abbrv`); *anything else* →
     spelled-out author-year. Rust read `AY` as author-year, `alpha` (a string
     nothing emits) as the abbreviated one, and every unknown value as
     `numbers`. Since `\bibliographystyle{alpha}` sets `CITE_STYLE=AY` (Perl
     `$BIBSTYLES`, `latex_constructs.pool.ltxml` L3953-3961 — the Rust table
     matches), **every alpha-styled document got the wrong label shape**, and
     natbib's `super` fell to numbers instead of author-year.
   * **`{ay}` and `{initial}` were keyed off the wrong name string.** Perl
     computes two (L318-337): the full `$sortnames` keys the sort, the SHORT
     `$names` ("Smith et al") keys disambiguation and the split-by-initial
     bucket. Rust used the full form for all three, so two 3+-author entries
     sharing a first author and year never collided and **neither was ever
     given its `a`/`b` suffix**.
   * **`unisort`** — now collates (primary-level UCA, no new dependency);
     divergence **#84** records what that does and does not reproduce.
   * **doc-global NUMBER** — was assigned in document-global sortkey order
     inside `get_bib_entries`; Perl assigns it in FORMAT order (`local $NUMBER`
     L55, `++$NUMBER` L418), which under `--splitbibliography` is
     initial-major. Moved to `process`, walking the same order the biblists are
     built in. **This changes no output today** and the PR says so: `post.rs`
     constructs the processor with `split = false` and `--splitbibliography` is
     in the deferred CLI cluster, so the split branch is unreachable; the
     non-split walk numbers in exactly the order the old pass did. It is a
     correctness fix for when that flag lands.
   * **`Formatter::Year` does NOT drop a suffix** — the audit item was read off
     the Perl source's sigil. `do_year` L613-615 reads the ARRAY `@…::SUFFIX`
     while L417 binds the SCALAR `$…::SUFFIX`, so the letter never reached the
     body in Perl either; measured Perl output is ` (1999)`, and `alpha.bst`
     agrees. **KNOWN_PERL_ERRORS #67**; the non-emission is now pinned by the
     guard so it is not "fixed" back.
   * **`make_alpha_label` byte-indexed a UTF-8 string** — `aa.len() > 3` /
     `&aa[..3]` over per-author initials, so a multi-byte initial (`Ångström`)
     could panic on a char boundary. Character-based now, and the stray
     `to_uppercase()` Perl's multi-name branch does not have is gone. Latent
     before, reachable for every `alpha` document after the citestyle repair —
     which is why it is in this change and not a follow-up.

   **Found, not fixed (separate defect):** a Rust `<ltx:biblist>` carries
   `xml:id` but no `fragid`, and the XSLT's `add_id` emits the HTML `id` from
   `@fragid` only — so Perl's `<ul id="bib.L1">` comes out as a bare `<ul>` in
   Rust. Pre-existing, unrelated to this change, and the XSLT template is
   byte-identical between the engines, so the divergence is upstream of it in
   whatever assigns `fragid` to a post-created node.
3. **Field-interpretation whitelist — RESOLVED by item 1, not by widening the
   list.** Flagged by the 2026-07-05 commit review of `ede2bdcc2c`: the
   `.bib`→XML path in `make_bibliography.rs` digested only 13 fields
   (author/editor/title/year/journal/journaltitle/booktitle/volume/number/
   issue/pages/publisher/note), while Perl's `BibTeX.pool.ltxml` has ~28
   `\bib@field@default@*` constructors that DO digest — incl. `abstract`
   (L708), `keywords` (L732), `annote` (L680), `series`, `institution`,
   `organization`, `school`, `edition`, `chapter`, `howpublished`,
   `translator`, `subtitle`, `type` — so Perl raised (and MergeStatus'ed) the
   undefined-macro errors those fields carry and Rust did not. (The commit's
   original "mirrors Perl" comment was factually inverted; corrected in-code
   2026-07-05.) The 2026-07-05 decision was to keep the narrow set as a first
   stage and widen it when the recursive core session landed. It landed, and it
   **digests fields the Perl way by construction** — there is no whitelist left
   to widen: the 13-name constant went with the string route, and the
   `\bib@field@default@*` name sets now match exactly (**45 unique names on each
   side**, `bibtex.rs` vs `BibTeX.pool.ltxml`; `diff` of the two sorted lists is
   empty). **Three of those constructors are a deliberate exception, not an
   omission:** `abstract`/`keywords`/`contents` are read `Verbatim` per
   divergence **#73**, and that is the sole surviving narrowing of Perl's set. The ADS/Zotero junk-field error floods the whitelist
   was suppressing are handled at their root instead, by the DATA-regime
   treatments (#74/#79).

Witness: 2605.00223 (ADS .bib: `{\'\i}`, `~` ties, `\aap`, bare DOIs).
