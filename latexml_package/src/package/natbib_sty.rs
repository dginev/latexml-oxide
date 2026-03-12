use crate::prelude::*;

LoadDefinitions!({
  // Package options — stubs
  DeclareOption!("numbers", "");
  DeclareOption!("super", "");
  DeclareOption!("authoryear", "");
  DeclareOption!("round", "");
  DeclareOption!("curly", "");
  DeclareOption!("square", "");
  DeclareOption!("angle", "");
  DeclareOption!("comma", "");
  DeclareOption!("semicolon", "");
  DeclareOption!("colon", "");
  DeclareOption!("nobibstyle", "");
  DeclareOption!("bibstyle", "");
  DeclareOption!("sort", "");
  DeclareOption!("sort&compress", "");
  DeclareOption!("compress", "");
  DeclareOption!("longnamesfirst", "");
  DeclareOption!("openbib", "");
  DeclareOption!("sectionbib", {
    AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:section");
  });
  DeclareOption!("nonamebreak", "");
  ProcessOptions!();

  // Core commands — stubs that pass through to basic \cite
  DefMacro!("\\citet OptionalMatch:* [][]Semiverbatim", "\\cite{#4}");
  DefMacro!("\\citep OptionalMatch:* [][]Semiverbatim", "\\cite{#4}");
  DefMacro!("\\citealt OptionalMatch:* [][]Semiverbatim", "\\cite{#4}");
  DefMacro!("\\citealp OptionalMatch:* [][]Semiverbatim", "\\cite{#4}");
  DefMacro!("\\citeauthor OptionalMatch:* []Semiverbatim", "\\cite{#3}");
  DefMacro!("\\citeyear []Semiverbatim", "\\cite{#2}");
  DefMacro!("\\citeyearpar []Semiverbatim", "\\cite{#2}");
  DefMacro!("\\Citet [][]Semiverbatim", "\\cite{#3}");
  DefMacro!("\\Citep [][]Semiverbatim", "\\cite{#3}");
  DefMacro!("\\Citealt [][]Semiverbatim", "\\cite{#3}");
  DefMacro!("\\Citealp [][]Semiverbatim", "\\cite{#3}");
  DefMacro!("\\Citeauthor []Semiverbatim", "\\cite{#2}");

  // Additional natbib macros
  DefMacro!("\\citenum Semiverbatim", "\\cite{#1}");
  DefMacro!("\\citefullauthor Semiverbatim", "\\cite{#1}");
  DefMacro!("\\bibpunct{}{}{}{}{}{}", "");
  DefMacro!("\\setcitestyle{}", "");
  DefMacro!("\\citestyle{}", "");
  DefMacro!("\\bibsection", "");
  DefMacro!("\\bibpreamble", "");
  DefMacro!("\\bibfont", "");
  DefMacro!("\\bibnumfmt{}", "#1");
  DefMacro!("\\bibhang", "");
  DefMacro!("\\bibsep", "");
  DefMacro!("\\defcitealias{}{}", "");
  DefMacro!("\\citetalias Semiverbatim", "\\cite{#1}");
  DefMacro!("\\citepalias Semiverbatim", "\\cite{#1}");
});
