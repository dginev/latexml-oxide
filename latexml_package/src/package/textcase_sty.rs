use crate::prelude::*;

// Port of LaTeXML/lib/LaTeXML/Package/textcase.sty.ltxml
// The textcase package provides \MakeTextUppercase, \MakeTextLowercase, \MakeTextTitlecase
// which in LaTeXML are simple aliases for the base case commands.
LoadDefinitions!({
  Let!("\\MakeTextUppercase", "\\MakeUppercase");
  Let!("\\MakeTextLowercase", "\\MakeLowercase");
  Let!("\\MakeTextTitlecase", "\\MakeTitlecase");
});
