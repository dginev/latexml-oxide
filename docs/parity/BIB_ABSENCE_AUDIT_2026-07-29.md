# Bibliography-absence audit — 2605/2606 sandboxes + full /data/arxiv (2026-07-29)

> **Mission.** Find every converted document that *lacks* any `class="ltx_bibitem"`
> entry — i.e. has no bibliography or a wrongly empty one — across (a) the
> `/data/arxiv/2605` sandbox (sandbox-13 rerun, 2026-07-27), (b) the
> `/data/arxiv/2606` sandbox (sandbox-14 rerun), and (c) the full arXiv corpus
> result zips under `/data/arxiv/<yymm>/` (2026-07 rerun). Then cluster causes
> and drive them down in mini-sprints (plan in `docs/SYNC_STATUS.md`, row TBD;
> related standing work: [`BIBLIOGRAPHY_WORKLIST.md`](BIBLIOGRAPHY_WORKLIST.md)).
>
> **Complete case lists** live beside this file in
> [`bib_absence_2026-07-29/`](bib_absence_2026-07-29/) — one TSV per corpus:
> `id, verdict, cortex_status, expect, srcsig, category, first_error_class`.

## Method (fail-safe toward flagging)

Two passes, both scripted (scan_bib.sh / pass2.sh, session scratchpad):

- **Pass 1** streams every `*.html` member plus the `status` member of each
  result zip through one `unzip -p | awk` and assigns a verdict:
  `ok` (≥1 `ltx_bibitem`), `empty_bib` (bibliography/biblist markup but zero
  items), `no_bib` (HTML, no bibliography markup at all), `no_html` (no HTML
  bytes — includes zero-byte HTML and corrupt zips), `no_result` (no zip).
  Anything unreadable lands in a flagged bucket, never in `ok`
  (CLAUDE.md canvas-signal rule).
- **Pass 2**, over flagged papers only, adds: **expect** — does the *source*
  ask for a bibliography (`.bbl`/`.bib` members; `\bibliography`,
  `thebibliography`, `\printbibliography`/`\addbibresource`, `\bibitem` in any
  `.tex`; `%auto-ignore` stub detection) — plus telemetry `category`,
  `output_bytes`, and the first ANSI-stripped `Error:`/`Fatal:` log line.

The result zips used: `oxidized_tex_to_html.sandbox-13.zip` (2605),
`…sandbox-14.zip` (2606), plain `oxidized_tex_to_html.zip` (corpus dirs,
2026-07 full rerun).

## Headline numbers

| corpus | docs | ok | flagged | **wrongly-missing** (HTML present, source wants a bib) | lost-doc (`no_html`) | wrongly-empty subset | legit (no source bib / `%auto-ignore`) | no_result |
|---|---|---|---|---|---|---|---|---|
| 2605 (sandbox-13) | 30 079 | 29 454 (97.9%) | 625 (2.08%) | **255** (249 + 6 legacy) | 257 (53 = `%auto-ignore` stubs) | 69 `empty_bib` | 107 | 6 |
| 2606 (sandbox-14) | 30 430 | 29 807 (98.0%) | 623 (2.05%) | **278** (272 + 6 legacy) | 213 (28 = stubs) | 79 `empty_bib` | 126 | 6 |
| full corpus (2026-07 rerun) | 2 790 409 | 2 637 113 (94.5%) | 153 296 (5.5%) | **52 299** (35 835 + 16 464 legacy) | 56 588 (11 957 = stubs) | 12 994 `empty_bib` | ~64 500 | 3 135 |

**Corpus total actionable: 52 299 wrongly-missing (1.87% of all docs) + 18 919
lost-doc fatals with bib intent + 3 091 `no_result` with bib intent.** "Legacy"
= source uses amsrefs/aastex/harvmac-era conventions (`\begin{biblist}`,
`\bib{`, `\reference`, `\begin{references}`, `\listrefs`, `\Refs`) that the
first-pass `expect` classifier missed (pass 2b; witness 0704.0808).

