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

Packages that extend the `array` column language (`w{c}{1cm}`, custom column
letters, bracketed first-row/last-col options) feed preambles through their own
parsers built on `\@mkpream`/`\newcolumntype`. LaTeXML replaces the whole
alignment pipeline with its own template reader (`latexml_core/alignment.rs`),
so raw-defined column machinery is bypassed and unknown template letters spray
`Unrecognized tabular template` warnings (nicematrix baseline: ~79k warnings on
one manual). **Plan**: make the template reader honor raw `\newcolumntype`
definitions and unknown-letter recovery without per-package bindings; measure
on nicematrix + tabularray manuals.

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

**The last non-policy oracle-clean residue is this family (scoped 2026-09-09; seven pdflatex-clean manuals: cjk-ko-doc, oblivoir-simpledoc, kotex-doc, kotex-utf-doc, sample-bxcjkjatype-beamer, bxcoloremoji-shortnames, gentombow).** All SHARED (same-host Perl errors on every one, five of them at its 100-error cap). Two mechanisms: (A) the pdfTeX BYTE mouth — cjkutf8-josa.sty:176-194 `\DeclareRobustCommand*\^^ea[2]` defines a control SYMBOL over the first UTF-8 byte of a hangul syllable and reads the next two bytes as arguments, and dhucs-trivcj.sty:18 `\ifx 가가` probes the engine by comparing two BYTES; our Unicode mouth tokenizes a syllable as one codepoint, so `\를`/`\은`/`\japanese` are undefined (kotex-utf-doc 27, the others 1-4). The only faithful mechanism is a pdfTeX byte-mouth mode in `latexml_core::mouth` (when the persona is pdfTeX and a package makes the high bytes catcode 12 — kotexutf.sty:40-41, CJK's `.bdg` — tokenize multibyte UTF-8 as catcode-12 bytes): a kernel program, MED-HIGH risk, no Perl to port from (Perl has no kotex/CJK binding), no hangul-keyed special case allowed. (B) the `\pdfoutput=0` persona (`pdftex.rs:11`, K6): gentombow.sty:582's driver branch and bxcoloremoji.sty:461's `\ifnum\pdfoutput>0` graphicx gate — the corpus-wide backend decision, not a local fix. Micro-gaps found on the way: `\nopagecolor` (color.sty:110, a no-op driver info) and beamer's `\trans…` transitions (beamerbaseoverlay.sty:755-772) — landed as batch 56bj; neither zeroes a doc alone. Notes `~/data/pk_agents/w22/cjk-residue/NOTES.md`.

## D9. pTeX/upTeX (Japanese) class ecosystem

`jsarticle.cls`/jsclasses raw-load under rawclasses and immediately hit the
pLaTeX kernel surface: `\hour`/`\minute` (plcore time registers, jsarticle.cls
L106 — sweep-11 first-error in 33 docs, witness bxbase/bxbase-ja), then
`\kanjiskip`, `\prebreakpenalty`, kanji character classes. This is the
same out-of-scope engine family as the CJK/luatexja cluster (pTeX primitives
outside the pdfTeX model): defining `\hour` alone just moves the failure one
primitive deeper. Catalog per bundle; a pLaTeX profile would be its own
mission-level decision.

Sweep-13 confirmation (2026-08-31): `\hour` remains rank-4 by bundles (16
bundles / 32 docs, bxbase/bxjaprnind…); `\epTeXinputencoding` (6 bundles,
asternote = jlreq class, 94 errors deep) and `\newXeTeXintercharclass`
(datetime2-* xe/lua test files, D6-adjacent) join the same catalog. Policy
unchanged: don't chase single primitives.

## D7. Documents needing shell-escape or external tools at author time

Manuals that `\input` files generated by their own build (e.g. piton's
`.pyluatex` caches, minted `frozencache`) fail on missing files. That is not a
kernel defect; catalog per bundle, mark the missing-file error expected.

**Development-tree `\input` paths (sweep 62, 2026-09-07).** tikz-ladder-doc,
tikz-relay-doc, tikz-sfc-doc and tikz-karnaugh-doc (293/292/172/130 errors, all
`pgfkeys` "I do not know the key '/tikz/…'") load their library with a bare
`\input ../tex/tikzlibrary<name>.code` (tikz-relay-doc.tex:148, the
`\usetikzlibrary` line above it commented out). `../tex/` exists only in the
authors' source tree, so pdflatex/lualatex abort on the same missing file (oracle
exit 1) and every later key is unknown. Not a kernel defect; a basename fallback
for a missing relative `\input` path would be a beyond-oracle divergence
(surpass-perl escalation, not taken). mercatormap (459) is shell-escape
(`shell_escape_excluded.tsv`).

