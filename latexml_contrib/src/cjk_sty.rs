use latexml_package::prelude::*;

LoadDefinitions!({
  // Graciously borrowed from arxiv-vanity/engrafo/latexml/packages/CJK.sty.ltxml
  // seems to make 1705.06031 work
  DefEnvironment!("{CJK}{}{}", "#body");
  DefEnvironment!("{CJK*}{}{}", "#body");
  DefMacro!("\\CJKfamily{}", "#1");
});
