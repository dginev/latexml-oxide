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

## D12. Deferred shared roots with a high-risk fix (wave 18, 2026-09-06)

Each is pdflatex/lualatex-clean, fails identically (or worse) in same-host Perl,
and has a root-cause report + red repro on disk; the fix is a mechanism change
whose blast radius outweighs the docs it frees today. Re-open when the mechanism
comes up for another reason.

| Root | Witnesses | Repro | What the fix is |
|---|---|---|---|
| pgf SVG driver group accounting: `pgfsys-latexml.def.ltxml:561-586` maps `\pgfsys@beginscope`/`@endscope` to real `\begingroup`/`\endgroup` (the pdf driver opens none); through `\pgfnode`/`\pgfmultipartnode` + `\pgfsys@begin@idscope` (pgfsys.code.tex:572-611) inside `\foreach` + `pgfscope` the count drifts and a `{` boxing frame stays open, so every later `\endgroup` reports "close non-boxing group" (the LaTeXML authors note the class at pgfsys-latexml.def.ltxml:887-890). SHARED: Perl 25 errors on the repro, Rust 24. | msc/msc (29 + PushbackLimit fatal from `codeexample` re-execution); modernposter/demo (16: the poster body is one document-spanning overlay `tikzpicture`, modernposter.cls:102-112, and `\maketitle`'s filled nodes replay their `\hbox` box with a scope `\begingroup` on top — `base_utilities.rs:3610` `predigest_box_contents_in_mode` → `end_mode` fails; Perl identical first error, 31 errors) | `tools/perfect_kernel/repros/graphics-tikz/msc_declinst_tikz_scope_desync.tex` (one `\declinst` inside `{msc}`), `graphics-tikz/modernposter_maketitle_hbox_mode.tex` | reconcile the node-box + idscope group pairs against the injected stomach groups in the SVG driver (the shared Perl semantics and their Rust realisation); a native msc.sty binding would re-derive the chart geometry. No stopgap pop on the errant `\endgroup`. |
| `\halign` inside display math whose `$#$` cells open `\hbox` (an active `<…>` that does `\ifhmode\else\expandafter\hbox\fi\bgroup…$\langle$…$\rangle$…\egroup`, abntexto.tex:79-81) with inline math inside the box: the mode-switch frames (`stomach.rs:753` "close a group that switched to mode", `:1112` "Attempt to end mode") never reconcile the nesting; the `\halign`, the display-math group and the enclosing list stay open and every later section lands inside the trapped `<ltx:inline-block>` (the 15 `<ltx:section> in <ltx:section>` errors are this collateral, not sectioning). Reached only through the manual's self-documentation trick (`\catcode\`\%=9 \input{abntexto.cls}`, :1037) that turns the class's commented `$$\offinterlineskip\halign{$#$\cr…}$$` (cls:727-736) live. SHARED: Perl 4 × "Attempt to end mode math". | abntexto/abntexto (~22 of 25 errors; the other root, amsmath's missing `\@saveprimitive\over\@@over`, landed in batch 56w) | `tools/perfect_kernel/repros/boxes-groups/reinput_display_halign.{tex,snippet}` | the parked mode-frame family: an `\hbox` opened inside a math cell must re-establish the math nesting boundary (tex.web `\displ@y`/alignment mode discipline) so the inner `$…$` closes inside the box and the `\halign` frame pops in display math. |
| zx-calculus self-loop wires: a `\ar[loop,in=…,out=…]` on one node drives `\zx@find@intersection@fakecenter` (tikzlibraryzx-calculus.code.tex:481-597) → `\pgfintersectionofpaths` on a DEGENERATE line (start==target); pgf's Bézier subdivision (pgflibraryintersections.code.tex:170/738, tolerance :689) never reaches its `\ifdim … < \pgfintersectiontolerance` stop in our engine — every pass adds a `\begingroup` pair until the stomach cycle guard Fatals (~60 s). SHARED: Perl runs away 197 s to the wall clock. | zx-calculus/zx-calculus (1 Fatal) | `graphics-tikz/zx_selfloop_intersection.tex` (+ `zx_selfloop_bare_CONTROL.tex`) | the pgfmath/`\ifdim` evaluation on the degenerate subdivision must converge as real pgf does — needs an instrumented `\pgfintersectionofpaths` trace; "Fatal stays Fatal" already holds (we stop in 60 s, Perl hangs). |
| ribbonproofs ribbon lists: etextools' `\ifintokslist`/for-loops scan token lists that carry conditional primitives as DATA (etextools.sty:773-777 `…\iffalse\ifcase\ifdefined…`); we lose a `\fi` at a mouth boundary and the `finish ribbons={c,d}` parse does not retire `c`, so `\com[finish ribbons={e},start ribbons={e}]` raises the package's own "already active" error. SHARED: Perl fails earlier (`\globcount` undefined at etextools.sty:905). | ribbonproofs/ribbonproofsmanual (3) | `parameter-conditional/etextools_ribbonlist_ribbonproofs.tex` (not reducible below the ~76-line proof body) | either an etextools binding or conditional-tokens-as-data in the scanner plus ribbon-state ordering — package-specific, HIGH. |
| mhequ single-line `{equ}`: `\@saveMHComms` `\let\\=\@MHcr` (mhequ.sty:181) with `\@MHcr` undefined in the `equ` path, restored by `\@restoreMHComms` (:184) which sits AFTER `\eqno{…}` — our `\eqno` (tex_math.rs:1790, = Perl TeX_Math.pool.ltxml:1239) gullet-scans the tag to `$$` and collects the `\let` instead of executing it, so `\\` stays undefined (3 errors; Perl 8). | mhequ/mhequ-example (3) | `kernel-alignment/mhequ_equ_cr.tex` | `\eqno`/`\leqno` digesting their tag material (assignments execute) — a shared display-math primitive rework; we already beat Perl. |
| psmatrix cell template: with the real `\psset` (batch 56x) pst-node's `\psm@endnode` runs and the `\halign` cell template's `\begingroup…\endgroup` pairs desync ("close a group that switched to mode restricted_horizontal" ×4, `\endgroup` non-boxing ×4); pstricks-add's colour keys also read `\pst@getcolor` = xcolor's `\XC@getcolor` (pstricks.sty:155), missing from `xcolor_sty.rs`. | dsptricks/dspTricksManual (100+, capped), pst-eucl-docBG | `graphics-tikz/psmatrix_psk_mnodesize_dsptricks.tex` (psk internals now defined) | alignment-engine root (the psmatrix template's group discipline) + `\XC@getcolor`/`\XC@usecolor` in the xcolor binding. |
| geometry global write-back: `elzcards.sty:396-397` reads `\textwidth`/`\textheight` into counters and fits 5in cards (:700) into the page box; our geometry binding sizes the SVG canvas only (OXIDIZED_DESIGN #99) and never writes the body box back to the global dims (`geometry_sty.rs:100-125` region; real geometry.sty assigns `\textwidth=\Gm@tw` etc.), so the article default 345pt < 5in and `\elzc@CalculaMatriz` (:281-305) raises "No space to print at least one card". SHARED: Perl no-ops geometry, 3 errors. | elzcards/elzcards-examples (1) | `layout-singles/geometry_noop_textwidth_elzcards.tex` (`~/data/pk_agents/w20/layout-singles/repros`) | overturn #99: geometry writes the computed body box to the global `\textwidth`/`\textheight`/paper dims — a flow-sizing policy change touching every geometry doc's width attributes; user decision, own branch. (Secondary, non-decisive: package-option splitting does not honour a brace-protected comma, `vmargin={0mm,0mm}` → 4 "Missing number" warnings.) |
| PDF persona for graphics extensions: `upmethodology-fmt.sty:460-463` picks `.pdftex_t/.pdf_tex` under `\ifpdf`, else `.pstex_t/.ps_tex`; both engines are DVI (`\pdfoutput=0`, `\ifpdf` false under the pdflatex profile), the doc ships only `figure_and_tex.pdftex_t`, so `\includefigurewtex` (:507-509) errmessages "File not found". `\IfFileExists`/`\filename@parse` are correct. SHARED (Perl fails differently: undefined `\@autolatex@wtfig@exttmp`). | upmethodology/upmethodology-doc (1) | `layout-singles/ifpdf_persona_graphics_ext_upmethodology.tex` | the K6 PDF-mode persona decision (KERNEL_CAPABILITIES 2026-09-05 K6 row; PLANS P16-vii) — a direct witness for "graphics extensions keyed on `\pdfoutput`". No `\ifpdf` stub. |
