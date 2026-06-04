use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: yfonts.sty.ltxml — Not much to do for LaTeXML.

  DefPrimitive!("\\frakfamily", None, font => {family => "fraktur"});
  // These font families are otherwise unrecognized...
  DefPrimitive!("\\swabfamily", None, font => {family => "schwabacher"});
  DefPrimitive!("\\gothfamily", None, font => {family => "gothic"});

  // SHOULD set up fancy initials...
  DefMacro!("\\initfamily", None);
  DefPrimitive!("\\fraklines", None);

  // Text-font commands. yfonts.sty source declares
  //   \DeclareTextFontCommand{\textfrak}{\frakfamily}
  //   \DeclareTextFontCommand{\textswab}{\swabfamily}
  //   \DeclareTextFontCommand{\textgoth}{\gothfamily}
  //   \DeclareTextFontCommand{\textinit}{\initfamily}
  // Since the binding intercepts the package load and the raw-load that
  // would otherwise execute `\DeclareTextFontCommand` is skipped, mirror
  // the resulting `\def\textswab#1{{\swabfamily #1}}` shape directly.
  // Witness: arXiv:1907.06086 `\textswab{f}_0^{-1}` inside math was
  // emitting `Error:undefined:\textswab`. Perl's yfonts.sty.ltxml also
  // omits these (relies on raw-load); this is a parity-augmenting fix.
  DefMacro!("\\textfrak{}", "{\\frakfamily #1}");
  DefMacro!("\\textswab{}", "{\\swabfamily #1}");
  DefMacro!("\\textgoth{}", "{\\gothfamily #1}");
  DefMacro!("\\textinit{}", "{\\initfamily #1}");

  DefMacro!("\\yinipar{}", "\\par\\noindent\\yinitpar{#1}");
  // SHOULD set the initial in fancy font.
  DefMacro!("\\yinitpar{}", "#1");

  // Nothing likely to ever be used, but for completeness...
  DefMacro!("\\gothdefault", "ygoth");
  DefMacro!("\\swabdefault", "yswab");
  DefMacro!("\\frakdefault", "yfrak");
  DefMacro!("\\initdefault", "yinitas");

  Let!("\\grq", "\\textquoteleft");
  Let!("\\grqq", "\\textquotedblleft");
});
