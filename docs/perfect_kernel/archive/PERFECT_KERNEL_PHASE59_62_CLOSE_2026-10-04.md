# Perfect kernel — phases 59-62: sweeps #135-148, the ranked path and stream G (archived 2026-10-04)

Moved verbatim from `docs/PERFECT_KERNEL.md` when the goal bar was met (sweeps #146-148, G1-G5) and the ranked path
closed. The live state (goal bar, state table, close status, open residuals) stays in `PERFECT_KERNEL.md`; open
run-329 work is in `SYNC_STATUS.md`.

Open at s135: **G1** tabularray (1 error; fixed by 59n, rank 3 — G1 is clear); ruled: chessboard_and_beamer's Fatal (D14, 2026-09-10: keep Perl's
single-pass overlays) and biblatex-ext's last error. **G2** philexmanual (fixed by 59m: 7 → 6, the 6 dangling D15) and
biblatex2bibitem-hyperref-result (5 `page.1` links, rank 7c); ruled: 3 biblatex cite/backref manuals (854 lines, D15),
7 section-in-item/figure (prerex's `\paragraph` inside its figure, OXIDIZED_DESIGN_DIVERGENCES #189), 5 singleton
dangling links (elsdoc, europecv, crossreftools, iodhbwm, jsonparse), biblatex-ext, chessboard_and_beamer. **G3** 142
manuals, 3,992 words, by sweep #132's recall-tail classes: bibliography rendering in post 37 manuals / 1,762 words
(ruling 7b), the residual "source words dropped" 72 / 1,377 (uantwerpendocs ×5 and g-brief ×2 among them, rank 5),
generated text 31 / 589 (unverified), arabi/samplebook 196 (rank 6), wheelchart 68 (newly scored). Accepted as no
conversion loss (`accepted_residuals.tsv` kind `recall`): montex, mlsquick, zanabazr (816 words — the golden's Type 3
bitmap fonts have no Unicode text layer; root-caused 2026-10-02) and 61 manuals' reference-side, date, slide-furniture
or external-tool gaps (2,396 words, from the s132 classification — agent-made, not yet verified per manual). **G4** pgf-interference-en 180.1 s (166 s
alone; rank 4). s134 → s135 had no regression: every recall mover improved (cmpj, serbian-def-cyr/proba, hindawi) but
datetime2-en-fulltext (one date word, the new `SOURCE_DATE_EPOCH` pin), no manual became invalid, every status change
was an improvement; the in-scope missing-word total rose 14,437 → 14,667 only because four manuals are newly scored
(kaytannollista-latexia, qworld, robustsample, wheelchart: +299 words).

**Sweep #136 (60b4, 2026-10-03)** — G4 met (pgf-interference-en 99 s, no timeout); G3 below 95 % 142 → 128 (the
uantwerpen, g-brief, showexpl, pgf-spectra, pdfcomment and arabi batches). Its G1/G2/recall regressions were bisected
over the batch binaries and fixed in 60d (TikZ nested pictures over-closing `svg:g`, KPE #440; a class `\maketitle`
table dropped by 59p's content test; an autoref name that is a drawing command) or recorded: withargs' `\fileversion`
is undefined as in pdflatex; xskak's and biblatex-gost's errors are RED repros (the latter needs a ruling, item 7e);
montex's recall drop is its golden's Latin text layer against 60b's Cyrillic.

**Sweep #137 (60f1, 2026-10-03)** — G1 4 → 1 (60d's fixes: codeanatomy.usage 32 → 0 errors, causets_example2 6 → 0,
argumentation-doc 1 → 0; mercatormap valid); G3 below 95 % 126 → 125 (testcv 88.6 → 100); G4 0 → 2: pgf-interference-de
and -en at the 180 s edge again (170-180 s on every sweep, -en timed out at s135; load-sensitive, the TikZ performance
item). One regression: etextools-examples 4 → 6 errors, from 60e2's per-call `\vadjust` group around a pre-tokenized short
verbatim (RED boxes-groups/vadjust_material_is_read_live; 60g reads the material live). Counts from today's lists (the
136 row was computed before montex/mlsquick were accepted: now 126 / 3,637).

