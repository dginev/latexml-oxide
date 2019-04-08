use crate::package::*;
//**********************************************************************
// Semi-Undocumented stuff
//**********************************************************************

LoadDefinitions!(state, {
  DefMacro!("\\@ifnextchar DefToken {}{}", sub[gullet, args, state] {
    unpack!(args => token, t_if, t_else);
    let token : Token = token.into();
    let next = gullet.read_non_space(state);
    // NOTE: Not actually substituting, but collapsing ## pairs!!!!
    // use \egroup for $next, if we've fallen off end?
    let next_test = next.as_ref().unwrap_or(&T_END!());
    let which = if XEquals!(&token, &next_test) {
      t_if
    } else {
      t_else
    };
    let mut result = which.substitute_parameters(Vec::new()).unlist();
    if let Some(t_next) = next {
      result.push(t_next);
    }
    result
  });
  Let!("\\kernel@ifnextchar", "\\@ifnextchar");
  Let!("\\@ifnext", "\\@ifnextchar"); // ????

  //======================================================================
  // Hair
  DefPrimitive!("\\makeatletter", {
    AssignCatcode!('@', Catcode::LETTER, Some(Scope::Local));
  });
  DefPrimitive!("\\makeatother", {
    AssignCatcode!('@', Catcode::OTHER, Some(Scope::Local));
  });

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
