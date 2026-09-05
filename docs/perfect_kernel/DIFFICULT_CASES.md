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