**Sweep #138 (60j4, 2026-10-03)** — G3 below 95 % 125 → 121, 3,629 → 3,553 missing words (glosmathtools en/fr
89.0/90.9 → 100, figbib_sample 54.2 → 95.8, clefval example-utf8 78.8 → 100: 60h/60i); G4 2 → 0 (pgf-interference-de
and -en back under 180 s); etextools-examples 6 → 1 error (60g reads `\vadjust` material live). No regression: every
status, validity and recall change was an improvement.

**Sweep #139 (60l5 = 60l + 60n, 2026-10-03)** — G2 1 → 0 (biblatex2bibitem-hyperref-result's 5 `page.1` links ruled D15,
7c); G3 below 95 % 121 → 118, 3,553 → 3,456 missing words (jacow-collaboration 92.7 → 100, achemso-demo 89.2 → 95.2,
wsemclassic-test 92.9 → 95.1, rpsample 89.6 → 92.1: 60n's author continuation lines and end-document `\nocite`);
tcolorbox 11 → 4 errors (still a timeout, G5). G4 0 → 2: pgf-interference-de and -en at 180.1 s again (170-172 s at
s136/s138; load-sensitive, the TikZ performance item). No other status, validity or recall change. A/B 60m2 → 60l5 on
3,003 arXiv papers: identical tallies, 156 byte-diffs, all size, whitespace/`<break/>` or author-block regrouping with no
word lost. Ruling 7b applied (`accepted_residuals.tsv`, words checked per manual against the s139 lists): 12 manuals whose
missing words are the style's bibliography or citation output (biblatex-apa6-test, biblatex-fiwi ×3, biblatex-software,
biblatex2bibitem ×2, issuulinks, munich, nmbib-sample, quantumview-template, shortmathj) and ribbonproofsmanual (its `.bib`
is not shipped); slice 6 adds four citation-rendering manuals (cms-noteref-demo, cms-notes-sample,
biblatex-true-citepages-omit-example, quantum-bibliographystyle-demo): G3 118 → 101, 2,818 missing words. Slice 6's real
losses (agent_reports/2026-10-03_g3_residual_slice6.md): `\printbibliography[title=,prenote=]` and `\defbibnote`
dropped silently (biblatex-apa-test, 22 manuals' headings), fontspec's missing `fontenc[TU]` (`<` `>` `|` as ¡ ¿ —,
12 manuals), jurabib's raw `\@citex` (jbtest). From sweep #140 the S3 audit drops text under `ltx_nodisplay` (60p's
hidden bibliography, acmart's `\Description`), as it already dropped inline `display:none`.

**Sweep #140 (60t3 = 60o-60t, 2026-10-03)** — G1 1 → 0 (biblatex-gost-examples 4 → 0 errors: 60o's autoref names wait
for an `\autoref`); G4 2 → 0 (pgf-interference-de and -en converted, 99.4 / 99.2 % recall); G3 below 95 % 101 → 99,
2,818 → 2,776 missing words against the s139 ruling set (sidenotesplus tests-sidenoteplus 90.0 → 98.2 and
quantumcubemodel-doc 93.3 → 99.0: 60p's `\fullcite`; xurl, latexbangla, caesar_example up). No regression: no validity
change, every status and recall change an improvement but tcolorbox's timeout (4 → 11 errors before the cutoff,
load-sensitive as at s136-s138). G3 residual slices 7a/7b (agent_reports/2026-10-03_g3_residual_slice7{a,b}.md, 61
manuals): 43 accepted as no conversion loss (reference-side, ruled furniture, bibliography rendering 7b) and hvpygmentex
excluded (shell escape), so the s140 lists give G3 56 / 2,054; their real losses went to the 61 train (jurabib's
citations, T1/T2 `^ ~`, the Unicode profiles' OpenType flag, `\pdfsetmatrix`, ltnews/knittingpattern copyright notes).
Open from the slices: crossreftools (`\ref` of a `\@currentlabel`-only label, Perl alike), ntgclass brief's sender,
coverpage (`\input` from a raw package forces `@` a letter), exam-n (an author without a title never reaches the HTML),
nomencl `stdsubgroups` headings, hindawi's locked `\title`, xassoccnt's free-standing `\addcontentsline`, uiucthesis's
replay gate (slice 5 R3); rulings asked: bfh-ci's title-page footer, tex-label's footer labels, vhistory's `.hst`.
Slices 1-6's prose verdicts rechecked word by word on s140 (agent_reports/2026-10-03_g3_rows_slices1-6_s140.md): 25 more
manuals accepted (reference-side text layers and included PDFs, 7b, furniture), rubik excluded (shell escape): the s140
lists give G3 30 / 841. Their real losses: wheelchart's `\iftotalpages` (aux page total), unicodefonttable's
comparison-only rows, imakeidx `\printindex[…]` (sbl ×2), `\printbibliography[heading=subbibliography]` titles outside
refsections, nomentbl/nomencl group headings, `\printbiblist`, and `\trivlist\item[label]` (webquiz; KPE #456, fixed 61p:
the itemization in the enclosing group, 17 arXiv papers' proof headings).

**Sweeps #146 (61p6) and #147 (61q2, the closing head, 2026-10-04)** — G1-G5 met: 0 unclean, 0 invalid, 0 below
95 % recall and 0 missing words against `accepted_residuals.tsv`, 0 over 180 s (pgf-interference-de/-en 169.7 / 168.2 s
run alone; their s145 timeouts ran beside an arXiv A/B), and the 68 canaries the identical set. Quality row: clean
1,590 → 1,592, timeouts 2 → 0, valid 1,577 → 1,580, missing words 13,770 → 13,739, cpu 0.92 → 0.84 h. Per-document
regressions vs s145: none (tcolorbox, a timeout canary, logs 4 → 11 errors before its cutoff, as at s136-s140). 61p's
arXiv A/B was count-neutral (2 statuses better, 0 worse, words +770) but its byte classification found 18 papers whose
proofs a trivlist label's font switch had set in small caps or bold — fixed by 61q (the label digested in a group,
latex.ltx:16028), and the A/B rows now fingerprint `font=`/`color=` histograms. 61q's A/B (61m17 → 61q2, 3,003
papers): errors 4,996 = 4,996, fatals 5 = 5, 2 statuses better and 0 worse, words +728, no `tex=` change, +0.7 % time;
`font=` changed in 17 papers, in each exactly one font per recovered trivlist label (2605.27137: 75 labels, 75 small-caps
attributes added) and none removed. Left for the close: the cortex reruns of 2605/2606 and the CSS review.