**Binary-dating caveat (matters for family sizing):** the corpus zips are from
the 2026-07-02..04 full rerun — a binary that PREDATES the R5
recursive-bib-session re-port (~2026-07-25). The F1 `$$`-cascade markers
(`Error:bibliography:convert`, `\end{bibtex@bibliography}`) score **zero** in
the corpus logs for that reason; F1's corpus footprint must be projected from
the sandbox rate (~0.11% of docs) or re-measured after the next corpus rerun.
Sandbox-13/14 (2026-07-27 binaries) carry the current code and are the
canonical measurement for engine-side families.

Notes:
- *wrongly-missing* = pass-1 `no_bib`/`empty_bib` **and** the source asks for a
  bibliography. This is the actionable core list.
- *lost-doc* = `no_html`: the entire document is absent (mostly
  `conversion_fatal`), bibliography is collateral. Counted separately so the
  bibliography-specific sprints aren't diluted by the general-fatal mission.
- A `no_html` with cortex **status 0** turned out to be almost entirely
  `%auto-ignore` withdrawal/PDF-only stubs (12-byte `.tex`, zero-byte HTML,
  `output_bytes:0`, category `ok`) — witness 2605.03462. Legit, excluded.

## Verdict × source-expectation × telemetry category (sandboxes)

Per-sandbox cross-tab (counts 2605 / 2606):

| pass-1 verdict | expect | category | 2605 | 2606 |
|---|---|---|---|---|
| no_html | yes | conversion_fatal | 202 | 178 |
| no_bib | yes | conversion_error | 144 | 154 |
| no_bib | no | ok *(legit)* | 83 | 101 |
| no_html | auto_ignore | ok *(legit)* | 53 | 28 |
| empty_bib | yes | conversion_fatal | 32 | 36 |
| no_bib | yes | conversion_fatal | 21 | 22 |
| empty_bib | yes | conversion_error | 20 | 18 |
| no_bib | yes | **ok (SILENT)** | 17 | 22 |
| empty_bib | yes | **ok (SILENT)** | 15 | 20 |
| no_bib | no | conversion_error | 26 | 25 |
| (small residual rows) | | | 12 | 19 |

The two **SILENT** rows — clean telemetry, source wants a bibliography, none in
the HTML — are the highest-value defect class (zero-error signal masking real
loss). 74 papers across both sandboxes.

## First-error classes over the flagged wants-bib set (both sandboxes)

`-` (none) 76 · `\lx@begin@alignment` 59 · `Fatal:Timeout:TokenLimit` 57 ·
`expected:\fi` 47 · `Fatal:Timeout:PushbackLimit` 28 · `unexpected:\endgroup`
21 · `unexpected:\lx@end@inline@math` 17 · long tail below ~15 each.

Bibliography-*specific* log lines across the same set:

| signal | papers |
|---|---|
| `Error:bibliography:convert` — recursive BibTeX session produced no bibliography (subset: `TooManyErrors` blowups with `\end{bibtex@bibliography}` cascades) | 69 |
| `Warning:bibliography:missing_keys` listing (often *all*) citation keys | 112 |
| `Error:malformed:ltx:bibliography` — `<ltx:bibliography>` opened inside `<ltx:XMath>` | 9 |
| `Error:undefined:\printbibliography` (biblatex path) | 8 |
| `Error:undefined:{thebibliography*}` (starred env) | 6 |

## Cause clusters

### Lost-doc fatals (`no_html` × `conversion_fatal`; 382 sandbox papers)

Tally over all 382 (subagent sweep, 2026-07-29): **Fatal:Timeout 216** (runaway
loops / TokenLimit) and **Fatal:TooManyErrors 166** (>100-error floods). Every
one carries a proper `Fatal:` diagnostic (no silent kills); RSS stays <2 GB for
361 of them, 2–8 GB for 21. Witnesses: timeout 2605.00058, 2605.00182,
2605.00503; error-flood 2605.00553, 2605.00601, 2605.01773. This bucket is the
general fatal-mining mission (bibliography is collateral), *plus* 8 papers with
a source zip but **no result zip at all** (`never_completed` candidates:
2605.07397, 2605.18285, 2605.13456, 2605.11365, 2605.01124, 2605.06546,
2606.28114, 2606.09822).

