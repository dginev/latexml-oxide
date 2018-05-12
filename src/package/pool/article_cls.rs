use package::*;
use rtx_core::state::State;
pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);
  LoadPool!("LaTeX");
  //**********************************************************************
  // Option handling
  for _option in [
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
  ].into_iter()
    .map(|s| s.to_string())
  {
    // DeclareOption!(option, None);
  }

  // DeclareOption!("openbib",
  // || { RequireResource!(None, type: "text/css", content: ".ltx_bibblock{display:block;}");
  // }); DeclareOption!("leqno", || { state.assign_mapping("DOCUMENT_CLASSES", "ltx_leqno": 1);
  // }); DeclareOption!("fleqn", || { state.assign_mapping("DOCUMENT_CLASSES", "ltx_fleqn": 1);
  // });

  // ProcessOptions!();

  //**********************************************************************
  // Document structure.
  RelaxNGSchema!("LaTeXML");
  RequireResource!("ltx-article.css");

  //**********************************************************************
  // The core sectioning commands are defined in LaTeX.pm
  // but the counter setup, etc, depends on article
  SetCounter!("secnumdepth", Number!(3), None);
  // NewCounter!("part",          "document",      idprefix => "Pt",  nested => ["section"]);
  // NewCounter("section",       "document",      idprefix => "S",   nested => ["subsection"]);
  // NewCounter("subsection",    "section",       idprefix => "SS",  nested => ["subsubsection"]);
  // NewCounter("subsubsection", "subsection",    idprefix => "SSS", nested => ["paragraph"]);
  // NewCounter("paragraph",     "subsubsection", idprefix => "P",   nested => ["subparagraph"]);
  // NewCounter("subparagraph", "paragraph", idprefix => "SP", nested => ["equation", "figure",
  // "table"]);

  // DefMacro("\thepart",          "\Roman{part}");
  // DefMacro("\thesection",       "\arabic{section}");
  // DefMacro("\thesubsection",    "\thesection.\arabic{subsection}");
  // DefMacro("\thesubsubsection", "\thesubsection.\arabic{subsubsection}");
  // DefMacro("\theparagraph",     "\thesubsubsection.\arabic{paragraph}");
  // DefMacro("\thesubparagraph",  "\theparagraph.\arabic{subparagraph}");
  // SetCounter(tocdepth => Number(3));

  // NewCounter("equation",       "document", idprefix => "E",  idwithin => "section");
  // NewCounter("@equationgroup", "document", idprefix => "EG", idwithin => "section");
  // NewCounter("figure",         "document", idprefix => "F",  idwithin => "section");
  // NewCounter("table",          "document", idprefix => "T",  idwithin => "section");

  // DefMacro("\theequation", "\arabic{equation}");
  // DefMacro("\thefigure",   "\arabic{figure}");
  // DefMacro("\thetable",    "\arabic{table}");

  // NewCounter("enumi",   undef, idwithin => "@itemizei", idprefix => "i");
  // NewCounter("enumii",  undef, idwithin => "enumi",     idprefix => "i");
  // NewCounter("enumiii", undef, idwithin => "enumii",    idprefix => "i");
  // NewCounter("enumiv",  undef, idwithin => "enumiii",   idprefix => "i");
  // DefMacro("\theenumi",   "\arabic{enumi}");
  // DefMacro("\theenumii",  "\alph{enumii}");
  // DefMacro("\theenumiii", "\roman{enumiii}");
  // DefMacro("\theenumiv",  "\Alph{enumiv}");

  // DefMacro("\refname", "References");

  // Tag("ltx:appendix", autoClose => 1);
  // DefMacro("\appendix", "\@appendix");
  // # Actually we should be using section counter
  // DefPrimitive("\@appendix", sub { startAppendices("section"); });

  Ok(())
}
