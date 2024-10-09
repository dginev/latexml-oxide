use crate::prelude::*;
LoadDefinitions!( {

LoadPool!("LaTeX");
//**********************************************************************
// Option handling
for option in &["10pt","11pt","12pt",
  "letterpaper", "legalpaper", "executivepaper", "a4paper", "a5paper", "b5paper",
  "landscape",
  "final", "draft",
  "oneside", "twoside",
  "openright", "openany",
  "notitlepage", "titlepage"] {
  DeclareOption!(option, None); }
DeclareOption!("onecolumn",
  r"\@twocolumnfalse\columnwidth\textwidth");
DeclareOption!("twocolumn",
  r"\@twocolumntrue\columnwidth\textwidth\advance\columnwidth-\columnsep\divide\columnwidth2\relax");
// DeclareOption!("openbib", sub {
//     RequireResource(None, type => "text/css", content => ".ltx_bibblock{display:block;}"); });
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

DefMacro!("\\@ptsize", "0");  // should depend on options...
TeX!(r"
\newif\if@restonecol
\newif\if@titlepage
\@titlepagefalse");

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
NewCounter!("footnote",     "chapter");

DefMacro!("\\thepart",          "\\Roman{part}");
DefMacro!("\\thechapter",       "\\arabic{chapter}");
DefMacro!("\\thesection",       "\\thechapter.\\arabic{section}");
DefMacro!("\\thesubsection",    "\\thesection.\\arabic{subsection}");
DefMacro!("\\thesubsubsection", "\\thesubsection.\\arabic{subsubsection}");
DefMacro!("\\theparagraph",     "\\thesubsubsection.\\arabic{paragraph}");
DefMacro!("\\thesubparagraph",  "\\theparagraph.\\arabic{subparagraph}");

DefMacro!("\\chaptermark{}", "");

NewCounter!("equation",       "chapter",  idprefix => "E");
NewCounter!("@equationgroup", "document", idprefix => "EG", idwithin => "section");
NewCounter!("figure",         "chapter",  idprefix => "F");
NewCounter!("table",          "chapter",  idprefix => "T");
DefMacro!("\\theequation", r"\ifnum\c@chapter>\z@\thechapter.\fi \arabic{equation}");
DefMacro!("\\thefigure",   r"\ifnum\c@chapter>\z@\thechapter.\fi \arabic{figure}");
DefMacro!("\\thetable",    r"\ifnum\c@chapter>\z@\thechapter.\fi \arabic{table}");
SetCounter!("tocdepth" => Number::new(2));

DefMacro!("\\theenumi",   "\\arabic{enumi}");
DefMacro!("\\theenumii",  "\\alph{enumii}");
DefMacro!("\\theenumiii", "\\roman{enumiii}");
DefMacro!("\\theenumiv",  "\\Alph{enumiv}");

DefMacro!("\\bibname", "Bibliography");

AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:chapter");

Tag!("ltx:appendix", auto_close => true);
DefMacro!("\\appendix", "\\@appendix");
DefPrimitive!("\\@appendix", { start_appendices("chapter"); });

// General document structure:
// \documentclass{..}
// preamble
// \begin{document}
// \frontmatter
DefPrimitive!("\\frontmatter", { AssignValue!("no_number_sections" => true); });
// frontmatter stuff
// \maketitle
// \include various preface, introduction, etc
// \mainmatter
DefPrimitive!("\\mainmatter", { AssignValue!("no_number_sections" => false); });
// \include various chapters, appendices
// \backmatter
DefPrimitive!("\\backmatter", None);
// commands for bibliography, indices
// \end{document}

});