**Sweep #148 (61t2, 2026-10-04)** re-checks the closing head after 61r–61t: G1–G5 met, with the same 68 canaries as
#147. Per-document changes against #147:
- nathguide: 63 → 36 errors.
- 30 pLaTeX manuals: +1 error each, a `\divide` by a zeroed `\baselineskip`. pLaTeX's format registers (plcore.ltx
  `\Cht`…) were unallocated, and 61r's `\setlength` then typesets the value as TeX does. Fixed by 61u.
- thesis-gwu: +1, `\widthof` without calc, as pdflatex.

**Cortex reruns of 2605/2606 (L6, worker 61q, 2026-10-04).**
- Against the last complete runs: 2605 errors 2,865 → 2,730 and fatals 77 → 73; 2606 errors 2,927 → 2,751 and fatals 100 → 95.
- No new Fatal cluster. 119 papers were worse by status; a paired container A/B (56il vs 61q) cut that to 48, and 61r/61s
  fixed their kernel causes:
  - a box closing on any end-group character;
  - `\setlength` read as one stream;
  - a versioned package's fallback binding run once;
  - pgf `@` arithmetic on the sp grid;
  - bindings scanning what their packages scan;
  - a bare `\array` giving `$` back.
- Second rerun on worker 61u (user, 2026-10-04; runs 327/328): 2605 fatals 73 → 68, errors 2,730 → 2,711; 2606 fatals
  95 → 83, errors 2,751 = 2,751; no new Fatal cluster. The newly fatal papers are fleet noise
  (`never_completed_with_retries`, and `TooManyErrors` papers that convert identically in both images standalone)
  except one panic (2605.18869, a column type expanded outside a preamble), fixed in 61v.
