use latexml_package::prelude::*;

LoadDefinitions!({
  // ed.sty — editorial notes. Optional arg is category/author, body
  // is the note text. Preserve the text as ltx:note with the
  // category as `name` attribute so reviewers can read what was
  // typed (content-preserving per [[feedback-content-preserving]]).
  // The prior Perl-matched empty expansion silently dropped author
  // editorial commentary, which is article material worth retaining
  // for the HTML/JATS output even if the original PDF target hid it.
  DefConstructor!("\\ednote[]{}",
    "<ltx:note role='editorial' name='#1'>#2</ltx:note>");
});
