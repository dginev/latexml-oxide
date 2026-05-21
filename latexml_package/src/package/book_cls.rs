use crate::prelude::*;

LoadDefinitions!({
  LoadPool!("LaTeX");
  //**********************************************************************
  // Option handling
  for option in &[
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
    "notitlepage",
    "titlepage",
  ] {
    DeclareOption!(option, None);
  }
  DeclareOption!("onecolumn", r"\@twocolumnfalse\columnwidth\textwidth");
  DeclareOption!(
    "twocolumn",
    r"\@twocolumntrue\columnwidth\textwidth\advance\columnwidth-\columnsep\divide\columnwidth2\relax"
  );
  // Perl book.cls.ltxml L33-34: `openbib` injects inline CSS to render
  // bib blocks as display blocks. Port via require_resource on an
  // anonymous Resource — matches article_cls/report_cls handlers.
  DeclareOption!("openbib", {
    use latexml_core::document::resource::Resource;
    require_resource(Resource {
      mimetype: "text/css".into(),
      content: ".ltx_bibblock{display:block;}".into(),
      ..Resource::default()
    });
  });
  DeclareOption!("leqno", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true); });
  DeclareOption!("fleqn", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_fleqn" => true); });

  ProcessOptions!();

  //**********************************************************************
  // Document structure.
  RelaxNGSchema!("LaTeXML");
  RequireResource!("ltx-book.css");

  // This makes the authors appear on 1 line;
  // for derived classes with multiple lines, map this to undef and add ltx_authors_multiline
  AddToMacro!("\\maketitle", "\\ltx@authors@oneline");

  DefMacro!("\\@ptsize", "0"); // should depend on options...
  DefMacro!("\\@pnumwidth", "1.55em");
  DefMacro!("\\@tocrmarg", "2.55em");
  DefMacro!("\\@dotsep", "4.5");
  DefRegister!("\\abovecaptionskip" => Glue::new(0));
  DefRegister!("\\belowcaptionskip" => Glue::new(0));
  DefRegister!("\\bibindent" => Dimension::new(0));

  TeX!(
    r"
\newif\if@restonecol
\newif\if@titlepage
\@titlepagefalse"
  );

  //**********************************************************************
  // The core sectioning commands are defined in LaTeX.pm
  // but the counter setup, etc, depends on article
  SetCounter!("secnumdepth", Number::new(2));
  NewCounter!("part",          "document",      idprefix => "Pt",  nested => vec!["chapter"]);
  NewCounter!("chapter",       "document",      idprefix => "Ch",  nested => vec!["section"]);
  NewCounter!("section",       "chapter",       idprefix => "S",   nested => vec!["subsection"]);
  NewCounter!("subsection",    "section",       idprefix => "SS",  nested => vec!["subsubsection"]);
  NewCounter!("subsubsection", "subsection",    idprefix => "SSS", nested => vec!["paragraph"]);
  NewCounter!("paragraph",     "subsubsection", idprefix => "P",   nested => vec!["subparagraph"]);
  NewCounter!("subparagraph", "paragraph", idprefix => "SP", nested => vec!["equation", "figure", "table"]);
  NewCounter!("footnote", "chapter");

  DefMacro!("\\thepart", "\\Roman{part}");
  DefMacro!("\\thechapter", "\\arabic{chapter}");
  DefMacro!("\\thesection", "\\thechapter.\\arabic{section}");
  DefMacro!("\\thesubsection", "\\thesection.\\arabic{subsection}");
  DefMacro!(
    "\\thesubsubsection",
    "\\thesubsection.\\arabic{subsubsection}"
  );
  DefMacro!("\\theparagraph", "\\thesubsubsection.\\arabic{paragraph}");
  DefMacro!("\\thesubparagraph", "\\theparagraph.\\arabic{subparagraph}");

  def_macro_noop("\\chaptermark{}")?;

  NewCounter!("equation",       "chapter",  idprefix => "E");
  NewCounter!("@equationgroup", "document", idprefix => "EG", idwithin => "section");
  NewCounter!("figure",         "chapter",  idprefix => "F");
  NewCounter!("table",          "chapter",  idprefix => "T");
  DefMacro!(
    "\\theequation",
    r"\ifnum\c@chapter>\z@\thechapter.\fi \arabic{equation}"
  );
  DefMacro!(
    "\\thefigure",
    r"\ifnum\c@chapter>\z@\thechapter.\fi \arabic{figure}"
  );
  DefMacro!(
    "\\thetable",
    r"\ifnum\c@chapter>\z@\thechapter.\fi \arabic{table}"
  );
  SetCounter!("tocdepth" => Number::new(2));

  DefMacro!("\\theenumi", "\\arabic{enumi}");
  DefMacro!("\\theenumii", "\\alph{enumii}");
  DefMacro!("\\theenumiii", "\\roman{enumiii}");
  DefMacro!("\\theenumiv", "\\Alph{enumiv}");

  DefMacro!("\\bibname", "Bibliography");

  AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:chapter");

  Tag!("ltx:appendix", auto_close => true);
  DefMacro!("\\appendix", "\\@appendix");
  DefPrimitive!("\\@appendix", {
    start_appendices("chapter");
  });

  // General document structure:
  // \documentclass{..}
  // preamble
  // \begin{document}
  // \frontmatter
  DefPrimitive!("\\frontmatter", {
    AssignValue!("no_number_sections" => true);
  });
  // frontmatter stuff
  // \maketitle
  // \include various preface, introduction, etc
  // \mainmatter
  DefPrimitive!("\\mainmatter", {
    AssignValue!("no_number_sections" => false);
  });
  // \include various chapters, appendices
  // \backmatter
  DefPrimitive!("\\backmatter", None);
  // commands for bibliography, indices
  // \end{document}

  DefPrimitive!("\\tiny",         None, font => {size => 5 });
  DefPrimitive!("\\scriptsize",   None, font => {size => 7 });
  DefPrimitive!("\\footnotesize", None, font => {size => 8 });
  DefPrimitive!("\\small",        None, font => {size => 9 });
  DefPrimitive!("\\normalsize",   None, font => {size => 10 });
  DefPrimitive!("\\large",        None, font => {size => 12 });
  DefPrimitive!("\\Large",        None, font => {size => 14.4 });
  DefPrimitive!("\\LARGE",        None, font => {size => 17.28 });
  DefPrimitive!("\\huge",         None, font => {size => 20.74 });
  DefPrimitive!("\\Huge",         None, font => {size => 29.8 });
});
