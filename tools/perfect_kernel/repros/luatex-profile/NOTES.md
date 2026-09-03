# luatex-profile — root-cause notes (Wave 14, w14)

Scope: 83 lualatex-oracle docs, converted under [luatex,rawstyles,rawclasses]latexml.sty.
Binary: /home/deyan/data/pk_bin/latexml_oxide.b54l (HEAD d1dd27af3c).

## PARKED (Japanese/pTeX/upTeX/kotex — list once, skip): 10 docs
bxcjkjatype/{bxcjkjatype-ja,bxcjkvert-ja}, bxcoloremoji/bxcoloremoji-ja,
bxjaprnind, bxjscls/bxjscls-manual, kanbun/kanbun-example (\luatexattributedef);
gckanbun/{kanshi-sample,whole-vert-sample}, kksymbols/kksymbols-doc
(\epTeXinputencoding); codebox/codebox-doc-en (\ltjsetparameter, LuaTeX-ja).
STOP: physics2/{physics2,physics2-legacy} — PLANS P75 (\the0 from unicode-math
\g__um_main_font_defined_bool; SHARED, Perl fails 102; needs unicode-math math).

## ROOT CAUSE 1 — \sys_if_engine_luatex:TF answers FALSE under [luatex]  (repro: sysengine_newXeTeXintercharclass_hang.tex)
Mechanism: expl3 l3sys computes \c_sys_engine_str via \str_const (expl3-code.tex:7846-7861)
at FORMAT-build time from \cs_if_exist:NT \tex_luatexversion:D. The [luatex] profile
(latexml_sty/mod.rs:132) defines \luatexversion only LATER at preload, so the const is
frozen to "pdftex" and \sys_if_engine_luatex:TF is FALSE permanently. Packages then take
their XeTeX/pdftex branch. Witness: polyglossia gloss-latin.ldf:125 \sys_if_engine_luatex:TF
-> else branch -> \newXeTeXintercharclass (undefined) x6 + \g_polyglossia_latin_*_class x6.
Classification: RUST-ONLY (the [luatex] profile is a Rust-only feature; its engine-identity
setup is incomplete). Verified: probe_engine.tex shows PROBE-LUATEX-FALSE / ENGINE-STR:pdftex.
Fix plan: latexml_sty/mod.rs DeclareOption!("luatex") block, after \def\luatexversion{121}:
re-run the l3sys engine detection via RawTeX (ExplSyntax):
  \cs_gset_nopar:Npn \c_sys_engine_str { luatex }
  \prg_set_conditional:Npnn \sys_if_engine_luatex: {p,T,F,TF} {\prg_return_true:}
  \prg_set_conditional:Npnn \sys_if_engine_pdftex: {p,T,F,TF} {\prg_return_false:}
  (also xetex/ptex/uptex -> false; and \c_sys_engine_exec_str/\c_sys_engine_format_str
   -> luatex/lualatex for completeness).
Verified inline (probe_fix.tex): flips PROBE-LUATEX-TRUE / ENGINE-STR:luatex, and removes
all 12 newXeTeXintercharclass/class errors from the polyglossia repro (iso_red 13 -> iso_fix 6).
Guard: 0 sys-engine errors + \sys_if_engine_luatex:TF picks the true branch.
Risk: MED (flipping to luatex-true routes packages into \directlua branches the bridge
no-ops; polyglossia's is a require() with no XML meaning — safe; other l3 packages need
a spot-check). Corpus gain: engine-identity foundation for ALL sys_if_engine_luatex docs.
SECOND BLOCKER on hang itself: \setotherlanguage{latin} then hits polyglossia's babel-compat
layer (\bbl@afterfi, \shorthandoff, \bbl@deactivate, "^ Script can only appear in math mode")
— a separate root cause (shared with the babel greek.polutoniko / \bbl@ cluster). hang x2 need
BOTH fixed to go green.

