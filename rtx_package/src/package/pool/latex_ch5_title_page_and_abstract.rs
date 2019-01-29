use crate::package::*;
LoadDefinitions!(state, {

//======================================================================
// C.5.4 The Title Page and Abstract
//======================================================================
// See frontmatter support in TeX.ltxml
DefMacro!("\\title{}", "\\@add@frontmatter{ltx:title}{#1}");

DefMacro!("\\sectionmark{}", "");
DefMacro!("\\subsectionmark{}", "");
DefMacro!("\\subsubsectionmark{}", "");
DefMacro!("\\paragraphmark{}", "");
DefMacro!("\\subparagraphmark{}", "");

});