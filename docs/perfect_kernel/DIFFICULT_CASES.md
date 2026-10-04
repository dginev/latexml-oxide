# Perfect Kernel — difficult / open-ended cases (catalog)

Cases where "perfect conversion" is not a mechanical kernel fix but a design
question. Each entry: what the construct is, why it is hard under raw
interpretation, and the current plan. Add cases as sweeps surface them; move
an entry to the ledger's fix log when it stops being open-ended.

## D1. PGF/TikZ drawing layer

Most modern package manuals draw their own figures with TikZ. Raw-interpreting
`tikz.sty` means emulating the pgf driver layer (`\pgfsysdriver`), specials,
and box measurements. LaTeXML(-oxide) has a curated tikz path producing SVG;
under `rawstyles` the curated binding still wins (bindings outrank raw — same
as Perl), so TikZ figures ride the existing support. **Open**: tikz *libraries*
loaded via `\usetikzlibrary{…}` raw-load pgf module files of very different
quality; catalog per-library breakage as it appears.

## D2. Alignment-preamble dialects (nicematrix, tabularray, …)

Packages extending the `array` column language feed preambles through their own `\@mkpream`/`\newcolumntype` parsers;
LaTeXML's template reader (`latexml_core/alignment.rs`) bypasses raw column machinery. nicematrix: 0 errors since
2026-09-04 (24 warnings at s134, was ~79k); tabularray runs through its binding (57co/57cp; its last in-scope error, the `[outer]`
`evaluate=`/`expand=` keys, fixed by 59n). Open: honour
raw `\newcolumntype` definitions without per-package bindings.

## D3. Verbatim-adjacent scanners (fancyvrb, shortvrb, listings, minted, piton)

Manuals demonstrate their own syntax inside verbatim variants with custom
catcode regimes, inline short-verb (`\MakeShortVerb{\|}`), and "example +
rendered result" environments that read the same body twice. Catcode-faithful
mouth behavior is kernel work and in scope; packages that shell out (minted,
piton with Python) can at best degrade to plain verbatim. **Open**: define the
degradation contract (content preserved, highlighting dropped).

## D4. Unsupported graphics backends / specials

Raw code that emits driver specials (`\special`, pdfTeX primitives like
`\pdfliteral`/`\pdfximage`, LuaTeX callbacks) has no meaning in XML. The
kernel should parse and no-op them *silently* where they are pure rendering,
and record a difficult-case entry where content is carried (e.g. annotations).

## D5. Placement semantics (floats, marginpar, side-notes, wrapfig)

PDF golden shows exact placement; XML deliberately abstracts it. "Perfect"
here = content present, order sensible, placement hints preserved as
attributes — not pixel parity. Audit rule for S3: every float/marginnote body
must exist in the XML; where it lands is not a defect.

## D6. LuaLaTeX-only manuals — REVISED 2026-08-31 (user directive)

