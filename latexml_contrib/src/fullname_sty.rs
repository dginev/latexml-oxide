use latexml_package::prelude::*;

LoadDefinitions!({
  // stub in for now.
  DefMacro!("\\citename{}", "#1, ");
  DefMacro!("\\shortcite", "\\cite");
  DefMacro!("\\namecite", "\\cite");
});
