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
  // `\input{fleqn.clo}` (fleqn_sty.rs `fleqn_clo`).
  DeclareOption!("fleqn", sub { fleqn_sty::fleqn_clo()?; });

  // article.cls:84-86: two-sided only by option (the kernel's `\@twosidefalse` is the default;
  // KNOWN_PERL_ERRORS #469: Perl's options are no-ops, as book's were for hep-ph0207204).
  DeclareOption!("oneside", r"\@twosidefalse\@mparswitchfalse");
  DeclareOption!("twoside", r"\@twosidetrue\@mparswitchtrue");
  ProcessOptions!();

  //**********************************************************************
  // Document structure.
  RelaxNGSchema!("LaTeXML");
  RequireResource!("ltx-article.css");

  // This makes the authors appear on 1 line;
  // for derived classes with multiple lines, map this to undef and add ltx_authors_multiline

  AddToMacro!("\\lx@maketitle@body", "\\ltx@authors@oneline");

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
  // article.cls:457/469 (report.cls, book.cls alike) define `\fnum@figure`/`\fnum@table`; letter.cls, without floats,
  // has none, but a letter document gets these through the OmniBus fallback (OD #579). Perl's classes define neither
  // and compose the tag in `\lx@@fnum@@`. Code that takes their current body needs one: babel's ini caption templates
  // (babel.sty:2959-2965, `\gdef\fnum@table{…{\[fnum@table]}…}`) captured the undefined token itself, so the new
  // `\fnum@table` called itself and the caption's `\refstepcounter` looped (2605.29738, Lithuanian on demand; Perl
  // loops too, KNOWN_PERL_ERRORS #654). Each is the tag LaTeXML composes, so a caption reads as before
  // (OXIDIZED_DESIGN_DIVERGENCES #579).
  DefMacro!("\\fnum@figure", "\\lx@@fnum@@{figure}");
  DefMacro!("\\fnum@table", "\\lx@@fnum@@{table}");

  DefMacro!("\\theenumi", "\\arabic{enumi}");
  DefMacro!("\\theenumii", "\\alph{enumii}");
  DefMacro!("\\theenumiii", "\\roman{enumiii}");
  DefMacro!("\\theenumiv", "\\Alph{enumiv}");

  // The class's names (article.cls:613-622), in its own load as the real class has them, with the values LaTeXML's
  // kernel predefines (Perl assigns only article.cls.ltxml:98 `\refname` here, the rest in
  // latex_constructs.pool.ltxml:719-722, "these actually should be in article/report/book"): a stand-in's copy that
  // `\LoadClass`es article finds them defined, so its `\newcommand` of one is LaTeX's "already defined"
  // (`state::ClassLoadFrame`, OXIDIZED_DESIGN_DIVERGENCES #732).
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename", "List of Tables");
  DefMacro!("\\refname", "References");
  DefMacro!("\\indexname", "Index");
  DefMacro!("\\figurename", "Figure");
  DefMacro!("\\tablename", "Table");
  DefMacro!("\\partname", "Part");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\abstractname", "Abstract");

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
  // size10.clo:54 (every `size*.clo`): `\normalsize` is EXECUTED right after
  // the size commands are defined, which is what sets `\@currsize` to the size
  // command itself (latex.ltx:14107 `\let\@currsize#1` via `\@setfontsize`;
  // our size switch lets it in dialect.rs). `\ifx\@currsize\normalsize` then
  // holds from the start (ltugboat's `\SMC` cascade, latex-doc-ptr; Gemini L5).
  Digest!("\\normalsize")?;


  // article.cls `\let\@openbib@code\@empty` (article.cls:585) — the default
  // the `openbib` option overrides; a copied `thebibliography` body reads it
  // (mciteplus_doc; Perl's class binding lacks the default too).
  RawTeX!(r"\let\@openbib@code\@empty");

  // article.cls:131-168: the `headings` and `myheadings` page styles. Running heads are page furniture, so
  // `\markboth`/`\markright` keep nothing (sect05.rs), but a document or a raw style calls the styles by name
  // (`\ps@headings` in 1004.5433's Mhdr.sty and 1301.4585; `\let\ps@plain\ps@myheadings`, adassconf.sty:534 in
  // 0711.0180, 0901.3729, 0906.2186) and `\pagestyle{headings}` runs them, as in LaTeX. Perl's binding defines
  // neither (KNOWN_PERL_ERRORS #760).
  RawTeX!(r"\if@twoside
  \def\ps@headings{%
      \let\@oddfoot\@empty\let\@evenfoot\@empty
      \def\@evenhead{\thepage\hfil\slshape\leftmark}%
      \def\@oddhead{{\slshape\rightmark}\hfil\thepage}%
      \let\@mkboth\markboth
    \def\sectionmark##1{%
      \markboth {\MakeUppercase{%
        \ifnum \c@secnumdepth >\z@
          \thesection\quad
        \fi
        ##1}}{}}%
    \def\subsectionmark##1{%
      \markright {%
        \ifnum \c@secnumdepth >\@ne
          \thesubsection\quad
        \fi
        ##1}}}
\else
  \def\ps@headings{%
    \let\@oddfoot\@empty
    \def\@oddhead{{\slshape\rightmark}\hfil\thepage}%
    \let\@mkboth\markboth
    \def\sectionmark##1{%
      \markright {\MakeUppercase{%
        \ifnum \c@secnumdepth >\m@ne
          \thesection\quad
        \fi
        ##1}}}}
\fi
\def\ps@myheadings{%
    \let\@oddfoot\@empty\let\@evenfoot\@empty
    \def\@evenhead{\thepage\hfil\slshape\leftmark}%
    \def\@oddhead{{\slshape\rightmark}\hfil\thepage}%
    \let\@mkboth\@gobbletwo
    \let\sectionmark\@gobble
    \let\subsectionmark\@gobble
    }");
  // article.cls:480-489: the caption box. The binding's `\caption` builds `ltx:caption` itself and never calls it,
  // but a raw package setting its own captions does — xtab.sty:84/121/132 `\@makecaption{\@nameuse{fnum@#1}}{…}`
  // (1010.5412), captcont.sty (1206.7007) — and documents save and wrap it (`\let\old@makecaption\@makecaption`;
  // 50 papers in all, 1011.6187). Its `Table 1: text` then stands in the text, as LaTeX sets it. Perl's binding
  // lacks it too (KNOWN_PERL_ERRORS #760).
  RawTeX!(r"\long\def\@makecaption#1#2{%
  \vskip\abovecaptionskip
  \sbox\@tempboxa{#1: #2}%
  \ifdim \wd\@tempboxa >\hsize
    #1: #2\par
  \else
    \global \@minipagefalse
    \hb@xt@\hsize{\hfil\box\@tempboxa\hfil}%
  \fi
  \vskip\belowcaptionskip}");
});