### `no_bib` + errors, HTML present (298 sandbox papers) — mostly collateral truncation

Subagent deep-dive (6 samples + cohort-wide heuristics), 2026-07-29:

- **Tail loss dominates**: of 234 cohort sources containing "acknowledg", only
  84 HTMLs retain it — 64% lost the document tail; median output/source byte
  ratio 5%; 41 papers are <3 KB stubs.
- **achemso `\iffalse` runaway** — `expected:\fi \iffalse` fires from the
  achemso/OmniBus *preamble path* (no `\iffalse` in user source); the body,
  including `\bibliography`, is skipped as a false branch. **36 of 42
  `expected:\fi` papers are `\documentclass{achemso}`** (~12% of the cohort).
  Witnesses: 2605.00451, 2606.00264.
- **Unclosed-group swallow after alignment errors** — after
  `unexpected:\lx@begin@alignment`, digestion continues (the `.bbl` is even
  read: log "Processing content main.bbl") but content lands in a never-closed
  group that is discarded; HTML truncates at the error site. Closing leaked
  groups at the error (or `\end{document}`) would recover the tail incl. the
  bibliography. Witnesses: 2605.05903, 2606.02744.
- **Document-build abort stub** — `Error:document:open_element_internal` →
  `Error:document:convert` (`latexml_oxide/src/converter.rs:392`) discards the
  ENTIRE document: identical 1809-byte "Untitled Document" stubs from 773 KB /
  143 KB sources. Violates the recover-partial-output principle. Witnesses:
  2605.00808, 2606.10727 (10 in cohort).
- **Post-stage collapse (tail-intact minority, ~84 papers)** — text complete
  through "References" but the post stage dies (XPath "growing nodeset" OOM →
  NULL XSLT context; `post:parse` null pointer), taking bib markup, citations
  and math with it. Same post-stage-blindness family as the
  `convert_and_post_clean` guard work. Witnesses: 2605.02373 (6.3 MB HTML, 0
  math, empty `[ ]` citations), 2605.17121 (31.8 MB HTML); biblatex-empty
  variant 2605.00270.

### The synthesized cause-family taxonomy (F1–F11)

Four further deep-dives (silent `.bib` loss, silent `.bbl` loss, recursive
bib-session failures, empty-bib cohort — all 2026-07-29, subagent sweeps with
same-host Perl cross-checks; the minimal repro inputs are preserved in
[`bib_absence_2026-07-29/repros/`](bib_absence_2026-07-29/repros/)) converge
on one taxonomy. Counts are sandbox-pair
counts unless marked corpus.

