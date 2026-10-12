//! Stub for pnas-new.cls (PNAS journal class).
//!
//! pnas-new.cls is the modern PNAS submission class. It is not in TeXLive;
//! authors bundle it with their paper. The class defines several
//! frontmatter helpers (\leadauthor, \correspondingauthor, \authorcontributions,
//! \significancestatement, \authordeclaration) and a \templatetype{...}
//! command that loads a per-article-type style (pnasresearcharticle.sty,
//! pnasperspective.sty etc.). Since our binding registry routes
//! pnas-new.cls → this binding, we provide gobble + frontmatter-preserve
//! stubs for the author-facing helpers and a \templatetype that defers to
//! \RequirePackage like the upstream does.
//!
//! Witness: 2305.01604 (significant 7-error gap on canvas, all from
//! undefined pnas-new frontmatter macros).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // pnas-new.cls:422 `\newcommand{\doi}[1]{\def\@doi{#1}}` stores the DOI for the page footer
  // (:268, :369-370): a setter taking a normal argument, often `\url{…}`, not OmniBus's `\doi`
  // that reads a DOI verbatim as a url (KNOWN_PERL_ERRORS #326), which printed "\urlwww.pnas…".
  // Witnesses 2605.03599, 2605.07504.
  // The front matter's DOI only: a `.bbl`'s `\doi` (pnas-new.bst:116-121; plainnat's), inside
  // `{thebibliography}`, meets the same setter in pdflatex and prints nothing. A body `\doi` after
  // `\maketitle` is a DOI too (the footer's changes with it); each call adds a note, where the
  // setter keeps the last.
  DefMacro!("\\doi{}", sub[(doi)] {
    let in_bibliography =
      do_expand(Tokens!(T_CS!("\\@currenvir")))?.to_string().trim() == "thebibliography";
    let mut tokens = if in_bibliography {
      vec![T_CS!("\\def"), T_CS!("\\@doi"), T_BEGIN!()]
    } else {
      mouth::tokenize_internal(TeXString::assembled("\\lx@add@pubnote[role=doi]{".to_string())).unlist()
    };
    tokens.extend(doi.unlist());
    tokens.push(T_END!());
    Ok(Tokens::new(tokens))
  });
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("graphicx");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("authblk");
  RequirePackage!("natbib");
  // pnas-new.cls L97 `\RequirePackage{xifthen}`, then L98-118 declares the
  // four booleans its own style files switch. Without them the FIRST line of
  // `pnasresearcharticle.sty` — `\setboolean{shortarticle}{true}` at L7 —
  // hits `Error:undefined:\setboolean` and is defined as `<ltx:ERROR/>`;
  // the article-type style then unravels and the document loses its tail,
  // bibliography included. 6 papers in the 2605+2606 bibliography-absence
  // cohort, all `\documentclass{pnas-new}`: 2606.02411, 2606.13799,
  // 2605.03599, 2605.07504, 2606.26469, 2606.29674.
  //
  // `ifthen` is raw-loaded from texmf (`ifthen_sty.rs` mirrors Perl's
  // `noltxml` passthrough), which is what supplies `\newboolean`,
  // `\setboolean` and `\ifthenelse`. The class ships with the paper and is
  // absent from TeX Live, so nothing else declares these.
  RequirePackage!("ifthen");
  RawTeX!(
    r"\newboolean{shortarticle}%
\newboolean{singlecolumn}%
\newboolean{displaywatermark}%
\newboolean{displaycopyright}%
\setboolean{displaywatermark}{true}%
\setboolean{displaycopyright}{false}"
  );
  // pnas-new.cls L17+ requires extarticle, lmodern, helvet, fontenc,
  // lettrine, ifpdf, ifxetex, tikz, mdframed, draftwatermark, textcomp,
  // colortbl, booktabs, algorithm, algpseudocode, changepage, geometry,
  // caption, etoolbox, fancyhdr, lastpage, titlesec, lineno, footmisc,
  // enumitem, sidecap, float, stfloats, marginnote.
  // Most of those are noop-loaded by OmniBus already; bring in the
  // critical ones explicitly so author content (booktabs rules, etc.) works.
  RequirePackage!("booktabs");
  RequirePackage!("fancyhdr");
  RequirePackage!("etoolbox");
  RequirePackage!("titlesec");
  RequirePackage!("caption");
  RequirePackage!("enumitem");
  // pnasresearcharticle.sty uses \afterpage and \mdfdefinestyle.
  RequirePackage!("afterpage");
  // pnas-new.cls:53-55 loads xcolor, tikz and `[framemethod=tikz]{mdframed}`; the option loads tikz (and xcolor)
  // in the mdframed binding, so a pnas document draws its `{tikzpicture}` without loading tikz itself.
  RequirePackage!("mdframed", options => vec!["framemethod=tikz".to_string()]);

  // pnas-new.cls L272: \newcommand{\leadauthor}[1]{\def\@leadauthor{#1}}
  // Preserve as frontmatter author-list note.
  DefMacro!(
    "\\leadauthor{}",
    "\\@add@frontmatter{ltx:note}[role=lead-author]{#1}"
  );
  // pnas-new.cls L275: \newcommand{\authorcontributions}[1]{\def\@authorcontributions{#1}}
  DefMacro!(
    "\\authorcontributions{}",
    "\\@add@frontmatter{ltx:note}[role=author-contributions]{#1}"
  );
  // pnas-new.cls L276: \newcommand{\authordeclaration}[1]{\def\@authordeclaration{#1}}
  DefMacro!(
    "\\authordeclaration{}",
    "\\@add@frontmatter{ltx:note}[role=author-declaration]{#1}"
  );
  // pnas-new.cls L278: \newcommand{\correspondingauthor}[1]{\def\@correspondingauthor{#1}}
  // Email content may contain `_` — Semiverbatim neutralizes catcode-8
  // chars so the note renders cleanly in horizontal mode.
  DefMacro!(
    "\\correspondingauthor Semiverbatim",
    "\\@add@frontmatter{ltx:note}[role=corresponding-author]{#1}"
  );
  // pnas-new.cls L279: \newcommand{\significancestatement}[1]{\def\@significancestatement{#1}}
  DefMacro!(
    "\\significancestatement{}",
    "\\@add@frontmatter{ltx:abstract}[role=significance]{#1}"
  );
  // pnas-new.cls: \templatetype{pkg} → \RequirePackage{pkg}. Defer to
  // \RequirePackage so per-article style packages (pnasresearcharticle.sty,
  // pnasperspective.sty etc.) load correctly.
  DefMacro!("\\templatetype{}", "\\RequirePackage{#1}");
  // pnas-new.cls uses \newif\ifshortarticle for layout switching. Provide
  // the conditional so authors can call \shortarticletrue/\shortarticlefalse.
  DefConditional!("\\ifshortarticle");
  // pnas-new.cls L362: \newif\ifsinglecolumn — single/double column layout.
  DefConditional!("\\ifsinglecolumn");
  // pnas-new.cls L269: \newcommand{\additionalelement}[1]{\def\@additionalelement{#1}}
  // Used by pnasresearcharticle.sty (which loads BEFORE pnas-new.cls
  // completes via \templatetype). Gobble cleanly.
  def_macro_noop("\\additionalelement{}")?;
  // pnas-new.cls L299: \newcommand{\abscontent}{...} — abstract-content
  // renderer that pnasresearcharticle.sty patches via \patchcmd. Provide
  // a noop so the patch silently fails without error.
  def_macro_noop("\\abscontent")?;
  def_macro_noop("\\abscontentformatted")?;

  // Other commonly used pnas-new frontmatter that this stub should preserve:
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  // \equalauthors / \etalauthors — joint-authorship markers.
  def_macro_noop("\\equalauthors{}")?;
  def_macro_noop("\\etalauthors{}")?;
  // \pnasbreak — content-level command, no rendering effect.
  def_macro_noop("\\pnasbreak")?;
  // pnas-new.cls:173 `\RequirePackage{jabbrv}`, the journal-abbreviation package that ships with the class (not in
  // TeX Live), whose `\JournalTitle{…}` the class's pnas-new.bst writes into every `.bbl` entry (witnesses 2304.12786,
  // 2303.10311, 2402.03593). Loaded when it is there; otherwise `\JournalTitle` prints the title as given.
  RawTeX!(
    r"\IfFileExists{jabbrv.sty}{\RequirePackage{jabbrv}}{\providecommand\JournalTitle[1]{#1}}"
  );
  // pnas-new.cls:270 `\dates{…}` stores the received/compiled line the title block prints under the authors
  // (:332, :342 `{\datesfont\@dates}`): a frontmatter date. Witnesses 2304.12786, 2402.03593, 2507.04951,
  // 2002.07301, 2303.10311, 1612.08995, 1708.03990 (`\dates` undefined, the first error).
  DefMacro!("\\dates{}", "\\@add@frontmatter{ltx:date}[role=dates]{#1}");
  // pnas-new.cls:284 `\dropcap{T}` is the first letter of the text, set as a dropped capital (lettrine): the letter.
  DefMacro!("\\dropcap{}", "#1");
  // pnas-new.cls:273 `\etal{…}` stores the running head's author line (page furniture), :321
  // `\verticaladjustment{len}` a title-block skip.
  def_macro_noop("\\etal{}")?;
  def_macro_noop("\\verticaladjustment{}")?;
  // pnas-new.cls:280-281 and :402-428: `\matmethods{…}`/`\acknow{…}` store the Materials and Methods and the
  // acknowledgments text, which `\showmatmethods`/`\showacknow` print where the document calls them.
  RawTeX!(
    r"\newcommand{\matmethods}[1]{\def\@matmethods{#1}}
\newcommand{\acknow}[1]{\def\@acknow{#1}}
\newcommand{\showmatmethods}{\@ifundefined{@matmethods}{}{\section*{Materials and Methods}\@matmethods}}
\newcommand{\showacknow}{\@ifundefined{@acknow}{}{\begin{acknowledgements}\@acknow\end{acknowledgements}}}
\newcommand\numberthis{\addtocounter{equation}{1}\tag{\theequation}}
\newcommand{\addtabletext}[1]{{\small#1}}
\newcommand{\copyrightstatement}{\, \textcopyright\, 2015 by The National Academy of Sciences of the USA}
\newcommand{\headerfont}{}\newcommand{\footerfont}{}\newcommand{\titlefont}{}\newcommand{\dropcapfont}{}
\newcommand{\datesfont}{}\newcommand{\absfont}{}\newcommand{\keywordsfont}{}\newcommand{\copyrightfont}{}"
  );
  // pnas-new.cls:441-450 `{widetext}`, a double-column float in print: its content in place.
  DefEnvironment!("{widetext}", "#body");
});