- 61v's arXiv A/B (61u4 → 61v4): errors, fatals, words, bibliographies and tables identical, no status change, +0.8 %
  time; `tex=` changed in 2 papers, both `\raisebox` reversions now carrying the raise measured with the box
  (2605.22405 `-.45\height` → -14.34pt, was -0.45pt; 2605.30146 `\depth` → 1.94pt, was 0pt).
- Workspaces cleaned (user-approved itemization, 2026-10-04): 1,163 paths, ~770 G; kept list in
  `~/data/pk_agents/main/CLEANUP_KEEPLIST.md`.
- **Stream G launched** (user, 2026-10-04): the full arXiv rerun, cortex run 329 (corpus `arXiv`, 2,908,567 papers,
  service `oxidized_tex_to_html`, started 13:06Z) on worker image `latexml-oxide/cortex-worker:61v` (94b5b38cbd,
  container `cortex-worker-61v`, 72 workers; ≈ 55 h at ~14.5 papers/s). Baselines: run 306 (2026-09-17, 2,883,701
  papers, 753,747 clean, 24,945 fatal) and run 303 (2026-08-23..26). Compare complete runs only.
- **Stream G stopped** (user, 2026-10-04): after 16,333 papers, 134 went worse into error or fatal against run 306;
  the remaining tasks stay paused until a fixed binary passes a new validation. 62a fixes the plain-document, Semiverbatim
  and DVI-option clusters (46 of the 134 → 0 on the host; 400-paper and 3,003-paper A/Bs without a worse status); 62b
  (3380ec483c) the `\emph`, comment/verbatim, acro, subfig clusters and KNOWN_PERL_ERRORS #467-#470. Fleet-environment
  validation of `cortex-worker:62b` on 527 run-329 papers: errors 2,170 → 590, Fatals 15 → 5, no status worse. Open
  items: `SYNC_STATUS.md` ("Stream G stopped").
