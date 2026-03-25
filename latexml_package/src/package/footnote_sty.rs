use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: footnote.sty.ltxml
  // Since we don't have problems with footnotes, there's not much to do here.
  DefMacro!("\\savenotes", None);
  DefMacro!("\\endsavenotes", None);
  DefMacro!("\\spewnotes", None);
  DefMacro!("\\csname minipage*\\endcsname", "\\minipage");
  DefMacro!("\\csname endminipage*\\endcsname", "\\endminipage");
  DefMacro!("\\makesavenoteenv[]{}",
    "\\if.#1.\\else\\newenvironment{#1}{\\begin{#2}}{\\end{#2}}\\fi");
});
