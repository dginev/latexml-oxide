//! elsart_support.sty — Elsevier article support (non-core additions)
//! Perl: elsart_support.sty.ltxml — 175 lines
//! Loads elsart_support_core and adds theorem/proof/section formatting
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("elsart_support_core");

  // Theorem stubs (if amsthm not loaded)
  DefMacro!("\\theoremstyle{}", "");
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true);

  // Proof environment — Perl L38-60
  DefEnvironment!("{proof}[]",
    "<ltx:proof><ltx:title font='italic' _force_font='true' class='ltx_runin'>#title</ltx:title>#body</ltx:proof>",
    properties => { stored_map!("title" => Stored::from("Proof")) }
  );

  // Section formatting — Perl L63-120
  // These customize section numbering and font for Elsevier style
  DefMacro!("\\elsartstyle", "");
  DefMacro!("\\semark{}",    "");
  DefMacro!("\\ssmark{}",    "");
  DefMacro!("\\sssmark{}",   "");
  DefMacro!("\\elsmarks",    "");

  // Abstract keywords with continuation
  DefMacro!("\\KWD{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  DefMacro!("\\AMS{}",  "\\@add@frontmatter{ltx:classification}[scheme=MSC]{#1}");
  DefMacro!("\\PAC{}",  "\\@add@frontmatter{ltx:classification}[scheme=PACS]{#1}");

  // Theorem environments — Perl L69-91
  RawTeX!("\\theoremstyle{plain}");
  RawTeX!("\\@ifundefined{cor}{\\newtheorem{cor}[thm]{Corollary}}{}");
  RawTeX!("\\@ifundefined{lem}{\\newtheorem{lem}[thm]{Lemma}}{}");
  RawTeX!("\\@ifundefined{claim}{\\newtheorem{claim}[thm]{Claim}}{}");
  RawTeX!("\\@ifundefined{conj}{\\newtheorem{conj}[thm]{Conjecture}}{}");
  RawTeX!("\\@ifundefined{prop}{\\newtheorem{prop}[thm]{Proposition}}{}");
  RawTeX!("\\@ifundefined{defn}{\\newtheorem{defn}[thm]{Definition}}{}");
  RawTeX!("\\@ifundefined{exmp}{\\newtheorem{exmp}[thm]{Example}}{}");
  RawTeX!("\\@ifundefined{rem}{\\newtheorem{rem}[thm]{Remark}}{}");
  RawTeX!("\\@ifundefined{note}{\\newtheorem{note}{Note}}{}");

  // Nuclear isotopes — Perl L60-65
  DefMacro!("\\nuc{}{}", "\\ensuremath{{}^{#2}\\mathrm{#1}}");
  DefMacro!("\\itnuc{}{}", "\\ensuremath{{}^{#2}\\textit{#1}}");

  // Caption continuations — Perl L108-110
  DefMacro!("\\contcaption", "\\caption{continued}");
  DefMacro!("\\contfigurecaption", "\\caption{continued}");
  DefMacro!("\\conttablecaption", "\\caption{continued}");

  // Bibliography — Perl L117-175
  DefEnvironment!("{subbibitems}", "#body");
  DefMacro!("\\cv", "");
  DefMacro!("\\biboptions{}", "");
  DefMacro!("\\bibliographystyle{}", "");
  DefMacro!("\\harvarditem[]{}{}{}",
    "\\bibitem[#2(#3)]{#4}");
  DefMacro!("\\harvardand", "\\&");
  DefMacro!("\\harvardurl{}", "\\url{#1}");
  DefMacro!("\\harvestremark{}", "");
  DefMacro!("\\harvardyearleft", "(");
  DefMacro!("\\harvardyearright", ")");
  DefMacro!("\\citestyle{}", "");

  // Shorthands — Perl L124-128
  DefMacro!("\\AND", "\\&");
  DefMacro!("\\etal", "et al.");
  DefMacro!("\\Elproofname", "Proof.");
  DefMacro!("\\proofname", "Proof.");

  // Dimensions — Perl L132-139
  DefMacro!("\\cropwidth", "297mm");
  DefMacro!("\\cropheight", "210mm");
  DefMacro!("\\cropleft", "0mm");
  DefMacro!("\\croptop", "0mm");
  DefRegister!("\\rulepreskip" => Dimension!("4pt"));
  DefMacro!("\\setleftmargin{}{}", "");

  // Misc — Perl L143-175
  Let!("\\realpageref", "\\pageref");
  DefMacro!("\\snm", "");
  DefEnvironment!("{NoHyper}", "#body");
  DefMacro!("\\mpfootnotemark", "");
  DefMacro!("\\FMSlash", "\\protect\\pFMSlash");
  DefMacro!("\\pFMSlash{}", "#1/");
  DefMacro!("\\pFMslash{}", "#1/");
  DefMacro!("\\Slashbox", "/");
  DefMacro!("\\slashbox", "/");
  DefMacro!("\\query{}", "");
});