- **Stream G resumed** (user, 2026-10-04, after the ACL ruling landed): 62c (70cf545b8a, a `\patchcmd` miss on the
  output routine succeeds; OXIDIZED_DESIGN_DIVERGENCES #441) validated on the same 527 papers in the fleet
  environment (`cortex-worker:62c`: errors 590 → 573, the 17 ACL-era papers 1 → 0, no status worse than 61v or 62b);
  run 329 resumed at 17:46Z on container `cortex-worker-62c` (72 workers), 2,891,776 tasks back to TODO. Of the 16,535
  papers 61v converted (16,333 at the stop, then its in-flight leases), its 1,698 Error and 78 Fatal results, plus 4
  Fatals that wrote no result, were set back to TODO at 17:54Z (user, 2026-10-04) so 62c reconverts them in run 329
  (ids by result-zip mtime before 17:40Z: `~/data/pk_agents/main/scratch_g/run329/rerun_ef/`); its clean and warning
  results stand. The 131 tasks the dispatcher still held from before the pause went to 62c. Compare complete runs
  only.
- The `\raisebox` ruling (61v), tabularx's own X (61s) and the pLaTeX registers (61u) are done; the remaining residuals
  (calc's `!` protocol, singletons) are listed in `SYNC_STATUS.md`.
- 61r's arXiv A/B (61m17 → 61r6): errors 4,996 = 4,996, fatals 5 = 5, 2 statuses better and 0 worse, −0.4 % time.
  `tex=` changed in 2 papers, both pgfplots ticks now printed as pdflatex prints them (2605.30713: `1\cdot 10^{-1}` →
  `0.1`; the sp sum 0.10002 has exponent −1). `font=` changed in 18: the 17 of 61q plus a 1/255 colour rounding
  (2605.24084).
- 61s's arXiv A/B (61m17 → 61s8): errors 4,996 → 3,524, fatals 5 → 3, 6 statuses better and 0 worse, words +140k,
  bibliographies +17, tables +22, −0.4 % time:
  - 2605.07596 (1,001 errors → 0, 3k → 104k words) and 2605.22562 are bare `\array`'s `$` (KPE #462).
  - 2605.26237's −5 % words are letters that A set as math after a `$$\array…\endarray$$` failed to close
    (single-letter math tokens 4,143 → 2,326).
  - The `tex=`/`font=` changes are those papers plus 61r's.

**Ranked path.** Batches come from the scoreboard (user re-steer 2026-10-01): S3 missing words, schema-invalid
in-scope manuals, timeouts. Each batch gets one reviewer round and one fix round; a synthetic finding becomes a RED
repro; one arXiv A/B at the ship candidate. Before taking a document, check it against every out-of-scope list (user
2026-10-02: out-of-scope marking exists to reach the goal sooner).
1. **Measure the bar** — DONE (f3c15b7daa, sweep #135). The lists are data beside the scoreboard: `tools/perfect_kernel/out_of_scope.tsv` (each
   ruled-out manual with its ruling and basis; the CJK clause applied to sources with ≥ 200 CJK characters or a
   zh/cn/jp/ja/tc name), `accepted_residuals.tsv` (ruled error and jing counts), and `run_doc.sh` pins
   `SOURCE_DATE_EPOCH` as `manual_net.sh:30` does (s132's recall drift was `\today`'s month). Next, **sweep #135** on
   the 59l head — the first sweep read against the bar — and re-cluster the goal set's recall tail
   (`sweep132_analysis/recall_tail_clusters.tsv`'s classes; a classified non-loss gets a list of its own).
2. **G2: philexmanual's anchor** — DONE (59m, fbca9a19da; KPE #424). `\hypertarget{id}{<display>}` puts the anchor before the display (hyperref's
   nesting-false order, hyperref.sty:4805-4810) and `localized_anchor` wraps only where the parent can hold an anchor
   (SHARED; root cause on file; LOW-MED; the gemini guard at `perfect_kernel_gemini.rs:2204` pins today's invalid
   shape).
3. **G1: tabularray's outer keys** — DONE (59n, ef92b4214a). `evaluate=`/`expand=` and `\SetTblrOuter` are ignored: collect the body and run
   tabularray's own preprocessing when they are present (RUST-ONLY, MED; root cause on file).
4. **G4: TikZ speed.** The audit's levers L1 (`macro_call` pstack path, −12..17 %) and L6 (`\expandafter`), one per
   measurement, until pgf-interference-en has margin under 180 s inside the sweep. 59q: L1's substitution half
   and the depth guard's cold path, fp1000 −7.8 %, en 164.5 → 156.6 s alone; open: L1's argument buffer, L6.
5. **G3: title-page and letter content** — DONE (59o g-brief, 59p uantwerpen). uantwerpendocs ×5 (59p) — eso-pic's `\AddToShipoutPicture*` title-page overlay
   (PERL-ORIGIN) and the frontmatter vocabulary gate's false negatives (RUST-ONLY); recall 83 → 96, 92 → 100,
   86 → 98.6; ruled 2026-10-02: the one-shot title-page overlay is kept whole, logo and form boxes included. g-brief ×2 — DONE (59o): the letter's sender (user 2026-10-01: kept) and addressee
   as frontmatter, with XSLT for the roles; beispiel2 60.7 → 97.6. Both designs on file.
6. **G3: losses not yet root-caused.** arabi/samplebook — DONE 59r (LAE/LFE fontmaps; the remaining S3 gap is the
   golden's text layer (presentation forms and slot codes, visual order), to be recorded in `accepted_residuals.tsv` at the next sweep; hvarabic is fontspec,
   not this); the residual class "source words dropped" (72 manuals, 1,377 words at
   s135), from the top — 59s took its kernel-level rows (braced `\openin` names, `\fileversion`, an unknown font
   family under pgf's `\nullfont`, textpos absolute blocks; pgf-pie, sepfootnotes, pdfcomment ×3, stubs, eso-pic);
   59t took nomentbl's 5-argument entries, glossaries `title=`, `\autoref` names (and two arXiv listings issues), 59u
   the showexpl preset (showexpl-test 50 → 95 %), 59v `\vsplit` forced breaks, 60e `\vadjust` material as its own
   paragraphs (KPE #441; open: material built where it is read and kept with its line, RED
   boxes-groups/vadjust_material_is_built_where_it_is_read, vadjust_material_stays_with_its_line); next (slice 3,
   agent_reports 2026-10-03): figbib `@fig` fields, clefval `.aux` values, glossary user keys and parent headings;
   nomencl group headings; the generated-text class (31 manuals, 589 words) to verify. Fidelity beside recall: the montex
   manuals' Cyrillic passages — DONE 60b (an LMC fontmap from the kmr fonts' encoding and ligature program, KPE #438);
   their Mongolian-script passages (LMS/LMO/LMU/LMA, no map) still read as transliteration (RED
   `fonts-nfss/lms_encoding_bicig`).
7. **Rulings** (each opens or closes a block of the bar; 7b, 7c, 7e ruled 2026-10-03 — b a later project, its G3 gaps
   ruled residuals per manual after sweep #139 (not class B wholesale: slices 4-5 found real losses inside several);
   c D15; e the autoref name evaluated lazily, errors only where an `\autoref` prints it, batch 60o):
   a. the bar itself;
   b. bibliography rendering in post — 40 manuals, 2,202 missing words at s132, the largest real-content class: the
      post-stage formatter does not print a biblatex/bibtex style's own words (K16, bibliographies from the style's
      programs; the harness has no biber `.bbl`) — in the goal, or a later project like math;
   c. biblatex2bibitem's `page.1` links — hyperref's page anchors, which the HTML has no pages for: D15 or not;
   d. token lists by reference (the structural TikZ lever) only if L1/L6 fall short. (The uantwerpen logo: ruled
      2026-10-02, the one-shot title-page overlay is kept whole.)
   e. an autoref name's diagnostics: 59t binds the name at the target (ruling 2026-10-02), so a name that cannot be
      typeset raises its errors at every target where TeX raises them only at an `\autoref` (biblatex-gost: hyperref's
      Russian `\cyr…` name under TU, 4 errors without any `\autoref`; RED singletons/autoref_name_evaluated_at_every_target).
      The same root sets a name that takes a required argument as the `~` alone (60d, DIVERGENCES #418): pdflatex
      prints a text-bearing one's text at an `\autoref` ("Hh1"), we " 1".
7f. **Finish plan (user rulings 2026-10-03, after sweep #143).** A gap is excluded without code only when no
   author-written content and nothing essential to quality (links to headings) is lost — the number of LaTeX passes
   does not decide it. Out of scope: unicodefonttable-samples, zed2e, xassoccnt, hindawi (`out_of_scope.tsv`).
   Class-level changes, in order: (i) the **second pass** a package needs, emulated per binding wherever the value is
   known by `\begin{document}` or can be filled at construction (user 2026-10-03: never a second invocation by the
   user; an in-process rerun only for the hardest case) — crossreftools' reference text and `\crtlistoflabels`;
   not page totals: HTML has no pages, so totalcount is undefined here and wheelchart's `\iftotalpages` block keeps
   the first-run reading (user 2026-10-03, `accepted_residuals.tsv`); vhistory's `.hst` (read at load, written after
   `\maketitle`) is the one in-process rerun candidate; rvwrite's quick links come from its Makefile (no pass makes
   them);
   (ii) **trivlist keeps its list** (webquiz; 17 arXiv papers' proof headings; KPE #456; 61p); (iii) **`\@currentlabel`
   reference text** (crossreftools; the arXiv idiom); (iv) **repeated `\printbibliography`**, printed only where an
   `ltx:bibliography` can stand (biblatex-apa-test, xurl). Then short-math-guide's availability marks, rvwrite's own
   boxed `\maketitle`, uiucthesis's replay gate. Sweep #145 (61m): G3 open = biblatex-apa-test, xurl (repeated
   `\printbibliography`), crossreftools (label list, `\@currentlabel` text), webquiz (trivlist), short-math-guide,
   rvwrite, uiucthesis; G4's two pgf-interference timeouts ran beside an arXiv A/B (170-172 s alone); codeanatomy.usage
   −0.8 is pdflatex's write/re-read artifact "[__codedoc_meta:n style]" now rendered "[⟨style⟩]" (no loss).
8. **Close.** The sweep that meets G1-G5; a review of every CSS change the goal made to `LaTeXML.css` (quality, and a
   port of what ar5iv needs to its standalone stylesheet, `~/git/ar5iv-css/css/` — user 2026-10-02; e.g. 59o's letter
   roles); the cortex reruns of 2605/2606 (L6); then stream G: the full arXiv rerun on the fleet.

