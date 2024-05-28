use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // Horizontal Mode primitives in Ch.25, pp.285--287

  // The following cause tex to start a new paragraph -- they switch to horizontal mode.
  // <horizontal command> = <letter> | <other> | \char | <chardef token>
  //    | \noboundary | \unhbox | \unhcopy | \valign | \vrule
  //    | \hskip | \hfil | \hfill | \hss | \hfilneg
  //    | \accent | \discretionary | \- | \<space> | $

  DefPrimitive!("\\noboundary", None);


  // Implement ???
  // DefMacro('\vrule','\relax');
  DefMacro!("\\valign", None);

  DefMacro!("\\vspace{}", "\\vskip#1\\relax");
  // \indent, \noindent, \par; see above.

});
