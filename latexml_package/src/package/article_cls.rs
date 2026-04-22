use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!( {
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
    DeclareOption!(*option, None);
  }

  DeclareOption!("onecolumn", r"\@twocolumnfalse\columnwidth\textwidth");
  DeclareOption!(
    "twocolumn",
    r"\@twocolumntrue\columnwidth\textwidth\advance\columnwidth-\columnsep\divide\columnwidth2\relax"
  );
  // Perl article.cls.ltxml L34-35: `openbib` injects an inline CSS
  // resource that switches bib blocks from flow to display layout.
  // Port via require_resource on an anonymous Resource (no name, so the
  // `<ltx:resource>` carries only mimetype + inline content).
  DeclareOption!("openbib", {
    use latexml_core::document::resource::Resource;
    require_resource(Resource {
      mimetype: "text/css".into(),
      content:  ".ltx_bibblock{display:block;}".into(),
      ..Resource::default()
    });
  });
  DeclareOption!("leqno", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true); });
  DeclareOption!("fleqn", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_fleqn" => true); });

  ProcessOptions!();

  //**********************************************************************
  // Document structure.
  RelaxNGSchema!("LaTeXML");
  RequireResource!("ltx-article.css");

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

  TeX!(r"
  \newif\if@restonecol
  \newif\if@titlepage
  \@titlepagefalse
  ");

  //**********************************************************************
  // The core sectioning commands are defined in LaTeX.pm
  // but the counter setup, etc, depends on article
  SetCounter!("secnumdepth", Number::new(3));
  NewCounter!("part",          "document",      idprefix => "Pt",  nested => vec!["section"]);
  NewCounter!("section",       "document",      idprefix => "S",   nested => vec!["subsection"]);
  NewCounter!("subsection",    "section",       idprefix => "SS",  nested => vec!["subsubsection"]);
  NewCounter!("subsubsection", "subsection",    idprefix => "SSS", nested => vec!["paragraph"]);
  NewCounter!("paragraph",     "subsubsection", idprefix => "P",   nested => vec!["subparagraph"]);
  NewCounter!("subparagraph", "paragraph", idprefix => "SP", nested => vec!["equation", "figure", "table"]);

  DefMacro!("\\thepart", "\\Roman{part}");
  DefMacro!("\\thesection", "\\arabic{section}");
  DefMacro!("\\thesubsection", "\\thesection.\\arabic{subsection}");
  DefMacro!(
    "\\thesubsubsection",
    "\\thesubsection.\\arabic{subsubsection}"
  );
  DefMacro!("\\theparagraph", "\\thesubsubsection.\\arabic{paragraph}");
  DefMacro!("\\thesubparagraph", "\\theparagraph.\\arabic{subparagraph}");
  SetCounter!("tocdepth", Number::new(3));

  NewCounter!("equation",       "document", idprefix => "E",  idwithin => "section");
  NewCounter!("@equationgroup", "document", idprefix => "EG", idwithin => "section");
  NewCounter!("figure",         "document", idprefix => "F",  idwithin => "section");
  NewCounter!("table",          "document", idprefix => "T",  idwithin => "section");

  DefMacro!("\\theequation", "\\arabic{equation}");
  DefMacro!("\\thefigure", "\\arabic{figure}");
  DefMacro!("\\thetable", "\\arabic{table}");

  DefMacro!("\\theenumi", "\\arabic{enumi}");
  DefMacro!("\\theenumii", "\\alph{enumii}");
  DefMacro!("\\theenumiii", "\\roman{enumiii}");
  DefMacro!("\\theenumiv", "\\Alph{enumiv}");

  DefMacro!("\\refname", "References");

  // Perl L100: article uses section-level bibliography in backmatter.
  // Rust was missing this mapping, so the chapterbib/bibunits/sectionbib
  // tweak chain had an empty baseline to modify.
  AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:section");

  Tag!("ltx:appendix", auto_close => true);
  DefMacro!("\\appendix", "\\@appendix");

  // Actually we should be using section counter
  DefPrimitive!("\\@appendix", { start_appendices("section"); });

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
