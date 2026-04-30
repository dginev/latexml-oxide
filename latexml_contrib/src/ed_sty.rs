use latexml_package::prelude::*;

LoadDefinitions!({
  // Perl ar5iv-bindings/ed.sty.ltxml: editorial notes stub — discards the
  // body. The optional arg is the category/author, the body is the note
  // text; neither is rendered in the XML output. Matches Perl's empty
  // expansion — purpose is just to make documents compile without
  // undefined-CS errors when ed.sty is loaded.
  DefMacro!("\\ednote[]{}", "");
});
