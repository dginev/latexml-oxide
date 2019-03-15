use crate::package::*;
use rtx_core::state::State;

LoadDefinitions!(stomach, state, {
  LoadPool!("LaTeX");
  //**********************************************************************
  // Option handling
  for option in [
    "10pt",
    "11pt",
    "12pt",
    "letterpaper",
    "legalpaper",
    "executivepaper",
    "a4paper",
    "a5paper",
    "b5paper",
    "landscape",
    "final",
    "draft",
    "oneside",
    "twoside",
    "openright",
    "openany",
    "onecolumn",
    "twocolumn",
    "notitlepage",
    "titlepage",
  ]
  .iter()
  {
    DeclareOption!(option.to_string(), None);
  }

  // TODO:
  // DeclareOption!("openbib",
  // || { RequireResource!(None, type: "text/css", content: ".ltx_bibblock{display:block;}");
  // }); DeclareOption!("leqno", || { state.assign_mapping("DOCUMENT_CLASSES", "ltx_leqno": 1);
  // }); DeclareOption!("fleqn", || { state.assign_mapping("DOCUMENT_CLASSES", "ltx_fleqn": 1);
  // });

  ProcessOptions!(stomach);

  //**********************************************************************
  // Document structure.
  RelaxNGSchema!("LaTeXML");
  RequireResource!("ltx-article.css");

  // This makes the authors appear on 1 line;
  // for derived classes with multiple lines, map this to undef and add ltx_authors_multiline

  AddToMacro!("\\maketitle", "\\ltx@authors@oneline");

  DefMacro!("\\@ptsize", "0"); // should depend on options...
  RawTeX!(
    r###"
  \newif\if@restonecol
  \newif\if@titlepage
  \@titlepagefalse
  "###
  );

  //**********************************************************************
  // The core sectioning commands are defined in LaTeX.pm
  // but the counter setup, etc, depends on article
  SetCounter!("secnumdepth", Number!(3), None);
  NewCounter!("part",          "document",      idprefix => "Pt",  nested => vec!["section"]);
  NewCounter!("section",       "document",      idprefix => "S",   nested => vec!["subsection"]);
  NewCounter!("subsection",    "section",       idprefix => "SS",  nested => vec!["subsubsection"]);
  NewCounter!("subsubsection", "subsection",    idprefix => "SSS", nested => vec!["paragraph"]);
  NewCounter!("paragraph",     "subsubsection", idprefix => "P",   nested => vec!["subparagraph"]);
  NewCounter!("subparagraph", "paragraph", idprefix => "SP", nested => vec!["equation", "figure", "table"]);

  DefMacro!("\\thepart", "\\Roman{part}");
  DefMacro!("\\thesection", "\\arabic{section}");
  DefMacro!("\\thesubsection", "\\thesection.\\arabic{subsection}");
  DefMacro!("\\thesubsubsection", "\\thesubsection.\\arabic{subsubsection}");
  DefMacro!("\\theparagraph", "\\thesubsubsection.\\arabic{paragraph}");
  DefMacro!("\\thesubparagraph", "\\theparagraph.\\arabic{subparagraph}");
  SetCounter!("tocdepth", Number!(3), None);

  NewCounter!("equation",       "document", idprefix => "E",  idwithin => "section");
  NewCounter!("@equationgroup", "document", idprefix => "EG", idwithin => "section");
  NewCounter!("figure",         "document", idprefix => "F",  idwithin => "section");
  NewCounter!("table",          "document", idprefix => "T",  idwithin => "section");

  DefMacro!("\\theequation", "\\arabic{equation}");
  DefMacro!("\\thefigure", "\\arabic{figure}");
  DefMacro!("\\thetable", "\\arabic{table}");

  NewCounter!("enumi",   "", idwithin => "@itemizei", idprefix => "i");
  NewCounter!("enumii",  "", idwithin => "enumi",     idprefix => "i");
  NewCounter!("enumiii", "", idwithin => "enumii",    idprefix => "i");
  NewCounter!("enumiv",  "", idwithin => "enumiii",   idprefix => "i");
  DefMacro!("\\theenumi", "\\arabic{enumi}");
  DefMacro!("\\theenumii", "\\alph{enumii}");
  DefMacro!("\\theenumiii", "\\roman{enumiii}");
  DefMacro!("\\theenumiv", "\\Alph{enumiv}");

  DefMacro!("\\refname", "References");

  Tag!("ltx:appendix", auto_close => true);
  DefMacro!("\\appendix", "\\@appendix");
  // Actually we should be using section counter
  // TODO:
  // DefPrimitive!("\\@appendix", sub { startAppendices("section"); });
});
