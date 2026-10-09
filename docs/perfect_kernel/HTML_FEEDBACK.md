# arXiv html_feedback — fidelity subgoal (living worklist)

User directive 2026-10-08: the reports readers filed at [arXiv/html_feedback](https://github.com/arXiv/html_feedback/issues)
name the defects they actually hit. Collect each report's paper (submission-only reports ignored), aggregate the reports
by the problem described, check each problem against **our current head**, and turn what persists into fixes and
perfect-kernel coverage. The reports were filed against arXiv's HTML, produced by Perl LaTeXML at the time; many predate
fixes, so every class is verified on our output before it becomes a row.

## Data (outside the repo, `~/data/html_feedback/`)

| file | what |
|---|---|
| `issues.jsonl` | all 6,950 issues (`gh api --paginate repos/arXiv/html_feedback/issues?state=all`), fetched 2026-10-08 |
| `parse.py` → `issues.tsv` | number, state, arXiv id, version, labels, title, description; 5,970 issues name an arXiv paper, **4,819 distinct papers**, 4,795 with local sources (`ids_local.txt`) |
| `work/classify.py`, `work/report.py` → `classes.tsv`, `TAXONOMY.md` | per issue: primary/secondary class (44 classes), actionable kind, confidence (maintainer label 2,985 / text rule 2,451 / guess 534); keyword rules, representative issues |
| `conv63l/<id>/` | baseline core-XML sweep of the 4,795 papers on head 63l (`1a2f6378ec`, `one.sh`: `--preload=ar5iv.sty --timeout=240`, 8 GB cap) |

Rerun the tallies: `python3 -I work/classify.py . && python3 -I work/report.py .`.

## Shape of the reports

| kind | issues | papers | open |
|---|---:|---:|---:|
| conversion | 4,430 | 3,746 | 2,525 |
| css-viewer (ar5iv-css: dark mode 330 papers, viewer UI 161, figure size 71, table overflow 63, a11y 53, mobile 34) | 769 | 723 | 143 |
| non-actionable (spam 214, errata about the paper 201, service health 114, unclear 38, kudos 21) | 771 | 566 | 49 |

## Baseline on head (63l, 4,795 papers)

Status: 1,483 no problem, 2,874 warnings, **426 error**, **3 fatal**, 9 without a status (8 PDF-only submissions,
`Fatal:invalid:not_tex_source`; 2307.07607 wall-clock timeout while loading `l3backend-pdftex.def` after cleveref).
Fatal: 2403.12744 (TooManyErrors), 2406.02363 (runaway `undefined:\maketitle` ×501), 2406.18128 (PushbackLimit).

First Error per paper, top clusters (log-scanner, `^(Error|Fatal):[^:\s]+:\S+`): `frontmatter:merged_creators` 111
(present in 118 — acmart 30, article 50, IEEEtran 9, interact 6), `unexpected:_` 22, `unexpected:&` 7,
`malformed:ltx:subsection` (in an item) 7, `undefined:\name` 7 (melba), `malformed:ltx:toccaption` 5, `undefined:\svgsetup` 5,
`unexpected:EOF` 5 (CCSXML), `malformed:ltx:break` (in a listing) 4, `misdefined:#` 4, `undefined:{biography}` 3,
`undefined:\xspace` 3, `undefined:\tbl` 3, `undefined:\usevalue` 3, `undefined:\thetitle` 3, `errmessage` 3.

## Conversion classes: papers, errors on head, verification

"Err" = papers ending with Error/Fatal on 63l. Verified = a sample of the reports checked against our XML
(FIXED / PERSISTS / VIEWER = presentational, XML right / UNCLEAR).

| class | papers | err | verified | rows |
|---|---:|---:|---|---|
| frontmatter-authors-affiliations | 547 | 50 | run-336 audits (A2) + 63m A/B over all 547 | HF2, A2 |
| tcolorbox-box | 422 | 31 | 15: 6 fixed, 5 viewer, 3 persist, 1 unclear | HF1, HF5, HF9 |
| figure-garbled-image-conversion | 260 | 23 | — needs post-processed HTML (images) | queue |
| tikz-pgf-diagrams | 239 | 18 | 15: 6 fixed, 7 persist, 1 viewer, 1 unclear; detectors vs 200-paper control: no empty/error/leaking picture, displaced text 8.5% vs 5.5% | HF11, HF12 |
| figure-missing | 209 | 14 | — needs HTML | queue |
| frontmatter-title-abstract-meta | 194 | 23 | — | queue |
| table-layout-broken | 174 | 13 | 15: 10 fixed, 1 persists, 3 viewer, 1 unclear; detector: ragged rows in 2 papers (0 in control) | HF13 |
| math-parse-wrong-rendering, math-symbols-macros, math-missing | 261 | 34 | parked: math stream | — |
| citation-links | 154 | 13 | — needs HTML (bibliography resolution) | queue |
| raw-latex-leak | 141 | 27 | 15: 12 fixed, 2 persist, 1 unclear; detector: ERROR elements in 9.3% vs 3.0% control | HF3, HF4, HF6, HF7 |
| figure-layout-subfigures-captions | 122 | 6 | — | queue |
| code-listing | 113 | 18 | — | queue |
| main-file-selection | 111 | 9 | — (ours or arXiv's autotex pick) | queue |
| document-truncated | 100 | 9 | — | queue |
| bibliography-formatting / -missing-entries | 97 / 65 | 10 / 4 | — | queue |
| other conversion classes (≤ 94 papers each) | — | — | — | queue |

## Ranked rows

| # | item | witnesses | state |
|---|---|---|---|
| **HF1** | An `lstlisting` inside a tcolorbox gets a body about one line high (230 of 290 in-box listings, 37 papers). Cause: the `\@@listings@block` whatsit (`listings_sty.rs:3116`) has no sizer, so `whatsit.rs:478` measures its body as one horizontal list, one line of the font (`font.rs:1701`); every measuring box (tcolorbox, pgf picture, minipage, `\resizebox`) and minted/piton/showexpl/tcblisting, which all go through `lst_process_display`, inherit it. TeX: aboveskip + N × `\baselineskip` + belowskip, depth 0, width `\linewidth` (listings.sty:703-705, :1765, :1818; lstmisc.sty:1285-1286, :1630/:1648); Perl has no sizer either and is far too tall (Whatsit.pm:252-255, Font.pm:667-682). Fix: record `\baselineskip`/`\linewidth` in `\@lst@startline`'s properties (:3437), aboveskip/belowskip in the block's; a sizer counting lines at `\@lst@endline` (ceil(width/linewidth) under `breaklines`); repro `boxes-groups/tcb_listing_body_height` (6 lines, TeX 84pt; scratch `~/data/pk_agents/main/scratch_hfb_verify1/rc_hf1/`); needs an OD entry | 2406.06469, 2604.25850, 2601.23265, 2402.10176, 2311.11482, 2402.07204 | **DONE 63n** (OXIDIZED_DESIGN_DIVERGENCES #462; guard `tcb_listing_body_height`) |
| **HF2** | Merged authors from the report set (`Error:frontmatter:merged_creators`, 118 papers): acmart comma list in one `\author` (30; A2 row 2b); marked lines wrapped in `\textbf{…}` (2304.13850, 2407.15815); a user separator macro `\authcomma` (2403.07809); names inside `\href{…}{\includegraphics… A, B}` (2312.07592); `Name1 Name2 ,` glued (2307.00040, 2504.09848) ; seen in the 63m A/B: an empty `\AND \\ Microsoft Research India` group read as a name, and a brace list split over two lines (2403.00393); marked affiliations inside a brace group `{\small $^\ast$A \\ $^\diamond$B}` (2408.00808); a note line `\emph{(Invited Paper)}` read as an affiliation (2408.00952); a mixed line `{paulgc, zhedong}@google.com, Google Inc.`; an accented name against an ASCII address (andresp, "André Susano Pinto", 2411.19722); a PoS `E-mail:\email{x}` line in a marked block leaves an email contact reading "E-mail:" (0811.4645, on HEAD too); an out-of-order address line is one contact, not each address to the name it spells (2509.10377, 2406.06326) | as listed | **OPEN**; 63m landed group lines, address lines, names-only lines, cross-group address owners, brace groups |
| **HF3** | A binding serves a class/package the paper ships and lacks what that file defines (Perl's bindings share the gaps): svmult `\runinhead` (svmult.cls:701; 1805.00023), interact `\tbl` (2312.11500, 2501.02233), IEEEtaes `\member` (2403.15966), neurips_2024 → xspace (neurips_2024.sty:22; 2401.08140, 24 errors), mdpi → soul `\hl` (mdpi.cls:48; 2312.16815), jcappub journal macros (2404.02153, 64 errors), `\svgsetup` (5), melba `\name`/`\aff` (7) | as listed | **DONE 63o** except neurips_2024 (a paper's own edited copy) and melba (HF6); KNOWN_PERL_ERRORS #536 |
| **HF4** | acmart `\Description` keeps its argument undigested, so markup leaks as raw TeX into the `aria-describedby` note (`acmart_cls.rs:223-225`; Perl digests it): 17 of 106 papers with such notes . Also (Perl-origin, Perl 0.8.8 the same): `\caption{…}\Description{…}\label{x}` labels the hidden note, not the float — `\label` takes the nearest id-bearing element from the current node's last child (`document.rs:6844-6866` `float_to_label`; Perl Document.pm:1098-1123), and the note moves into the caption only at float close (`latex_constructs/mod.rs:3361`), so `\ref{x}` points at the note (2403.09168: 10 labels). acmart.cls:895 `\Description` typesets nothing. Fix in the binding: move the note into an existing caption at construct time; repro `scratch_hfb_verify1/rc_hf4/desc_label.tex` | 2403.09168, 2401.04997, 2304.01062, 2312.11013 | **DONE 63n** (KNOWN_PERL_ERRORS #535; guards `acmart_description_label_after_names_the_float`, `acmart_description_markup_is_digested`) |
| **HF5** | A font-size switch covering a whole tcolorbox body is lost (`fontupper=\footnotesize`, a leading `\scriptsize`, `\small`) | 2410.10630, 2310.04475, 2605.23904 | **OPEN** |
| **HF6** | Content LaTeX typesets later is digested at once: melba.cls:279-280 defines `\aff`/`\name` inside its `\maketitle` group (244 errors); an acmart abstract using macros `\include`d later (80 errors) | 2405.09787, 2510.00328 | **melba DONE 63q** (the title code's definitions open at `\@author` are in force for the author content, 7 witnesses 2-3 errors to 0; OXIDIZED_DESIGN_DIVERGENCES #464, KNOWN_PERL_ERRORS #538). OPEN, low: acmart's abstract is typeset at `\maketitle` (`\Collect@Body`), LaTeXML digests it at once, so macros a later `\include{macros.tex}` defines are undefined there (2510.00328, 4 errors; Perl the same) |
| **HF7** | Error clusters of the report set beyond HF2/HF3: `unexpected:_` (22 papers), `unexpected:&` (7), `ltx:subsection` in an item (7), `ltx:toccaption` in a block (5), CCSXML EOF (5), `ltx:break` in a listing (4), `misdefined:#` (4) | per cluster: 2308.06669, 2205.04522, 2304.10050, 2312.04535, 2401.14656, 2301.04312, 2310.04475 | **OPEN**, triage per cluster (`canvas-triage`) |
| **HF8** | tcolorbox residuals: display math in a box body measured short (2310.02875, 2509.09737); a `remember picture,overlay` node is a 1×1 SVG (2111.13530); a box wrapping the abstract hides it from the frontmatter (microsoft-tech-report.sty:84, 2605.23904); boxes wider than the geometry text width (2409.00729, 2308.10974) | as listed | **OPEN** |
| **HF9** | Small: ulem's declaration form `{\ul …}` underlines one token (`ulem_sty.rs:84-98` lets it to `\uline`; ulem.sty:286-293; 2406.03441); `\textcircled{\raisebox{…}{1}}` prints the raise's source (`sect13.rs:1199`, KNOWN_PERL_ERRORS #361; 2311.00960, 2402.11753); refcount `\getrefnumber` without an `.aux` gives 0 (refcount.sty:233; 2601.14040) | as listed | **OPEN**, low |
| **HF10** | Verify the queued classes on head (sample ≈15 reports each, as above; the figure, citation and bibliography classes need post-processed HTML): figure-garbled, figure-missing, title-abstract-meta, citation-links, subfigures, code-listing, main-file-selection, document-truncated, bibliography | — | **OPEN** (tikz, table done) |
| **HF11** | pgf shapes whose text anchor is `\savedanchor\text` (diamond, diamond split: pgflibraryshapes.geometric.code.tex:277, pgflibraryshapes.multipart.code.tex:379) print their text far from the shape when amsmath is loaded: amstext's `\text` is locked (`amstext_sty.rs:5-6`; Perl amstext.sty.ltxml:23-25 the same), so `\anchor{text}{\text}` runs the amstext macro. Repro `hfb_verify2/probe/diamond5.tex` | 2605.26428, 2505.21642, 2602.00180 | **OPEN** (Perl-origin) |
| **HF12** | TikZ-class residuals: `\includestandalone` with only the `.tex` shipped becomes `\includegraphics{x}` with a `.tex` candidate nothing renders (`standalone_sty.rs:109-112`; standalone.sty:1037-1043 inputs the `.tex`; 10 papers: 2608.05283, 2403.01643, 2403.17633, 2404.19456, 2412.01410, 2410.02545, 2412.12317, 2504.17583, 2505.19304, 2508.06316); blkarray lacks `\BAhhline`, `\BA@colsep` (2301.06399); `\rotatebox{90}{$\in$}` in a math array loses the rotation (2601.07627); Inkscape `.pdf_tex` multi-line labels shift (2211.08508, XML or CSS unsettled); post-processing `trim`/`clip` of a PDF whose CropBox differs from its MediaBox (2312.04615); a multi-line node's lines wrap in the browser (2107.02270, CSS `nowrap`) | as listed | **OPEN**; `\includestandalone` DONE 63p (OXIDIZED_DESIGN_DIVERGENCES #463, KNOWN_PERL_ERRORS #537); residual: acmart `\Description` takes only a `<graphics>` as the described image, so a figure now holding a `<picture>` warns (2404.19456) |
| **HF13** | Table structure: empty-column pruning drops an empty spanning header cell instead of shrinking it, so the header row slides (`alignment/normalize.rs:472-540`, Perl Alignment.pm:812; 2406.06521 Table 2); empty-row pruning keeps a `\multirow{5}`'s rowspan 5 (`normalize.rs:248`, Perl Alignment.pm:741; 2507.20312 Table 2); `\lxRequireResource[content=…]` builds the resource with empty content (`latexml_sty/mod.rs:807-820`; Perl latexml.sty.ltxml:246-248 passes the options; 2606.12996, Rust-only) | as listed | **DONE 63s**: the spanner shrinks, a covered `\multirow` spans no rows of its own (OXIDIZED_DESIGN_DIVERGENCES #467, Perl the same); `content=` passed and kept under ar5iv (#468) |

Settled for this subgoal: the css-viewer kind goes to ar5iv-css triage, not here; the math classes wait for the math
stream (parked 2026-10-02); 2307.07607's timeout and the 8 PDF-only submissions are pipeline, not conversion.
