//! Stub for tac.cls (Theory and Applications of Categories journal).
//!
//! tac.cls is a paper-bundled class from TAC (theory + applications of
//! categories). It extends article with custom AMS subject-classification
//! commands (`\amsclass`, `\subjclass`) and journal frontmatter.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");

  // tac.cls L339: \amsclass{<codes>} — preamble AMS subject classification.
  // L341: \let\subjclass\amsclass. Witness 2312.12356.
  DefMacro!("\\amsclass{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#1}");
  Let!("\\subjclass", "\\amsclass");
  // tac.cls custom frontmatter helpers — preserve author content as
  // ltx:note where possible, no-op otherwise.
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\runninghead{}",
    "\\@add@frontmatter{ltx:note}[role=runninghead]{#1}");
  DefMacro!("\\volyear{}",
    "\\@add@frontmatter{ltx:note}[role=volume-year]{#1}");
  DefMacro!("\\volnum{}",
    "\\@add@frontmatter{ltx:note}[role=volume-number]{#1}");
  // tac.cls L317: \address{...} — author postal address note.
  DefMacro!("\\address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}");
  // tac.cls L322: \eaddress{...} — email/electronic address note.
  DefMacro!("\\eaddress{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  // tac.cls L337: \copyrightyear{...} — journal frontmatter copyright year.
  DefMacro!("\\copyrightyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyright-year]{#1}");
  // tac.cls dedicatedto / submitted / received / revised — frontmatter
  // pubdate annotations.
  DefMacro!("\\dedicatedto{}",
    "\\@add@frontmatter{ltx:note}[role=dedication]{#1}");
  DefMacro!("\\submitted{}",
    "\\@add@frontmatter{ltx:note}[role=submitted]{#1}");
  DefMacro!("\\received{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}");
  DefMacro!("\\revised{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}");
  DefMacro!("\\Vol{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\No{}",
    "\\@add@frontmatter{ltx:note}[role=number]{#1}");
  DefMacro!("\\pages{}",
    "\\@add@frontmatter{ltx:note}[role=pages]{#1}");

  // tac.cls L315: \def\CR{\\\phantom{\rm Email: }} — line-break helper
  // in author block. Use a no-op \\ since phantom is layout-only.
  DefMacro!("\\CR", "\\\\");

  // tac.cls L192-200: theorem-likes. Pre-register so they don't fire
  // undefined errors. \newtheoremrm{remark} is just like \newtheorem
  // (the rm marker is roman-text styling, irrelevant for HTML).
  RawTeX!(r"\newtheorem{theorem}{Theorem}");
  RawTeX!(r"\newtheorem{proposition}[theorem]{Proposition}");
  RawTeX!(r"\newtheorem{corollary}[theorem]{Corollary}");
  RawTeX!(r"\newtheorem{lemma}[theorem]{Lemma}");
  RawTeX!(r"\newtheorem{definition}[theorem]{Definition}");
  RawTeX!(r"\newtheorem{scholium}[theorem]{Scholium}");
  RawTeX!(r"\newtheorem{assumption}[theorem]{Assumption}");
  RawTeX!(r"\newtheorem{remark}[theorem]{Remark}");
  RawTeX!(r"\newtheorem{example}[theorem]{Example}");
  RawTeX!(r"\newtheorem{notation}[theorem]{Notation}");
  Let!("\\newtheoremrm", "\\newtheorem");
});