## ROOT CAUSE 2 — \mathup undefined (unicode-math)  (repro: mathup_unicodemath_toptesi.tex)
Mechanism: unicode-math-luatex.sty:2306 (and -xetex.sty:2288) \cs_set_protected:Npn \mathup
{ \mathrm }. The contrib stub latexml_contrib/src/unicode_math_sty.rs never loads either
engine file and defines only the \sym* family; the \math* alphabet aliases are missing.
Witness: toptesi topcoman.sty:76 \mathup{\mu} (both toptesi-example-luatex & -xetex).
Classification: SHARED (Perl also errors \mathup undefined — Perl has no unicode-math binding
and raw unicode-math needs xe/luatex; Perl count=2 [\mathup,\symup], oracle lualatex clean =>
surpass-approved). Verified RED (2 err) -> GREEN (0 real err) with \def\mathup{\mathrm}:
XML has <Math tex="\mathrm{x}+\mathrm{y}"> with upright XMTok x.
Fix plan: unicode_math_sty.rs LoadDefinitions, add the \math* family mirroring \sym*:
  \mathup->\mathrm (unicode-math-luatex.sty:2306, exact), plus \mathbfup->\mathbf,
  \mathbfit (like \symbfit), \mathsfup->\mathsf, \mathsfit->\mathsf, \mathbfsfup->\mathsf,
  \mathbbit->\mathbb, \mathscr->\mathcal, \mathbfscr/\mathbfcal->\mathcal,
  \mathbffrak->\mathfrak, \mathbfsf->\mathsf. (\mathit/\mathbf/\mathrm/\mathcal/\mathbb/
  \mathfrak/\mathsf/\mathtt already exist from kernel/amsmath — leave.)
Guard: 0 errors + <Math> tex="\mathrm{x}" for \mathup{x}. Risk: LOW. Corpus gain: 2 docs
(toptesi luatex+xetex, each sole-error -> likely fully green).

## ROOT CAUSE 3 — \SOUL@setup undefined (soul-ori internals not exposed by binding)  (repro: soulsetup_highlightx.tex)
Mechanism: \RequirePackage{soul} resolves to soul_sty.rs (Perl soul.sty.ltxml): the public
API (\so/\ul/\hl/\st/\sodef) reimplemented as DefConstructors, NOT soul-ori's classic
internals. Real soul.sty:162/185 \input/\RequirePackage{soul-ori} which defines \SOUL@setup /
\SOUL@preamble / \SOUL@postamble / \SOUL@everyhyphen / \SOUL@setkern / \SOUL@charkern /
\SOUL@hyphkern. highlightx.sty:193 (\SurlignerTexte) and proofread.sty:74 (\hilite) call
\SOUL@setup then override \SOUL@preamble/postamble to inject a tikz remember-picture overlay
highlight. Binding omits the internals -> \SOUL@setup undefined.
Classification: SHARED (Perl also errors \SOUL@setup undefined; Perl count=1; oracle lualatex
clean => surpass-approved). Verified RED (minimal: \usepackage{soul}\SOUL@setup).
Fix plan (checkpoint-N, deeper): the tikz-overlay highlight is purely presentational (no page
geometry in LaTeXML). Cleanest faithful option: define soul-ori's \SOUL@setup + the \SOUL@*
family the two packages touch as harmless no-ops in soul_sty.rs (letterspacing/kerning/hyphen
node loop has no XML meaning), so highlightx \SurlignerTexte / proofread \hilite degrade to
plain typeset text. Needs verifying highlightx/proofread go green (tikz overlay + simplekv).
Risk: MED. Corpus gain: 2 docs (highlightx-doc, proofread/example) — needs confirmation.

## Lower-priority / deferred clusters (candidate list, ranked below)
- \uselanguage x2 (beamertheme-mirage zh: mirage-beamer-zh, mirage-poster-zh): \uselanguage is
  translator.sty:20 (has binding: translator_sty.rs raw-loads). Undefined because these are
  ctexbeamer docs and beamer's translator load path is incomplete; the -en siblings already
  fail on \beamertemplatedotitem / \XKV@cc. beamer-blocked => ~0 corpus gain from \uselanguage
  alone. DEPRIORITIZE (not a translator bug).
- greek.polutoniko babel option x2 (greek-fontenc alphabeta-doc, hyperref-with-greek): babel
  language-option cluster; same family as polyglossia \bbl@ 2nd blocker above.
- \patch x2 + \@tabbing@" x2 (greek-fontenc char-list/char-list-alphabeta, test-lgrenc/
  textalpha-doc): LGR encoding + tabbing shorthands (lgrenc.def).
- \pdfmeta_xmp_xmlns_new:nn x2 (zugferd x2): l3pdfmeta xmp metadata surface — absorb layer.
- \TikzEveryCell (nicematrix-french, 429 err) / nicematrix (39 err): nicematrix+tikz — large.
- \XKV@cc (mirage-poster-en, 12): xkeyval internal under beamer-poster.
- \runcite (biblatex-chicago cms-legal-sample): biblatex.
- \textsection between \csname..\endcsname (clefval example-utf8): csname/catcode.
- "Keyboard character used is undefined in inputencoding utf8" (sduthesis-demo): utf8 input.

