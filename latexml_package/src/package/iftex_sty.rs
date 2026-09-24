/// Perl: iftex.sty.ltxml — TeX engine detection conditionals
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // eTeX is always "true" for LaTeXML; the pdfTeX/LuaTeX pair follows the
  // opt-in `luatex` latexml.sty profile (user decision 2026-08-31): a
  // LuaLaTeX-authored document converted with
  // `--preload=[…,luatex]latexml.sty` identifies as LuaTeX so its
  // engine-detection branches run; everything else stays the pdfTeX model.
  // Consulting the state value HERE (not a load-time constant) matters
  // because this binding may load after latexml.sty processed its options.
  DefConditional!("\\ifetex", { true });
  DefConditional!("\\ifeTeX", { true });
  DefConditional!("\\ifpdftex", { !unicode_engine_profile() });
  DefConditional!("\\ifPDFTeX", { !unicode_engine_profile() });
  DefConditional!("\\ifluatex", { lookup_bool("LUATEX_PROFILE") });
  DefConditional!("\\ifLuaTeX", { lookup_bool("LUATEX_PROFILE") });
  // iftex.sty:272-291: `\ifpdf` is TRUE on LuaTeX whenever
  // `tex.outputmode`/`tex.pdfoutput` > 0 — always, LuaTeX defaults to PDF
  // output — so a lualatex-oracle document sees the PDF branch
  // (tikzrput.sty defines `\rput` only inside `\ifpdf…\fi`: pgfornament
  // ornaments/tikzrput). The pdfTeX persona keeps FALSE (Perl; the K6
  // PDF-mode question is separate). Guard:
  // `perfect_kernel_batch56::ifpdf_is_true_under_the_luatex_profile`.
  DefConditional!("\\ifpdf", { lookup_bool("LUATEX_PROFILE") });
  // iftex.sty:269-270: the legacy ifpdf.sty setters.
  RawTeX!(r"\def\pdftrue{\let\ifpdf\iftrue}\def\pdffalse{\let\ifpdf\iffalse}");
  // The opt-in `xetex` profile (latexml.sty, OXIDIZED_DESIGN #283).
  DefConditional!("\\ifxetex", { lookup_bool("XETEX_PROFILE") });
  DefConditional!("\\ifXeTeX", { lookup_bool("XETEX_PROFILE") });
  // All others are false
  DefConditional!("\\ifluahbtex");
  DefConditional!("\\ifLuaHBTeX");
  DefConditional!("\\ifptex");
  DefConditional!("\\ifpTeX");
  DefConditional!("\\ifuptex");
  DefConditional!("\\ifupTeX");
  DefConditional!("\\ifptexng");
  DefConditional!("\\ifpTeXng");
  DefConditional!("\\ifvtex");
  DefConditional!("\\ifVTeX");
  DefConditional!("\\ifalephtex");
  DefConditional!("\\ifAlephTeX");
  DefConditional!("\\iftutex", { unicode_engine_profile() });
  DefConditional!("\\ifTUTeX", { unicode_engine_profile() });
  DefConditional!("\\iftexpadtex");
  DefConditional!("\\ifTexpadTeX");
  DefConditional!("\\ifhint");
  DefConditional!("\\ifHINT");

  define_xetex_interchar()?;
  // `\RequireXeTeX` and `\RequireTUTeX` pass (Perl makes every `\Require<engine>` a
  // no-op, iftex.sty.ltxml:52-64): what XeTeX gives a document — UTF-8 input,
  // OpenType font selection, script switching — the Unicode-native engine and
  // its fontspec/polyglossia/unicode-math bindings already provide, so an
  // XeLaTeX paper converts under the default persona (11 papers of arXiv 2605
  // halted, 2605.02089, 2605.16477). The other engines keep iftex's halt.
  // iftex.sty:42-52 + :78-91 verbatim: `\Require<engine>` tests the engine
  // conditional and otherwise writes the banner and HALTS via the
  // `\batchmode\read -1` terminal read (tex.web §484 → Fatal here). Formerly
  // 13 no-ops, which let XeTeX-only packages (bidi, ucharclasses) run their
  // bodies on undefined XeTeX primitives — the sweep-57 alloc_failed cluster.
  RawTeX!(
    r"\def\IFTEX@Require#1#2#3{#1\else\newlinechar 64\relax\errorcontextlines -1\relax
  \immediate\write20{@********************************************@* #2 is required to compile this document.@* Sorry!@********************************************}%
  \batchmode\read -1 to \@tempa #3}
\protected\def\RequireeTeX{\IFTEX@Require\ifetex{eTeX}\fi}
\protected\def\RequirePDFTeX{\IFTEX@Require\ifpdftex{pdfTeX}\fi}
\protected\def\RequireXeTeX{\lx@xetex@interchar}
\protected\def\RequireLuaTeX{\IFTEX@Require\ifluatex{LuaTeX}\fi}
\protected\def\RequireLuaHBTeX{\IFTEX@Require\ifluahbtex{LuaHBTeX}\fi}
\protected\def\RequireLuaMetaTeX{\IFTEX@Require\ifluahbtex{LuaMetaTeX}\fi}
\protected\def\RequirepTeX{\IFTEX@Require\ifptex{pTeX}\fi}
\protected\def\RequireupTeX{\IFTEX@Require\ifuptex{upTeX}\fi}
\protected\def\RequirepTeXng{\IFTEX@Require\ifptexng{pTeX-ng}\fi}
\protected\def\RequireVTeX{\IFTEX@Require\ifvtex{VTeX}\fi}
\protected\def\RequireAlephTeX{\IFTEX@Require\ifalephtex{AlephTeX}\fi}
\protected\def\RequireTUTeX{}
\protected\def\RequireTexpadTeX{\IFTEX@Require\iftexpadtex{TexpadTeX}\fi}
\protected\def\RequireHINT{\IFTEX@Require\ifhint{HINT}\fi}"
  );
});

