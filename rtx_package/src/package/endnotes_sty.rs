use crate::package::*;

LoadDefinitions!({

  NewCounter!("endnote");
  DefMacro!("\\theendnote",         None, "\\arabic{endnote}");
  DefMacro!("\\endnotetyperefname", None, "endnote");

  // \theenmark  Should be assigned to the mark, by \endnote,\endnotemark !

  // \enotesize
  // \@makeentext to format the text of the endnote; not used (yet)!!!

  // This is NOT correct; it should be edef"d after the counter is stepped...
  DefMacro!("\\theenmark",    "\\theendnote");
  DefMacro!("\\makeenmark",   r"\hbox{\textsuperscript{\normalfont\theenmark}}");
  DefMacro!("\\fnum@endnote", "\\makeenmark");

  DefMacro!("\\ext@endnote", None, "ent");

  DefMacro!("\\endnote",     "\\lx@note{endnote}");
  DefMacro!("\\endnotemark", "\\lx@notemark{endnote}");
  DefMacro!("\\endnotetext", "\\lx@notetext{endnote}");

  // \addtoendnotes{text}
  DefMacro!("\\addtoendnotes{}", "");

  DefMacro!("\\notesname", "Notes");

  // Note: NOT called \printendnotes!
  DefConstructor!(T_CS!("\\theendnotes"), None,
    "<ltx:TOC lists='ent' scope='global' show='refnum > note'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => { stored_map!("name" => stomach::digest(T_CS!("\\notesname"))?) });

});