## ROOT CAUSE 4 (Checkpoint N #2) — \@ifundefined pollutes to \relax, breaking polyglossia's babelsh.def load  (repro: sysengine_newXeTeXintercharclass_hang.tex 2nd blocker)
Symptom on hang/hang, hang/sample (after RC1 landed in b54m): \bbl@afterfi undefined,
\shorthandoff undefined, \bbl@deactivate undefined, "^ Script can only appear in math mode" x4.
Mechanism (the babel question, answered): polyglossia does NOT route through the babel binding.
gloss-latin.ldf:591 does \@ifundefined{initiate@active@char}{\input{babelsh.def}}{} — babelsh.def
is polyglossia's OWN bundled file (texmf .../polyglossia/babelsh.def, "taken verbatim from babel
v3.76") that defines the whole shorthand surface it needs: \bbl@afterfi (:54), \initiate@active@char
(:278), \bbl@activate (:403), \bbl@deactivate (:407), \declare@shorthand (:420), \languageshorthands
(:490), \shorthandoff/\shorthandon, \bbl@allowhyphens (:605). babelsh.def:1-4 gates itself with
  \ifx\initiate@active@char\@undefined \else \bbl@afterfi\endinput \fi
i.e. it loads ONLY if the name is GENUINELY \@undefined. But our \@ifundefined
(latexml_engine/src/base_utilities.rs:58 \lx@ifundefined, :108 assign_meaning(&cs,\relax)) sets the
probed name to \relax (old-\csname pollution; Perl Base_Utility.pool.ltxml L23-31 does the same).
So after gloss-latin.ldf:591's \@ifundefined, \initiate@active@char is \relax (not \@undefined),
babelsh.def:1 goes to its \else, hits \bbl@afterfi\endinput while \bbl@afterfi is still undefined,
and the whole shorthand surface never loads. Modern real latex.ltx:1729-1737 defines \@ifundefined
with \ifcsname (NON-polluting) precisely to avoid this; the L1738 polluting form is the pre-\ifcsname
fallback only. So this is NOT a babel-binding gap and NOT a polyglossia-binding need — it is the
KERNEL \@ifundefined diverging from latex.ltx:1729.
Proof: probe_pollute.tex -> A-POLLUTED + \bbl@afterfi error + shorthandoff UNDEF;
       probe_nonpollute.tex (\ifcsname guard) -> babelsh.def loads fully, shorthandoff/bbl@activate
       DEF, 0 errors; probe_fullhang.tex (internals present + RC1 in b54m) -> polyglossia+latin path
       is CLEAN (0 real errors).
