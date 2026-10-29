use crate::prelude::*;

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own):
/// book: its title page (`titlepage`, its default) prints the notes in the body's arabic numbers; without one
/// (`notitlepage`) the kernel declares the title hook (`declare_fnsymbol_title_block`), which wins
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "book",
  binding: "book.cls",
  title_marks: FmTitleMarks::Body,
  cite: "book.cls:217 (OD #851, `declare_fnsymbol_title_block`)",
  ..FM_KERNEL_CONVENTIONS
};

LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
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
  // `\input{fleqn.clo}` (fleqn_sty.rs `fleqn_clo`).
  DeclareOption!("fleqn", sub { fleqn_sty::fleqn_clo()?; });
  // Kernel contract (book.cls L52/L98/L119): `\newif\if@openright`, default
  // TRUE via \ExecuteOptions{...,openright}. Neither Perl's book.cls.ltxml
  // nor report.cls.ltxml declare the switch, but derived classes and docs
  // poke it directly (toptesi.sty L329-342 front/main/backmatter; the
  // amscls-doc Author_Handbooks read it at body L49/L60/L170) — the
  // sweep-11/12 `\if@openright` cluster. Same `\if@mainmatter` precedent
  // as commit dba2a7eab0. Declared BEFORE ProcessOptions so the class
  // options actually run.
  RawTeX!(r"\newif\if@openright \@openrighttrue");
  DeclareOption!("openright", r"\@openrighttrue");
  DeclareOption!("openany", r"\@openrightfalse");
  // book.cls:86-88 and :119 `\ExecuteOptions{…,twoside,…}`: a book is two-sided by default, and a class or document
  // branching on `\if@twoside` takes that branch (hep-ph0207204's own `\if@twoside` `\ps@headings`; Perl's no-op
  // options left the kernel's `\@twosidefalse`, latex_base.rs).
  DeclareOption!("oneside", r"\@twosidefalse\@mparswitchfalse");
  DeclareOption!("twoside", r"\@twosidetrue\@mparswitchtrue");
  RawTeX!(r"\@twosidetrue\@mparswitchtrue");

  ProcessOptions!();
  // The page a float's panels are set in rows against (the panel grid rule, `declare_panel_page`): book.cls's in two
  // columns, as its size file sets it for the paper and size options (`standard_class_twocolumn_page`: 229.5pt
  // columns of a 469pt text on letter paper, 221/452pt on A4), when book is the document's class (review rev79c2
  // finding 1: a `figure*` stated 2 columns where the PDF gives 2.044). One column is the registers' page.
  declare_standard_class_page("book");
  // book.cls L339-348: \@endpart, called by \@part/\@spart — raw classes built on
  // this class invoke it directly (toptesi.sty L448; witness toptesi-example
  // L265 \part). The two-sided blank verso (`\null\thispagestyle{empty}\newpage`)
  // is page furniture: it still counts a page, but its `\null` box left an empty
  // paragraph (hpsdiss, nddiss2e, turabian-thesis, suftesi, bookest, nostarch).
  RawTeX!(
    r"\def\@endpart{\vfil\newpage
  \if@twoside\if@openright\thispagestyle{empty}\newpage\fi\fi
  \if@tempswa\twocolumn\fi}"
  );

  //**********************************************************************
  // Document structure.
  RelaxNGSchema!("LaTeXML");
  RequireResource!("ltx-book.css");

  // This makes the authors appear on 1 line;
  // for derived classes with multiple lines, map this to undef and add ltx_authors_multiline
  AddToMacro!("\\lx@maketitle@body", "\\ltx@authors@oneline");
  // book.cls's title block: `\@fnsymbol` marks only under `notitlepage` (`titlepage` is the default), the footnote
  // counter reset after it, when book is the document's class.
  engine::latex_constructs::declare_fnsymbol_title_block("book", Some(true))?;

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
\@titlepagetrue"
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
  // `\@makeschapterhead{title}` typesets an unnumbered chapter head (book.cls:396-405);
  // classes and packages call it directly for their own starred heads — a
  // renewed `theindex` (ryethesis.cls:481-490 `\begin{multicols}{2}[\@makeschapterhead
  // {\indexname}…]`), toc/lof renewals, apacite, sectsty. It is `\@schapter`'s
  // dispatch (latex_constructs_rust_only.rs), as the class's `\@schapter` is this
  // head plus `\@afterheading`; article.cls has none (texilikechaps.sty:89 tests
  // `\@ifundefined{@makeschapterhead}` to detect a chapter class).
  DefMacro!(
    "\\@makeschapterhead{}",
    "\\@startsection{chapter}{0}{}{}{}{}*{#1}"
  );

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
  // article.cls:457/469 (report.cls, book.cls alike) define `\fnum@figure`/`\fnum@table`; letter.cls, without floats,
  // has none, but a letter document gets these through the OmniBus fallback (OD #579). Perl's classes define neither
  // and compose the tag in `\lx@@fnum@@`. Code that takes their current body needs one: babel's ini caption templates
  // (babel.sty:2959-2965, `\gdef\fnum@table{…{\[fnum@table]}…}`) captured the undefined token itself, so the new
  // `\fnum@table` called itself and the caption's `\refstepcounter` looped (2605.29738, Lithuanian on demand; Perl
  // loops too, KNOWN_PERL_ERRORS #654). Each is the tag LaTeXML composes, so a caption reads as before
  // (OXIDIZED_DESIGN_DIVERGENCES #579).
  DefMacro!("\\fnum@figure", "\\lx@@fnum@@{figure}");
  DefMacro!("\\fnum@table", "\\lx@@fnum@@{table}");
  SetCounter!("tocdepth" => Number::new(2));

  DefMacro!("\\theenumi", "\\arabic{enumi}");
  DefMacro!("\\theenumii", "\\alph{enumii}");
  DefMacro!("\\theenumiii", "\\roman{enumiii}");
  DefMacro!("\\theenumiv", "\\Alph{enumiv}");

  // The class's names (book.cls:718-727), in its own load as the real class has them, with the values LaTeXML's kernel
  // predefines (Perl assigns only book.cls.ltxml:100 `\bibname` here, the rest in latex_constructs.pool.ltxml:719-722,
  // "these actually should be in article/report/book"): a stand-in's copy that `\LoadClass`es book finds them defined,
  // so its `\newcommand` of one is LaTeX's "already defined" (`state::ClassLoadFrame`, OXIDIZED_DESIGN_DIVERGENCES
  // #732).
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename", "List of Tables");
  DefMacro!("\\bibname", "Bibliography");
  DefMacro!("\\indexname", "Index");
  DefMacro!("\\figurename", "Figure");
  DefMacro!("\\tablename", "Table");
  DefMacro!("\\partname", "Part");
  DefMacro!("\\chaptername", "Chapter");
  DefMacro!("\\appendixname", "Appendix");

  AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:chapter");

  Tag!("ltx:appendix", auto_close => true);
  // book.cls:520-524 `\appendix` also `\setcounter{section}{0}` (Perl's book.cls.ltxml leaves it): a back-matter
  // chapter steps no chapter, so its sections' count ran on into the appendix (".3" for pdflatex's ".1"), and each
  // `\appendix` resets it, a second one too (`\@appendix` is `\relax` once the appendices start). svmult's main-matter
  // `\appendix` is a `\par` that resets nothing (svmult_cls.rs). Guards `perfect_kernel_batch78a::{
  // book_appendix_after_backmatter_restarts_sections, book_second_appendix_restarts_sections}`.
  DefMacro!(
    "\\appendix",
    "\\@appendix\\global\\c@section\\z@\\gdef\\@chapapp{\\appendixname}"
  );
  DefPrimitive!("\\@appendix", {
    start_appendices("chapter");
  });
  // book.cls:283 `\newcommand\@chapapp{\chaptername}` (`\appendix` re-points it to
  // `\appendixname`): classes built on this one print it in their headings
  // (utexasthesis; class census 2026-09-24).
  DefMacro!("\\@chapapp", "\\chaptername");

  // General document structure:
  // \documentclass{..}
  // preamble
  // \begin{document}
  // \frontmatter
  DefPrimitive!("\\frontmatter", {
    AssignValue!("no_number_sections" => true);
    AssignValue!("no_number_chapters" => false);
  });
  // frontmatter stuff
  // \maketitle
  // \include various preface, introduction, etc
  // \mainmatter
  DefPrimitive!("\\mainmatter", {
    AssignValue!("no_number_sections" => false);
    AssignValue!("no_number_chapters" => false);
  });
  // \include various chapters, appendices
  // \backmatter
  // book.cls:296-299 `\@mainmatterfalse`, by which `\@chapter` (:355-366) and `\@makechapterhead` (:376-385) leave a
  // back-matter chapter unnumbered and its counter alone; its sections keep numbering under the last main chapter
  // ("1.2"), so only the chapters lose their numbers (`no_number_chapters`, sect04.rs), not every level
  // (`no_number_sections`; scrbook's back-matter appendix precedent, koma_script.rs `koma_book_matter`). Perl's is a
  // no-op (book.cls.ltxml:121; KNOWN_PERL_ERRORS #1016): "Chapter 2 Afterword". Guard
  // `perfect_kernel_batch78a::book_backmatter_chapters_are_unnumbered`.
  DefPrimitive!("\\backmatter", {
    AssignValue!("no_number_sections" => false);
    AssignValue!("no_number_chapters" => true);
  });
  // commands for bibliography, indices
  // \end{document}

  // Kernel contract (real book.cls L~380): `\newif\if@mainmatter`, default
  // true; `\frontmatter`/`\backmatter` set it false, `\mainmatter` true.
  // Neither Perl's book.cls.ltxml nor this binding established the
  // conditional, so book-family raw classes/docs poking it directly
  // (`\@mainmatterfalse` in amsdtx/amsldoc, tufte-book, ryethesis — 4 TL doc
  // bundles) errored `undefined`. Wrap the number-toggling primitives above
  // so the flag tracks the matter, exactly as real book.cls couples them.
  RawTeX!(r"\newif\if@mainmatter \@mainmattertrue");
  Let!("\\lx@book@frontmatter", "\\frontmatter");
  Let!("\\lx@book@mainmatter", "\\mainmatter");
  Let!("\\lx@book@backmatter", "\\backmatter");
  // book.cls:284-291 also switch the page numbering (roman, then arabic), which `\thepage` shows.
  DefMacro!(
    "\\frontmatter",
    "\\lx@book@frontmatter\\@mainmatterfalse\\pagenumbering{roman}"
  );
  DefMacro!(
    "\\mainmatter",
    "\\lx@book@mainmatter\\@mainmattertrue\\pagenumbering{arabic}"
  );
  DefMacro!("\\backmatter", "\\lx@book@backmatter\\@mainmatterfalse");

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
  // book.cls `\let\@openbib@code\@empty` (article.cls:585) — the default
  // the `openbib` option overrides; a copied `thebibliography` body reads it
  // (mciteplus_doc; Perl's class binding lacks the default too).
  RawTeX!(r"\let\@openbib@code\@empty");
});