**tcolorbox `compilable listing` / `pdf comment` (sweep 66: beamertheme-rainbow/-spectrum/-tcolorbox docs, didec; root-caused 2026-09-09).** Every `Package tcolorbox Error` is one cascade from tcblistingscore.code.tex:167-181 `\__tcbox_run_system_command:n` refusing without shell escape ("You must invoke LaTeX with the -shell-escape flag"), then tcbskins.code.tex:1684-1706's missing sub-PDF and the undefined `\pdfpages`. pdflatex without `-shell-escape` fails identically; Perl explodes earlier on raw minted v3. Three bundles were already in `shell_escape_excluded.tsv`; didec added.

## D8. expl3-heavy packages

Raw interpretation of l3-programming-layer packages exercises expl3 the
hardest (regex VM, intarray, fp). Known open expl3 gaps are tracked in the
main memory/SYNC docs; entries here should reference the specific manual +
first error rather than duplicating that tracking.

## D10. forest.sty — full support is a standing side goal (user directive 2026-09-05)

**Status:** `latexml_contrib/src/forest_sty.rs` is a discard stub (batch 56k made
its diagnostic a Warn: the `{forest}` body is dropped, nothing is drawn). Not a
perfect-kernel target row, but the user asked that complete support be recorded
and not forgotten: forest is heavily used on arXiv (linguistics trees, decision
and proof trees), and three TL manuals (forest-quickstart, fragoli, milsymb) plus
forest-doc/forest-libs exercise it end to end.

**What "complete" means.** forest.sty (9,259 lines, expl3 + pgfkeys) parses a
bracket notation into a tree, lays it out (its own packing algorithm,
`forest-lib-edges.sty` edge styles, `forest-lib-linguistics.sty` presets) and
draws it with TikZ. Faithful support has two candidate shapes, to be decided when
it is taken up:
1. **Overlay binding (K1 shape):** raw-load forest.sty and let its bracket
   parser, keys and layout run on our TikZ layer (D1) — the layout uses
   `\pgfmath` and pgf coordinates throughout, so this is gated on D1's fidelity
   and on the expl3/pgfkeys machinery already in place. Perl raw-loads forest
   and dies on `\forestversion` (misdefined), so this is beyond-Perl.
2. **Native tree model:** parse the bracket notation natively (it is a small
   grammar: `[label, keys [child] …]`) into an `<ltx:picture>`-free structural
   tree (nested lists or a dedicated tree element with a CSS renderer), which
   is what an accessible web rendering of a syntax tree wants anyway. Drawing
   fidelity is lower; structure fidelity and accessibility are higher.

**Evidence to keep:** the discard stub's witnesses (forest-quickstart,
fragoli_doc, milsymb; sweep #41), the forest-doc `\DocInput` example environment
(fixed in 56i for the listings side, the trees themselves still discarded),
arXiv usage counts to be measured with the corpus scanner before choosing the
shape. Guards to keep green while touching it: `perfect_kernel_batch54::forest_bare_cs_form_discards_body`,
`perfect_kernel_batch56::{forest_stub_is_a_warning, forest_docinput_lstenv_writefile_gobbles_doc_percent}`.

Audit note (K1 step 3 pass two, 2026-09-07): `latexml_contrib/src/forest_sty.rs`
discards the whole `{forest}` body into an `<ltx:ERROR>` (mirrors ar5iv's
`discard_env_body`; Rust emits no `Error:` line, so the 3 oracle-clean forest
manuals read "clean" while every tree is lost — the highest content-risk stub
in the corpus). A raw load needs forest.sty:49-61's tikz (+`shapes`,`fit`,`calc`),
`pgfopts`, `elocalloc`, `environ`, `xparse`, `inlinedef` (catcode-`#` tricks) and
forest's own bracket parser; the bracket parser and `inlinedef` are the hard
blockers, the rest exists. A real port = the bracket grammar as a Rust reader
producing the tree as nested `ltx:para`/lists plus tikz for the drawing.


## D11. Quick-failure registry — fast fatals that still owe a root cause (user directive 2026-09-05)

"Failing quickly is better UX and DX than failing with a huge performance
regression" — so a conversion that cannot succeed says so in seconds instead of
grinding to a cap. Every such bail is recorded here with its witnesses and the
work that would turn it into a conversion; none is a final answer.