Classification: SHARED (Perl pollutes identically: babelsh.perl.tex -> PERL-POLLUTED-relax +
\bbl@afterfi at babelsh.def:3; Perl count=1). Oracle lualatex/pdflatex clean (modern kernel) =>
surpass-approved.
Fix plan: latexml_engine/src/base_utilities.rs, \lx@ifundefined (fn body L58-112): DELETE the
pollution — the `assign_meaning(&cs, lookup_meaning(&TOKEN_RELAX), None)` at L108 — so the undefined
branch just returns if_token without overwriting the cs. That makes \@ifundefined non-polluting,
matching latex.ltx:1729-1737 (\ifcsname form). The autoload path (L80-103, smfart 2507.23241v1
witness) is unaffected/strengthened: removing the overwrite only makes it MORE conservative (it
already guarded the assign with `if !is_autoload`). Optional full-fidelity refinement: also treat a
\relax-valued cs as undefined (kernel's \@ifundefin@d@i \ifx...\relax step) — separate, not needed
for babelsh.
Guard: perfect_kernel batch NN — assert (1) \@ifundefined{zz@undef}{}{} leaves \zz@undef genuinely
\@undefined (\ifx\zz@undef\@undefined true); (2) the polyglossia-latin repro: 0 errors + a <ltx:p>
containing "latin"/"text". Risk: MED (broad kernel change; every \@ifundefined caller — validate via
full suite; the pollution has been the behavior since commit 5732f3c3b4, CHANGELOG L1143). 
Corpus gain (residue, PROVEN): hang/hang 17->~0 and hang/sample 37->~0 (both = RC1 + RC4, fully
clear per probe_fullhang). Beyond residue: any polyglossia doc using a babelshorthands language
(19 gloss-*.ldf: latin, german, russian, italian, czech, polish, portuguese, dutch, slovak,
ukrainian, catalan, croatian, finnish, ...), plus any package that loads a reentrancy-guarded .def
via \@ifundefined{sentinel}{\input file}.
NOT SHARED with the greek.polutoniko cluster: alphabeta-doc:66 \usepackage[greek.polutoniko,english]
{babel} fails "Unknown option 'greek.polutoniko'" — babel's DOTTED-MODIFIER option syntax
(language.modifier), a separate babel-binding option-parsing gap, unrelated to \@ifundefined.

## ROOT CAUSE 5 (Checkpoint N #3) — babel language.modifier options: process_options reads the State VecDeque, not the babel-rewritten \opt@<pkg> macro  (repro: babelmodifier_greek_polutoniko.tex)
Chosen over RC3-soul by corpus gain: both greek.polutoniko docs (alphabeta-doc, hyperref-with-greek)
have ONLY this 1 error and reach 0 with the modifier stripped (verified: alphabeta-stripped,
hyp-stripped -> 0 errors, complete). RC3-soul at best clears 1 doc (proofread has \LL/\FL/\ctable too).
Mechanism (the babel dotted-modifier question, answered):
- \usepackage[greek.polutoniko,english]{babel}. babel.sty:316-347 preprocesses BEFORE \ProcessOptions:
  \bbl@tempd (:322) splits each option on '.', \bbl@tempe/\bbl@csarg\edef{mod@greek} (:320-321) stores
  \bbl@mod@greek = polutoniko, and :347 rewrites the MACRO \opt@babel.sty := \bbl@tempc = the
  modifier-STRIPPED "greek,english". \ProcessOptions* (:414) then processes it; greek.ldf later reads
  \BabelModifiers = \bbl@mod@greek (babel.sty:4136-4137) to turn on polytonic. So polutoniko needs
  ONLY \bbl@mod@greek — which our engine ALREADY sets correctly (probe: MODGREEK=[polutoniko],
  OPTBABEL=[greek,english]).
- The break: real \ProcessOptions reads \@curroptions := \@ptionlist{\@currname.\@currext}
  (latex.ltx:18557) and \@ptionlist (:18393) expands the MACRO \opt@babel.sty (= rewritten
  "greek,english"). Our \ProcessOptions is a Rust primitive (latex_constructs.rs:5300) -> process_options
  (latexml_core/src/binding/content.rs:1890), which at :1904-1905 reads the STATE VecDeque
  opt@babel.sty (still the raw "greek.polutoniko,english") — babel's macro rewrite is invisible to it.
  So \bbl@load@language{greek.polutoniko} -> \InputIfFileExists{greek.polutoniko.ldf} fails ->
  babel.sty:4140 \bbl@error{unknown-package-option} "Unknown option 'greek.polutoniko'".
Classification: SHARED (Perl ProcessOptions reads LookupValue('opt@...') the same State store; Perl
fails identically "Unknown option 'greek.polutoniko'" at babel.sty:4301, count=2). Oracle
lualatex/pdflatex clean (real \@ptionlist reads the macro) => surpass-approved.
Fix plan (CORE, not babel_sty.rs): process_options (content.rs:1904-1905) should build current_options
from the \opt@<name>.<ext> MACRO expansion (comma-split, trimmed) — faithful \@ptionlist
(latex.ltx:18393) — falling back to the VecDeque only when the macro is undefined. The macro is proven
in-sync with the VecDeque for ordinary packages (graphicx=[final,draft], article=[11pt,twocolumn]),
so behavior is unchanged EXCEPT when a package deliberately rewrites \opt@<pkg> before \ProcessOptions
— exactly the LaTeX-standard idiom babel uses and that we must honor. NOT babel_sty.rs: putting it
there would duplicate babel's modifier parser and miss the general case (es-*, german variants, other
packages that rewrite \opt@). babel_sty.rs already pre-allocates \l@polutonikogreek (:19) as a related
partial workaround; leave it.
Guard: perfect_kernel batch NN — babelmodifier_greek_polutoniko repro: 0 errors + \bbl@mod@greek
expands to "polutoniko" + a body <ltx:p>; plus a direct assertion that after a package \def-rewrites
\opt@<pkg>.<ext>, its \ProcessOptions processes the rewritten list.
Risk: MED (process_options is on every package/class load path; validated by the in-sync evidence,
gate on full suite). Corpus gain (residue, PROVEN): alphabeta-doc 1->0, hyperref-with-greek 1->0
(both reach clean, verified with modifier stripped). Beyond residue: any doc using babel
language.modifier syntax (es-tabla/es-cuadro spanish, german variants, ...) and any package rewriting
\opt@<pkg> pre-\ProcessOptions.
Dead ends: suspected the modifier preprocessing (babel.sty:322-347) didn't run under our raw load —
it DID (\bbl@mod@greek and the rewritten \opt@babel.sty macro are both correct); the sole break is
process_options reading the wrong (State-VecDeque) copy of the option list.