**F1 — In-bib `$$` entry-poisoning cascade. GENUINE RUST; the biggest single
bug.** A `.bib` field containing *adjacent* inline math — Google-Scholar title
mangles `$\{$LLMs$\}$$\{$…$\}$` / `$$\backslash$alpha $`, APS/CMS exporter
juxtapositions `${\mathrm{La}}_{2}$${\mathrm{CuO}}_{4}$`, `T$'$$\to$` — reads
as a display-math `$$` inside `bib@entry`; the math never closes, the entry
group unwind cascades through `\end{bibtex@bibliography}`
(`unexpected:} … switched to mode math` → `malformed:ltx:XMTok/bib-*` flood →
`TooManyErrors` → "Recursive BibTeX conversion produced no bibliography").
**One bad entry voids the whole bibliography** and drives the paper to
status 3. 66/69 `bibliography:convert` papers (A1 GS-braces 19, A2
GS-backslash 17, A3 physics-exporter 32) + 2 stray-backslash brace-eaters
(`journal={\aap\ }`, `…\}`) + the sibling `{\color{red}…}`-in-`\csname` poison.
0/69 ship a `.bbl` fallback. Perl: same text → "Missing $ inserted"-style
recovery, References kept (min repro: Rust 18 errors, Perl 0). bibtex+pdflatex
likewise keep the list. Witnesses: 2605.03129, 2605.00125, 2605.27979,
2605.13649, 2606.25054, 2606.28450, 2605.01115 (Perl 31 bibitems / sandbox
Rust 0; today's binary 14 — pre-poison entries survive), 2606.03480. Fix
direction: treat `$$` in the restricted-horizontal bib context as TeX does
*and* contain damage per-entry at the `\end{bib@entry}` boundary (close
dangling math/box frames, drop that entry only).

**F2 — Rust-made SILENCE around real loss.** The loss is often parity; the
*zero-error signal* is ours. (a) Missing-bibliography-file: Perl raises
`Error:missing_file` (`Post/MakeBibliography.pm:139`); the Rust port emits
only `Info!("bibliography","missing",…)` (`latexml_post/src/make_bibliography.rs:298-303`)
— exactly why bibunits/missing-`.bib` papers land in telemetry `ok`. (b) EOF
with an open environment: Perl `Error:expected:\end{split}`, Rust silent
(2605.19817). (c) No signal when MakeBibliography selects zero entries —
`N bibentries, 0 cited` + `Missing Entry for citation` are Info/Warning only
(24/38 resp. 20/38 of the silent cohort carry them). Restoring (a)+(b) is
parity-correct and moves ~16 silent papers into visible categories.

**F3 — `.bbl`-path fidelity: loss REAL vs the arXiv-PDF ground truth, mostly
shared-with-Perl.** (a) *Empty-arg* `\bibliography{}` with a jobname `.bbl`
shipped: `\lx@ifusebbl` returns on empty arg with no message
(`latexml_engine/src/latex_constructs.rs` ~L8253 = Perl
`latex_constructs.pool.ltxml:3901`), while real latex.ltx does
`\@input@{\jobname.bbl}` unconditionally — full parity, silent in both
engines; 7 papers incl. the GWTC-5 LIGO set; witness 2605.27226 (+min repro).
(b) *REVTeX 4-2 `auto@bib`*: `\bibliography` commented out, `.bbl` shipped,
real class auto-inputs `\jobname.bbl` at end-document (revtex4-2.cls L5972,
L7275) — not emulated by either engine; 3 papers; witness 2605.03978.
(c) *bibunits `\putbib`* resolves through the BibTeX path only; the shipped
`bu1/bu2.bbl` (what pdflatex reads) are never consulted → empty References
sections; 14 silent + 23 in the empty-bib cohort (overlapping); witnesses
2606.04416 (Perl-verified same loss, but Perl raises the F2(a) Error),
2605.21570.

**F4 — biblatex binding defects. GENUINE RUST** (in surpass code Perl lacks):
(a) `\refcontext[]{}` noop **eats a following `\printbibliography` as its `{}`
argument** (`latexml_contrib/src/biblatex_sty.rs:2193`; 2606.11276,
2606.02676); (b) `\addbibresource` signature lacks the `[]` optional, so
`[location=local]` corrupts the resource list (`biblatex_sty.rs:1781`;
2605.27263); (c) `\addbibresource` living inside an *unbound* `.cls` is
dependency-mined but never registered (2605.23724); (d) apa6 OmniBus
dep-mining takes the biblatex branch the document didn't, re-`\let`ting
`\bibliography` (2605.14990). Plus the plain gaps: `undefined:\printbibliography`
8, `undefined:{thebibliography*}` 6.

**F5 — Silent mid-document swallow. GENUINE RUST; top content-integrity
find.** Whole document tails — bibliography included — vanish with **zero
errors**: (a) runaway listings/verbatim renders the rest of the paper as
literal text (the HTML tail shows `\bibliographystyle{…} \bibliography{…}
\end{document}` *as text*) or drops it — 6 witnesses, 2606.08339 reproduced
on today's binary, also 2605.03954, 2606.05629, 2606.10056, 2605.28598,
2606.23302; (b) ar5iv-preload group leak: `\xpretocmd\myinline{…\bgroup…}`
under raw-loaded xpatch/collectbox leaks a `\bgroup`, swallowing line 209→EOF
(2606.16679; plain Rust and Perl both fine — fleet-config-only); (c)
unterminated `split` swallow (2605.19817 — loss parity, silence ours, F2(b));
(d) alignment unclosed-group swallow — digestion continues, the `.bbl` is even
read, but everything lands in a discarded group (2605.05903, 2606.02744).

**F6 — achemso `\iffalse` runaway. GENUINE RUST, concentrated.** An `\iffalse`
opened in the achemso/OmniBus *preamble path* (no `\iffalse` in user source)
never closes; the whole body is skipped as a false branch → title+abstract
stubs. 36/42 sandbox `expected:\fi` papers; corpus first-error `expected:\fi`
= 1 700. Witnesses 2605.00451, 2606.00264.

**F7 — Document-build abort stub. GENUINE RUST, violates recover-partial.**
`Error:document:open_element_internal` → `Error:document:convert`
(`latexml_oxide/src/converter.rs:392`) discards the ENTIRE built document —
identical 1809-byte "Untitled Document" stubs after 44 s of work. Sandbox 10;
corpus first-error count 436. Witnesses 2605.00808, 2606.10727.

**F8 — Post-stage collapse on huge documents.** Text complete through
"References", then the post stage dies (XPath "growing nodeset" OOM → NULL
XSLT context; `post:parse` null pointer), taking bib markup, citations, math.
Corpus first-errors: `post:convert` 520, `xpath:findnodes` 177. Witnesses
2605.02373, 2605.17121. Related: 9 papers `<ltx:bibliography>` inside
`<ltx:XMath>` (`malformed:ltx:bibliography`).

**F9 — Faithful-to-broken-source (PDF lacks the references too; no parity
work, surpass-tier only).** (a) Shipped raw styles clobber `\cite`
(aaai/iccc/flairs/kr/achicago/harvard/fixbib under `--includestyles`): session
runs, "N bibentries, 0 cited", bold `?` citations — Perl identical under the
same flag; none ship a `.bbl`, so the arXiv PDF shows `?` too. 11 silent + 28
+ 20 empty-bib papers. Witnesses 2605.07102, 2606.32016, 2605.15421,
2605.00671 (the `\affiliations` cluster). (b) `\nocite{*}`-only docs: the
star key is skipped line-for-line as in Perl (`MakeBibliography.pm:279-313` =
`make_bibliography.rs:570-583`); 7 papers. (c) imsart aux-write path, no
`.bbl` (2). (d) `\printbibliography`/`\bibliography` commented out or orphaned
`.bbl` (2605.29754, 2606.09394, 2606.31667). Surpass levers if ever wanted:
bind the aaai family, implement bibtex-true `\nocite{*}`, read `bu*.bbl`.

**F10 — Lost-doc umbrella (the general fatal mission, bib collateral).**
Sandbox 382 `no_html` fatals: Fatal:Timeout 216, Fatal:TooManyErrors 166
(some of the flood cases are F1 upstream). Corpus: 41 941 `no_html` status-3.
Plus 8 papers with a source but NO result zip (`never_completed`).

**F11 — Harness/corpus-prep.** (a) Decoy-toplevel selection: cortex converts
an IEEE-copyright stub `arXiv.tex` while `00README` lists the real
`main_RAL.tex` (2606.01946). (b) 4 empty 2606 paper dirs with stray top-level
`.gz` files (corpus-prep artifact).

**Classifier limitation (affects the corpus lists):** pass-2 `expect` missed
amsrefs (`\begin{biblist}`/`\bib{`), aastex `\reference`, harvmac
`\listrefs`, `\begin{references}` conventions — witness 0704.0808 (amsrefs,
empty biblist, `expect=no`). A pass-2b re-check with extended signals over all
`expect∈{no,no_tex}` rows corrects the corpus totals below.

## Full-corpus results

Verdict distribution over 2 790 409 docs: `ok` 2 637 113 · `no_bib` 80 579 ·
`no_html` 56 588 · `empty_bib` 12 994 · `no_result` 3 135. Expect over the
flagged set (after pass 2b): `yes` 57 845 · `yes_legacy` 17 055 · `no` 57 870 ·
`auto_ignore` 13 871 · `no_tex` 6 651 · `no_src` 4.

First-error classes over the 35 835 non-legacy wrongly-missing: **silent (`-`)
12 583 (35%)** · `\lx@begin@alignment` 2 356 · `expected:\fi` 1 700 (F6
achemso) · `unexpected:\omit` 713 · `post:convert` 520 (F8) ·
`\lx@end@inline@math` 458 · `document:open_element_internal` 436 (F7) ·
`undefined:\setboolean` 416 · `\endgroup` 407 · `\lx@tag@intags` 372 · long
tail. The silent block decomposes by srcsig: `bbl,bibcmd|empty_bib` 2 987 ·
`bbl,bib,bibcmd|no_bib` 1 201 · `bibcmd|no_bib` 1 170 (names a `.bib` not in
the zip) · `bbl,bibcmd|no_bib` 1 084 · `thebib,bibitem|no_bib` 958 · … — i.e.
the F2/F3 `.bbl`-path and selection-zero families dominate corpus silence.

Bibliography log markers over all 93 573 corpus `no_bib`+`empty_bib` papers
(pre-re-port binary): `Missing Entry for citation` 32 244 ·
`bibentries, 0 cited` 12 388 · `missing_keys` 11 976 · `Couldn't find usable
bibliography` 691 · `bibliography:convert`/`bibtex@bibliography` **0** (see
the binary-dating caveat).

Legacy-convention signal counts (pass 2b, over `expect∈{no,no_tex}` rows;
overlapping): amsrefs `\bib{` 6 479 · `biblist` 6 217 · `\begin{references}`
5 186 · harvmac `\listrefs` 4 311 · aastex `\reference` 4 223 · `\Refs` 483.
Spot-checks 0704.0777 (harvmac) and 0704.0420 (`references` env): **zero
errors, no bibliography** — F12 is a silent family. Rust ships bindings
(`harvmac_tex.rs`, `aastex*_*.rs`, `amsrefs_sty.rs`); Perl has only
`amsrefs.sty.ltxml` — harvmac/aastex-era handling is already beyond-Perl
territory.

## Mini-sprint plan

Ordered by measurement-integrity-first, then impact ÷ effort. Each sprint is
self-contained (fresh branch + PR per project policy); sandbox-13/14 rates are
the acceptance metric, re-measured on the witnesses named above.

| sprint | family | scope & fix direction | est. size | recovers (sandbox pair / corpus proj.) |
|---|---|---|---|---|
| **S1 — Un-silence the loss** | F2 | Restore the two parity Error raises: `make_bibliography.rs:298-303` Info→Error (Perl `MakeBibliography.pm:139`); EOF-with-open-environment error. Consider Warning→Error for all-keys-`missing_keys`. Guards: the 2605.27226 / 2605.19817 / 2606.04416 witnesses must stop reporting `ok`. | small (1-2 days) | signal only, ~28 sandbox papers leave `ok`; unlocks honest telemetry for S2-S8 |
| **S2 — `$$`-in-bib containment** | F1 | Treat `$$` in the bib restricted-horizontal context as TeX does; per-entry error containment at `\end{bib@entry}` (close dangling math/box frames, drop only that entry). Min-repros ready ([`bib_absence_2026-07-29/repros/f1_bib_cascade/`](bib_absence_2026-07-29/repros/f1_bib_cascade/), incl. `bibbisect.py`). | medium | ~69 papers + 68 fatals / pair; projected ~3 200 corpus-wide once the re-port binary ships |
| **S3 — `.bbl` fidelity** | F3 | (a) empty-arg `\bibliography{}` → `\jobname.bbl` fallback (= latex.ltx, beyond-Perl-vs-broken doc); (b) REVTeX 4-2 `auto@bib` end-document `.bbl` input; (c) bibunits `\putbib` reads shipped `bu<N>.bbl`. PDF ground truth per config-driven rule. | medium | ~47 / pair; corpus: the 2 987 `bbl,bibcmd|empty_bib` + 1 084 `bbl,bibcmd|no_bib` silent blocks are the upper bound |
| **S4 — biblatex bindings** | F4 | `\refcontext[]{}` must not eat `\printbibliography` (`biblatex_sty.rs:2193`); `\addbibresource[]` optional (`:1781`); resource registration from unbound-class dep-mining; apa6 branch fidelity. | small-medium | ~18 / pair + corpus `undefined:\addbibresource` 83, `\printbibliography` cluster |
| **S5 — achemso `\iffalse`** | F6 | One preamble-path bug in the achemso/OmniBus route; body skipped as false branch. | small | 36 / pair; corpus `expected:\fi` 1 700 |
| **S6 — Never discard a built document** | F7 | `document:convert` abort → salvage the partial DOM instead of the 1809-byte stub (recover-partial principle, cf. `fatal-stays-fatal` policy). | small-medium | 10 / pair; corpus 436 |
| **S7 — Silent swallow / group leaks** | F5 | Runaway listings/verbatim; leaked `\bgroup` under ar5iv raw-load; alignment unclosed-group discard (close leaked groups at error or `\end{document}`); each sub-family needs min-repro first. Hardest, highest content-integrity value — bib is just the visible symptom. | large (own session per sub-family) | ~6 silent + alignment cohort / pair; corpus `\lx@begin@alignment` 2 356 |
| **S8 — Post-stage robustness** | F8 | XPath growing-nodeset OOM, NULL XSLT context, `post:parse` null pointer on huge docs; also `<ltx:bibliography>`-in-XMath placement. | medium | ~13 diagnosed + 84-cohort tail / pair; corpus 520+177 |
| **S9 — Legacy conventions** | F12 | Triage first: why do harvmac `\listrefs` / aastex `references` / amsrefs docs silently lose bibs despite Rust bindings (0704.0777, 0704.0420, 0704.0808 witnesses); then fix per convention. | triage then per-convention | corpus 16 464 (largest corpus family) |
| **S10 — Surpass-tier rulings** | F9 | USER DECISION needed, not code: bind the aaai/iccc family (PDFs broken too — surpass-Perl territory), bibtex-true `\nocite{*}` (recorded PARITY 2026-07-14), read `bu*.bbl`(→S3c). Present measured tradeoffs, don't pre-decide. | discussion | ~66 / pair if all approved |
| *(routed)* | F10 | Lost-doc fatals → existing fatal-mining mission (216 timeout + 166 flood / pair; the flood subset shrinks with S2). 8 `never_completed` no-result papers attached. | — | — |
| *(routed)* | F11 | Decoy-toplevel selection (2606.01946) → cortex main-file-selection issue; 4 empty 2606 dirs → corpus-prep. | — | — |

Follow-up measurement: after S1+S2 land, rerun the 2605/2606 sandboxes and
re-run `scan_bib.sh` (~40 s/sandbox) — the wrongly-missing count and the
silent share are the two tracked numbers. The full-corpus numbers reset at the
next fleet rerun (post-re-port binary), which also gives F1 its true corpus
footprint. Side product for R9-BST: the `.bib`+`.bst`-with-no-`.bbl`
population is directly derivable from the corpus list's srcsig column.