A Lua interpreter (`texlua`) may be assumed wherever TeX Live is installed,
so LuaTeX ESCAPES are now in scope: `latexml_engine::lua_bridge` runs a
persistent per-conversion texlua; `\lx@directlua` evaluates chunks with
LuaTeX-manual semantics (job-persistent state, `tex.print`/`tex.sprint`
re-entering the input with current catcodes), and the luacode.sty binding
maps `\luadirect`/`\luaexec`/`\luastring*`/`{luacode}`(`*`) onto it.
The strategy question — native emulation vs rebinding into our XML model —
is settled in [`LUA_REBINDING.md`](LUA_REBINDING.md) (2026-08-31): texlua has
no engine, so every `tex.*` touchpoint is our shim by construction; shims are
tiered translate / mirror / absorb. `tex.count`/`tex.dimen` reads AND writes
now mirror the live Rust State over the pipe (no more stub zeros); `require`
resolves texmf Lua modules via kpse + lualibs. Out of scope remains only the
node/font/callback layer (typesetter internals — binding territory when a
package's node output carries content). The engine deliberately does NOT
define `\directlua` itself: that name is the LuaTeX-detection probe for
babel & friends, and claiming it flips whole package ecosystems onto luatex
code paths (26 suite tests red). fontspec-style font selection remains
absorbable presentation (see fontspec cluster).

**Residue closed (2026-10-02).** The seven pdflatex-clean CJK manuals named 2026-09-09 (cjk-ko-doc, oblivoir-simpledoc,
kotex-doc, kotex-utf-doc, sample-bxcjkjatype-beamer, bxcoloremoji-shortnames, gentombow) convert with 0 errors at s134
(K10 pdfTeX byte mouth, 56bl; K6 `\pdfoutput` persona, 56id; `\nopagecolor` and beamer `\trans…`, 56bj). Japanese,
Chinese and Korean manuals are out of scope since 2026-10-01 (PERFECT_KERNEL.md → Scope).

## D9. pTeX/upTeX (Japanese) class ecosystem — out of scope (CJK, user 2026-10-01)

jsclasses/jlreq raw loads reach the pLaTeX kernel surface (`\hour`/`\minute`, `\kanjiskip`, `\epTeXinputencoding`,
kanji classes; witnesses bxbase-ja, asternote and the jlreq family); K12 (56ds) gives pTeX control words their letters
under a pLaTeX class. Crash canaries; settled: don't chase single pTeX primitives.

## D7. Documents needing shell-escape or external tools at author time

Manuals that `\input` files generated by their own build (e.g. piton's
`.pyluatex` caches, minted `frozencache`) fail on missing files. That is not a
kernel defect; catalog per bundle, mark the missing-file error expected.

Settled: development-tree `\input ../tex/tikzlibrary<name>.code` (tikz-ladder/-relay/-sfc/-karnaugh-doc) fails in pdflatex
too (oracle exit 1); a basename fallback would be a beyond-oracle divergence, not taken. tcolorbox `compilable listing` /
`pdf comment` needs `-shell-escape` (tcblistingscore.code.tex:167-181): beamertheme-rainbow/-spectrum/-tcolorbox docs and
didec are in `shell_escape_excluded.tsv`.

## D10. forest.sty — native tree model (standing side goal, user 2026-09-05; SYNC R8b)

`latexml_contrib/src/forest_sty.rs` parses the bracket grammar (forest.sty:1413-1655, `{forest}` :8506-8515, `\Forest`
:8666-8680) into nested `<ltx:inline-enumerate class="ltx_forest_children">` / `<ltx:inline-item class="ltx_forest_node">`;
labels digest as TeX (56bu), the node keys `edge label`/`tier`/`phantom`/`name` and `\forestset` styles are structure, every
form is an `ltx:inline-block`, key processing is bounded (56iz; DIVERGENCES #302) — the "native tree model" shape; Perl
raw-loads forest and dies on `\forestversion`. Open: drawing/layout fidelity (forest's packing, forest-lib-edges/-linguistics),
forest-doc's `\DocInput` gaps (`\@escapeifif`, `\forest@file@copy`; lualatex-unclean, out of scope). Witnesses
forest-quickstart (99.7 % at s134), fragoli_doc (91.7 %). Guards `perfect_kernel_batch54::forest_bare_cs_form_discards_body`,
`perfect_kernel_batch56::{forest_stub_is_a_warning, forest_three_level_semantic_tree, forest_docinput_lstenv_writefile_gobbles_doc_percent}`.

## D11. Quick-failure registry — fast fatals that still owe a root cause (user directive 2026-09-05)

"Failing quickly is better UX and DX than failing with a huge performance
regression" — so a conversion that cannot succeed says so in seconds instead of
grinding to a cap. Every such bail is recorded here with its witnesses and the
work that would turn it into a conversion; none is a final answer.

Every bail registered here (2026-09-05 → 09-17) is landed or out of scope:
- Landed: wheelchart's `Fatal:Timeout:Convert` (59b pgfmath ternary precedence, KPE #420; 59c 118 s; 59e
  `\f@nch@setoffs`, KPE #422); the `oom:alloc_failed` 3,288,334,336-byte cluster (56ak, `\batchmode\read -1`, KPE #213);
  kaytannollista's math-prime runaway (56bu: unicode-math re-binds `'` at begin-document).
- Out of scope (crash canaries): the pTeX bails — `ajmacros_sty.rs` Fatal and jarticle.cls:94-97's byte pair
  (platexsheet-jsclasses, sample-jsclasses, wtref-ja, jpneduenumerate, platexcheat; D9) and jlreq's `\epTeXinputencoding`
  → PushbackLimit (`~/data/pk_agents/w22/eptex-pushback/repro.tex`); csvsimple-l3 and mercatormap (shell-escape,
  `TooManyErrors`); tikz-among-us (the `animate` binding emits one frame, guard
  `perfect_kernel_gemini::animate_multiframe_single_frame`); yquant-doc (the minted stub never closes the doc's
  `\begingroup`, `~/data/pk_agents/w22/yquant/repros/minipage_minted_group.tex`).
- Settled, no code change: bibleref-parse (`\brp@ifcs` on `\"` loops in real pdflatex too; Perl stops only via the KPE #123
  backtick quirk we do not copy — guards `fpeval_register_right_operand_of_comparison`,
  `backquote_charcode_of_other_backslash`; a generalized no-progress guard is MED-HIGH risk, not pursued); chemexec ×2 /
  quickreaction (their `\usepackage`s left TL2025 — oracle-incomplete; chemexec.sty:922 `\edef\empty`; repro
  `~/data/pk_agents/w22/chemfig-nodes/repros/min_node.tex`).
The sweep-57/58 fatal tallies (2026-09-06/07) stay in this file's git history.

## D12. Deferred shared roots (wave 18, 2026-09-06) — closed

Landed: msc (56ac, `\globaldefs` vs save frames; KPE #209, DIVERGENCES #215), modernposter (56ad, document hooks inline;
KPE #211, DIVERGENCES #217), abntexto `\halign` cell-head peek (56ab, DIVERGENCES #211), zx-calculus arc bisection (56ac
closed form; KPE #210, DIVERGENCES #216; guard `line_and_arc_intersection_is_closed_form`) and the tikz-cd-in-amsmath
align count (56ae; guard `tikzcd_matrix_inside_an_amsmath_cell_closes_its_math`), ribbonproofs pgfmath `min`/`max` fold
(56ac; guard `pgfmath_min_max_fold_over_every_argument`), mhequ `\eqno` (56ab, DIVERGENCES #213), psmatrix/dsptricks
`\pst@object` (56af; guard `pst_object_dispatches_to_the_object_body`), upmethodology `\ifpdf` persona (K6, 56id).
Out of scope (oracle-unclean): ualberta (0 errors at s134 — re-run `~/data/pk_agents/w22/color-group/repro_b.tex` before
treating the pgfplots `scatter` `\aftergroup` frame leak as gone; pgfplots scatter is arXiv-relevant), tikzviolinplots
(stringstrings' `\protect\OE` byte, SHARED; `~/data/pk_agents/w22/pgf-pair/probe_ba.tex`), tikzpingus-doc (shading
regeneration, not isolated; `~/data/pk_agents/w22/pgf-pair/NOTES.md`). elzcards: D13.
Settled dead end: inserting the missing `}` at `\endgroup` (tex.web §1064 off_save) reaches the nested consumer, not the
owning box reader — doubles the count.

## D13. Manuals broken by their own class on TL2025 (SHARED with pdflatex)

Every witness below except xebaposter and elzcards is lualatex/pdflatex-unclean on TL2025, i.e. a crash canary since the
2026-09-29 scope rule; the kernel facts stay recorded.

- **cnltx-doc `{multicols}` undefined** (35 manuals; schule the same root): cnltx-doc.cls:728 `\AfterPackage!{hyperref}{…}`
  runs grouped, latex.ltx:18699-18703 refuses a grouped load, multicol never loads (pdflatex: same two errors). General
  rule: a grouped package load sets a global loaded-flag (latex.ltx:18482; Package.pm:2326) that blocks the top-level
  reload. No faithful fix; RED by design `loader/multicols_afterpackage_group`. Settled: the multicol binding does load;
  `!`-label parsing and `\@ifpackageloaded` are not the discriminator.
- **`#` (catcode PARAM) reaching the stomach** (31 manuals): the emitter (stomach.rs:2119-2133) is Perl Stomach.pm:192-201 /
  tex.web §1049; always an upstream failure pdflatex hits too (TL2025 `\pdftex_if_engine:TF` removal, absent classes).
  SHARED, no action. Repros `~/data/pk_agents/w22/param-hash/`.
- **Text-mode `_` from an "Anonymous String"**: shape (1) chemfig `\everyeof{\_nil}…\scantokens` (chemfig.tex:1051-1053):
  our `\scantokens` (etex.rs:465) inserts no `\everyeof` — Perl shares it (eTeX.pool.ltxml:251-258); PARKED, two attempts
  regressed l3doc (prerequisite: stop expandable `\verb` scanning in edef-style bodies; repro
  `~/data/pk_agents/w22/text-underscore/repros/shape1_chemfig_everyeof.tex`). Shape (2) `\fcolorbox` landed 56at; (3)-(5)
  SHARED/persona/deferred. Settled: `\DeclareRobustCommand\0` works.
- **frankenstein self-documenting manuals** (`\DocInput{X.sty}`): doc.sty:622 `\code` refuses compsci.sty:510's
  `\newcommand*\code`, so the prose's `\FOO`s execute, as in pdflatex; Perl drops the body (Package.pm:2289-2291). SHARED,
  content kept, no faithful fix. Repros `~/data/pk_agents/w22/frankenstein/repros/`.
- **manyind/mindsample** — landed 56bp (DIVERGENCES #222); 0 errors, 100 %.
- **chinesechess** — `\draw_linewidth:n` is l3draw version skew (SHARED); the dimension-overflow clamp landed
  (`numeric_ops.rs` `MAX_DIMEN`, tex.web §101/§460).
- **xebaposter/poster** — resolved: 0 errors, schema-valid at s134. Dead ends kept: renaming the capture to `ltx:sidebar`
  (structural, not Flow); widening `sectional-block` (leading empty `ltx:p` invalid).
- **elzcards/elzcards-examples** — OUT OF SCOPE (user 2026-10-02; PERFECT_KERNEL.md → Scope; geometry #99). The
  `landscape` order dependence is fixed (geometry_sty.rs:44-52); open side findings (`\stop` = `\endinput`, kernel paper
  swap) are in HANDOFF; unverified since 56bp: `\usepackage[vmargin={0mm,0mm}]{geometry}` losing brace-protected commas.
- **etoolbox environment hooks** — settled dead end: one `@environment@X@atbegin` store for every name DOUBLE-FIRES (the
  magic `\begin{X}` constructor, dialect.rs:1244-1247, digests the store AND fires `\UseHook{env/X/begin}`); the
  `verbatim` predicate feeds the one store-only constructor (mod.rs:788-799). cora-macs-doc's 28 errors: Perl's XML is
  byte-identical.

## D14. beamer overlays are one pass here, many passes in beamer

**Witness:** chessboard/chessboard_and_beamer (pdflatex-clean, in scope; 3 errors + `Fatal:Mouth:EoF`). Real beamer
typesets a frame once per overlay slide (beamerbaseframe.sty), so each `\only<n>` sees a fresh skak game; our binding and
Perl (beamer.cls.ltxml:793-834) take every overlay in one pass. **User decision 2026-09-10: keep Perl's single pass**;
documented residue (RED `expl3/s41msg_chessboard_beamer_only_gamestate`).

## D15. Content-preservation audit findings (stage 2, from 2026-09-17)

The S3 recall audit (`tools/perfect_kernel/s3_sweep.sh` over the S0∧S1 slice) and the
markup census expose losses that the error-free stage could not see. Each row names
the mechanism, its witnesses, and the disposition.

- **Measurement artifacts, not loss — do not re-chase** (sweep #104 recall triage,
  `archive/CONTENT_RECALL_TRIAGE_2026-09-19.md`): a golden that is not the `.tex`'s own PDF (curated in
  `tools/perfect_kernel/golden_reference.tsv`: geradwp, dinbrief, JACoW A4/Letter; modular is out of scope); a non-Latin
  golden that `pdftotext` garbles while the XML has the text (greek-fontenc/test-tuenc-greek, litetable zh-cn/zh-hk);
  content that is graphics (bookcover, tkz-grapheur, chessboard-skakps, writeongrid, pgf-spectra, tikz-kalender); listing
  identifiers counted as missing (timeop, showexpl, pygmentex); embedded external PDFs (newpax/doc-use-pax,
  doc-use-newpax). arabi/samplebook: the loss was real — no LAE/LFE fontmap, every
  Arabic letter decoded to nothing (0 code points; Perl the same) — LANDED 59r (DIVERGENCES #415: 17,175 Arabic code
  points; letter coverage of the golden 99.96 % once its text layer is decoded), the rest a reference artifact (the
  golden's text layer mixes presentation forms and LAE slot codes, in visual order, split at glyph joins); montex/mlsquick/zanabazr accepted as
  reference-side (the golden's Type 3 bitmap fonts have no Unicode text layer; `accepted_residuals.tsv`, 2026-10-02).

- **`\renewenvironment{document}` around a `\loop … \input` of full documents** (base/ltnews, base/l3news): resolved in
  phase 56 — a second `\begin{document}` fires no hooks (KPE #143); 99.9 % recall at s134.
- **A class `\maketitle` built from private fields** (exam-n.cls:609-764): resolved by 56gj's maketitle capture
  (template-master 36.5 → 100 %). Classes laying out fields inside `\maketitle` itself (uantwerpendocs shipout-picture
  title pages): LANDED 59p (one-shot overlay kept whole, user ruling 2026-10-02, DIVERGENCES #413; gate rules #265).
- **JACoW_LaTeX_A4/_Letter**: the shipped PDF includes the commented-out annex (`JACoW_LaTeX_A4.tex:499-501`); curated in
  `golden_reference.tsv` (100 % at s134). Perl loses the `Itemize` list (3 errors).
- **German letter classes.** dinbrief: golden is the full `.dtx` build (curated in `golden_reference.tsv`; 99.8 % at s134).
  g-brief beispiel2 / beispiel: sender, recipient and bank blocks are typeset only in `\thispagestyle{firstpage}`'s
  `\@oddhead`/`\@oddfoot` (g-brief2.cls:252-253, :310-427) — LANDED 59o (user ruling 2026-10-01: creator role=sender /
  addressee, OXIDIZED_DESIGN_DIVERGENCES #412): beispiel2 60.7 → 97.6 %, beispiel 71.4 → 91.4 %, ApplicationLetter
  85.7 → 96.7 %.
- **The 45-60 % "uncategorized" recall family (2026-09-18) is classified, closed:** pecha/showexpl PDF-font and verbatim
  artifacts; figbib a bibtex multipass (`.fig` aux, SHARED; list via `\fbList`, 56dc); quotchap/fbithesis Perl error-dumps
  inflate Perl's recall; simplecd fixed 56ct; sim-os-menus' 120 missing words are text inside
  `\includegraphics[page=…]{ProfLycee-doc.pdf}` (unrecoverable; lualatex-unclean anyway).
- **S2: dangling `\hyperlink` targets stay (SHARED; RULED 2026-09-18, as pdflatex).** `\hyperlink{name}{text}` emits
  `<ref idref="name">` unconditionally (hyperref.sty.ltxml:231-234 = hyperref_sty.rs:712) while the `\hypertarget` never
  reaches the core XML (biblatex `begentry` inside a deferred `\printbibliography`, endnote anchors, `{comment}`); Perl's
  core XML is byte-identical. At s134: biblatex-gost-examples 770, cms-dates-intro 80, cms-trad-appendix 4, philexmanual 6
  of 7, europecv 3, elsdoc 2 jing lines. A surpass option for biblatex `cite.<n>@<key>` anchors needs a ruling
  (`~/data/pk_agents/main/agent_reports/2026-10-02_biblatex_idref_rootcause.md`). Root-causer `regr87/s2_idref/NOTES.md`.
- **S2: `class` values are NMTOKENs and `xml:id`s valid (RULED 2026-09-18: surpass).** `Document::set_attribute` reduces
  class tokens to NameChar (listings' `ltx_lst_language_{TeX}`, `C++`, `\lx@add@class` `@` names, fontawesome `fa-*`);
  `xml:id` goes through `clean_id` before dedup. manptp's two roots fixed: counter `\theequation@ID` formatter under a
  raw class (56dh), `\inst` as an affiliation store (56di) and K11 store reroute (56dj).
- **Empty-body XML under 3,000 bytes (157 docs, 2026-09-18): no RUST-ONLY drop.** 76 complete and short, 36 with a PDF of
  < 30 words, 7 bibliography-only, 13 rendered through external resources (D4/D7: frontespizio `\jobname-frn.eps`, arabi
  `\special{ps:}`, textpos/eso-pic, newpax overlays, pygmentex, nomencl `.nls`). The uni-titlepage (13) and
  `\enddocument`-redefinition (ltnews, l3news, tools-overview) members are resolved (s134 recall 99.9-100 %).
  Classifier `~/data/pk_agents/w22/empty_xml/NOTES.md`.
- **Bibliography-only bodies** (`\nocite{*}` + `\bibliography`; shipunov/rusnat-ex1-ru, bookshelf/spines, biblatex
  samples): the core XML carries the `<bibliography files=…>` placeholder; judge them on the HTML root (`post_sweep.sh`).
- **`\DocInput{<file>.sty}` of a package with a compiled binding typesets nothing (SHARED, parked):** the body-level input
  routes to `input_definitions(notex)` (content.rs:1706) and the binding short-circuits the raw typeset (:1723), as Perl's
  `loadTeXDefinitions` (Package.pm:2298); forcing raw content for bound files was litigated at content.rs:1725-1740 (HIGH
  risk). frankenstein is the only bundle that `\DocInput`s a `.sty` (lualatex-unclean: out of scope).
- **Not losses (`~/data/pk_agents/w22/recall_mid3/NOTES.md`):** geradwp (golden is the class documentation; curated in
  `golden_reference.tsv`), pdfreview (overlaid source PDF pages), elsdoc (`\includeclip` of sample-manuscript PDFs; only
  rvdtx's `\setbox\topbox` title block is a real ~1 % loss — 325 missing words at s134, mostly the clipped PDFs).
  milsymb is out of scope (PythonTeX, user 2026-10-02); skeldoc's enotez bodies landed in 58l (KPE #416).
- **Not losses (s125 tail, `~/data/pk_agents/w70/scratch-streamA126/`):** modular (TL flattened the bundle; lualatex-unclean,
  out of scope), beamertheme-mirage-doc (14 embedded PDF pages; xeCJK spacing and hologo's `\HoLogo@La` split words for the
  audit only), matapli, webquiz (`\includepdf`, Perl parity), bibarts (register = external `bibsort` `.prr`, TL ships no
  binary — SHARED), abntex2cite(-alf) `.bst`-only references (DEFERRED_FAMILIES; its live residuals: SYNC_STATUS rc131 row).
- **Structure-loss signals (131 clean docs) are not a general drop**: ~90 % displayed source or macro bodies; the rest
  element-name mismatches the check must accept (a `\title` footnote is `<pubnote>`, an `\author` one `<contact
  role="note">`, byte-identical to Perl; minipage footnotes `<note>`; grid's `gridenv` the shared `\box0`+`\vadjust`
  path). Notes `~/data/pk_agents/w22/structure_loss/NOTES.md`.
