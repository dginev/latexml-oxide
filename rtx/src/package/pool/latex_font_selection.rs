use crate::package::*;

pub fn load_definitions(outer_state: &mut State) -> Result<()> {
  SetupBindingMacros!(outer_state);

  // TODO
  DefMacro!("\\normalfont", "");

  //======================================================================
  // Hair
  DefPrimitive!("\\makeatletter", sub[stomach, whatsit, state] { state.assign_catcode('@', Catcode::LETTER, Some(Scope::Local)); Ok(vec![]) });
  DefPrimitive!("\\makeatother",  sub[stomach, whatsit, state] { state.assign_catcode('@', Catcode::OTHER, Some(Scope::Local)); Ok(vec![]) });

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
  DefMacro!(
    "\\sectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subsectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subsubsectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\paragraphtyperefname",
    "\\lx@paragraphsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subparagraphtyperefname",
    "\\lx@paragraphsign\\lx@ignorehardspaces"
  );

  // These really should be robust! which is a source of expand timing issues!
  DefMacro!("\\textmd{}",     "\\ifmmode\\textmd@math{#1}\\else{\\mdseries #1}\\fi",       protected => 1);
  DefMacro!("\\textbf{}",     "\\ifmmode\\textbf@math{#1}\\else{\\bfseries #1}\\fi",       protected => 1);
  DefMacro!("\\textrm{}",     "\\ifmmode\\textrm@math{#1}\\else{\\rmfamily #1}\\fi",       protected => 1);
  DefMacro!("\\textsf{}",     "\\ifmmode\\textsf@math{#1}\\else{\\sffamily #1}\\fi",       protected => 1);
  DefMacro!("\\texttt{}",     "\\ifmmode\\texttt@math{#1}\\else{\\ttfamily #1}\\fi",       protected => 1);
  DefMacro!("\\textup{}",     "\\ifmmode\\textup@math{#1}\\else{\\upshape #1}\\fi",        protected => 1);
  DefMacro!("\\textit{}",     "\\ifmmode\\textit@math{#1}\\else{\\itshape #1}\\fi",        protected => 1);
  DefMacro!("\\textsl{}",     "\\ifmmode\\textsl@math{#1}\\else{\\slshape #1}\\fi",        protected => 1);
  DefMacro!("\\textsc{}",     "\\ifmmode\\textsc@math{#1}\\else{\\scshape #1}\\fi",        protected => 1);
  DefMacro!("\\textnormal{}", "\\ifmmode\\textnormal@math{#1}\\else{\\normalfont #1}\\fi", protected => 1);

  // TODO:
  DefMacroI!(T_CS!("\\ttfamily"), None, Tokens!());

  Ok(())
}
