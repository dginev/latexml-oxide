use crate::{engine::latex_constructs::toc_select, prelude::*};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tocbibind.sty.ltxml defines nothing. tocbibind.sty's state is read by packages built around it — tocloft's
  // `\tableofcontents` tests `\if@dotoctoc` then `\if@bibchapter` (tocloft.sty:105-114) — so the conditionals are
  // tocbibind.sty's (:38-69): each `\if@doto…` true unless its `not…` option clears it, `\if@bibchapter` true when the
  // class has chapters. Without `\if@bibchapter` a skipped branch paired its `\else`/`\fi` with the outer conditional
  // (a stray `\fi`: SciPost.cls loads tocloft raw and this binding; 1811.09408, 2105.01655, 2203.11601). Witnesses
  // 2408.01486 (SciPost probing `\if@dotoctoc`), 2003.02382 (`\if@dotoclof`/`\if@dotoclot` under `[nottoc]`).
  Digest!(r"\newcommand{\@bibquit}{}\newif\if@bibchapter
\@ifundefined{chapter}{\@bibchapterfalse}{\@bibchaptertrue}
\newif\if@inltxdoc
\@ifclassloaded{ltxdoc}{\@inltxdoctrue}{\@inltxdocfalse}
\newif\if@dotocbib\@dotocbibtrue
\newif\if@dotocind\@dotocindtrue
\newif\if@dotoctoc\@dotoctoctrue
\newif\if@dotoclot\@dotoclottrue
\newif\if@dotoclof\@dotocloftrue
\newif\if@donumbib\@donumbibfalse
\newif\if@donumindex\@donumindexfalse
\newif\if@dot@cb@bsection\@dot@cb@bsectionfalse")?;
  DeclareOption!("section", { Digest!(r"\@bibchapterfalse")?; });
  DeclareOption!("notbib", { Digest!(r"\@dotocbibfalse")?; });
  DeclareOption!("notindex", { Digest!(r"\@dotocindfalse")?; });
  DeclareOption!("nottoc", { Digest!(r"\@dotoctocfalse")?; });
  DeclareOption!("notlot", { Digest!(r"\@dotoclotfalse")?; });
  DeclareOption!("notlof", { Digest!(r"\@dotocloffalse")?; });
  DeclareOption!("numbib", { Digest!(r"\@donumbibtrue")?; });
  DeclareOption!("numindex", { Digest!(r"\@donumindextrue")?; });
  // `chapter` only warns when the class has no chapters (tocbibind.sty:75-78); `other` and `none` (:80-90).
  DeclareOption!("chapter", None);
  DeclareOption!("other", { Digest!(r"\@bibchapterfalse")?; });
  DeclareOption!("none", {
    Digest!(r"\@dotocbibfalse\@dotocindfalse\@dotoctocfalse\@dotoclotfalse\@dotocloffalse\@donumbibfalse\@donumindexfalse")?;
  });

  ProcessOptions!();

  // The user commands and the heading internals, as tocbibind.sty:99-152 and :250-316 define them (Perl's binding
  // defines none: KNOWN_PERL_ERRORS #800). The names are the headings the paper sets: `\settocbibname` (46 papers;
  // 0710.3210, 1106.3946, 1401.4404) renames the bibliography, `\settocname` the contents (2010.07703, 2012.02570),
  // `\setlofname`/`\setlotname`/`\setindexname` the other lists (0710.3210); `\tocfile{<title>}{<ext>}` (:263-271)
  // is a list of `<ext>` headed `<title>` — `\tocfile{\listalgorithmcfname}{loa}` (1712.01817, 2409.09967),
  // `\tocsection\tocfile{\listfigurename}{lof}` (1605.05950, 2204.09786); etoc's tocbibind style calls `\toc@start`,
  // `\toc@section{\@tocextra}{\contentsname}` and `\toc@finish` (etoc.sty:1784-1812; 2412.08175).
  //
  // `\tocbibname` (:115-123) names `\bibname` when the class defines it, else `\refname`. LaTeXML's kernel defines a
  // `\bibname` default for every class (sect11.rs) and natbib's binding a `\refname` (natbib_sty.rs), so the class's
  // own name is told by its sectioning, as the standard classes pair them: a class with chapters names its
  // bibliography `\bibname` (book, report: pdflatex "Bibliography"), one without `\refname` (article: "References";
  // OXIDIZED_DESIGN_DIVERGENCES #620).
  RawTeX!(r"\if@inltxdoc \@dotocindfalse \fi
\newcommand{\@tocextra}{section}
\newcommand{\tocotherhead}[1]{\renewcommand{\@tocextra}{#1}}
\newcommand{\tocetcmark}[1]{\@mkboth{\MakeUppercase{#1}}{\MakeUppercase{#1}}}
\newcommand{\prw@mkboth}[1]{\tocetcmark{#1}}
\newcommand{\toc@section}[2]{\@nameuse{#1}*{#2\prw@mkboth{#2}}\addcontentsline{toc}{#1}{#2}}
\newcommand{\toc@headstar}[2]{\@nameuse{#1}*{{#2}}}
\newcommand{\toc@chapter}[1]{\chapter*{#1}\prw@mkboth{#1}\addcontentsline{toc}{chapter}{#1}}
\newcommand{\tocbibname}{\@ifundefined{chapter}{\@ifundefined{refname}{\bibname}{\refname}}{\bibname}}
\providecommand{\indexname}{Index}
\newcommand{\setindexname}[1]{\renewcommand{\indexname}{#1}}
\providecommand{\contentsname}{Contents}
\newcommand{\settocname}[1]{\renewcommand{\contentsname}{#1}}
\providecommand{\listtablename}{List of Tables}
\newcommand{\setlotname}[1]{\renewcommand{\listtablename}{#1}}
\providecommand{\listfigurename}{List of Figures}
\newcommand{\setlofname}[1]{\renewcommand{\listfigurename}{#1}}
\newcommand{\settocbibname}[1]{\renewcommand{\tocbibname}{#1}}
\newcommand{\t@cb@bchapsec}{\if@bibchapter\if@donumbib\chapter{\tocbibname}\else\toc@chapter{\tocbibname}\fi
  \else\if@donumbib\@nameuse{\@tocextra}{\tocbibname}\else\toc@section{\@tocextra}{\tocbibname}\fi\fi}
\newcommand{\t@cb@bsection}{\if@donumbib\@nameuse{\@tocextra}{\tocbibname}\else\toc@section{\@tocextra}{\tocbibname}\fi}
\newcommand{\toc@start}{\if@bibchapter\if@twocolumn\@restonecoltrue\onecolumn\else\@restonecolfalse\fi\fi}
\newcommand{\toc@finish}{\if@bibchapter\if@restonecol\twocolumn\fi\fi}
\newcommand{\tocfile}[2]{\toc@start\lx@tocbibind@tocfile{#1}{#2}\toc@finish}
\newcommand{\tocsection}{\renewcommand{\toc@section}[2]{\@nameuse{##1}{##2}}}");
  // With the bibliography in the contents (`\if@dotocbib`, the default), tocbibind heads it with `\tocbibname`: its
  // `thebibliography` opens with `\bibsection` = `\t@cb@bchapsec` (:154-159), and at the document's start natbib's
  // `\bibsection` becomes `\t@cb@bchapsec`, or `\t@cb@bsection` under natbib's or chapterbib's `sectionbib`
  // (:184-198). The bibliography constructor reads its heading from a `\bibsection` of the form
  // `\<unit>*{<title>}` (`begin_bibliography_clean`), so the heading these conditionals select is written in that
  // form once the options are settled (OXIDIZED_DESIGN_DIVERGENCES #620).
  RawTeX!(r"\AtBeginDocument{%
\@ifpackagewith{natbib}{sectionbib}{\@dot@cb@bsectiontrue}{}%
\@ifpackagewith{chapterbib}{sectionbib}{\@dot@cb@bsectiontrue}{}%
\if@dotocbib
  \@tempswafalse
  \if@dot@cb@bsection\else\if@bibchapter\@tempswatrue\fi\fi
  \if@tempswa
    \if@donumbib\def\bibsection{\chapter{\tocbibname}}\else\def\bibsection{\chapter*{\tocbibname}}\fi
  \else
    \if@donumbib
      \edef\bibsection{\expandafter\noexpand\csname\@tocextra\endcsname{\noexpand\tocbibname}}%
    \else
      \edef\bibsection{\expandafter\noexpand\csname\@tocextra\endcsname*{\noexpand\tocbibname}}%
    \fi
  \fi
\fi}");
  // `\tocfile`'s heading and list (`\toc@chapter`/`\toc@section` then `\@starttoc{<ext>}`): a table of contents
  // titled `<title>` listing `<ext>`, as the kernel's `\tableofcontents`/`\listoffigures` are (sect04.rs).
  DefConstructor!("\\lx@tocbibind@tocfile{}{}",
    "<ltx:TOC lists='#lists' scope='global' select='#select'><ltx:title>#1</ltx:title></ltx:TOC>",
    properties => sub[args] {
      let lists = args[1].as_ref().map(|ext| ext.to_string()).unwrap_or_default();
      let select = if lists == "toc" { toc_select()? } else { String::new() };
      Ok(stored_map!("lists" => lists, "select" => select))
    });
});