| Bail (site) | Witnesses | Time before → after | What would remove the bail |
|---|---|---|---|
| `ajmacros.sty` Fatal (`ajmacros_sty.rs`, batch 56l) — japanese-otf's ISO-2022-JP-named recursive kanji scanners loop aperiodically without pTeX's kanji token model | platexsheet-jsclasses, sample-jsclasses, wtref-ja, jpneduenumerate | 250–264 s → < 1 s | §D9: a kanji token model (`\kcatcode`, JIS/UTF-8 kanji as single tokens) in the mouth; then ajmacros runs raw |
| Unbalanced-expansion Fatal (`Expandable::new`, batch 56l; Perl parity) — jarticle.cls:94-97 `\ds@tate`'s ISO-2022-JP byte pair whose `%` eats a brace | platexcheat sample, platexsheet | 253–259 s → ~2 s | the same kanji source model (the `%` is the second byte of a kanji) |
| Runaway cap `TooManyErrors` (500 identical errors) — csvsimple's Java CSV-Sorter, `\file_input:n` of shell-escape products | csvsimple-l3, mercatormap | minutes → seconds after the cap | K8: degrade the offending construct instead of discarding the document; shell-escape emulation is a user decision |
| Sweep-57 fatal-class tally (2026-09-06, 111 status-3 docs, 2 oracle-clean = bibarts, chessboard): `TooManyErrors:MaxLimit(100)` 35 · **`oom:alloc_failed` of exactly 3,288,334,336 bytes 23 docs** (LANDED batch 56ak: the `\batchmode\read -1` halt idiom was a no-op, so XeTeX-only packages looped on `\XeTeXcharclass` until the log buffer's next doubling failed; now one Fatal in seconds, KPE #213) · `Timeout:PushbackLimit` 10 · `Mouth:EoF` 9 · `Timeout:MemoryBudget` 8 (source2e, datatool-user, glossaries-extra…) · `Timeout:Recursion` 3 · the D9 pTeX bails 5 · rest singles | as listed | — | the alloc cluster is a kernel defect (an identical size across unrelated docs); MemoryBudget docs are the largest manuals (a real memory/perf lever, PERFORMANCE.md); the rest are the D6/D9/D7 families |
| Sweep-58 fatal residue after 56ak, grouped by first error (2026-09-07; excludes the D9 pTeX bails and the memory-fuse docs): `\epTeXinputencoding` undefined → `PushbackLimit` **6 docs** (gckanbun, asternote, hideanswer, inlinelabel, jpnedumathsymbols, jpneduenumerate — all `jlreq`; lualatex oracle exit 1 for every one, so they run under the pdfTeX identity and jlreq.cls:492 correctly takes its (u)pLaTeX branch = D9 pTeX primitives. PARKED 2026-09-07; the runaway afterwards is an unreduced l3keys `\keys_set` re-unread loop (`\__kernel_tl_set:Nx` of the choice/finally save-restore) confined to jlreq's platex path, repro `~/data/pk_agents/w22/eptex-pushback/repro.tex`; no in-scope lever short of giving lualatex-authored-but-failing docs the luatex identity, which still needs luatexja) · class-requires-XeLaTeX/LuaLaTeX → `Recursion` 2 (thuthesis, nxuthesis: `\directlua` detection, D6 policy) · tikzpingus (`\lxSVG@sh@defs` — our own name — → PushbackLimit) · tikzviolinplots (`Extra \else` → Recursion) · neoschool-fr (`[cmyk]{…}` taken as a colour NAME → EoF) · ualberta (non-boxing `\endgroup` → PushbackLimit) · xytree (`\ex` undefined → Recursion) · singles typog, tikz-optics (`../` library path), pldocverb (`\hour`, D9), pas-crosswords (`\lstset` undefined), dmlb-template (`\mya`), stex-doc (archive), fixdif-zh-cn (xelatex-only); out of scope: resolsysteme ×2, robust-externalize, tikzfxgraph (shell-escape). Oracle-clean among them: bibarts, chessboard (recorded verdicts). | as listed | — | root-causers on the epTeX chain, the pgf pair, and neoschool/ualberta in flight (2026-09-07) |
| `Timeout:MemoryBudget` in 13 s — tikz-among-us/tikz-among-us: NOT a loop or a wrong unit; `\begin{animateinline}…\multiframe{180}{rt=0+1}{<tikzpicture>}` (tex:756-765, animate.sty:2369-2394 bounded `\whiledo`) materializes 180 SVG frames in the document tree (~39 MB each; pdflatex ships each frame as a Form XObject and frees it). SHARED: Perl has no animate binding either. Repro `~/data/pk_agents/w22/among-us/repro.tex` (1 character, fuse at ~150 frames under `--max-memory=1536`). The `_ in math mode` errors are the doc's missing local FHZ-* packages (`\href` undefined). The other fuse docs (source2e/source3, glossaries ×3, datatool-user, tcolorbox, pgf-spectra) are genuine big manuals (≤ 4 MB logs, no repeated line) = the memory lever for a quiet machine (PERFORMANCE.md), not loops. A root-causer's side claim that ifthen `\whiledo`/`\equal` break under the `luatex` preload did NOT reproduce (ifthen, xifthen, animate+xifthen all clean under both preloads, 2026-09-07). | tikz-among-us (oracle lualatex exit 1) | beyond-Perl binding: an `animate` binding emitting ONE representative frame (guard `animate_multiframe_single_frame`: 0 errors, `count(//svg:svg)=1`) | registered 2026-09-07; value = fleet stability, not a clean doc |
| `\end{minipage} Attempt to end mode internal_vertical` ×84 — yquant/yquant-doc (shell-escape-EXCLUDED): the doc's `option` environment opens a `\begingroup` that only minted's real inline processor (`\RobustMintInlineProcess@ii`, patched by the doc via `\patch@mintinline`) closes; the `minted` stub (`latexml_contrib/src/minted_sty.rs`, `\tex`→`\lstinline`) never runs it, so every `\end{minipage}` meets the open group. Perl (raw minted2) has no minipage error. Fix would be a patchable stub processor — version-specific; deferred (excluded doc). Repro `~/data/pk_agents/w22/yquant/repros/minipage_minted_group.tex`. | yquant-doc | minted stub processor hook | registered 2026-09-09 |
| `Fatal:Timeout:Recursion` "Infinite expansion loop: a window of 2 token(s) repeated 100+ times" — yquant/yquant-doc (shell-escape-EXCLUDED), sweep 67, surfaced once batch 56bd defined `\gundef` and yquant's register-group cleanup ran further. The doc's `minipage`/minted imbalance (row above) still precedes it. Root not isolated; yquant's own language parser (`yquant-lang.sty`) is arXiv-relevant, so worth a bounded root-cause when a yquant arXiv witness appears. | yquant-doc | — | registered 2026-09-09 |

## D12. Deferred shared roots with a high-risk fix (wave 18, 2026-09-06)

Each is pdflatex/lualatex-clean, fails identically (or worse) in same-host Perl,
and has a root-cause report + red repro on disk; the fix is a mechanism change
whose blast radius outweighs the docs it frees today. Re-open when the mechanism
comes up for another reason.

| Root | Witnesses | Repro | What the fix is |
|---|---|---|---|
| pgf SVG driver group accounting: `pgfsys-latexml.def.ltxml:561-586` maps `\pgfsys@beginscope`/`@endscope` to real `\begingroup`/`\endgroup` (the pdf driver opens none); through `\pgfnode`/`\pgfmultipartnode` + `\pgfsys@begin@idscope` (pgfsys.code.tex:572-611) inside `\foreach` + `pgfscope` the count drifts and a `{` boxing frame stays open, so every later `\endgroup` reports "close non-boxing group" (the LaTeXML authors note the class at pgfsys-latexml.def.ltxml:887-890). SHARED: Perl 25 errors on the repro, Rust 24. | msc/msc (29 + PushbackLimit fatal from `codeexample` re-execution); modernposter/demo (16: the poster body is one document-spanning overlay `tikzpicture`, modernposter.cls:102-112, and `\maketitle`'s filled nodes replay their `\hbox` box with a scope `\begingroup` on top — `base_utilities.rs:3610` `predigest_box_contents_in_mode` → `end_mode` fails; Perl identical first error, 31 errors) | `tools/perfect_kernel/repros/graphics-tikz/msc_declinst_tikz_scope_desync.tex` (one `\declinst` inside `{msc}`), `graphics-tikz/modernposter_maketitle_hbox_mode.tex` | PARTIAL → msc LANDED batch 56ac: the msc "crossing" was `\globaldefs=1` (msc.sty:2616 `\msc@global@set`) globalizing our save-frame bookkeeping, so a closed `{` group stayed "current" (KPE #209, DIVERGENCES #215; msc 21→0, 10 `<svg:g>`). modernposter LANDED batch 56ad: its root was the document HOOKS digested in isolated mouths — the `\AfterEndPreamble` opener and the `\AtEndDocument{\end{tikzpicture}}` closer never met the galley's box reader (latex.ltx:15255-15259, etoolbox.sty:1774-1776 run them inline; KPE #211, DIVERGENCES #217; 14→0). Still open: psmatrix/dsptricks (the `\halign` cell template family, K9 Stage 3). Settled dead end: inserting the missing `}` at `\endgroup` (§1064 off_save) reaches the nested consumer, not the owning box reader — doubles the count. Sweep 54: modernposter/demo 15→1; the residual `No shape named `sep'` (demo.tex:53 `\node[...inner sep=...]`? a pgfkeys parse in our tikz binding) is a separate small root. |
| `\halign` inside display math whose `$#$` cells open `\hbox` (an active `<…>` that does `\ifhmode\else\expandafter\hbox\fi\bgroup…$\langle$…$\rangle$…\egroup`, abntexto.tex:79-81) with inline math inside the box: the mode-switch frames (`stomach.rs:753` "close a group that switched to mode", `:1112` "Attempt to end mode") never reconcile the nesting; the `\halign`, the display-math group and the enclosing list stay open and every later section lands inside the trapped `<ltx:inline-block>` (the 15 `<ltx:section> in <ltx:section>` errors are this collateral, not sectioning). Reached only through the manual's self-documentation trick (`\catcode\`\%=9 \input{abntexto.cls}`, :1037) that turns the class's commented `$$\offinterlineskip\halign{$#$\cr…}$$` (cls:727-736) live. SHARED: Perl 4 × "Attempt to end mode math". | abntexto/abntexto (~22 of 25 errors; the other root, amsmath's missing `\@saveprimitive\over\@@over`, landed in batch 56w) | `tools/perfect_kernel/repros/boxes-groups/reinput_display_halign.{tex,snippet}` | LANDED batch 56ab: the cell-head peek runs in internal vertical mode (tex.web §15510 `align_peek` before `init_row`), so the `\ifhmode\else\hbox` head keeps its box (DIVERGENCES #211). |
| zx-calculus self-loop wires — RE-ROOTED: not the intersections library (never loaded; a bare `\pgfintersectionofpaths` converges). A tikz-cd self-arrow on a `rounded rectangle` node queries the corner border → pgf `\pgfmathpointintersectionoflineandarc` (pgfmathcalc.code.tex:366-468) bisects until `\ifdim\x pt=\q pt` (:447) — exact only in pgf's fixed-point trig; our float trig (pgfmath_code_tex.rs) stays ~0.0005° off, 2 empty boxes per iteration → the 50,000-box cycle fatal. SHARED (Perl float trig, spins to its wall clock). | zx-calculus/zx-calculus (Fatal); arXiv 2201.09268 class (callout nodes) | `tools/perfect_kernel/repros/graphics-tikz/zx_roundedrect_selfloop_arc_bisection.tex` (rectangle control in the guard) | LANDED batch 56ac: closed-form binding (KPE #210, DIVERGENCES #216; guard `line_and_arc_intersection_is_closed_form`). Sweep 53: the fatal is gone; 49 errors remain — `\lx@begin@alignment`/`\endgroup`/`\hbox`/`\vbox` mode-frame mismatches in "Anonymous String" mouths around `\zx{…}` inside `$…$` (a tikz-cd matrix built in isolated digests): the K9 family, next root-causer. RE-ROOTED again (wave 22 residual): not an isolated mouth — a tikz-cd matrix inside an amsmath cell; `\lxSVG@halign` (pgfsys_latexml_def.rs) decremented the align-group count UNCONDITIONALLY where the standard `\halign` does so only for a `{` opener, and pgf matrices open `\halign\bgroup`, so the enclosing `align` lost a level and its cell's closing hidden `$` was never recognized (RUST-ONLY: Perl 41 on the repro, worse). LANDED batch 56ae: conditional decrement; repro `repros/kernel-alignment/tikzcd_in_amsmath_cell_align_state.tex`, guard `tikzcd_matrix_inside_an_amsmath_cell_closes_its_math`; the manual's remaining 3 = `\got@maxcolwd` (amsmath gather), separate. |
| ribbonproofs ribbon lists — RE-ROOTED: the `\fi`/`\iffalse` cascade was a downstream symptom. Root = native pgfmath `min()`/`max()` were BINARY (`pgfmath_code_tex.rs` `pgfmath_apply_fn` dropped `args[2..]`, `min(x)` read as `min(x,0)`) where pgfmathfunctions.misc.code.tex:292-336 folds the whole list (`\pgfmathmin@@`, sentinel ±16383pt); ribbonproofs.sty:1213 `min(\@leftPositions)` gave a wrong `\@stepLeft`, a ribbon re-started "already active" and the `\PackageError` inside the tikz `\foreach` cascaded into `expected:\fi`. etextools' conditional-tokens-as-data scan is handled correctly (verified). RUST-ONLY (Perl dies earlier on `\globcount`, 3). | ribbonproofs/ribbonproofsmanual (3) | `tools/perfect_kernel/repros/graphics-tikz/ribbonproofs_pgfmath_min_variadic.tex` | LANDED batch 56ac: variadic fold (unit test `min_max_fold_over_every_argument`, guard `pgfmath_min_max_fold_over_every_argument`). |
| mhequ single-line `{equ}`: `\@saveMHComms` `\let\\=\@MHcr` (mhequ.sty:181) with `\@MHcr` undefined in the `equ` path, restored by `\@restoreMHComms` (:184) which sits AFTER `\eqno{…}` — our `\eqno` (tex_math.rs:1790, = Perl TeX_Math.pool.ltxml:1239) gullet-scans the tag to `$$` and collects the `\let` instead of executing it, so `\\` stays undefined (3 errors; Perl 8). | mhequ/mhequ-example (3) | `kernel-alignment/mhequ_equ_cr.tex` | LANDED batch 56ab: `\eqno`/`\leqno` digest the tag as a bounded math sub-body (tex.web §21745; DIVERGENCES #213). |
| psmatrix cell template: with the real `\psset` (batch 56x) pst-node's `\psm@endnode` runs and the `\halign` cell template's `\begingroup…\endgroup` pairs desync ("close a group that switched to mode restricted_horizontal" ×4, `\endgroup` non-boxing ×4); pstricks-add's colour keys also read `\pst@getcolor` = xcolor's `\XC@getcolor` (pstricks.sty:155), missing from `xcolor_sty.rs`. | dsptricks/dspTricksManual (100+, capped), pst-eucl-docBG | `graphics-tikz/psmatrix_psk_mnodesize_dsptricks.tex` (psk internals now defined) | LANDED batch 56af — the Stage-3 diagnosis was WRONG: the u-part never opened its node box because `pstricks_support_sty.rs` stubbed `\pst@object{}` as `#1` (typeset the object NAME; pstricks.tex:1453-1461 dispatches to `\<name>@i` — no Perl counterpart to the stub), so `\psm@beginnode`'s `\pst@object{psm@beginnode}` produced text and the v-part `\psm@endnode@i` closers popped the alignment's own cell frame. Stub removed: single-cell and dsptricks repros 0 errors (4 cells). Guard `pst_object_dispatches_to_the_object_body`. The row/column boxing frames are FINE for a properly nested template (tex.web nest levels vs our frames only matter for real straddles, none seen). |
| geometry global write-back: `elzcards.sty:396-397` reads `\textwidth`/`\textheight` into counters and fits 5in cards (:700) into the page box; our geometry binding sizes the SVG canvas only (OXIDIZED_DESIGN #99) and never writes the body box back to the global dims (`geometry_sty.rs:100-125` region; real geometry.sty assigns `\textwidth=\Gm@tw` etc.), so the article default 345pt < 5in and `\elzc@CalculaMatriz` (:281-305) raises "No space to print at least one card". SHARED: Perl no-ops geometry, 3 errors. | elzcards/elzcards-examples (1) | `layout-singles/geometry_noop_textwidth_elzcards.tex` (`~/data/pk_agents/w20/layout-singles/repros`) | overturn #99: geometry writes the computed body box to the global `\textwidth`/`\textheight`/paper dims — a flow-sizing policy change touching every geometry doc's width attributes; user decision, own branch. (Secondary, non-decisive: package-option splitting does not honour a brace-protected comma, `vmargin={0mm,0mm}` → 4 "Missing number" warnings.) |
| PDF persona for graphics extensions: `upmethodology-fmt.sty:460-463` picks `.pdftex_t/.pdf_tex` under `\ifpdf`, else `.pstex_t/.ps_tex`; both engines are DVI (`\pdfoutput=0`, `\ifpdf` false under the pdflatex profile), the doc ships only `figure_and_tex.pdftex_t`, so `\includefigurewtex` (:507-509) errmessages "File not found". `\IfFileExists`/`\filename@parse` are correct. SHARED (Perl fails differently: undefined `\@autolatex@wtfig@exttmp`). | upmethodology/upmethodology-doc (1) | `layout-singles/ifpdf_persona_graphics_ext_upmethodology.tex` | the K6 PDF-mode persona decision (KERNEL_CAPABILITIES 2026-09-05 K6 row; PLANS P16-vii) — a direct witness for "graphics extensions keyed on `\pdfoutput`". No `\ifpdf` stub. |
| pgfplots `scatter` markers leak one boxing `{` frame each: `\aftergroup\pgfplots@scatter@plot@mark` (pgfplots.markers.code.tex:178-214, an xdef'd `\begingroup…\endgroup` body deferred out of the marker box) — `LXML_TRACE_FRAMES` shows the `{` pushed in restricted_horizontal never popped; `\end{axis}`'s `\endgroup` cascade underflows ("close non-boxing group"), a second axis hits "nested axis" and error recovery re-unreads past the pushback cap. Inline repro `~/data/pk_agents/w22/color-group/repro_b.tex` (`\addplot+[scatter,mark=*] coordinates {…}`, ≥32 errors; non-scatter plots clean). | ualberta/ualberta (oracle lualatex exit 1) | RUST-ONLY engine (`\aftergroup` vs box reader) | OPEN 2026-09-07 — needs a token-level trace of the marker box; also the pushback runaway on an unclosable `\endgroup` is a robustness cap worth its own guard |
| stringstrings' encoded blank space is the robust `\protect\OE` (stringstrings.sty:82-94 `\SaveOEthel`); its non-`\edef` byte machinery (`\@rotate`/`\@treatleadingspaces` :1580+, `\@gobblearg`/`\@DiscardNextChar` :1546-1560, `\isnextbyte` :915) must see that multi-token unit as one byte; here `\OE`'s decomposition leaks `O`,`E`,`\else`,`\fi` into `\edef\@x{\if\SignalChar\@x F\else T\fi}` → "Extra `\else`" ×2 → expansion runaway. Trigger = a leading space (a newline inside `\violinsetoptions[…]{…}`'s option list, tikzviolinplots.sty:201-228 `\noblanks[e]`+`\whereisword[q]`). **SHARED**: Perl fails identically (2 Error + `Fatal:terminate`). Repro `~/data/pk_agents/w22/pgf-pair/probe_ba.tex` (the inline-options twin is clean). | tikzviolinplots/tikzviolinplots (oracle lualatex, renders) | SHARED kernel — `\protect` must freeze to `\relax` on the non-`\edef` discard path so one encoded byte = one `\@gobble` (latex.ltx robust-command semantics) | OPEN 2026-09-07, surpass-tier (1 doc); design the `\protect` rule first, MED risk |
| pgf shading regenerated at use time (`\pgfuseshading` → `\pgfshadepath`, pgfcoreshade.code.tex:769-810) fails to reinstall `\@pgfshading<xname>!`, so `\pgfsys@shadinginsidepgfpicture{\relax}` runs our `\lxSVG@sh@defs`/`@pos`/`@sh` trio undefined (pgfsys_latexml_def.rs:1616-1760, Perl pgfsys-latexml.def.ltxml:672-724) and the stream derails into a PushbackLimit; fingerprint = three "Illegal unit of measure (pt inserted) at Anonymous String" just before. Candidates: the global `\@pgfshading<xname>!` lost across `\pgfmath@smuggleone` on our opaque primitive (:794-802), or the malformed-spec dimension math in `\lxSVG@sh@create`. 12 isolation probes clean — fires only in the full manual's showcase+tcolorbox+tikzducks context. | tikzpingus/tikzpingus-doc (oracle lualatex, renders) | not isolated; likely SHARED (faithful Perl translation) | OPEN 2026-09-07 — needs bisection of the full doc (notes `~/data/pk_agents/w22/pgf-pair/NOTES.md`) |

## D13. Manuals broken by their own class on TL2025 (SHARED with pdflatex)

**cnltx-doc `{multicols}` undefined (sweep 62: 35 manuals, 21 as first error;
root-caused 2026-09-09).** cnltx-doc.cls:728 defers `\RequirePackage{multicol,ragged2e}`
with scrlfile's deprecated `\AfterPackage!{hyperref}{…}` (scrlfile-hook.sty:209), and
hyperref only loads from `\AtEndPreamble` (cls:879), so the body runs from the
`file/hyperref.sty/after` hook at `\currentgrouplevel>0`; latex.ltx:18699-18703
`\@fileswithoptions` refuses a grouped load ("Loading a class or package in a group"),
multicol never loads, and `\begin{multicols}` (cls:796) is undefined. pdflatex on the real
class stops at cls:736 with exactly those two errors; Perl LaTeXML and oxide both load the
binding inside the group and lose the local `{multicols}` at the pop (same end state,
Perl 2 errors, oxide 1). Members: bohr_en, cnltx_en, cntformats_en, currency_doc,
dashrulex, easybook, elements-manual, embrac_en, enotez_en, fnpct-manual,
guitarchordschemes_en, idxcmds_en, leadsheets_en, passopt, schule, snotez-manual,
spbmark, syntaxdi, tasks-manual, translations-manual, utfsym (first error), plus the
chemmacros/acro family where another cnltx-doc root fires first (carbohydrates_en,
chemformula-manual, chemnum_en, chemgreek_en, endiagram_en, ghsystem-manual,
modiagram_en, substances_en, exsheets_en, xsim-manual, scaletextbullet, thalie, pixelart,
convert-jpfonts). No faithful fix produces the environment (a global package definition
inside a group would diverge from both oracles); expected gain 0. Repro
`tools/perfect_kernel/repros/loader/multicols_afterpackage_group.tex` (RED by design).
Settled dead ends: the multicol binding does load (log `Loading multicol_sty.rs`); the
`!`/label parse and `\@ifpackageloaded` are not the discriminator; raw
`\AddToHook{file/*/after}` bodies persist.

**`#` (catcode PARAM) reaching the stomach (sweep 62: 31 manuals, 12 as the dominant
class; root-caused 2026-09-09) — SHARED, no action.** The emitter
(`stomach.rs:2119-2133`) is the faithful port of Perl Stomach.pm:192-201 and of tex.web
§1049 ("You can't use macro parameter character #"): a `#` at digestion is always the
symptom of an upstream failure that pdflatex hits too. Roots: ltxmdf.cls:46
`\pdftex_if_engine:TF` (removed from l3kernel in TL2025 → its branches run as bare groups,
refcount's `\rc@RobustDefOne` never defined; mdframed-example ×4, fullwidth), classes and
packages absent from TeX Live (amltxdoc.cls: keyval2e-guide, storecmd-guide; packagedoc.cls:
underoverlap; noweb.sty: biocon; pas-doc.sty: pas-cv), a companion file not co-located
(tagpdf-code's `tagpdf-docelements.tex`), cnltx-doc (translations-manual, above), and
broken sources where pdflatex reports as much or more (changelayout-guide 102 vs our 101,
xwatermark-examples2 96 vs 25). The one pdflatex-clean member, l2tabu (2 errors), takes
scrbase.sty:1424-1444 `\ifpdfoutput`'s FALSE branch because both latexml engines set
`\pdfoutput=0` (pdfTeX.pool.ltxml:23, `pdftex.rs:11`) and that branch holds the document's
own buggy `\newcommand` — a K6 persona question (PDF-mode identity), not a cluster fix.
Repros and oracle logs: `~/data/pk_agents/w22/param-hash/`.

**Text-mode `_` from an "Anonymous String" (sweep 62: 16 manuals dominant, 237 errors;
root-caused 2026-09-09) — five shapes, one landed.** The message site is
`tex_math.rs:261` (a cat-8 `_` digested in text mode); the locator names the string
mouth, not the source. (1) chemfig re-scans molecules with `\everyeof{\_nil}…\scantokens`
(chemfig.tex:1051-1053) and our `\scantokens` (`etex.rs:465`) never inserts `\everyeof` at
the pseudo-file end, so the `\_nil`-delimited capture runs on and the molecule leaks to text
digestion — Perl shares it (eTeX.pool.ltxml:251-258, `\everyeof` "NOT used anywhere");
the wiring exists but both prior attempts regressed the l3doc family (see the `etex.rs`
comment; prerequisite = stop expandable `\verb` scanning inside edef-style bodies) — PARKED,
witnesses chemexec ×2, carbohydrates_en, quickreaction; repro
`~/data/pk_agents/w22/text-underscore/repros/shape1_chemfig_everyeof.tex`. (2) `\fcolorbox`
digested its color-name arguments (hobete_doc, 30) — landed as batch 56at. (3) the source's
own `_`/`^` in text (tikz-among-us, pst-eucl-docBG, resolsysteme-doc, egpeirce's document
positions) — SHARED with pdflatex. (4) LuaTeX-detection halts and Lua-as-TeX (fontscale's
beery.cls:29 `\sys_if_engine_luatex:F`, responsive's linebreaker.sty, pyluatex docs) —
persona/parked. (5) name re-scans through `digest()` in dun19expl3, paracol-man,
regulatory ×2 — confounded by a source-tree `.dtx` FindFile that Perl skips; deferred
per doc. Settled: `\DeclareRobustCommand\0` works (dun19expl3's `\0` is a
`\loadglsentries` context issue).

**frankenstein self-documenting manuals (sweep 66: attrib 54, dialogue 9, lgreekuse,
blkcntrl/lips/slemph `missing_file:\aftergroup` ×2, achicago/abbrevs csname leaks;
root-caused 2026-09-09) — SHARED with pdflatex, content kept.** Every driver is
`ltxdoc` + `\ProcessDTXFile{X.sty}` + `\DocInput{X.sty}`, i.e. the package's own
`%`-prose executes as LaTeX (doc.sty:895-897 `\MakePercentIgnore`). That prose is
`\cs\FOO` throughout, and compsci.sty:998-1003 `\cs@cmd@ungrouped` wraps
`\code{\FOO}` in `\begingroup…\aftergroup…\endgroup`; `\code` should be compsci's
url-verbatim (compsci.sty:510 `\newcommand*\code`) but doc.sty:622 already defines
`\code` as the identity, so the `\newcommand*` is refused (pdflatex: "Command \code
already defined", 21 errors on slemph) and every `\FOO` in the prose EXECUTES:
`\ProcessDTXFile` swallows the trailing `\aftergroup` as its file name, `\usepackage`
runs in the body, `\attrib` opens a box the `\endgroup` cannot close, moredefs'
`\futurelet` star parser leaks into a `\csname`. Perl reports 0 errors only because
its raw-`.sty` `\input` is reload-protected (Package.pm:2289-2291) and the whole
documentation body is DROPPED; our content re-read (PLANS P66, `content.rs:1712-1734`)
executes it as pdflatex does. The `\aftergroup`/`\futurelet`/`\code`-verbatim primitives
are clean (probe 0 errors). No faithful fix; an Error→Warning downgrade of a nested
missing `\input` inside a definitions re-read would save 2 lines per doc and nothing
else. Repros `~/data/pk_agents/w22/frankenstein/repros/` (`frank_docbody_reinput.tex`).

**schule (sweep 68, 4 errors) is the same cnltx root (root-causer 2026-09-09).** Three of its four errors share it: cnltx-doc.cls:728 `\AfterPackage!{hyperref}{…}` — with scrlfile raw-loaded without `withdeprecated` (scrlfile.sty:64-92), xparse's `\AfterPackage` reads `!` as the package name and the block runs at once as a plain group, so cls:729 `\newrobustcmd*\cnltx@tableofcontents` and cls:736 `\RequirePackage{multicol,ragged2e}` are group-local (latex.ltx:18699 loads anyway; the `\ver@ragged2e.sty` flag is set GLOBAL at latex.ltx:18482, mirrored by `binding/content.rs:748-948` and Perl Package.pm:2326) and the top-level cls:805 `\RequirePackage{marginnote,ragged2e}` is refused as already loaded → `\RaggedRight` undefined too. General rule: **a grouped package load sets a global loaded-flag that blocks the top-level reload, so the group-local macros are never restored** — faithful to both engines, no non-divergent fix. The fourth error, `\draw` undefined inside the `[siunitx,european]` circuitikz example of fachPhysik.tex:27 (a cnltx `example` re-`\input`), did not shrink in budget: plain `\begin{circuitikz}\draw…` is clean, and the tikzpicture examples of the same manual keep `\draw`; likely RUST-ONLY, one diagram, open. Notes `~/data/pk_agents/w22/schule/NOTES.md`.
