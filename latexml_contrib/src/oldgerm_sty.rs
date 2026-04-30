use latexml_package::prelude::*;

LoadDefinitions!({
  // Not much to do for LaTeXML, similar to yfonts.sty.ltxml
  DefPrimitive!("\\frakfamily", None, font => {family => "fraktur"});
  // These font families are otherwise unrecognized...
  DefPrimitive!("\\swabfamily", None, font => {family => "schwabacher"});
  DefPrimitive!("\\gothfamily", None, font => {family => "gothic"});

  // Nothing likely to ever be used, but for completeness...
  DefMacro!("\\gothdefault", "ygoth");
  DefMacro!("\\swabdefault", "yswab");
  DefMacro!("\\frakdefault", "yfrak");

  RawTeX!(
    "\\DeclareTextFontCommand{\\textgoth}{\\gothfamily}\n\\DeclareTextFontCommand{\\textswab}{\\swabfamily}\n\\DeclareTextFontCommand{\\textfrak}{\\frakfamily}"
  );
});
