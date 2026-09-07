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
  DefConditional!("\\ifpdftex", { !lookup_bool("LUATEX_PROFILE") });
  DefConditional!("\\ifPDFTeX", { !lookup_bool("LUATEX_PROFILE") });
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
  // All others are false
  DefConditional!("\\ifxetex");
  DefConditional!("\\ifXeTeX");
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
  DefConditional!("\\iftutex", { lookup_bool("LUATEX_PROFILE") });
  DefConditional!("\\ifTUTeX", { lookup_bool("LUATEX_PROFILE") });
  DefConditional!("\\iftexpadtex");
  DefConditional!("\\ifTexpadTeX");
  DefConditional!("\\ifhint");
  DefConditional!("\\ifHINT");

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
\protected\def\RequireXeTeX{\IFTEX@Require\ifxetex{XeTeX}\fi}
\protected\def\RequireLuaTeX{\IFTEX@Require\ifluatex{LuaTeX}\fi}
\protected\def\RequireLuaHBTeX{\IFTEX@Require\ifluahbtex{LuaHBTeX}\fi}
\protected\def\RequireLuaMetaTeX{\IFTEX@Require\ifluahbtex{LuaMetaTeX}\fi}
\protected\def\RequirepTeX{\IFTEX@Require\ifptex{pTeX}\fi}
\protected\def\RequireupTeX{\IFTEX@Require\ifuptex{upTeX}\fi}
\protected\def\RequirepTeXng{\IFTEX@Require\ifptexng{pTeX-ng}\fi}
\protected\def\RequireVTeX{\IFTEX@Require\ifvtex{VTeX}\fi}
\protected\def\RequireAlephTeX{\IFTEX@Require\ifalephtex{AlephTeX}\fi}
\protected\def\RequireTUTeX{\IFTEX@Require\iftutex{TUTeX}\fi}
\protected\def\RequireTexpadTeX{\IFTEX@Require\iftexpadtex{TexpadTeX}\fi}
\protected\def\RequireHINT{\IFTEX@Require\ifhint{HINT}\fi}"
  );
});