/// XeTeX's inter-character primitives, for the packages that DECLARE themselves
/// XeTeX-only: `\RequireXeTeX` installs them (ucharclasses.sty:987, then its
/// `\XeTeXcharclass` loops over whole Unicode blocks). With them undefined, the
/// loop `\XeTeXcharclass \count \class` degraded into a spurious assignment and
/// never ended (latexbangla: 95k warnings, then a 6.3 GB `alloc_failed`; the
/// sweep-57 cluster). They read their real arguments — `<number> [=] <number>`,
/// `<number> <number> [=] {<tokens>}`, an integer parameter — and change nothing:
/// the classes drive font switching the Unicode-native engine does not need.
/// Installed on request, or by latexml.sty's `xetex` profile (run at the format
/// seam, `latex.rs`), because amsmath, mathastext, minted2 and others probe
/// `\XeTeXcharclass` to detect XeTeX (ordinary pdfLaTeX documents must not take
/// those branches). `\newXeTeXintercharclass` is latex.ltx:22025-22031's
/// allocator, over a fresh counter (latex.ltx `\countdef`s register 257, which
/// the pdfTeX-persona format may already use).
pub fn define_xetex_interchar() -> Result<()> {
  DefPrimitive!("\\lx@XeTeXcharclass Number OptionalMatch:= Number", sub[(_c, _eq, _k)] {});
  DefPrimitive!("\\lx@XeTeXinterchartoks Number Number OptionalMatch:= {}", sub[(_a, _b, _eq, _t)] {});
  DefRegister!("\\lx@XeTeXinterchartokenstate" => Number::new(0));
  RawTeX!(
    r"\def\lx@xetex@interchar{\ifx\XeTeXcharclass\@undefined
  \let\XeTeXcharclass\lx@XeTeXcharclass
  \let\XeTeXinterchartoks\lx@XeTeXinterchartoks
  \let\XeTeXinterchartokenstate\lx@XeTeXinterchartokenstate
  \chardef\e@alloc@intercharclass@top=4095
  \newcount\xe@alloc@intercharclass
  \def\newXeTeXintercharclass{\e@alloc\XeTeXcharclass\chardef\xe@alloc@intercharclass\m@ne\e@alloc@intercharclass@top}%
  \fi}"
  );
  Ok(())
}
