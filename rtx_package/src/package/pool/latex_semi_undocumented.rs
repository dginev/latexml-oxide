use crate::package::*;
LoadDefinitions!(state, {
  //======================================================================
  // Hair
  DefPrimitive!("\\makeatletter", sub { AssignCatcode!('@', Catcode::LETTER, Some(Scope::Local)); });
  DefPrimitive!("\\makeatother",  sub { AssignCatcode!('@', Catcode::OTHER, Some(Scope::Local)); });

  //**********************************************************************
  // Sundry (is this ams ?)
  DefMacro!("\\textprime", "\u{00B4}"); // ACUTE ACCENT

  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");
  //**********************************************************************
  // Should be defined in each (or many) package, but it"s not going to
  // get set correctly or maintained, so...
  DefMacro!("\\fileversion", "");
  DefMacro!("\\filedate", "");

  // Ultimately these may be overridden by babel, or otherwise,
  // various of these are defined in various places by different classes.
  DefMacro!("\\chaptername", "Chapter");
  DefMacro!("\\partname", "Part");
  // The rest of these are defined in some classes, but not most.
  //DefMacroI("\sectionname",       undef, "Section");
  //DefMacroI("\subsectionname",    undef, "Subsection");
  //DefMacroI("\subsubsectionname", undef, "Subsubsection");
  //DefMacroI("\paragraphname",     undef, "Paragraph");
  //DefMacroI("\subparagraphname",  undef, "Subparagraph");

  DefMacro!("\\appendixname", "Appendix");
  // These aren"t defined in LaTeX,
  // these definitions will give us more meaningful typerefnum"s
  DefMacro!("\\sectiontyperefname", "\\lx@sectionsign\\lx@ignorehardspaces");
  DefMacro!("\\subsectiontyperefname", "\\lx@sectionsign\\lx@ignorehardspaces");
  DefMacro!("\\subsubsectiontyperefname", "\\lx@sectionsign\\lx@ignorehardspaces");
  DefMacro!("\\paragraphtyperefname", "\\lx@paragraphsign\\lx@ignorehardspaces");
  DefMacro!("\\subparagraphtyperefname", "\\lx@paragraphsign\\lx@ignorehardspaces");
});
