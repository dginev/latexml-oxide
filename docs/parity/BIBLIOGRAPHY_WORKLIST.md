# Bibliography worklist — targets + the MakeBibliography re-port

> Lifted out of `docs/SYNC_STATUS.md` on 2026-07-25. Two related bodies of
> work: the surveyed "missing references" target list (2026-07-12) and the
> MakeBibliography full-parity re-port (user directive 2026-07-04 — reuse TeX
> interpretation, no special-case parser).

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

Fix: both `.bib` read sites (`pre_bibtex::new_from_file` engine-side,
`make_bibliography::convert_bib_file_to_xml` post-side) now decode via the new
shared `latexml_core::mouth::decode_input_bytes` — UTF-8, else a **Latin-1
passthrough** (lossless byte → char, so accented names survive intact instead of
collapsing to U+FFFD; legacy `.bib` files are overwhelmingly Latin-1/Cp1252).
The Mouth's own no-encoding branch now calls the same helper, so there is one
implementation rather than three (the "bespoke duplicate shadowing a faithful
port" anti-pattern has already bitten twice here).

Breadth: 17 papers corpus-wide, 10 of them EMPTY. All 10 recovered: **0 → 336
references** (7/15/5/22/57/25/39/48/108/10), 0 dangling.

Red/green tests: `pre_bibtex::tests::non_utf8_bib_file_is_read_not_rejected`
(engine reader) **and** `06_cluster_regressions::non_utf8_bib_file_still_yields_a_bibliography`
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
Guard: `06_cluster_regressions::bib_field_markup_survives_into_the_bibliography`
(fixture carries a `\&` so a text-escaping regression cannot pass silently).

FULL RE-PORT remaining (post-release):
1. Replace `convert_bib_file_to_xml` with the recursive core conversion
   (`DigestionMode::BibTeX` + `PreBibTeX` + bibtex.rs already exist):
   inject from latexml_oxide's post-orchestration (latexml_post cannot
   depend on the converter); recover class+packages(+options) preloads from
   the document PIs; isolate/accumulate REPORT counters + log around the
   recursive session; single combined pass for multiple raw bibs
   (cross-bib @string sharing); prefer `<name>.bib.xml`; kpsewhich +
   literaldata inputs. Deletes the string parser + metadata fallback
   (~770 lines).
2. Secondary parity gaps from the audit: `unisort` (Unicode collation) vs
   `Vec::sort()`; citestyle semantics swapped (`AY` should be the
   abbreviated `[AA+yy]` label, not full author-year); `Formatter::Year`
   drops the disambiguation `@SUFFIX`; document-global NUMBER across split
   documents.
3. **Field-interpretation whitelist (first stage, not yet Perl-faithful)** —
   flagged by the 2026-07-05 commit review of `ede2bdcc2c`. The `.bib`→XML
   path (`make_bibliography.rs`) only digests 13 fields
   (author/editor/title/year/journal/journaltitle/booktitle/volume/number/
   issue/pages/publisher/note). Perl's `BibTeX.pool.ltxml` has ~28
   `\bib@field@default@*` constructors that DO digest — incl. `abstract`
   (L708), `keywords` (L732), `annote` (L680), `series`, `institution`,
   `organization`, `school`, `edition`, `chapter`, `howpublished`,
   `translator`, `subtitle`, `type` — so Perl raises (and MergeStatus'es) the
   undefined-macro errors those fields carry, while Rust currently does NOT.
   The commit's original "mirrors Perl" comment was factually inverted
   (corrected in-code 2026-07-05). Decision (user, 2026-07-05): keep the
   narrow set FOR NOW as a first stage — it suppresses the junk-field error
   floods of ADS/Zotero exports — but the eventual target is Perl's full
   rendering-field set. Bounded blast radius: this path only fires for raw
   `.bib` inputs WITHOUT a `.bbl`. Widen when the full re-port (item 1) lands
   the recursive core session, which digests fields the Perl way by
   construction.

Witness: 2605.00223 (ADS .bib: `{\'\i}`, `~` ties, `\aap`, bare DOIs).
