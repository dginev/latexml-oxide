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
  RequirePackage!("mdframed");

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
});
